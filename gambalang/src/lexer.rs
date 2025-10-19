use crate::error::GambaError;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Number(f64),
    String(String),

    // delimitadores
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    // operadores
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    PipeOp,      // |>
    DoubleColon, // ::

    Comma,
    Newline,

    // palavras-chave
    Fn,
    When,
    Bool(bool),

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer<'a> {
    src: &'a str,
    chars: Vec<char>,
    i: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            chars: src.chars().collect(),
            i: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).cloned()
    }
    fn peek2(&self) -> Option<char> {
        self.chars.get(self.i + 1).cloned()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.chars.get(self.i).cloned();
        if let Some(c) = ch {
            self.i += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            line: self.line,
            col: self.col,
        }
    }

    fn skip_spaces(&mut self) -> Option<Token> {
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\r') => {
                    self.bump();
                }
                Some('\n') => {
                    self.bump();
                    return Some(self.make_token(TokenKind::Newline));
                }
                Some('/') if self.peek2() == Some('/') => {
                    // comentario de linha
                    while let Some(c) = self.peek() {
                        self.bump();
                        if c == '\n' {
                            return Some(self.make_token(TokenKind::Newline));
                        }
                    }
                    return None;
                }
                _ => break,
            }
        }
        None
    }

    fn lex_number(
        &mut self,
        start_line: usize,
        start_col: usize,
        first: char,
    ) -> Result<Token, GambaError> {
        let mut s = String::new();
        s.push(first);
        let mut has_dot = first == '.';
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.bump();
            } else if c == '.' && !has_dot {
                has_dot = true;
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        match s.parse::<f64>() {
            Ok(n) => Ok(Token {
                kind: TokenKind::Number(n),
                line: start_line,
                col: start_col,
            }),
            Err(_) => Err(GambaError::lex("número inválido", start_line, start_col)),
        }
    }

    fn lex_ident(&mut self, start_line: usize, start_col: usize, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek() {
            // unicode friendly: letras e digitos de verdade, e _
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        let kind = match s.as_str() {
            "fn" => TokenKind::Fn,
            "when" => TokenKind::When,
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _ => TokenKind::Ident(s),
        };
        Token {
            kind,
            line: start_line,
            col: start_col,
        }
    }

    fn lex_string(&mut self, start_line: usize, start_col: usize) -> Result<Token, GambaError> {
        let mut s = String::new();
        while let Some(c) = self.bump() {
            match c {
                '"' => {
                    return Ok(Token {
                        kind: TokenKind::String(s),
                        line: start_line,
                        col: start_col,
                    })
                }
                '\\' => {
                    if let Some(esc) = self.bump() {
                        match esc {
                            'n' => s.push('\n'),
                            't' => s.push('\t'),
                            'r' => s.push('\r'),
                            '\\' => s.push('\\'),
                            '"' => s.push('"'),
                            other => {
                                s.push('\\');
                                s.push(other);
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ => s.push(c),
            }
        }
        Err(GambaError::lex(
            "string não finalizada",
            start_line,
            start_col,
        ))
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, GambaError> {
        let mut tokens = Vec::new();
        while let Some(_) = self.peek() {
            if let Some(nl) = self.skip_spaces() {
                tokens.push(nl);
                continue;
            }
            let line = self.line;
            let col = self.col;
            let Some(ch) = self.bump() else {
                break;
            };
            let tk = match ch {
                '(' => self.make_token(TokenKind::LParen),
                ')' => self.make_token(TokenKind::RParen),
                '{' => self.make_token(TokenKind::LBrace),
                '}' => self.make_token(TokenKind::RBrace),
                '[' => self.make_token(TokenKind::LBracket),
                ']' => self.make_token(TokenKind::RBracket),
                ',' => self.make_token(TokenKind::Comma),
                '+' => self.make_token(TokenKind::Plus),
                '-' => self.make_token(TokenKind::Minus),
                '*' => self.make_token(TokenKind::Star),
                '/' => self.make_token(TokenKind::Slash),
                '%' => self.make_token(TokenKind::Percent),
                ':' if self.peek() == Some(':') => {
                    self.bump();
                    self.make_token(TokenKind::DoubleColon)
                }
                '|' if self.peek() == Some('>') => {
                    self.bump();
                    self.make_token(TokenKind::PipeOp)
                }
                '"' => {
                    tokens.push(self.lex_string(line, col)?);
                    continue;
                }
                c if c.is_ascii_digit() => {
                    tokens.push(self.lex_number(line, col, c)?);
                    continue;
                }
                c if c.is_alphabetic() || c == '_' => {
                    tokens.push(self.lex_ident(line, col, c));
                    continue;
                }
                '\n' => self.make_token(TokenKind::Newline),
                _ => {
                    return Err(GambaError::lex(
                        format!("caractere inesperado: '{}'", ch),
                        line,
                        col,
                    ))
                }
            };
            tokens.push(tk);
        }
        tokens.push(Token {
            kind: TokenKind::Eof,
            line: self.line,
            col: self.col,
        });
        Ok(tokens)
    }
}
