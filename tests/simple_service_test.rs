use dist_agent_lang::lexer::Lexer;
use dist_agent_lang::parser::Parser;

fn main() {
    println!("Simple Service Parsing Test");
    println!("===========================");

    // Test service parsing
    let test_code = r#"
        @trust("hybrid")
        service MyService {
            balance: int = 100,
            name: string,
            
            fn get_balance() -> int {
                return self.balance;
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
            println!(
                "   First 10 tokens: {:?}",
                &tokens[..std::cmp::min(10, tokens.len())]
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
                println!("   Attributes: {}", service.attributes.len());
                for attr in &service.attributes {
                    println!("     - @{}", attr.name);
                }
                println!("   Fields: {}", service.fields.len());
                for field in &service.fields {
                    println!("     - {}: {}", field.name, field.field_type);
                }
                println!("   Methods: {}", service.methods.len());
                for method in &service.methods {
                    println!("     - {}()", method.name);
                }
            }
        }
    } else {
        println!("❌ No service statement found!");
        println!("   Available statement types:");
        for stmt in &program.statements {
            match stmt {
                dist_agent_lang::parser::ast::Statement::Expression(_) => {
                    println!("     - Expression")
                }
                dist_agent_lang::parser::ast::Statement::Let(_) => println!("     - Let"),
                dist_agent_lang::parser::ast::Statement::Return(_) => println!("     - Return"),
                dist_agent_lang::parser::ast::Statement::Block(_) => println!("     - Block"),
                dist_agent_lang::parser::ast::Statement::Function(_) => println!("     - Function"),
                dist_agent_lang::parser::ast::Statement::Service(_) => println!("     - Service"),
                dist_agent_lang::parser::ast::Statement::Spawn(_) => println!("     - Spawn"),
                dist_agent_lang::parser::ast::Statement::Agent(_) => println!("     - Agent"),
                dist_agent_lang::parser::ast::Statement::Message(_) => println!("     - Message"),
                dist_agent_lang::parser::ast::Statement::Event(_) => println!("     - Event"),
                dist_agent_lang::parser::ast::Statement::If(_) => println!("     - If"),
                dist_agent_lang::parser::ast::Statement::While(_) => println!("     - While"),
                dist_agent_lang::parser::ast::Statement::ForIn(_) => println!("     - ForIn"),
                dist_agent_lang::parser::ast::Statement::Try(_) => println!("     - Try"),
                dist_agent_lang::parser::ast::Statement::Break(_) => println!("     - Break"),
                dist_agent_lang::parser::ast::Statement::Continue(_) => println!("     - Continue"),
                dist_agent_lang::parser::ast::Statement::Loop(_) => println!("     - Loop"),
                dist_agent_lang::parser::ast::Statement::Match(_) => println!("     - Match"),
                dist_agent_lang::parser::ast::Statement::Import(_) => println!("     - Import"),
            }
        }
    }

    println!("\n🎉 Service parsing test completed!");
}
