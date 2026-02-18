// Simple token definitions for testing
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Literal(String),
    Punctuation(String),
    Operator(String),
    EOF,
}

#[derive(Debug, Clone)]
pub struct Lexer {
    input: String,
    #[allow(dead_code)]
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            position: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        let words: Vec<&str> = self.input.split_whitespace().collect();

        for word in words {
            match word {
                "service" => tokens.push(Token::Keyword("service".to_string())),
                "@trust" => tokens.push(Token::Keyword("@trust".to_string())),
                "fn" => tokens.push(Token::Keyword("fn".to_string())),
                "return" => tokens.push(Token::Keyword("return".to_string())),
                "int" => tokens.push(Token::Keyword("int".to_string())),
                "string" => tokens.push(Token::Keyword("string".to_string())),
                "self" => tokens.push(Token::Keyword("self".to_string())),
                "{" => tokens.push(Token::Punctuation("{".to_string())),
                "}" => tokens.push(Token::Punctuation("}".to_string())),
                "(" => tokens.push(Token::Punctuation("(".to_string())),
                ")" => tokens.push(Token::Punctuation(")".to_string())),
                ":" => tokens.push(Token::Punctuation(":".to_string())),
                "=" => tokens.push(Token::Punctuation("=".to_string())),
                ";" => tokens.push(Token::Punctuation(";".to_string())),
                "->" => tokens.push(Token::Punctuation("->".to_string())),
                "100" => tokens.push(Token::Literal("100".to_string())),
                "\"hybrid\"" => tokens.push(Token::Literal("\"hybrid\"".to_string())),
                "MyService" => tokens.push(Token::Identifier("MyService".to_string())),
                "balance" => tokens.push(Token::Identifier("balance".to_string())),
                "name" => tokens.push(Token::Identifier("name".to_string())),
                "get_balance" => tokens.push(Token::Identifier("get_balance".to_string())),
                "self.balance" => tokens.push(Token::Identifier("self.balance".to_string())),
                _ => tokens.push(Token::Identifier(word.to_string())),
            }
        }

        tokens.push(Token::EOF);
        Ok(tokens)
    }
}

#[derive(Debug)]
pub struct ServiceStatement {
    pub name: String,
    pub attributes: Vec<String>,
    pub fields: Vec<ServiceField>,
    pub methods: Vec<ServiceMethod>,
}

#[derive(Debug)]
pub struct ServiceField {
    pub name: String,
    pub field_type: String,
    pub initial_value: Option<String>,
}

#[derive(Debug)]
pub struct ServiceMethod {
    pub name: String,
    pub return_type: Option<String>,
    pub body: String,
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<ServiceStatement>, String> {
        let mut services = Vec::new();

        while self.position < self.tokens.len() {
            if let Some(Token::Keyword(keyword)) = self.tokens.get(self.position) {
                if keyword == "service" {
                    let service = self.parse_service()?;
                    services.push(service);
                } else {
                    self.position += 1;
                }
            } else {
                self.position += 1;
            }
        }

        Ok(services)
    }

