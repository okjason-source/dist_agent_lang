use std::process;
use dist_agent_lang::lexer::Lexer;
use dist_agent_lang::parser::Parser;

fn test_code(name: &str, source: &str) {
    println!("Testing: {}", name);
    println!("Source: {}", source);
    
    let lexer = Lexer::new(source);
    match lexer.tokenize() {
        Ok(tokens) => {
            println!("✅ Tokenization successful! {} tokens", tokens.len());
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(ast) => {
                    println!("✅ Parsing successful! {} statements", ast.statements.len());
                }
                Err(e) => {
                    println!("❌ Parsing failed: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            println!("❌ Tokenization failed: {}", e);
            process::exit(1);
        }
    }
    println!();
}

fn main() {
    // Test simple expressions
    test_code("simple_expressions", "let x = 42 + 10 * 2; let y = (x + 5) / 3;");
    
    // Test function definitions
    test_code("function_definitions", r#"
        fn add(a: int, b: int) -> int {
            return a + b;
        }
        fn multiply(x: int, y: int) -> int {
            return x * y;
        }
    "#);
    
    // Test complex statements
    test_code("complex_statements", r#"
        @txn @secure
        fn process_data(data: string) -> bool {
            let mut result = 0;
            if data.len() > 10 {
                result = data.len() * 2;
            } else {
                result = data.len() + 5;
            }
            return result > 20;
        }
    "#);
    
    println!("All tests passed!");
}
