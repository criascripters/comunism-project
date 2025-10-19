//! gamba: linguagem funcional com pipes e I/O pra CLIs
//!
//! use `eval_str` para rodar um código fonte Gamba e obter um Value

pub mod ast;
pub mod builtins;
pub mod env;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;

pub use crate::env::{Runtime, Value};
pub use crate::error::GambaError;

/// avalia um código fonte Gamba em um runtime novo com builtins carregados
/// retorna o valor da ultima expressão do programa
pub fn eval_str(src: &str) -> Result<Value, GambaError> {
    let mut lx = lexer::Lexer::new(src);
    let tokens = lx.tokenize()?;
    let mut p = parser::Parser::new(tokens);
    let program = p.parse_program()?;
    let mut rt = Runtime::with_builtins();
    interpreter::eval_program(&mut rt, program)
}

/// cria um runtime reutilizável (mantem estado de nomes definidos) com builtins e
/// avalia o código no mesmo escopo (util para REPL)
pub fn eval_in_runtime(rt: &mut Runtime, src: &str) -> Result<Value, GambaError> {
    let mut lx = lexer::Lexer::new(src);
    let tokens = lx.tokenize()?;
    let mut p = parser::Parser::new(tokens);
    let program = p.parse_program()?;
    interpreter::eval_program(rt, program)
}
