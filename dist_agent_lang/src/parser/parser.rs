use crate::lexer::tokens::{Token, Keyword, Operator, Punctuation, Literal};
use crate::parser::ast::{
    Expression, Statement, Program, LetStatement, ReturnStatement, FunctionStatement, 
    Parameter, FunctionCall, SpawnStatement, AgentStatement, MessageStatement, 
    EventStatement, IfStatement, TryStatement, CatchBlock, Attribute, BlockStatement,
    AttributeTarget, ServiceStatement, ServiceField, EventDeclaration, CompilationTargetInfo,
    FieldVisibility
};
use crate::parser::error::{ParserError, ErrorContext};
use std::collections::HashMap;

pub struct Parser {
    tokens: Vec<Token>,
    token_positions: Vec<(usize, usize)>, // (line, column) for each token
}

impl Parser {
    const MAX_RECURSION_DEPTH: usize = 100;
    
    pub fn new(tokens: Vec<Token>) -> Self {
        // For backward compatibility, create empty positions
        // In the future, we should pass tokens with positions
        Self { 
            tokens,
            token_positions: Vec::new(),
        }
    }

    pub fn new_with_positions(tokens_with_pos: Vec<crate::lexer::tokens::TokenWithPosition>) -> Self {
        let mut tokens = Vec::new();
        let mut positions = Vec::new();
        for twp in tokens_with_pos {
            positions.push((twp.line, twp.column));
            tokens.push(twp.token);
        }
        Self { tokens, token_positions: positions }
    }

    fn get_token_position(&self, position: usize) -> (usize, usize) {
        if position < self.token_positions.len() {
            self.token_positions[position]
        } else {
            // Fallback: try to calculate from token index
            // This is a rough estimate
            (1, 1)
        }
    }

    fn error_unexpected_token(&self, position: usize, expected: &[&str]) -> ParserError {
        let token = self.tokens.get(position).unwrap_or(&Token::EOF);
        let (line, column) = self.get_token_position(position);
        ParserError::unexpected_token(token, expected, line, column)
    }

    pub fn parse(&mut self) -> Result<Program, ParserError> {
        let mut program = Program::new();
        let mut position = 0;
        
        while position < self.tokens.len() {
            let (new_position, statement) = self.parse_statement(position, 0)?;
            program.add_statement(statement);
            position = new_position;
        }
        
        Ok(program)
    }

    fn parse_statement(&mut self, position: usize, depth: usize) -> Result<(usize, Statement), ParserError> {
        if depth > Self::MAX_RECURSION_DEPTH {
            let (line, column) = self.get_token_position(position);
            return Err(ParserError::SemanticError {
                message: format!("Maximum recursion depth ({}) exceeded in statement parsing", Self::MAX_RECURSION_DEPTH),
                line,
                context: ErrorContext::new(),
            });
        }
        // Check for attributes first
        if let Some(Token::Punctuation(Punctuation::At)) = self.tokens.get(position) {
            // Attributes can appear before function or service declarations
            let mut current_position = position;
            let mut attributes = Vec::new();
            
            // Collect all attributes
            while current_position < self.tokens.len() {
                if let Some(Token::Punctuation(Punctuation::At)) = self.tokens.get(current_position) {
                    let (new_pos, attr) = self.parse_attribute(current_position)?;
                    attributes.push(attr);
                    current_position = new_pos;
                } else {
                    // No more attributes, break
                    break;
                }
            }
            
            // After attributes, we must have a function declaration (async or regular) or service declaration
            if let Some(Token::Keyword(Keyword::Fn)) = self.tokens.get(current_position) {
                let (new_pos, mut func_stmt) = self.parse_function_statement(current_position)?;
                // Set the collected attributes
                if let Statement::Function(ref mut func) = func_stmt {
                    func.attributes = attributes;
                }
                return Ok((new_pos, func_stmt));
            } else if let Some(Token::Keyword(Keyword::Async)) = self.tokens.get(current_position) {
                let (new_pos, mut func_stmt) = self.parse_async_function_statement(current_position)?;
                // Set the collected attributes
                if let Statement::Function(ref mut func) = func_stmt {
                    func.attributes = attributes;
                }
                return Ok((new_pos, func_stmt));
            } else if let Some(Token::Keyword(Keyword::Service)) = self.tokens.get(current_position) {
                let (new_pos, service_stmt) = self.parse_service_statement_with_attributes(current_position, attributes)?;
                return Ok((new_pos, service_stmt));
            } else {
                // Get the actual token for better error reporting
                let expected = vec!["function declaration", "service declaration"];
                return Err(self.error_unexpected_token(current_position, &expected));
            }
        }
        
        if let Some(Token::Keyword(Keyword::Let)) = self.tokens.get(position) {
            self.parse_let_statement(position)
        } else if let Some(Token::Keyword(Keyword::Fn)) = self.tokens.get(position) {
            self.parse_function_statement(position)
        } else if let Some(Token::Keyword(Keyword::Spawn)) = self.tokens.get(position) {
            self.parse_spawn_statement(position)
        } else if let Some(Token::Keyword(Keyword::Agent)) = self.tokens.get(position) {
            self.parse_agent_statement(position)
        } else if let Some(Token::Keyword(Keyword::Msg)) = self.tokens.get(position) {
            self.parse_message_statement(position)
        } else if let Some(Token::Keyword(Keyword::Event)) = self.tokens.get(position) {
            self.parse_event_statement(position)
        } else if let Some(Token::Keyword(Keyword::If)) = self.tokens.get(position) {
            self.parse_if_statement(position)
        } else if let Some(Token::Keyword(Keyword::Try)) = self.tokens.get(position) {
            self.parse_try_statement(position)
        } else if let Some(Token::Keyword(Keyword::Service)) = self.tokens.get(position) {
            self.parse_service_statement(position)
        } else if let Some(Token::Keyword(Keyword::Return)) = self.tokens.get(position) {
            self.parse_return_statement(position)
        } else if let Some(Token::Punctuation(Punctuation::LeftBrace)) = self.tokens.get(position) {
            let (new_position, block) = self.parse_block_statement(position, depth + 1)?;
            Ok((new_position, Statement::Block(block)))
        } else if let Some(Token::Punctuation(Punctuation::Semicolon)) = self.tokens.get(position) {
            // Skip semicolons (empty statements)
            Ok((position + 1, Statement::Expression(Expression::Literal(Literal::Null))))
        } else if let Some(Token::EOF) = self.tokens.get(position) {
            // Skip EOF tokens
            Ok((position + 1, Statement::Expression(Expression::Literal(Literal::Null))))
        } else {
            // Before falling through to expression parsing, check if this might be a service declaration
            // that wasn't caught earlier (shouldn't happen, but defensive check)
            if let Some(Token::Keyword(Keyword::Service)) = self.tokens.get(position) {
                // This should have been caught at line 92, but if we get here, try parsing as service
                return self.parse_service_statement(position);
            }
            let (new_position, expr) = self.parse_expression(position)?;
            Ok((new_position, Statement::Expression(expr)))
        }
    }

    fn parse_let_statement(&self, position: usize) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'let'
        
        // Check for 'mut' keyword
        let _is_mutable = if let Some(Token::Keyword(Keyword::Mut)) = self.tokens.get(current_position) {
            current_position += 1;
            true
        } else {
            false
        };
        
        // Use expect_identifier_or_keyword to allow keywords as variable names (e.g., "agent", "ai", "chain")
        let (new_position, name) = self.expect_identifier_or_keyword(current_position)?;
        current_position = new_position;
        
        let (new_position, _) = self.expect_token(current_position, &Token::Operator(Operator::Assign))?;
        current_position = new_position;
        
