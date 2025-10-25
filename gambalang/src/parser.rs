use crate::ast::*;
use crate::error::{GambaError, ParseContext, Suggestion};
use crate::lexer::{Token, TokenKind};

// permitir argumento negativo por justaposição (ex.: f -1)
// isso pode fazer "f - 1" ser lido como f(-1), por isso tá setado como false por enquanto
const ALLOW_NEG_ARG_JUXTA: bool = false;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Juxta {
    Allow,
    Forbid,
}

pub struct Parser {
    tokens: Vec<Token>,
    i: usize,
    context_stack: Vec<ParseContext>, // pilha de contextos
}

// tenta extrair func e args de uma chamada existente
// funciona pra chamadas com parênteses e justaposiçao
// agora também devolve a posição do call site, se existir
fn can_take_juxta(callee: &Expr) -> bool {
    // so permite justaposição quando o alvo é algo claramente chamavel
    // isso evita casos como "64 x" virarem uma chamada acidental
    matches!(
        callee,
        Expr::Ident(_) | Expr::Lambda { .. } | Expr::Call { .. } | Expr::Group(_)
    )
}

fn extract_call_info(expr: Expr) -> Option<(Expr, Vec<Expr>, Option<(usize, usize)>)> {
    match expr {
        Expr::Call {
            func,
            args,
            call_line,
            call_col,
            ..
        } => Some((*func, args, Some((call_line, call_col)))),
        _ => None,
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            i: 0,
            context_stack: vec![ParseContext::TopLevel],
        }
    }

    fn push_context(&mut self, ctx: ParseContext) {
        self.context_stack.push(ctx);
    }

    fn pop_context(&mut self) {
        if self.context_stack.len() > 1 {
            self.context_stack.pop();
        }
    }

    fn current_context(&self) -> ParseContext {
        self.context_stack
            .last()
            .cloned()
            .unwrap_or(ParseContext::TopLevel)
    }

    fn parse_error(&self, msg: impl Into<String>) -> GambaError {
        let t = self.peek();
        GambaError::parse_with_context(msg, t.line, t.col, self.current_context())
    }

    fn expect_error(&self, expected: &TokenKind, found: &Token) -> GambaError {
        self.parse_error(format!(
            "esperado {:?}, encontrado {:?}",
            expected, found.kind
        ))
        .add_suggestion(self.context_specific_hint(expected, &found.kind))
    }

    fn context_specific_hint(&self, expected: &TokenKind, found: &TokenKind) -> Suggestion {
        match self.current_context() {
            ParseContext::FunctionArgs { ref func_name } => {
                if matches!(expected, TokenKind::RParen) && matches!(found, TokenKind::Bar) {
                    Suggestion::new(format!(
                        "ao chamar '{}', bar-lambdas precisam ser coletadas por justaposição ou dentro de parênteses",
                        func_name
                    ))
                    .with_fix(format!("tente: {}(|x| {{ ... }})", func_name))
                } else {
                    Suggestion::new("verifique vírgulas, parênteses e chaves")
                }
            }
            ParseContext::PipelineRHS => {
                Suggestion::new("no RHS do pipe, use funções, chamadas ou lambdas explícitas")
            }
            ParseContext::ListLiteral => {
                Suggestion::new("elementos de lista devem ser separados por espaços ou vírgulas")
            }
            _ => Suggestion::new("revise a sintaxe neste ponto"),
        }
    }
    fn peek(&self) -> &Token {
        &self.tokens[self.i]
    }
    fn at(&self, k: &TokenKind) -> bool {
        &self.peek().kind == k
    }
    fn next(&mut self) -> &Token {
        let t = &self.tokens[self.i];
        self.i += 1;
        t
    }
    fn bump_if(&mut self, k: &TokenKind) -> bool {
        if self.at(k) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, k: &TokenKind) -> Result<Token, GambaError> {
        if self.at(k) {
            Ok(self.peek().clone())
        } else {
            let t = self.peek().clone();
            Err(self.expect_error(k, &t))
        }
    }
    fn consume(&mut self, k: &TokenKind) -> Result<Token, GambaError> {
        let tok = self.expect(k)?;
        self.i += 1;
        Ok(tok)
    }
    fn skip_newlines(&mut self) {
        while self.at(&TokenKind::Newline) {
            self.i += 1;
        }
    }

    // consome "(when (cond)) { then } { else }" ou "(when cond) { then } { else }"
    fn parse_parenthesized_when(&mut self) -> Result<Expr, GambaError> {
        //  vê '(' e deve tratar "(when ...)" em duas variantes:
        // 1) (when cond) { then } { else }
        // 2) (when cond { then } { else })
        //
        // passo: consome '(' e 'when', le a condição (com ou sem parênteses),
        // depois decide se fecha o ')' agora ou depois dos blocos

        self.consume(&TokenKind::LParen)?; // '('
        self.consume(&TokenKind::When)?; // 'when'

        // condição: pode ser "(cond)" ou sem parênteses
        let cond = if self.bump_if(&TokenKind::LParen) {
            let e = self.parse_expr()?;
            self.consume(&TokenKind::RParen)?; // fecha parenteses da condição
            e
        } else {
            // sem parênteses extras: nao deixa juxta capturar '{'
            self.parse_expr_with_juxta(Juxta::Forbid)?
        };

        self.skip_newlines();

        // duas variantes:
        // a) (when cond) {then}{else}  => fecha parentese agora, blocos depois
        // b) (when cond {then}{else})  => blocos agora, fecha parentese depois
        if self.bump_if(&TokenKind::RParen) {
            // variante a)
            self.skip_newlines();
            let then_b = self.parse_block()?;
            self.skip_newlines();
            let else_b = self.parse_block()?;
            Ok(Expr::When {
                cond: Box::new(cond),
                then_branch: then_b,
                else_branch: else_b,
            })
        } else if self.at(&TokenKind::LBrace) {
            // variante b)
            let then_b = self.parse_block()?;
            self.skip_newlines();
            let else_b = self.parse_block()?;
            self.skip_newlines();
            self.consume(&TokenKind::RParen)?; // fecha o parentese depois dos blocos
            Ok(Expr::When {
                cond: Box::new(cond),
                then_branch: then_b,
                else_branch: else_b,
            })
        } else {
            // forma inesperada: ajuda com uma mensagem clara
            let t = self.peek().clone();
            Err(self.parse_error(
                "após '(when cond' esperava ')' seguido de '{', ou então '{' diretamente.
             exemplos válidos:
  (when cond) { ... } { ... }
  (when cond { ... } { ... })",
            ))
        }
    }

    // rhs "atômico" para let: |lambda|, when, fn, {bloco}, e (when ...){..}{..}
    fn parse_let_rhs_atom(&mut self) -> Result<Expr, GambaError> {
        self.skip_newlines();

        // caso especial: (when ...) nas duas variantes suportadas
        if self.at(&TokenKind::LParen)
            && self.i + 1 < self.tokens.len()
            && matches!(self.tokens[self.i + 1].kind, TokenKind::When)
        {
            return self.parse_parenthesized_when();
        }

        // regra geral: parseie a expressão inteira como RHS do let,
        // incluindo pipes, binarios, chamadas etc
        self.parse_expr_with_juxta(Juxta::Allow)
    }

    pub fn parse_program(&mut self) -> Result<Program, GambaError> {
        let mut items = Vec::new();
        self.skip_newlines();
        while !self.at(&TokenKind::Eof) {
            items.push(self.parse_expr_with_juxta(Juxta::Allow)?);
            self.skip_newlines();
        }
        Ok(Program { items })
    }

    fn parse_expr(&mut self) -> Result<Expr, GambaError> {
        self.parse_expr_with_juxta(Juxta::Allow)
    }

    fn parse_expr_with_juxta(&mut self, juxta: Juxta) -> Result<Expr, GambaError> {
        self.skip_newlines();
        // let binding: ident :: expr
        if let TokenKind::Ident(name) = self.peek().kind.clone() {
            if self.i + 1 < self.tokens.len()
                && self.tokens[self.i + 1].kind == TokenKind::DoubleColon
            {
                self.i += 2; // consome ident e ::
                self.skip_newlines();

                // usa o rhs atomico para let: |lambda|, when, fn, {bloco}, e (when ...){..}{..}
                let expr = self.parse_let_rhs_atom()?;

                return Ok(Expr::Let {
                    name,
                    expr: Box::new(expr),
                });
            }
        }

        // pipeline
        // topo: nao permitir juxta atravessando newline
        let mut left = self.parse_binop_expr_with_juxta_opts(0, juxta, false)?;
        while self.bump_if(&TokenKind::PipeOp) {
            self.skip_newlines();
            // RHS de pipe: permitir juxta atravessando newline (ex.: "10 |> add\n5")
            let right = self.parse_binop_expr_with_juxta_opts(0, juxta, true)?;

            // permite parenteses no alvo do pipe: x |> (f a)
            let target = match right.clone() {
                Expr::Group(inner) => *inner,
                other => other,
            };

            // token atual (ou próximo) serve como fallback de posição do call site
            let t = self.peek().clone();

            left = match extract_call_info(target.clone()) {
                Some((func, mut args, loc)) => {
                    // insere LHS como primeiro argumento da chamada existente
                    args.insert(0, left);
                    let (line, col) = loc.unwrap_or((t.line, t.col));
                    Expr::Call {
                        func: Box::new(func),
                        args,
                        call_line: line,
                        call_col: col,
                        style: CallStyle::Pipe,
                    }
                }
                None => {
                    // RHS não é chamada => chamar rhs(LHS)
                    match target {
                        Expr::Ident(_) | Expr::Lambda { .. } => Expr::Call {
                            func: Box::new(target),
                            args: vec![left],
                            call_line: t.line,
                            call_col: t.col,
                            style: CallStyle::Pipe,
                        },
                        _ => {
                            let t = self.peek().clone();
                            return Err(self.parse_error(
                                "após |> o alvo deve ser uma função (nome, chamada ou lambda). \
Ex.: xs |> map(fn x { ... })  |  x |> f(a)  |  x |> (fn v { ... })",
                            ));
                        }
                    }
                }
            };
        }
        Ok(left)
    }

    fn parse_block(&mut self) -> Result<Block, GambaError> {
        self.consume(&TokenKind::LBrace)?;
        let mut items = Vec::new();
        self.skip_newlines();
        while !self.at(&TokenKind::RBrace) {
            items.push(self.parse_expr()?);
            self.skip_newlines();
        }
        self.consume(&TokenKind::RBrace)?;
        Ok(Block { items })
    }

    fn parse_binop_expr(&mut self) -> Result<Expr, GambaError> {
        self.parse_binop_expr_with_juxta(0, Juxta::Allow)
    }
    fn op_precedence(op: &BinaryOp) -> u8 {
        match op {
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 20,
            BinaryOp::Add | BinaryOp::Sub => 10,
            // comparações tem precedência menor que soma/subtração
            BinaryOp::Gt | BinaryOp::Ge | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Eq => 5,
            BinaryOp::And => 3, // &&
            BinaryOp::Or => 2,  // ||
        }
    }
    fn token_to_binop(tok: &TokenKind) -> Option<BinaryOp> {
        match tok {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            TokenKind::Star => Some(BinaryOp::Mul),
            TokenKind::Slash => Some(BinaryOp::Div),
            TokenKind::Percent => Some(BinaryOp::Mod),
            TokenKind::Greater => Some(BinaryOp::Gt),
            TokenKind::GreaterEqual => Some(BinaryOp::Ge),
            TokenKind::Less => Some(BinaryOp::Lt),
            TokenKind::LessEqual => Some(BinaryOp::Le),
            TokenKind::EqualEqual => Some(BinaryOp::Eq),
            TokenKind::AndAnd => Some(BinaryOp::And),
            TokenKind::OrOr => Some(BinaryOp::Or),

            _ => None,
        }
    }
    fn parse_binop_expr_with_juxta(
        &mut self,
        min_prec: u8,
        juxta: Juxta,
    ) -> Result<Expr, GambaError> {
        self.parse_binop_expr_with_juxta_opts(min_prec, juxta, false)
    }

    // nova: controla se juxta pode atravessar newline (só usa true no RHS de pipe)
    fn parse_binop_expr_with_juxta_opts(
        &mut self,
        min_prec: u8,
        juxta: Juxta,
        allow_newline_juxta: bool,
    ) -> Result<Expr, GambaError> {
        let mut left = self.parse_call_expr_with_juxta_opts(juxta, allow_newline_juxta)?;
        loop {
            self.skip_newlines();
            let op = if let Some(op) = Self::token_to_binop(&self.peek().kind) {
                op
            } else {
                break;
            };
            let prec = Self::op_precedence(&op);
            if prec < min_prec {
                break;
            }
            self.next();
            let right =
                self.parse_binop_expr_with_juxta_opts(prec + 1, juxta, allow_newline_juxta)?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_lambda_body_expr(&mut self) -> Result<Block, GambaError> {
        // coorpo de lambda sem chaves: parseia exatamente uma expressão,
        // com juxta proibida (para não capturar args externos), mas com
        // binários funcionando normalmente
        let expr = self.parse_binop_expr_with_juxta_opts(0, Juxta::Forbid, false)?;
        Ok(Block { items: vec![expr] })
    }

    fn parse_bar_lambda(&mut self) -> Result<Expr, GambaError> {
        // ja sabemos que o próximo token é '|'
        self.consume(&TokenKind::Bar)?;
        let mut params = Vec::new();

        // zero-args: '||'
        if self.bump_if(&TokenKind::Bar) {
            // nada
        } else {
            loop {
                match self.peek().kind.clone() {
                    TokenKind::Ident(name) => {
                        self.next();
                        params.push(name);
                    }
                    _ => {
                        let t = self.peek().clone();
                        return Err(self.parse_error("esperado nome de parâmetro dentro de | |"));
                    }
                }
                self.skip_newlines();
                // virgulas opcionais entre parametros
                if self.bump_if(&TokenKind::Comma) {
                    self.skip_newlines();
                }
                if self.bump_if(&TokenKind::Bar) {
                    break;
                }
            }
        }

        // garante que espaços/linhas entre "|params|" e o corpo nao atrapalham
        self.skip_newlines();

        // corpo: ou { bloco } ou expressão simples
        let body = if self.at(&TokenKind::LBrace) {
            self.parse_block()?
        } else {
            // corpo sem chaves: uma única expressão, juxta proibida
            self.parse_lambda_body_expr()?
        };
        Ok(Expr::Lambda { params, body })
    }

    fn parse_call_expr(&mut self) -> Result<Expr, GambaError> {
        self.parse_call_expr_with_juxta(Juxta::Allow)
    }

    // versão original (compat): sem juxta atravessando newline
    fn parse_call_expr_with_juxta(&mut self, juxta: Juxta) -> Result<Expr, GambaError> {
        self.parse_call_expr_with_juxta_opts(juxta, false)
    }

    // controla se juxta pode atravessar newline
    fn parse_call_expr_with_juxta_opts(
        &mut self,
        juxta: Juxta,
        allow_newline_juxta: bool,
    ) -> Result<Expr, GambaError> {
        let mut callee = self.parse_primary()?;
        // depois de consumir um bar-lambda como arg por juxta, nao aceitar mais args por juxta
        let mut stop_juxta_args = false;
        // lembrar se o último passo foi uma chamada com parênteses (pra tratar newline)
        let mut last_step_was_parens_call = false;

        loop {
            // no RHS do pipe pode atravessar newline; registra se tem newline aqui
            let crossed_newline =
                allow_newline_juxta && matches!(self.peek().kind, TokenKind::Newline);
            if allow_newline_juxta {
                self.skip_newlines();
            }

            // caso 1: chamada com parênteses f(a, b) sempre permitida
            if self.at(&TokenKind::LParen) {
                // track context: dentro de argumentos de função
                if let Expr::Ident(ref name) = callee {
                    self.push_context(ParseContext::FunctionArgs {
                        func_name: name.clone(),
                    });
                }

                let lpar = self.consume(&TokenKind::LParen)?;
                let mut args = Vec::new();
                self.skip_newlines();
                if !self.at(&TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr_with_juxta(Juxta::Forbid)?);
                        self.skip_newlines();
                        if !self.bump_if(&TokenKind::Comma) {
                            break;
                        }
                        self.skip_newlines();
                    }
                }
                self.consume(&TokenKind::RParen)?;

                self.pop_context(); // sai do contexto de argumentos
                                    // se a chamada com parênteses recebeu uma lambda, nao aceitar mais args por juxtaposicão
                let had_lambda_arg = args.iter().any(|a| matches!(a, Expr::Lambda { .. }));
                callee = Expr::Call {
                    func: Box::new(callee),
                    args,
                    call_line: lpar.line,
                    call_col: lpar.col,
                    style: CallStyle::Parens,
                };
                if had_lambda_arg {
                    stop_juxta_args = true; // bar-lambda já sela a juxta
                }
                last_step_was_parens_call = true;
                continue; // permite f(a)(b)
            }

            // caso 2: bar-lambda sempre pode ser coletada como argumento (exceção a juxta=Forbid)
            // isso permite `filter |x| { ... }` em qualquer contexto, incluindo dentro de listas/argumentos
            if self.at(&TokenKind::Bar) && can_take_juxta(&callee) {
                let call_site = self.peek().clone();
                let lam = self.parse_bar_lambda()?;
                stop_juxta_args = true; // bar-lambda sela a juxta seguinte
                callee = match callee {
                    Expr::Call {
                        func,
                        mut args,
                        call_line,
                        call_col,
                        style,
                    } => {
                        args.push(lam);
                        Expr::Call {
                            func,
                            args,
                            call_line,
                            call_col,
                            style,
                        }
                    }
                    _ => Expr::Call {
                        func: Box::new(callee),
                        args: vec![lam],
                        call_line: call_site.line,
                        call_col: call_site.col,
                        style: CallStyle::Juxta,
                    },
                };
                last_step_was_parens_call = false;
                continue;
            }

            // caso 3: chamada por justaposição f a b (apenas se permitido)
            if juxta == Juxta::Allow {
                // se atravessou newline, e o último passo foi uma chamada com parênteses,
                // não absorver mais argumentos por juxta (evita colar a próxima linha)
                if allow_newline_juxta && crossed_newline && last_step_was_parens_call {
                    break;
                }
                // evita vazamento: apos newline, não absorver Ident seguido de '(' como arg
                if allow_newline_juxta && crossed_newline {
                    if matches!(self.peek().kind, TokenKind::Ident(_))
                        && (self.i + 1) < self.tokens.len()
                        && matches!(self.tokens[self.i + 1].kind, TokenKind::LParen)
                    {
                        break;
                    }
                }

                // se acaba de consumir um bar-lambda por juxta, encerra a coleta por juxta
                if stop_juxta_args {
                    break;
                }

                // não transformar qualquer coisa em alvo chamavel
                if !can_take_juxta(&callee) {
                    break; // encerra a coleta por juxta pra não criar chamadas nonsense
                }

                // suporte opcional a argumento iniciado por '-' (unario)
                let minus_starts_arg = if ALLOW_NEG_ARG_JUXTA {
                    matches!(self.peek().kind, TokenKind::Minus)
                        && (self.i + 1) < self.tokens.len()
                        && matches!(
                            self.tokens[self.i + 1].kind,
                            TokenKind::Number(_)
                                | TokenKind::LParen
                                | TokenKind::Ident(_)
                                | TokenKind::String(_)
                                | TokenKind::Bool(_)
                                | TokenKind::Bar
                                | TokenKind::LBracket
                                | TokenKind::LBrace
                                | TokenKind::Fn
                        )
                } else {
                    false
                };

                let next_is_arg = matches!(
                    self.peek().kind,
                    TokenKind::Number(_)
                        | TokenKind::String(_)
                        | TokenKind::Bool(_)
                        | TokenKind::Ident(_)
                        | TokenKind::Bar
                        | TokenKind::LBracket
                        | TokenKind::LBrace
                        | TokenKind::Fn
                ) || minus_starts_arg;

                if next_is_arg {
                    // posição da chamada por justaposição = início do argumento
                    let call_site = self.peek().clone();

                    // bar-lambda já foi tratada acima (caso 2), aqui so primários normais
                    let arg = self.parse_primary()?;

                    callee = match callee {
                        Expr::Call {
                            func,
                            mut args,
                            call_line,
                            call_col,
                            style,
                        } => {
                            args.push(arg);
                            Expr::Call {
                                func,
                                args,
                                call_line,
                                call_col,
                                style,
                            }
                        }
                        _ => Expr::Call {
                            func: Box::new(callee),
                            args: vec![arg],
                            call_line: call_site.line,
                            call_col: call_site.col,
                            style: CallStyle::Juxta,
                        },
                    };
                    // um passo de juxta não é "parens"; reset da flag
                    last_step_was_parens_call = false;
                    continue;
                }
            }

            // nenhuma continuação de chamada; reset flag
            last_step_was_parens_call = false;
            break;
        }
        Ok(callee)
    }

    fn parse_param_list(&mut self) -> Result<Vec<String>, GambaError> {
        if self.bump_if(&TokenKind::LParen) {
            // fn(a,b) ou fn()
            let mut params = Vec::new();
            self.skip_newlines();
            if !self.at(&TokenKind::RParen) {
                loop {
                    if let TokenKind::Ident(name) = self.peek().kind.clone() {
                        self.next();
                        params.push(name);
                    } else {
                        let t = self.peek().clone();
                        return Err(self.parse_error("Esperado nome de parâmetro"));
                    }
                    self.skip_newlines();
                    if !self.bump_if(&TokenKind::Comma) {
                        break;
                    }
                    self.skip_newlines();
                }
            }
            self.consume(&TokenKind::RParen)?;
            return Ok(params);
        }
        if let TokenKind::Ident(name) = self.peek().kind.clone() {
            self.next();
            return Ok(vec![name]);
        } // fn x
        if self.at(&TokenKind::LBrace) {
            return Ok(vec![]);
        } // fn { }
        let t = self.peek().clone();
        Err(self.parse_error("esperado '(', nome de parâmetro, ou '{' após 'fn'"))
    }

    fn parse_list(&mut self) -> Result<Expr, GambaError> {
        self.consume(&TokenKind::LBracket)?;
        let mut items = Vec::new();
        self.skip_newlines();
        while !self.at(&TokenKind::RBracket) {
            // itens de lista: sem justaposição
            items.push(self.parse_expr_with_juxta(Juxta::Forbid)?);
            self.skip_newlines();
            // vírgulas opcionais
            if self.bump_if(&TokenKind::Comma) {
                self.skip_newlines();
            }
        }
        self.consume(&TokenKind::RBracket)?;
        Ok(Expr::List(items))
    }

    fn parse_primary(&mut self) -> Result<Expr, GambaError> {
        self.skip_newlines();
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Number(n) => {
                self.next();
                Ok(Expr::Number(n))
            }
            TokenKind::String(s) => {
                self.next();
                Ok(Expr::String(s))
            }
            TokenKind::Bool(b) => {
                self.next();
                Ok(Expr::Bool(b))
            }

            // !expr
            TokenKind::Bang => {
                self.next();
                let e = self.parse_primary()?;
                Ok(Expr::UnaryNot(Box::new(e)))
            }

            TokenKind::Bar => {
                // lambda estilo rust: |params| expr ou |params| { ... }
                self.parse_bar_lambda()
            }

            // || como lambda zero-args (equivale a "| |")
            TokenKind::OrOr => {
                self.next();
                // corpo pode ser { bloco } ou expressao simples
                self.skip_newlines();
                let body = if self.at(&TokenKind::LBrace) {
                    self.parse_block()?
                } else {
                    self.parse_lambda_body_expr()?
                };
                Ok(Expr::Lambda {
                    params: vec![],
                    body,
                })
            }
            TokenKind::Ident(name) => {
                // suporte inline a: nome :: expr em qualquer posição
                // (alem do atalho já feito no topo de parse_expr_with_juxta)
                if self.i + 1 < self.tokens.len()
                    && matches!(self.tokens[self.i + 1].kind, TokenKind::DoubleColon)
                {
                    // consome ident e ::
                    self.i += 2;
                    self.skip_newlines();
                    // rhs atômico (|...|, when, fn, { }, (when ...), ou expressao comum)
                    let expr = self.parse_let_rhs_atom()?;
                    return Ok(Expr::Let {
                        name,
                        expr: Box::new(expr),
                    });
                }

                // caso comum: só um identificador
                self.next();
                Ok(Expr::Ident(name))
            }
            TokenKind::LParen => {
                // se for "(when ...){...}{...}", consome como um when completo
                if self.i + 1 < self.tokens.len()
                    && matches!(self.tokens[self.i + 1].kind, TokenKind::When)
                {
                    return self.parse_parenthesized_when();
                }
                // caso normal: grupo
                self.next();
                let e = self.parse_expr()?;
                self.consume(&TokenKind::RParen)?;
                Ok(Expr::Group(Box::new(e)))
            }
            TokenKind::LBracket => self.parse_list(),
            TokenKind::LBrace => Ok(Expr::Block(self.parse_block()?)),
            TokenKind::Minus => {
                self.next();
                let e = self.parse_primary()?;
                Ok(Expr::UnaryMinus(Box::new(e)))
            }
            TokenKind::Fn => {
                self.next();
                let params = self.parse_param_list()?;
                let body = self.parse_block()?;
                Ok(Expr::Lambda { params, body })
            }
            // when como expressão primária (agora com ou sem parênteses)
            TokenKind::When => {
                self.next();
                // suporta: when (cond) { ... } { ... }  ou  when cond { ... } { ... }
                let cond = if self.bump_if(&TokenKind::LParen) {
                    let e = self.parse_expr()?;
                    self.consume(&TokenKind::RParen)?;
                    e
                } else {
                    // sem parênteses: parseia a condição sem permitir juxta,
                    // assim o próximo '{' não vira argumento por juxta
                    self.parse_expr_with_juxta(Juxta::Forbid)?
                };
                self.skip_newlines();
                let then_b = self.parse_block()?;
                self.skip_newlines();
                let else_b = self.parse_block()?;
                Ok(Expr::When {
                    cond: Box::new(cond),
                    then_branch: then_b,
                    else_branch: else_b,
                })
            }
            _ => Err(self.parse_error("token inesperado na expressão")),
        }
    }
}
