use dist_agent_lang::lexer::Lexer;
use dist_agent_lang::parser::Parser;
use dist_agent_lang::runtime::Runtime;

fn main() {
    println!("Testing Service Statement Parsing");
    println!("==================================");

    // Test 1: Basic service with fields and methods
    let test_code = r#"
        @trust("hybrid")
        @secure
        service MyService {
            balance: int = 100,
            name: string,
            
            fn get_balance() -> int {
                return self.balance;
            }
            
            fn set_balance(new_balance: int) {
                self.balance = new_balance;
            }
        }
    "#;

    println!("Test Code:");
    println!("{}", test_code);
    println!();

    // Tokenize
    println!("1. Tokenizing...");
    let tokens = match Lexer::new(test_code).tokenize() {
        Ok(tokens) => {
            println!(
                "✅ Tokenization successful! Generated {} tokens",
                tokens.len()
            );
            tokens
        }
        Err(e) => {
            eprintln!("❌ Tokenization failed: {}", e);
            return;
        }
    };

    // Parse
    println!("\n2. Parsing...");
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(program) => {
            println!(
                "✅ Parsing successful! Parsed {} statements",
                program.statements.len()
            );
            program
        }
        Err(e) => {
            eprintln!("❌ Parsing failed: {}", e);
            return;
        }
    };

    // Check for service statement
    println!("\n3. Checking for service statement...");
    let has_service = program
        .statements
        .iter()
        .any(|stmt| matches!(stmt, dist_agent_lang::parser::ast::Statement::Service(_)));

    if has_service {
        println!("✅ Service statement found!");

        // Print service details
        for stmt in &program.statements {
            if let dist_agent_lang::parser::ast::Statement::Service(service) = stmt {
                println!("   Service Name: {}", service.name);
                println!("   Attributes: {:?}", service.attributes);
                println!("   Fields: {}", service.fields.len());
                for field in &service.fields {
                    println!("     - {}: {}", field.name, field.field_type);
                }
                println!("   Methods: {}", service.methods.len());
                for method in &service.methods {
                    println!("     - {}()", method.name);
                }
                println!("   Events: {}", service.events.len());
            }
        }
    } else {
        println!("❌ No service statement found!");
    }

    // Execute (if parsing was successful)
    println!("\n4. Executing...");
    let mut runtime = Runtime::new();
    match runtime.execute_program(program) {
        Ok(result) => {
            println!("✅ Execution successful!");
            println!("   Result: {:?}", result);
        }
        Err(e) => {
            println!("❌ Execution failed: {}", e);
        }
    }

    println!("\n🎉 Service parsing test completed!");
}