        let (new_position, value) = self.parse_expression(current_position)?;
        current_position = new_position;
        
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Semicolon))?;
        current_position = new_position;
        
        Ok((current_position, Statement::Let(LetStatement { name, value })))
    }

    fn parse_return_statement(&self, position: usize) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'return'
        
        let value = if let Some(Token::Punctuation(Punctuation::Semicolon)) = self.tokens.get(current_position) {
            None
        } else {
            let (new_position, expr) = self.parse_expression(current_position)?;
            current_position = new_position;
            Some(expr)
        };
        
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Semicolon))?;
        Ok((new_position, Statement::Return(ReturnStatement { value })))
    }

    fn parse_block_statement(&mut self, position: usize, depth: usize) -> Result<(usize, BlockStatement), ParserError> {
        if depth > Self::MAX_RECURSION_DEPTH {
            let (line, column) = self.get_token_position(position);
            return Err(ParserError::SemanticError {
                message: format!("Maximum recursion depth ({}) exceeded in block statement parsing", Self::MAX_RECURSION_DEPTH),
                line,
                context: ErrorContext::new(),
            });
        }
        
        let (position, _) = self.expect_token(position, &Token::Punctuation(Punctuation::LeftBrace))?;
        
        let mut block = BlockStatement::new();
        let mut current_position = position;
        
        while current_position < self.tokens.len() {
            if let Some(Token::Punctuation(Punctuation::RightBrace)) = self.tokens.get(current_position) {
                let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::RightBrace))?;
                return Ok((new_position, block));
            }
            
            let (new_position, statement) = self.parse_statement(current_position, depth + 1)?;
            block.add_statement(statement);
            current_position = new_position;
        }
        
        Err(ParserError::unexpected_eof("}"))
    }

    fn parse_expression(&self, position: usize) -> Result<(usize, Expression), ParserError> {
        self.parse_assignment(position, 0) // Start with depth 0 for top-level expressions
    }
    
    fn parse_expression_with_depth(&self, position: usize, depth: usize) -> Result<(usize, Expression), ParserError> {
        self.parse_assignment(position, depth)
    }

    fn parse_assignment(&self, position: usize, depth: usize) -> Result<(usize, Expression), ParserError> {
        // Prevent stack overflow from infinite recursion
        if depth > Self::MAX_RECURSION_DEPTH {
            let (line, column) = self.get_token_position(position);
            return Err(ParserError::SemanticError {
                message: format!(
                    "Maximum recursion depth ({}) exceeded at line {}, column {}. This may indicate malformed input or a parser bug.",
                    Self::MAX_RECURSION_DEPTH, line, column
                ),
                line,
                context: ErrorContext::new(),
            });
        }
        
        let (position, expr) = self.parse_or(position, depth)?;
        
        if let Some(Token::Operator(Operator::Assign)) = self.tokens.get(position) {
            let (position, _) = self.expect_token(position, &Token::Operator(Operator::Assign))?;
            let (position, value) = self.parse_assignment(position, depth + 1)?;
            
            match expr {
                Expression::Identifier(name) => {
                    return Ok((position, Expression::Assignment(name, Box::new(value))));
                }
                Expression::FieldAccess(object_expr, field_name) => {
                    return Ok((position, Expression::FieldAssignment(object_expr, field_name, Box::new(value))));
                }
                Expression::FunctionCall(call) if call.name == "__index__" && call.arguments.len() == 2 => {
                    // Array assignment: arr[index] = value
                    // Represent as a function call: __index_assign__(arr, index, value)
                    return Ok((position, Expression::FunctionCall(FunctionCall {
                        name: "__index_assign__".to_string(),
                        arguments: vec![call.arguments[0].clone(), call.arguments[1].clone(), value],
                    })));
                }
                _ => {
                    return Err(ParserError::invalid_function_call(
                        "assignment",
                        "Invalid assignment target - expected identifier, field access, or array access",
                        1
                    ));
                }
            }
        }
        
        Ok((position, expr))
    }

    fn parse_or(&self, position: usize, depth: usize) -> Result<(usize, Expression), ParserError> {
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_and(current_position, depth)?;
        current_position = new_position;
        
        while current_position < self.tokens.len() {
            if let Some(Token::Operator(Operator::Or)) = self.tokens.get(current_position) {
                let (new_pos, _) = self.expect_token(current_position, &Token::Operator(Operator::Or))?;
                let (new_pos, right) = self.parse_and(new_pos, depth)?;
                expr = Expression::BinaryOp(Box::new(expr), Operator::Or, Box::new(right));
                current_position = new_pos;
            } else {
                break;
            }
        }
        
        Ok((current_position, expr))
    }

    fn parse_and(&self, position: usize, depth: usize) -> Result<(usize, Expression), ParserError> {
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_equality(current_position, depth)?;
        current_position = new_position;
        
        while current_position < self.tokens.len() {
            if let Some(Token::Operator(Operator::And)) = self.tokens.get(current_position) {
                let (new_pos, _) = self.expect_token(current_position, &Token::Operator(Operator::And))?;
                let (new_pos, right) = self.parse_equality(new_pos, depth)?;
                expr = Expression::BinaryOp(Box::new(expr), Operator::And, Box::new(right));
                current_position = new_pos;
            } else {
                break;
            }
        }
        
        Ok((current_position, expr))
    }

    fn parse_equality(&self, position: usize, depth: usize) -> Result<(usize, Expression), ParserError> {
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_comparison(current_position, depth)?;
        current_position = new_position;
        
        while current_position < self.tokens.len() {
            if let Some(Token::Operator(Operator::Equal)) = self.tokens.get(current_position) {
                let (new_pos, _) = self.expect_token(current_position, &Token::Operator(Operator::Equal))?;
                let (new_pos, right) = self.parse_comparison(new_pos, depth)?;
                expr = Expression::BinaryOp(Box::new(expr), Operator::Equal, Box::new(right));
                current_position = new_pos;
            } else if let Some(Token::Operator(Operator::NotEqual)) = self.tokens.get(current_position) {
                let (new_pos, _) = self.expect_token(current_position, &Token::Operator(Operator::NotEqual))?;
                let (new_pos, right) = self.parse_comparison(new_pos, depth)?;
                expr = Expression::BinaryOp(Box::new(expr), Operator::NotEqual, Box::new(right));
                current_position = new_pos;
            } else {
                break;
            }
        }
        
        Ok((current_position, expr))
    }

    fn parse_comparison(&self, position: usize, depth: usize) -> Result<(usize, Expression), ParserError> {
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_term(current_position, depth)?;
        current_position = new_position;
        
        while current_position < self.tokens.len() {
            if let Some(Token::Operator(op)) = self.tokens.get(current_position) {
                match op {
                    Operator::Less | Operator::LessEqual | Operator::Greater | Operator::GreaterEqual => {
                        let (new_pos, _) = self.expect_token(current_position, &Token::Operator(op.clone()))?;
                        let (new_pos, right) = self.parse_term(new_pos, depth)?;
                        expr = Expression::BinaryOp(Box::new(expr), op.clone(), Box::new(right));
                        current_position = new_pos;
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }
        
        Ok((current_position, expr))
    }

    fn parse_term(&self, position: usize, depth: usize) -> Result<(usize, Expression), ParserError> {
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_factor(current_position, depth)?;
        current_position = new_position;
        
        while current_position < self.tokens.len() {
            if let Some(Token::Operator(op)) = self.tokens.get(current_position) {
                match op {
                    Operator::Plus | Operator::Minus => {
                        let (new_pos, _) = self.expect_token(current_position, &Token::Operator(op.clone()))?;
                        let (new_pos, right) = self.parse_factor(new_pos, depth)?;
                        expr = Expression::BinaryOp(Box::new(expr), op.clone(), Box::new(right));
                        current_position = new_pos;
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }
        
        Ok((current_position, expr))
    }

    fn parse_factor(&self, position: usize, depth: usize) -> Result<(usize, Expression), ParserError> {
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_unary(current_position, depth)?;
        current_position = new_position;
        
        while current_position < self.tokens.len() {
            if let Some(Token::Operator(op)) = self.tokens.get(current_position) {
                match op {
                    Operator::Star | Operator::Slash => {
                        let (new_pos, _) = self.expect_token(current_position, &Token::Operator(op.clone()))?;
                        let (new_pos, right) = self.parse_unary(new_pos, depth)?;
                        expr = Expression::BinaryOp(Box::new(expr), op.clone(), Box::new(right));
                        current_position = new_pos;
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }
        
        Ok((current_position, expr))
    }

    fn parse_unary(&self, position: usize, depth: usize) -> Result<(usize, Expression), ParserError> {
        if let Some(Token::Operator(op)) = self.tokens.get(position) {
            match op {
                Operator::Minus | Operator::Not => {
                    let (position, _) = self.expect_token(position, &Token::Operator(op.clone()))?;
                    let (position, right) = self.parse_unary(position, depth)?;
                    return Ok((position, Expression::UnaryOp(op.clone(), Box::new(right))));
                }
                _ => {}
            }
        }
        
        // Parse primary expression, then handle postfix operations (array access, field access)
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_primary(current_position, depth)?;
        current_position = new_position;
        
        // Handle postfix operations: array access [index] and chained field access .field
        while current_position < self.tokens.len() {
            // Array access: expr[index]
            if let Some(Token::Punctuation(Punctuation::LeftBracket)) = self.tokens.get(current_position) {
                let (new_pos, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::LeftBracket))?;
                current_position = new_pos;
                
                // Parse index expression
                let (new_pos, index_expr) = self.parse_expression_with_depth(current_position, depth + 1)?;
                current_position = new_pos;
                
                // Expect closing bracket
                let (new_pos, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::RightBracket))?;
                current_position = new_pos;
                
                // Represent array access as a function call for now
                // In the future, we could add IndexAccess to Expression enum
                expr = Expression::FunctionCall(FunctionCall {
                    name: "__index__".to_string(),
                    arguments: vec![expr, index_expr],
                });
                continue;
            }
            
            // Chained field access: expr.field (for cases like self.balances[key])
            if let Some(Token::Punctuation(Punctuation::Dot)) = self.tokens.get(current_position) {
                let (new_pos, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Dot))?;
                current_position = new_pos;
                
                let (new_pos, field_name) = self.expect_identifier_or_keyword(current_position)?;
                current_position = new_pos;
                
                // Check if this is a method call: expr.field()
                if let Some(Token::Punctuation(Punctuation::LeftParen)) = self.tokens.get(current_position) {
                    let (new_pos, arguments) = self.parse_function_arguments(current_position, depth)?;
                    current_position = new_pos;
                    // Create a function call with the field access as the name
                    expr = Expression::FunctionCall(FunctionCall {
                        name: format!("{}.{}", match &expr {
                            Expression::Identifier(name) => name.clone(),
                            Expression::FieldAccess(obj, field) => format!("{}.{}", match obj.as_ref() {
                                Expression::Identifier(n) => n.clone(),
                                _ => "self".to_string(),
                            }, field),
                            _ => "self".to_string(),
                        }, field_name),
                        arguments,
                    });
                } else {
                    expr = Expression::FieldAccess(Box::new(expr), field_name);
                }
                continue;
            }
            
            break;
        }
        
        Ok((current_position, expr))
    }

    fn parse_primary(&self, position: usize, depth: usize) -> Result<(usize, Expression), ParserError> {
        if let Some(token) = self.tokens.get(position) {
            match token {
                Token::Literal(Literal::Int(value)) => {
                    return Ok((position + 1, Expression::Literal(Literal::Int(*value))));
                }
                Token::Literal(Literal::String(value)) => {
                    return Ok((position + 1, Expression::Literal(Literal::String(value.clone()))));
                }
                Token::Literal(Literal::Bool(value)) => {
                    return Ok((position + 1, Expression::Literal(Literal::Bool(*value))));
                }
                Token::Literal(Literal::Null) => {
                    return Ok((position + 1, Expression::Literal(Literal::Null)));
                }
                Token::Identifier(name) => {
                    let namespace_name = name.clone();
                    
                    // Check if this is a namespace call (identifier::identifier)
                    if let Some(Token::Punctuation(Punctuation::DoubleColon)) = self.tokens.get(position + 1) {
                        let (new_position, _) = self.expect_token(position + 1, &Token::Punctuation(Punctuation::DoubleColon))?;
                        let (new_position, method_name) = self.expect_identifier_or_keyword(new_position)?;
                        
                        // Check if this is a function call
                        if let Some(Token::Punctuation(Punctuation::LeftParen)) = self.tokens.get(new_position) {
                            let (new_position, arguments) = self.parse_function_arguments(new_position, depth)?;
                            return Ok((new_position, Expression::FunctionCall(FunctionCall {
                                name: format!("{}::{}", namespace_name, method_name),
                                arguments,
                            })));
                        } else {
                            return Ok((new_position, Expression::Identifier(format!("{}::{}", namespace_name, method_name))));
                        }
                    }
                    // Check if this is a function call
                    else if let Some(Token::Punctuation(Punctuation::LeftParen)) = self.tokens.get(position + 1) {
                        let (new_position, arguments) = self.parse_function_arguments(position + 1, depth)?;
                        return Ok((new_position, Expression::FunctionCall(FunctionCall {
                            name: namespace_name.clone(),
                            arguments,
                        })));
                    }
                    // Check if this is a field access (identifier.field) or method call (identifier.field())
                    else if let Some(Token::Punctuation(Punctuation::Dot)) = self.tokens.get(position + 1) {
                        let (new_position, _) = self.expect_token(position + 1, &Token::Punctuation(Punctuation::Dot))?;
                        let (new_position, field_name) = self.expect_identifier_or_keyword(new_position)?;
                        // Check if this is a method call: identifier.field()
                        if let Some(Token::Punctuation(Punctuation::LeftParen)) = self.tokens.get(new_position) {
                            let (new_pos, arguments) = self.parse_function_arguments(new_position, depth)?;
                            return Ok((new_pos, Expression::FunctionCall(FunctionCall {
                                name: format!("{}.{}", namespace_name, field_name),
                                arguments,
                            })));
                        } else {
                            return Ok((new_position, Expression::FieldAccess(
                                Box::new(Expression::Identifier(namespace_name)),
                                field_name
                            )));
                        }
                    } else {
                        return Ok((position + 1, Expression::Identifier(namespace_name.clone())));
                    }
                }
                Token::Keyword(Keyword::Service) => {
                    let namespace_name = "service".to_string();

                    // Check if this is a namespace call (service::identifier)
                    if let Some(Token::Punctuation(Punctuation::DoubleColon)) = self.tokens.get(position + 1) {
                        let (new_position, _) = self.expect_token(position + 1, &Token::Punctuation(Punctuation::DoubleColon))?;
                        let (new_position, method_name) = self.expect_identifier_or_keyword(new_position)?;
                                  // Check if this is a function call
                        if let Some(Token::Punctuation(Punctuation::LeftParen)) = self.tokens.get(new_position) {
                            let (new_position, arguments) = self.parse_function_arguments(new_position, depth)?;
                            return Ok((new_position, Expression::FunctionCall(FunctionCall {
                                name: format!("{}::{}", namespace_name, method_name),
                                arguments,
                            })));
                        } else {
                            return Ok((new_position, Expression::Identifier(format!("{}::{}", namespace_name, method_name))));
                        }
                    } else {
                        return Ok((position + 1, Expression::Identifier(namespace_name.clone())));
                    }
                }
                Token::Keyword(Keyword::Ai) => {
                    let namespace_name = "ai".to_string();

                    // Check if this is a namespace call (ai::identifier)
                    if let Some(Token::Punctuation(Punctuation::DoubleColon)) = self.tokens.get(position + 1) {
                        let (new_position, _) = self.expect_token(position + 1, &Token::Punctuation(Punctuation::DoubleColon))?;
                        let (new_position, method_name) = self.expect_identifier_or_keyword(new_position)?;

                        // Check if this is a function call
                        if let Some(Token::Punctuation(Punctuation::LeftParen)) = self.tokens.get(new_position) {
                            let (new_position, arguments) = self.parse_function_arguments(new_position, depth)?;
                            return Ok((new_position, Expression::FunctionCall(FunctionCall {
                                name: format!("{}::{}", namespace_name, method_name),
                                arguments,
                            })));
                        } else {
                            return Ok((new_position, Expression::Identifier(format!("{}::{}", namespace_name, method_name))));
                        }
                    } else {
                        return Ok((position + 1, Expression::Identifier(namespace_name.clone())));
                    }
                }
                Token::Keyword(_) => {
                    // Allow keywords to be used as identifiers in expressions (e.g., "chain" as a variable name)
                    let (new_position, name) = self.expect_identifier_or_keyword(position)?;
                    return Ok((new_position, Expression::Identifier(name)));
                }

                Token::Punctuation(Punctuation::LeftParen) => {
                    let (position, _) = self.expect_token(position, &Token::Punctuation(Punctuation::LeftParen))?;
                    let (position, expr) = self.parse_expression_with_depth(position, depth + 1)?;
                    let (position, _) = self.expect_token(position, &Token::Punctuation(Punctuation::RightParen))?;
                    return Ok((position, expr));
                }
                Token::Punctuation(Punctuation::LeftBrace) => {
                    let (position, object_literal) = self.parse_object_literal(position)?;
                    return Ok((position, Expression::ObjectLiteral(object_literal)));
                }
                Token::Punctuation(Punctuation::LeftBracket) => {
                    // Array literal: [expr1, expr2, ...]
                    // Note: Array access expr[index] is handled in parse_unary postfix operations
                    let (position, array_literal) = self.parse_array_literal(position)?;
                    return Ok((position, Expression::ArrayLiteral(array_literal)));
                }
                Token::Operator(Operator::Not) => {
                    // Handle macro calls like vec!["read"]
                    let (position, _) = self.expect_token(position, &Token::Operator(Operator::Not))?;
                    
                    // Expect a left parenthesis after the !
                    let (position, _) = self.expect_token(position, &Token::Punctuation(Punctuation::LeftParen))?;
                    
                    // Parse the arguments
                    let (position, arguments) = self.parse_function_arguments(position, depth)?;
                    
                    // Expect a right parenthesis
                    let (position, _) = self.expect_token(position, &Token::Punctuation(Punctuation::RightParen))?;
                    
                    // For now, treat it as a function call with "macro" prefix
                    // In a real implementation, this would be more sophisticated
                    return Ok((position, Expression::FunctionCall(FunctionCall {
                        name: "macro".to_string(),
                        arguments,
                    })));
                }
                Token::Keyword(Keyword::Await) => {
                    let (position, _) = self.expect_token(position, &Token::Keyword(Keyword::Await))?;
                    let (position, expr) = self.parse_primary(position, depth)?;
                    return Ok((position, Expression::Await(Box::new(expr))));
                }
                Token::Keyword(Keyword::Throw) => {
                    let (position, _) = self.expect_token(position, &Token::Keyword(Keyword::Throw))?;
                    let (position, expr) = self.parse_expression_with_depth(position, depth + 1)?;
                    return Ok((position, Expression::Throw(Box::new(expr))));
                }
                _ => {}
            }
        }
        
        Err(ParserError::unexpected_token(
            self.tokens.get(position).unwrap_or(&Token::EOF),
            &["expression"],
            1,
            1
        ))
    }

    fn parse_function_statement(&mut self, position: usize) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'fn'
        
        let (new_position, name) = self.expect_identifier(current_position)?;
        current_position = new_position;
        
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::LeftParen))?;
        current_position = new_position;
        
        let (new_position, parameters) = self.parse_parameters(current_position)?;
        current_position = new_position;
        
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::RightParen))?;
        current_position = new_position;
        
        // Parse return type if present
        let return_type = if let Some(Token::Punctuation(Punctuation::Arrow)) = self.tokens.get(current_position) {
            let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Arrow))?;
            current_position = new_position;
            let (new_position, return_type) = self.parse_type_expression(current_position)?;
            current_position = new_position;
            Some(return_type)
        } else {
            None
        };
        
        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;
        
        Ok((current_position, Statement::Function(FunctionStatement {
            name,
            parameters,
            return_type,
            body,
            attributes: Vec::new(),
            is_async: false,
        })))
    }

    fn parse_async_function_statement(&mut self, position: usize) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'async'
        
        // Must be followed by 'fn'
        let (new_position, _) = self.expect_token(current_position, &Token::Keyword(Keyword::Fn))?;
        current_position = new_position;
        
        let (new_position, name) = self.expect_identifier(current_position)?;
        current_position = new_position;
        
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::LeftParen))?;
        current_position = new_position;
        
        let (new_position, parameters) = self.parse_parameters(current_position)?;
        current_position = new_position;
        
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::RightParen))?;
        current_position = new_position;
        
        // Parse return type if present
        let return_type = if let Some(Token::Punctuation(Punctuation::Arrow)) = self.tokens.get(current_position) {
            let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Arrow))?;
            current_position = new_position;
            let (new_position, return_type) = self.parse_type_expression(current_position)?;
            current_position = new_position;
            Some(return_type)
        } else {
            None
        };
        
        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;
        
        Ok((current_position, Statement::Function(FunctionStatement {
            name,
            parameters,
            return_type,
            body,
            attributes: Vec::new(),
            is_async: true,
        })))
    }

    fn parse_parameters(&self, position: usize) -> Result<(usize, Vec<Parameter>), ParserError> {
        let mut current_position = position;
        let mut parameters = Vec::new();
        
        // Check if parameters list is empty
        if let Some(Token::Punctuation(Punctuation::RightParen)) = self.tokens.get(current_position) {
            return Ok((current_position, parameters));
        }
        
        loop {
            // Allow keywords as parameter names (e.g., "chain" can be a parameter name)
            let (new_position, name) = self.expect_identifier_or_keyword(current_position)?;
            current_position = new_position;
            
            // Parse type annotation if present (supports generics and keywords like list<int>)
            let param_type = if let Some(Token::Punctuation(Punctuation::Colon)) = self.tokens.get(current_position) {
                let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Colon))?;
                current_position = new_position;
                let (new_position, param_type) = self.parse_type_expression(current_position)?;
                current_position = new_position;
                Some(param_type)
            } else {
                None
            };
            
            parameters.push(Parameter { name, param_type });
            
            // Check for comma or end of parameters
            if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position) {
                let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Comma))?;
                current_position = new_position;
            } else {
                break;
            }
        }
        
        Ok((current_position, parameters))
    }

    fn parse_spawn_statement(&mut self, position: usize) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'spawn'

        let (new_position, agent_name) = self.expect_identifier(current_position)?;
        current_position = new_position;

        // Check for optional type specification (agent_name:type)
        let mut agent_type = None;
        if let Some(Token::Punctuation(Punctuation::Colon)) = self.tokens.get(current_position) {
            current_position += 1; // consume ':'
            let (new_position, type_name) = self.expect_identifier(current_position)?;
            current_position = new_position;
            agent_type = Some(type_name);
        }

        // Check for optional configuration block
        let mut config = None;
        if let Some(Token::Punctuation(Punctuation::LeftBrace)) = self.tokens.get(current_position) {
            if agent_type.is_some() { // Only allow config if we have a type
                let (new_position, config_map) = self.parse_object_literal(current_position)?;
                current_position = new_position;
                config = Some(config_map);
            }
        }

        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;

        Ok((current_position, Statement::Spawn(SpawnStatement {
            agent_name,
            agent_type,
            config,
            body,
        })))
    }

    fn parse_agent_statement(&mut self, position: usize) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'agent'

        let (new_position, name) = self.expect_identifier(current_position)?;
        current_position = new_position;

        // Parse agent type (required for agent statements)
        let agent_type = if let Some(Token::Punctuation(Punctuation::Colon)) = self.tokens.get(current_position) {
            current_position += 1; // consume ':'
            let (new_position, type_str) = self.expect_identifier(current_position)?;
            current_position = new_position;

            match type_str.as_str() {
                "ai" => crate::parser::ast::AgentType::AI,
                "system" => crate::parser::ast::AgentType::System,
                "worker" => crate::parser::ast::AgentType::Worker,
                _ => crate::parser::ast::AgentType::Custom(type_str),
            }
        } else {
            return Err(ParserError::unexpected_token(&self.tokens[current_position], &[":"], current_position, 0));
        };

        // Parse configuration block
        let (new_position, config) = self.parse_object_literal(current_position)?;
        current_position = new_position;

        // Parse optional capabilities
        let mut capabilities = Vec::new();
        if let Some(Token::Keyword(Keyword::With)) = self.tokens.get(current_position) {
            current_position += 1; // consume 'with'
            let (new_position, caps) = self.parse_capabilities_list(current_position)?;
            current_position = new_position;
            capabilities = caps;
        }

        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;

        Ok((current_position, Statement::Agent(AgentStatement {
            name,
            agent_type,
            config,
            capabilities,
            body,
        })))
    }

    fn parse_object_literal(&self, position: usize) -> Result<(usize, HashMap<String, Expression>), ParserError> {
        let mut current_position = position;

        // Expect opening brace
        if let Some(Token::Punctuation(Punctuation::LeftBrace)) = self.tokens.get(current_position) {
            current_position += 1;
        } else {
            return Err(ParserError::unexpected_token(&self.tokens[current_position], &["{"], current_position, 0));
        }

        let mut properties = HashMap::new();

        // Parse properties until closing brace
        while let Some(token) = self.tokens.get(current_position) {
            match token {
                Token::Punctuation(Punctuation::RightBrace) => {
                    current_position += 1;
                    break;
                }
                Token::Identifier(key) => {
                    current_position += 1; // consume key

                    // Expect colon
                    if let Some(Token::Punctuation(Punctuation::Colon)) = self.tokens.get(current_position) {
                        current_position += 1; // consume ':'
                    } else {
                        return Err(ParserError::unexpected_token(&self.tokens[current_position], &[":"], current_position, 0));
                    }

                    // Parse value expression
                    let (new_position, value) = self.parse_expression(current_position)?;
                    current_position = new_position;

                    properties.insert(key.clone(), value);

                    // Check for comma (optional for last property)
                    if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position) {
                        current_position += 1; // consume ','
                    }
                }
                Token::Literal(Literal::String(key)) => {
                    current_position += 1; // consume key

                    // Expect colon
                    if let Some(Token::Punctuation(Punctuation::Colon)) = self.tokens.get(current_position) {
                        current_position += 1; // consume ':'
                    } else {
                        return Err(ParserError::unexpected_token(&self.tokens[current_position], &[":"], current_position, 0));
                    }

                    // Parse value expression
                    let (new_position, value) = self.parse_expression(current_position)?;
                    current_position = new_position;

                    properties.insert(key.clone(), value);

                    // Check for comma (optional for last property)
                    if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position) {
                        current_position += 1; // consume ','
                    }
                }
                _ => {
                    return Err(ParserError::unexpected_token(&self.tokens[current_position], &["property key", "}"], current_position, 0));
                }
            }
        }

        Ok((current_position, properties))
    }

    fn parse_array_literal(&self, position: usize) -> Result<(usize, Vec<Expression>), ParserError> {
        let mut current_position = position;

        // Expect opening bracket
        if let Some(Token::Punctuation(Punctuation::LeftBracket)) = self.tokens.get(current_position) {
            current_position += 1;
        } else {
            let (line, column) = self.get_token_position(position);
            return Err(ParserError::unexpected_token(
                &self.tokens[position],
                &["["],
                line,
                column,
            ));
        }

        let mut elements = Vec::new();

        // Parse elements until closing bracket
        while let Some(token) = self.tokens.get(current_position) {
            match token {
                Token::Punctuation(Punctuation::RightBracket) => {
                    current_position += 1;
                    break;
                }
                _ => {
                    // Parse expression (can be any expression: literal, identifier, function call, etc.)
                    let (new_position, expr) = self.parse_expression(current_position)?;
                    current_position = new_position;
                    elements.push(expr);

                    // Check for comma (optional for last element)
                    if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position) {
                        current_position += 1; // consume ','
                    } else if let Some(Token::Punctuation(Punctuation::RightBracket)) = self.tokens.get(current_position) {
                        // Allow trailing comma or no comma before closing bracket
                        continue;
                    } else {
                        // If we don't have a comma or closing bracket, it's an error
                        let (line, column) = self.get_token_position(current_position);
                        return Err(ParserError::unexpected_token(
                            &self.tokens[current_position],
                            &[",", "]"],
                            line,
                            column,
                        ));
                    }
                }
            }
        }

        Ok((current_position, elements))
    }

    fn parse_capabilities_list(&self, position: usize) -> Result<(usize, Vec<String>), ParserError> {
        let mut current_position = position;
        let mut capabilities = Vec::new();

        // Expect opening bracket
        if let Some(Token::Punctuation(Punctuation::LeftBracket)) = self.tokens.get(current_position) {
            current_position += 1;
        } else {
            return Err(ParserError::unexpected_token(&self.tokens[current_position], &["["], current_position, 0));
        }

        // Parse capabilities until closing bracket
        while let Some(token) = self.tokens.get(current_position) {
            match token {
                Token::Punctuation(Punctuation::RightBracket) => {
                    current_position += 1;
                    break;
                }
                Token::Literal(Literal::String(capability)) => {
                    current_position += 1; // consume capability
                    capabilities.push(capability.clone());

                    // Check for comma
                    if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position) {
                        current_position += 1; // consume ','
                    }
                }
                _ => {
                    return Err(ParserError::unexpected_token(&self.tokens[current_position], &["string literal", "]"], current_position, 0));
                }
            }
        }

        Ok((current_position, capabilities))
    }

    fn parse_message_statement(&self, position: usize) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'msg'
        
        let (new_position, recipient) = self.expect_identifier(current_position)?;
        current_position = new_position;
        
        let (new_position, _) = self.expect_token(current_position, &Token::Keyword(Keyword::With))?;
        current_position = new_position;
        
        let (new_position, data) = self.parse_message_data(current_position)?;
        current_position = new_position;
        
        Ok((current_position, Statement::Message(MessageStatement {
            recipient,
            data,
        })))
    }

    fn parse_event_statement(&self, position: usize) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'event'
        
        let (new_position, event_name) = self.expect_identifier(current_position)?;
        current_position = new_position;
        
        let (new_position, data) = self.parse_message_data(current_position)?;
        current_position = new_position;
        
        Ok((current_position, Statement::Event(EventStatement {
            event_name,
            data,
        })))
    }

    fn parse_message_data(&self, position: usize) -> Result<(usize, HashMap<String, Expression>), ParserError> {
        let mut current_position = position;
        let mut data = HashMap::new();
        
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::LeftBrace))?;
        current_position = new_position;
        
        // Check if data is empty
        if let Some(Token::Punctuation(Punctuation::RightBrace)) = self.tokens.get(current_position) {
            let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::RightBrace))?;
            return Ok((new_position, data));
        }
        
        loop {
            let (new_position, key) = self.expect_identifier(current_position)?;
            current_position = new_position;
            
            let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Colon))?;
            current_position = new_position;
            
            let (new_position, value) = self.parse_expression(current_position)?;
            current_position = new_position;
            
            data.insert(key, value);
            
            // Check for comma or end of data
            if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position) {
                let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Comma))?;
                current_position = new_position;
            } else {
                break;
            }
        }
        
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::RightBrace))?;
        Ok((new_position, data))
    }

    fn parse_if_statement(&mut self, position: usize) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'if'
        
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::LeftParen))?;
        current_position = new_position;
        
        let (new_position, condition) = self.parse_expression(current_position)?;
        current_position = new_position;
        
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::RightParen))?;
        current_position = new_position;
        
        let (new_position, consequence) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;
        
        // Parse else block if present
        let alternative = if let Some(Token::Keyword(Keyword::Else)) = self.tokens.get(current_position) {
            let (new_pos, _) = self.expect_token(current_position, &Token::Keyword(Keyword::Else))?;
            let (new_pos, else_block) = self.parse_block_statement(new_pos, 0)?;
            current_position = new_pos;
            Some(else_block)
        } else {
            None
        };
        
        Ok((current_position, Statement::If(IfStatement {
            condition,
            consequence,
            alternative,
        })))
    }

    fn parse_function_arguments(&self, position: usize, depth: usize) -> Result<(usize, Vec<Expression>), ParserError> {
        let mut current_position = position;
        let mut arguments = Vec::new();
        
        // Skip the opening parenthesis
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::LeftParen))?;
        current_position = new_position;
        
        // Check if arguments list is empty
        if let Some(Token::Punctuation(Punctuation::RightParen)) = self.tokens.get(current_position) {
            let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::RightParen))?;
            return Ok((new_position, arguments));
        }
        
        loop {
            let (new_position, argument) = self.parse_expression_with_depth(current_position, depth + 1)?;
            current_position = new_position;
            arguments.push(argument);
            
            // Check for comma or end of arguments
            if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position) {
                let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Comma))?;
                current_position = new_position;
            } else {
                break;
            }
        }
        
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::RightParen))?;
        Ok((new_position, arguments))
    }

    fn parse_attribute(&self, position: usize) -> Result<(usize, Attribute), ParserError> {
        let mut current_position = position + 1; // consume '@'
        
        // Parse attribute name (can be identifier or keyword)
        let name = if let Some(Token::Identifier(name)) = self.tokens.get(current_position) {
            let (new_position, _) = self.expect_identifier(current_position)?;
            current_position = new_position;
            name.clone()
        } else if let Some(Token::Keyword(keyword)) = self.tokens.get(current_position) {
            let (new_position, _) = self.expect_token(current_position, &Token::Keyword(keyword.clone()))?;
            current_position = new_position;
            // Convert keyword to string representation
            match keyword {
                Keyword::Txn => "txn".to_string(),
                Keyword::Secure => "secure".to_string(),
                Keyword::Limit => "limit".to_string(),
                Keyword::Trust => "trust".to_string(),
                Keyword::Chain => "chain".to_string(),
                _ => format!("{:?}", keyword).to_lowercase(),
            }
        } else {
            return Err(ParserError::unexpected_token(
                self.tokens.get(current_position).unwrap_or(&Token::EOF),
                &["identifier", "keyword"],
                1,
                1
            ));
        };
        
        let mut parameters = Vec::new();
        
        // Check if parameters are present
        if let Some(Token::Punctuation(Punctuation::LeftParen)) = self.tokens.get(current_position) {
            let (new_pos, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::LeftParen))?;
            current_position = new_pos;
            
            // Check if parameters list is empty
            if let Some(Token::Punctuation(Punctuation::RightParen)) = self.tokens.get(current_position) {
                let (new_pos, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::RightParen))?;
                return Ok((new_pos, Attribute { 
                    name, 
                    parameters,
                    target: AttributeTarget::Function, // Default target
                }));
            }
            
            loop {
                let (new_pos, param) = self.parse_expression(current_position)?; // Top-level, depth 0
                current_position = new_pos;
                parameters.push(param);
                
                // Check for comma or end of parameters
                if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position) {
                    let (new_pos, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Comma))?;
                    current_position = new_pos;
                } else {
                    break;
                }
            }
            
            let (new_pos, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::RightParen))?;
            current_position = new_pos;
        }
        
        // Determine target based on context (default to Function for now)
        let target = AttributeTarget::Function;
        
        Ok((current_position, Attribute { 
            name, 
            parameters,
            target,
        }))
    }

    fn parse_try_statement(&mut self, position: usize) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'try'
        
        // Parse try block
        let (new_position, try_block) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;
        
        let mut catch_blocks = Vec::new();
        let mut finally_block = None;
        
        // Parse catch blocks
        while let Some(Token::Keyword(Keyword::Catch)) = self.tokens.get(current_position) {
            let (new_position, catch_block) = self.parse_catch_block(current_position)?;
            current_position = new_position;
            catch_blocks.push(catch_block);
        }
        
        // Parse finally block if present
        if let Some(Token::Keyword(Keyword::Finally)) = self.tokens.get(current_position) {
            let (new_position, finally) = self.parse_finally_block(current_position)?;
            current_position = new_position;
            finally_block = Some(finally);
        }
        
        Ok((current_position, Statement::Try(TryStatement {
            try_block,
            catch_blocks,
            finally_block,
        })))
    }

    fn parse_catch_block(&mut self, position: usize) -> Result<(usize, CatchBlock), ParserError> {
        let mut current_position = position + 1; // consume 'catch'
        
        let mut error_type = None;
        let mut error_variable = None;
        
        // Check if catch has parameters: catch (ErrorType error_var)
        if let Some(Token::Punctuation(Punctuation::LeftParen)) = self.tokens.get(current_position) {
            let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::LeftParen))?;
            current_position = new_position;
            
            // Parse error type if present
            if let Some(Token::Identifier(type_name)) = self.tokens.get(current_position) {
                let (new_position, _) = self.expect_identifier(current_position)?;
                current_position = new_position;
                error_type = Some(type_name.clone());
                
                // Parse error variable name
                if let Some(Token::Identifier(var_name)) = self.tokens.get(current_position) {
                    let (new_position, _) = self.expect_identifier(current_position)?;
                    current_position = new_position;
                    error_variable = Some(var_name.clone());
                }
            }
            
            let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::RightParen))?;
            current_position = new_position;
        }
        
        // Parse catch body
        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;
        
        Ok((current_position, CatchBlock {
            error_type,
            error_variable,
            body,
        }))
    }

    fn parse_finally_block(&mut self, position: usize) -> Result<(usize, BlockStatement), ParserError> {
        let mut current_position = position + 1; // consume 'finally'
        
        // Parse finally body
        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;
        
        Ok((current_position, body))
    }

    // NEW: Service statement parsing with pre-parsed attributes
    fn parse_service_statement_with_attributes(&mut self, position: usize, pre_parsed_attributes: Vec<Attribute>) -> Result<(usize, Statement), ParserError> {
        // Expect 'service' keyword
        let (new_position, _) = self.expect_token(position, &Token::Keyword(Keyword::Service))?;
        let mut current_position = new_position;
        
        // Parse service name
        let (new_position, name) = self.expect_identifier(current_position)?;
        current_position = new_position;
        
        // Use the pre-parsed attributes
        let mut attributes = pre_parsed_attributes;
        let mut compilation_target = None;
        
        // Parse any additional attributes that might come after the service name
        while let Some(Token::Punctuation(Punctuation::At)) = self.tokens.get(current_position) {
            // Check if this is a @compile_target attribute
            if let Some(Token::Keyword(Keyword::CompileTarget)) = self.tokens.get(current_position + 1) {
                let (new_position, target_info) = self.parse_compile_target_attribute(current_position)?;
                compilation_target = Some(target_info);
                current_position = new_position;
            } else {
                let (new_position, attr) = self.parse_attribute(current_position)?;
                attributes.push(attr);
                current_position = new_position;
            }
        }
        
        // Expect opening brace
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::LeftBrace))?;
        current_position = new_position;
        
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut events = Vec::new();
        
        // Parse service body
        while current_position < self.tokens.len() {
            match self.tokens.get(current_position) {
                Some(Token::Keyword(Keyword::Fn)) => {
                    let (new_position, method) = self.parse_function_statement(current_position)?;
                    if let Statement::Function(func) = method {
                        methods.push(func);
                    }
                    current_position = new_position;
                }
                Some(Token::Keyword(Keyword::Event)) => {
                    let (new_position, event) = self.parse_event_declaration(current_position)?;
                    events.push(event);
                    current_position = new_position;
                }
                Some(Token::Punctuation(Punctuation::RightBrace)) => {
                    current_position += 1;
                    break;
                }
                _ => {
                    // Try to parse field declaration
                    let (new_position, field) = self.parse_service_field(current_position)?;
                    fields.push(field);
                    current_position = new_position;
                }
            }
        }
        
        let service_stmt = ServiceStatement {
            name,
            attributes,
            fields,
            methods,
            events,
            compilation_target,
        };
        
        // Validate target constraints if compilation target is specified
        if let Some(_) = &service_stmt.compilation_target {
            self.validate_target_constraints(&service_stmt)?;
        }
        
        Ok((current_position, Statement::Service(service_stmt)))
    }

    // NEW: Service statement parsing
    fn parse_service_statement(&mut self, position: usize) -> Result<(usize, Statement), ParserError> {
        // Expect 'service' keyword
        let (new_position, _) = self.expect_token(position, &Token::Keyword(Keyword::Service))?;
        let mut current_position = new_position;
        
        // Parse service name
        let (new_position, name) = self.expect_identifier(current_position)?;
        current_position = new_position;
        
        // Parse attributes (if any)
        let mut attributes = Vec::new();
        let mut compilation_target = None;
        
        while let Some(Token::Punctuation(Punctuation::At)) = self.tokens.get(current_position) {
            // Check if this is a @compile_target attribute
            if let Some(Token::Keyword(Keyword::CompileTarget)) = self.tokens.get(current_position + 1) {
                let (new_position, target_info) = self.parse_compile_target_attribute(current_position)?;
                compilation_target = Some(target_info);
                current_position = new_position;
            } else {
                let (new_position, attr) = self.parse_attribute(current_position)?;
                attributes.push(attr);
                current_position = new_position;
            }
        }
        
        // Expect opening brace
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::LeftBrace))?;
        current_position = new_position;
        
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut events = Vec::new();
        
        // Parse service body
        while current_position < self.tokens.len() {
            match self.tokens.get(current_position) {
                Some(Token::Keyword(Keyword::Fn)) => {
                    let (new_position, method) = self.parse_function_statement(current_position)?;
                    if let Statement::Function(func) = method {
                        methods.push(func);
                    }
                    current_position = new_position;
                }
                Some(Token::Keyword(Keyword::Event)) => {
                    let (new_position, event) = self.parse_event_declaration(current_position)?;
                    events.push(event);
                    current_position = new_position;
                }
                Some(Token::Punctuation(Punctuation::RightBrace)) => {
                    current_position += 1;
                    break;
                }
                _ => {
                    // Try to parse field declaration
                    let (new_position, field) = self.parse_service_field(current_position)?;
                    fields.push(field);
                    current_position = new_position;
                }
            }
        }
        
        let service_stmt = ServiceStatement {
            name,
            attributes,
            fields,
            methods,
            events,
            compilation_target,
        };
        
        // Validate target constraints if compilation target is specified
        if let Some(_) = &service_stmt.compilation_target {
            self.validate_target_constraints(&service_stmt)?;
        }
        
        Ok((current_position, Statement::Service(service_stmt)))
    }

    fn parse_service_field(&self, position: usize) -> Result<(usize, ServiceField), ParserError> {
        let mut current_position = position;
        
        // Parse field name
        let (new_position, name) = self.expect_identifier(current_position)?;
        current_position = new_position;
        
        // Expect colon
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Colon))?;
        current_position = new_position;
        
        // Parse field type (supports generics like map<string, int>)
        let (new_position, field_type) = self.parse_type_expression(current_position)?;
        current_position = new_position;
        
        // Parse initial value if present
        let initial_value = if let Some(Token::Operator(Operator::Assign)) = self.tokens.get(current_position) {
            let (new_position, _) = self.expect_token(current_position, &Token::Operator(Operator::Assign))?;
            current_position = new_position;
            let (new_position, value) = self.parse_expression(current_position)?;
            current_position = new_position;
            Some(value)
        } else {
            None
        };
        
        // Expect semicolon
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Semicolon))?;
        current_position = new_position;
        
        let field = ServiceField {
            name,
            field_type,
            initial_value,
            visibility: FieldVisibility::Public, // Default for now
        };
        
        Ok((current_position, field))
    }

    /// Parse a type expression, supporting both simple types and generics
    /// Examples: "string", "int", "map<string, int>", "list<string>"
    fn parse_type_expression(&self, position: usize) -> Result<(usize, String), ParserError> {
        let mut current_position = position;
        
        // Parse base type name (can be identifier or keyword like "list")
        let base_type = if let Some(Token::Identifier(name)) = self.tokens.get(current_position) {
            let (new_position, _) = self.expect_identifier(current_position)?;
            current_position = new_position;
            name.clone()
        } else if let Some(Token::Keyword(keyword)) = self.tokens.get(current_position) {
            let (new_position, _) = self.expect_token(current_position, &Token::Keyword(keyword.clone()))?;
            current_position = new_position;
            // Convert keyword to string representation
            match keyword {
                Keyword::List => "list".to_string(),
                Keyword::Map => "map".to_string(),
                Keyword::Set => "set".to_string(),
                Keyword::Option => "option".to_string(),
                Keyword::Result => "result".to_string(),
                _ => format!("{:?}", keyword).to_lowercase(),
            }
        } else {
            return Err(ParserError::unexpected_token(
                self.tokens.get(current_position).unwrap_or(&Token::EOF),
                &["identifier", "keyword"],
                1,
                1
            ));
        };
        
        // Check if this is a generic type (has <)
        if let Some(Token::Operator(Operator::Less)) = self.tokens.get(current_position) {
            // Parse generic parameters
            let (new_pos, _) = self.expect_token(current_position, &Token::Operator(Operator::Less))?;
            current_position = new_pos;
            
            let mut type_params = Vec::new();
            
            // Parse type parameters
            loop {
                let (new_pos, param_type) = self.parse_type_expression(current_position)?;
                current_position = new_pos;
                type_params.push(param_type);
                
                // Check for comma or closing bracket
                if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position) {
                    let (new_pos, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Comma))?;
                    current_position = new_pos;
                } else if let Some(Token::Operator(Operator::Greater)) = self.tokens.get(current_position) {
                    let (new_pos, _) = self.expect_token(current_position, &Token::Operator(Operator::Greater))?;
                    current_position = new_pos;
                    break;
                } else {
                    return Err(ParserError::unexpected_token(
                        self.tokens.get(current_position).unwrap_or(&Token::EOF),
                        &[",", ">"],
                        1,
                        1
                    ));
                }
            }
            
            // Build the type string: "map<string, int>"
            let type_str = format!("{}<{}>", base_type, type_params.join(", "));
            Ok((current_position, type_str))
        } else {
            // Simple type
            Ok((current_position, base_type))
        }
    }

    fn parse_event_declaration(&self, position: usize) -> Result<(usize, EventDeclaration), ParserError> {
        let mut current_position = position + 1; // consume 'event'
        
        // Parse event name
        let (new_position, name) = self.expect_identifier(current_position)?;
        current_position = new_position;
        
        // Parse parameters
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::LeftParen))?;
        current_position = new_position;
        
        let mut parameters = Vec::new();
        while current_position < self.tokens.len() {
            if let Some(Token::Punctuation(Punctuation::RightParen)) = self.tokens.get(current_position) {
                current_position += 1;
                break;
            }
            
            // Parse single parameter
            let (new_position, param_name) = self.expect_identifier(current_position)?;
            current_position = new_position;
            
            // Check for type annotation (supports generics and keywords like list<int>)
            let param_type = if let Some(Token::Punctuation(Punctuation::Colon)) = self.tokens.get(current_position) {
                let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Colon))?;
                current_position = new_position;
                let (new_position, type_name) = self.parse_type_expression(current_position)?;
                current_position = new_position;
                Some(type_name)
            } else {
                None
            };
            
            parameters.push(Parameter {
                name: param_name,
                param_type,
            });
            
            // Check for comma
            if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position) {
                current_position += 1;
            }
        }
        
        // Expect semicolon
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::Semicolon))?;
        current_position = new_position;
        
        Ok((current_position, EventDeclaration { name, parameters }))
    }

    // NEW: Compilation Target Parsing
    fn parse_compile_target_attribute(&mut self, position: usize) -> Result<(usize, CompilationTargetInfo), ParserError> {
        let mut current_position = position + 1; // consume '@compile_target'
        
        // Expect opening parenthesis
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::LeftParen))?;
        current_position = new_position;
        
        // Parse target name (string literal)
        let target_name = if let Some(Token::Literal(Literal::String(name))) = self.tokens.get(current_position) {
            let (new_position, _) = self.expect_token(current_position, &Token::Literal(Literal::String(name.clone())))?;
            current_position = new_position;
            name.clone()
        } else {
            return Err(ParserError::unexpected_token(
                self.tokens.get(current_position).unwrap_or(&Token::EOF),
                &["string literal"],
                1,
                1
            ));
        };
        
        // Expect closing parenthesis
        let (new_position, _) = self.expect_token(current_position, &Token::Punctuation(Punctuation::RightParen))?;
        current_position = new_position;
        
        // Parse target from string
        let target = crate::lexer::tokens::CompilationTarget::from_string(&target_name)
            .ok_or_else(|| ParserError::unexpected_token(
                self.tokens.get(position).unwrap_or(&Token::EOF),
                &["valid compilation target"],
                1,
                1
            ))?;
        
        // Get constraints for this target
        let constraints = crate::lexer::tokens::get_target_constraints()
            .get(&target)
            .cloned()
            .unwrap_or_else(|| crate::lexer::tokens::TargetConstraint::new(target.clone()));
        
        let target_info = CompilationTargetInfo {
            target,
            constraints,
            validation_errors: Vec::new(),
        };
        
        Ok((current_position, target_info))
    }
    
    fn validate_target_constraints(&self, service: &ServiceStatement) -> Result<(), ParserError> {
        if let Some(ref target_info) = service.compilation_target {
            let constraint = &target_info.constraints;
            
            // Check required attributes
            let mut found_required_attrs = Vec::new();
            for attr in &service.attributes {
                if constraint.required_attributes.contains(&attr.name) {
                    found_required_attrs.push(attr.name.clone());
                }
            }
            
            if found_required_attrs.len() < constraint.required_attributes.len() {
                let missing: Vec<String> = constraint.required_attributes
                    .iter()
                    .filter(|attr| !found_required_attrs.contains(attr))
                    .cloned()
                    .collect();
                
                return Err(ParserError::unexpected_token(
                    self.tokens.get(0).unwrap_or(&Token::EOF),
                    &[&format!("Missing required attributes: {:?}", missing)],
                    1,
                    1
                ));
            }
            
            // Check method operations (simplified validation)
            for method in &service.methods {
                // For now, we'll do basic validation
                // In a full implementation, we'd analyze the method body
                if method.name.contains("web::") && target_info.target == crate::lexer::tokens::CompilationTarget::Blockchain {
                    return Err(ParserError::unexpected_token(
                        self.tokens.get(0).unwrap_or(&Token::EOF),
                        &["Web operations not allowed in blockchain target"],
                        1,
                        1
                    ));
                }
                
                if method.name.contains("chain::") && target_info.target == crate::lexer::tokens::CompilationTarget::WebAssembly {
                    return Err(ParserError::unexpected_token(
                        self.tokens.get(0).unwrap_or(&Token::EOF),
                        &["Blockchain operations not allowed in wasm target"],
                        1,
                        1
                    ));
                }
            }
        }
        
        Ok(())
    }

    // Helper methods
    fn expect_token(&self, position: usize, expected: &Token) -> Result<(usize, &Token), ParserError> {
        if let Some(token) = self.tokens.get(position) {
            if token == expected {
                Ok((position + 1, token))
            } else {
                Err(self.error_unexpected_token(position, &[&format!("{:?}", expected)]))
            }
        } else {
            Err(ParserError::unexpected_eof(&format!("{:?}", expected)))
        }
    }

    fn expect_identifier(&self, position: usize) -> Result<(usize, String), ParserError> {
        if let Some(Token::Identifier(name)) = self.tokens.get(position) {
            Ok((position + 1, name.clone()))
        } else {
            Err(self.error_unexpected_token(position, &["identifier"]))
        }
    }

    fn expect_identifier_or_keyword(&self, position: usize) -> Result<(usize, String), ParserError> {
        if let Some(Token::Identifier(name)) = self.tokens.get(position) {
            Ok((position + 1, name.clone()))
        } else if let Some(Token::Keyword(keyword)) = self.tokens.get(position) {
            // Convert keyword to string representation
            let name = match keyword {
                Keyword::Service => "service".to_string(),
                Keyword::Ai => "ai".to_string(),
                Keyword::Chain => "chain".to_string(),
                Keyword::Mobile => "mobile".to_string(),
                Keyword::Desktop => "desktop".to_string(),
                Keyword::Iot => "iot".to_string(),
                Keyword::Persistent => "persistent".to_string(),
                Keyword::Cached => "cached".to_string(),
                Keyword::Versioned => "versioned".to_string(),
                Keyword::Deprecated => "deprecated".to_string(),
                Keyword::CompileTarget => "compile_target".to_string(),
                Keyword::Interface => "interface".to_string(),
                Keyword::Txn => "txn".to_string(),
                Keyword::Secure => "secure".to_string(),
                Keyword::Limit => "limit".to_string(),
                Keyword::Trust => "trust".to_string(),
                Keyword::Import => "import".to_string(),
                Keyword::Export => "export".to_string(),
                Keyword::As => "as".to_string(),
                Keyword::Private => "private".to_string(),
                Keyword::Audit => "audit".to_string(),
                Keyword::With => "with".to_string(),
                Keyword::Finally => "finally".to_string(),
                Keyword::Await => "await".to_string(),
                Keyword::Async => "async".to_string(),
                Keyword::Result => "Result".to_string(),
                Keyword::Option => "Option".to_string(),
                Keyword::Some => "Some".to_string(),
                Keyword::None => "None".to_string(),
                Keyword::Ok => "Ok".to_string(),
                Keyword::Err => "Err".to_string(),
                Keyword::List => "List".to_string(),
                Keyword::Map => "Map".to_string(),
                Keyword::Set => "Set".to_string(),
                Keyword::Generic => "Generic".to_string(),
                Keyword::Box => "Box".to_string(),
                Keyword::Ref => "ref".to_string(),
                Keyword::Mut => "mut".to_string(),
                Keyword::Const => "const".to_string(),
                Keyword::Static => "static".to_string(),
                Keyword::Extern => "extern".to_string(),
                Keyword::Crate => "crate".to_string(),
                Keyword::Super => "super".to_string(),
                Keyword::Self_ => "self".to_string(),
                Keyword::SelfType => "Self".to_string(),
                _ => format!("{:?}", keyword).to_lowercase(),
            };
            Ok((position + 1, name))
        } else {
            Err(ParserError::unexpected_token(
                self.tokens.get(position).unwrap_or(&Token::EOF),
                &["identifier", "keyword"],
                1,
                1
            ))
        }
    }
}