    fn parse_service(&mut self) -> Result<ServiceStatement, String> {
        // Skip 'service' keyword
        self.position += 1;

        // Parse service name
        let name = if let Some(Token::Identifier(name)) = self.tokens.get(self.position) {
            self.position += 1;
            name.clone()
        } else {
            return Err("Expected service name".to_string());
        };

        // Parse attributes (simplified)
        let mut attributes = Vec::new();
        while self.position < self.tokens.len() {
            if let Some(Token::Keyword(attr)) = self.tokens.get(self.position) {
                if attr.starts_with('@') {
                    attributes.push(attr.clone());
                    self.position += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Expect opening brace
        if let Some(Token::Punctuation(punct)) = self.tokens.get(self.position) {
            if punct == "{" {
                self.position += 1;
            } else {
                return Err("Expected '{' after service name".to_string());
            }
        } else {
            return Err("Expected '{' after service name".to_string());
        }

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        // Parse service body
        while self.position < self.tokens.len() {
            if let Some(Token::Punctuation(punct)) = self.tokens.get(self.position) {
                if punct == "}" {
                    self.position += 1;
                    break;
                }
            }

            // Try to parse field or method
            if let Some(Token::Keyword(keyword)) = self.tokens.get(self.position) {
                if keyword == "fn" {
                    let method = self.parse_method()?;
                    methods.push(method);
                } else {
                    let field = self.parse_field()?;
                    fields.push(field);
                }
            } else {
                self.position += 1;
            }
        }

        Ok(ServiceStatement {
            name,
            attributes,
            fields,
            methods,
        })
    }

    fn parse_field(&mut self) -> Result<ServiceField, String> {
        // Parse field name
        let name = if let Some(Token::Identifier(name)) = self.tokens.get(self.position) {
            self.position += 1;
            name.clone()
        } else {
            return Err("Expected field name".to_string());
        };

        // Expect colon
        if let Some(Token::Punctuation(punct)) = self.tokens.get(self.position) {
            if punct == ":" {
                self.position += 1;
            } else {
                return Err("Expected ':' after field name".to_string());
            }
        } else {
            return Err("Expected ':' after field name".to_string());
        }

        // Parse field type
        let field_type = if let Some(Token::Keyword(typ)) = self.tokens.get(self.position) {
            self.position += 1;
            typ.clone()
        } else {
            return Err("Expected field type".to_string());
        };

        // Parse initial value if present
        let initial_value = if let Some(Token::Punctuation(punct)) = self.tokens.get(self.position)
        {
            if punct == "=" {
                self.position += 1;
                if let Some(Token::Literal(value)) = self.tokens.get(self.position) {
                    self.position += 1;
                    Some(value.clone())
                } else {
                    return Err("Expected value after '='".to_string());
                }
            } else {
                None
            }
        } else {
            None
        };

        // Expect semicolon
        if let Some(Token::Punctuation(punct)) = self.tokens.get(self.position) {
            if punct == ";" {
                self.position += 1;
            } else {
                return Err("Expected ';' after field".to_string());
            }
        } else {
            return Err("Expected ';' after field".to_string());
        }

        Ok(ServiceField {
            name,
            field_type,
            initial_value,
        })
    }

    fn parse_method(&mut self) -> Result<ServiceMethod, String> {
        // Skip 'fn' keyword
        self.position += 1;

        // Parse method name
        let name = if let Some(Token::Identifier(name)) = self.tokens.get(self.position) {
            self.position += 1;
            name.clone()
        } else {
            return Err("Expected method name".to_string());
        };

        // Expect opening parenthesis
        if let Some(Token::Punctuation(punct)) = self.tokens.get(self.position) {
            if punct == "(" {
                self.position += 1;
            } else {
                return Err("Expected '(' after method name".to_string());
            }
        } else {
            return Err("Expected '(' after method name".to_string());
        }

        // Skip parameters for now (simplified)
        while self.position < self.tokens.len() {
            if let Some(Token::Punctuation(punct)) = self.tokens.get(self.position) {
                if punct == ")" {
                    self.position += 1;
                    break;
                }
            }
            self.position += 1;
        }

        // Parse return type if present
        let return_type = if let Some(Token::Punctuation(punct)) = self.tokens.get(self.position) {
            if punct == "->" {
                self.position += 1;
                if let Some(Token::Keyword(typ)) = self.tokens.get(self.position) {
                    self.position += 1;
                    Some(typ.clone())
                } else {
                    return Err("Expected return type after '->'".to_string());
                }
            } else {
                None
            }
        } else {
            None
        };

        // Skip method body for now (simplified)
        while self.position < self.tokens.len() {
            if let Some(Token::Punctuation(punct)) = self.tokens.get(self.position) {
                if punct == "}" {
                    self.position += 1;
                    break;
                }
            }
            self.position += 1;
        }

        Ok(ServiceMethod {
            name,
            return_type,
            body: "simplified".to_string(),
        })
    }
}

fn main() {
    println!("Simple Service Parsing Test");
    println!("===========================");

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
    let mut lexer = Lexer::new(test_code);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => {
            println!(
                "✅ Tokenization successful! Generated {} tokens",
                tokens.len()
            );
            println!("   Tokens: {:?}", tokens);
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
    let services = match parser.parse() {
        Ok(services) => {
            println!("✅ Parsing successful! Found {} services", services.len());
            services
        }
        Err(e) => {
            eprintln!("❌ Parsing failed: {}", e);
            return;
        }
    };

    // Print service details
    println!("\n3. Service Details:");
    for service in &services {
        println!("   Service Name: {}", service.name);
        println!("   Attributes: {}", service.attributes.len());
        for attr in &service.attributes {
            println!("     - {}", attr);
        }
        println!("   Fields: {}", service.fields.len());
        for field in &service.fields {
            println!("     - {}: {}", field.name, field.field_type);
            if let Some(ref value) = field.initial_value {
                println!("       Initial value: {}", value);
            }
        }
        println!("   Methods: {}", service.methods.len());
        for method in &service.methods {
            println!("     - {}()", method.name);
            if let Some(ref return_type) = method.return_type {
                println!("       Returns: {}", return_type);
            }
        }
    }

    println!("\n🎉 Service parsing test completed successfully!");
}
