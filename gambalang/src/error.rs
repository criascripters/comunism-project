use std::fmt;

/// contexto de parsing: rastreia onde tá na árvore sintática
#[derive(Debug, Clone, PartialEq)]
pub enum ParseContext {
    TopLevel,
    FunctionArgs { func_name: String },
    ListLiteral,
    PipelineRHS,
    LetBinding { name: String },
    WhenCondition,
    LambdaBody,
    BinaryExpr,
}

/// sugestões de correção contextuais
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub fix: Option<String>, // correção automática sugerida
}

impl Suggestion {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            fix: None,
        }
    }

    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct GambaError {
    pub kind: ErrorKind,
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub context: Option<ParseContext>,
    pub suggestions: Vec<Suggestion>,
    pub snippet: Option<String>, // trecho de código relevante
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Lex,
    Parse,
    Runtime,
}

impl GambaError {
    pub fn lex(msg: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            kind: ErrorKind::Lex,
            message: msg.into(),
            line,
            col,
            context: None,
            suggestions: vec![],
            snippet: None,
        }
    }

    pub fn parse(msg: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            kind: ErrorKind::Parse,
            message: msg.into(),
            line,
            col,
            context: None,
            suggestions: vec![],
            snippet: None,
        }
    }

    pub fn parse_with_context(
        msg: impl Into<String>,
        line: usize,
        col: usize,
        ctx: ParseContext,
    ) -> Self {
        let mut err = Self::parse(msg, line, col);
        err.context = Some(ctx);
        err.auto_suggest();
        err
    }

    pub fn runtime(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Runtime,
            message: msg.into(),
            line: 0,
            col: 0,
            context: None,
            suggestions: vec![],
            snippet: None,
        }
    }

    // permite anotar posição em erros de runtime (call site, etc)
    // so preenche se ainda não tiver posição.
    pub fn with_pos(mut self, line: usize, col: usize) -> Self {
        if let ErrorKind::Runtime = self.kind {
            if self.line == 0 && self.col == 0 && line > 0 {
                self.line = line;
                self.col = col;
            }
        }
        self
    }

    pub fn with_context(mut self, ctx: ParseContext) -> Self {
        self.context = Some(ctx);
        self.auto_suggest();
        self
    }

    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    pub fn add_suggestion(mut self, sugg: Suggestion) -> Self {
        self.suggestions.push(sugg);
        self
    }

    /// gera sugestoes automáticas baseadas no contexto e mensagem
    fn auto_suggest(&mut self) {
        // padrao: "esperado X, encontrado Y"
        if self.message.contains("esperado RParen, encontrado Bar") {
            if matches!(self.context, Some(ParseContext::FunctionArgs { .. })) {
                self.suggestions.push(
                    Suggestion::new(
                        "bar-lambdas (|x| { ... }) funcionam melhor fora de contextos com juxta desabilitada"
                    )
                    .with_fix("tente usar parênteses: filter(|x| { ... })")
                );
            } else if matches!(self.context, Some(ParseContext::PipelineRHS)) {
                self.suggestions.push(
                    Suggestion::new(
                        "no lado direito de um pipe, bar-lambdas devem ser argumentos diretos",
                    )
                    .with_fix("use: xs |> filter |x| { ... }  ou  xs |> filter(|x| { ... })"),
                );
            }
        }

        // padrao: nome não encontrado
        if self.message.starts_with("nome não encontrado:") {
            self.suggestions.push(Suggestion::new(
                "verifique se você definiu este nome antes de usá-lo com '::'",
            ));
            self.suggestions.push(Suggestion::new(
                "lembre que gamba é case-sensitive (maiúsculas ≠ minúsculas)",
            ));
        }

        // padrao: arity mismatch
        if self.message.contains("esperava") && self.message.contains("argumentos") {
            if let Some(ctx) = &self.context {
                if matches!(ctx, ParseContext::PipelineRHS) {
                    self.suggestions.push(
                        Suggestion::new(
                            "no pipe (|>), o valor à esquerda vira o primeiro argumento da função",
                        )
                        .with_fix("exemplo: 10 |> add(5)  equivale a  add(10, 5)"),
                    );
                }
            }
        }

        // padrao: tentativa de chamar não-função
        if self
            .message
            .contains("tentativa de chamar um valor não-função")
        {
            self.suggestions.push(Suggestion::new(
                "verifique se o nome não foi sobrescrito por um let posterior",
            ));
            self.suggestions.push(Suggestion::new(
                "funções vêm do ambiente, builtins ou de definições com 'fn'",
            ));
        }
    }
}

impl fmt::Display for GambaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // header do erro
        match self.kind {
            ErrorKind::Lex | ErrorKind::Parse => write!(
                f,
                "erro de {:?} em {}:{}\n{}",
                self.kind, self.line, self.col, self.message
            ),
            ErrorKind::Runtime => {
                if self.line > 0 {
                    write!(
                        f,
                        "erro de runtime em {}:{}\n{}",
                        self.line, self.col, self.message
                    )
                } else {
                    write!(f, "erro de runtime\n{}", self.message)
                }
            }
        }?;

        // contexto
        if let Some(ctx) = &self.context {
            write!(f, "\ncontexto: ")?;
            match ctx {
                ParseContext::TopLevel => write!(f, "nível superior do programa")?,
                ParseContext::FunctionArgs { func_name } => {
                    write!(f, "argumentos da função '{}'", func_name)?
                }
                ParseContext::ListLiteral => write!(f, "dentro de uma lista [...]")?,
                ParseContext::PipelineRHS => write!(f, "lado direito de um pipe (|>)")?,
                ParseContext::LetBinding { name } => write!(f, "definição de '{}'", name)?,
                ParseContext::WhenCondition => write!(f, "condição de 'when'")?,
                ParseContext::LambdaBody => write!(f, "corpo de função lambda")?,
                ParseContext::BinaryExpr => write!(f, "expressão binária")?,
            }
        }

        // snipper de código
        if let Some(snippet) = &self.snippet {
            write!(f, "\ntrecho de código:\n{}", snippet)?;
        }

        // sugestoes
        if !self.suggestions.is_empty() {
            write!(f, "\nsugestões:")?;
            for (i, sugg) in self.suggestions.iter().enumerate() {
                write!(f, "\n  {}. {}", i + 1, sugg.message)?;
                if let Some(fix) = &sugg.fix {
                    write!(f, "\n     → {}", fix)?;
                }
            }
        }

        Ok(())
    }
}

impl std::error::Error for GambaError {}
