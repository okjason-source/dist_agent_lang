// dist_agent_lang library
// This file makes the project available as a library

pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod stdlib;
pub mod testing;
pub mod performance;
pub mod http_server;

// Re-export main components for easy access
pub use lexer::{Lexer, tokens::Token};
pub use parser::{Parser, ast, error::ParserError};
pub use runtime::{Runtime, values::Value};

// For external integrations
pub fn parse_source(source: &str) -> Result<ast::Program, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    let mut parser = Parser::new(tokens);
    parser.parse().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

pub fn execute_source(source: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let program = parse_source(source)?;
    let mut runtime = Runtime::new();
    let result = runtime.execute_program(program).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(result.unwrap_or(Value::Null))
}
