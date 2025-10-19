use crate::ast::*;
use crate::error::GambaError;
use crate::lexer::{Token, TokenKind};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Juxta {
    Allow,
    Forbid,
}

pub struct Parser {
    tokens: Vec<Token>,
    i: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, i: 0 }
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
            Err(GambaError::parse(
                format!("esperado {:?}, encontrado {:?}", k, t.kind),
                t.line,
                t.col,
            ))
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
                let expr = self.parse_expr_with_juxta(juxta)?;
                return Ok(Expr::Let {
                    name,
                    expr: Box::new(expr),
                });
            }
        }

        // pipeline
        let mut left = self.parse_binop_expr_with_juxta(0, juxta)?;
        while self.bump_if(&TokenKind::PipeOp) {
            self.skip_newlines();
            let right = self.parse_binop_expr_with_juxta(0, juxta)?;

            // permite parênteses no alvo do pipe: x |> (f a)
            let target = match right.clone() {
                Expr::Group(inner) => *inner,
                other => other,
            };

            left = match target {
                Expr::Call { func, mut args } => {
                    args.insert(0, left);
                    Expr::Call { func, args }
                }
                Expr::Ident(_) | Expr::Lambda { .. } => Expr::Call {
                    func: Box::new(target),
                    args: vec![left],
                },
                _ => {
                    let t = self.peek().clone();
                    return Err(GambaError::parse(
                        "após |> o alvo deve ser uma função (nome, chamada ou lambda). \
Ex.: xs |> map(fn x { ... })  |  x |> f(a)  |  x |> (fn v { ... })",
                        t.line,
                        t.col,
                    ));
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
        }
    }
    fn token_to_binop(tok: &TokenKind) -> Option<BinaryOp> {
        match tok {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            TokenKind::Star => Some(BinaryOp::Mul),
            TokenKind::Slash => Some(BinaryOp::Div),
            TokenKind::Percent => Some(BinaryOp::Mod),
            _ => None,
        }
    }
    fn parse_binop_expr_with_juxta(
        &mut self,
        min_prec: u8,
        juxta: Juxta,
    ) -> Result<Expr, GambaError> {
        let mut left = self.parse_call_expr_with_juxta(juxta)?;
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
            let right = self.parse_binop_expr_with_juxta(prec + 1, juxta)?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }
    
    fn parse_call_expr(&mut self) -> Result<Expr, GambaError> {
        self.parse_call_expr_with_juxta(Juxta::Allow)
    }

    fn parse_call_expr_with_juxta(&mut self, juxta: Juxta) -> Result<Expr, GambaError> {
        let mut callee = self.parse_primary()?;

        loop {
            // caso 1: chamada com parênteses f(a, b)
            if self.bump_if(&TokenKind::LParen) {
                let mut args = Vec::new();
                self.skip_newlines();
                if !self.at(&TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr_with_juxta(Juxta::Allow)?);
                        self.skip_newlines();
                        if !self.bump_if(&TokenKind::Comma) {
                            break;
                        }
                        self.skip_newlines();
                    }
                }
                self.consume(&TokenKind::RParen)?;
                callee = Expr::Call {
                    func: Box::new(callee),
                    args,
                };
                continue; // permite f(a)(b)
            }

            // Caso 2: Chamada por justaposição f a b (somente se permitido)
            if juxta == Juxta::Allow {
                let next_is_arg = matches!(
                    self.peek().kind,
                    TokenKind::Number(_)
                        | TokenKind::String(_)
                        | TokenKind::Bool(_)
                        | TokenKind::Ident(_)
                        | TokenKind::LBracket
                        | TokenKind::LBrace
                        | TokenKind::Fn
                );

                if next_is_arg {
                    let arg = self.parse_primary()?;
                    callee = match callee {
                        Expr::Call { func, mut args } => {
                            args.push(arg);
                            Expr::Call { func, args }
                        }
                        _ => Expr::Call {
                            func: Box::new(callee),
                            args: vec![arg],
                        },
                    };
                    continue;
                }
            }

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
                        return Err(GambaError::parse(
                            "Esperado nome de parâmetro",
                            t.line,
                            t.col,
                        ));
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
        Err(GambaError::parse(
            "esperado '(', nome de parâmetro, ou '{' após 'fn'",
            t.line,
            t.col,
        ))
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
            TokenKind::Ident(name) => {
                self.next();
                Ok(Expr::Ident(name))
            }
            TokenKind::LParen => {
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
            // when como expressão primária (permite pipes naturalmente)
            TokenKind::When => {
                self.next();
                self.consume(&TokenKind::LParen)?;
                let cond = self.parse_expr()?;
                self.consume(&TokenKind::RParen)?;
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
            _ => Err(GambaError::parse(
                "token inesperado na expressão",
                tok.line,
                tok.col,
            )),
        }
    }
}
