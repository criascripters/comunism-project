use std::fmt;

#[derive(Debug, Clone)]
pub struct GambaError {
    pub kind: ErrorKind,
    pub message: String,
    pub line: usize,
    pub col: usize,
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
        }
    }
    pub fn parse(msg: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            kind: ErrorKind::Parse,
            message: msg.into(),
            line,
            col,
        }
    }
    pub fn runtime(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Runtime,
            message: msg.into(),
            line: 0,
            col: 0,
        }
    }
}

impl fmt::Display for GambaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::Lex | ErrorKind::Parse => write!(
                f,
                "{:?} error at {}:{}: {}",
                self.kind, self.line, self.col, self.message
            ),
            ErrorKind::Runtime => write!(f, "runtime error: {}", self.message),
        }
    }
}

impl std::error::Error for GambaError {}
