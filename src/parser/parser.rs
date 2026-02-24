use crate::lexer::tokens::{Keyword, Literal, Operator, Punctuation, Token};
use crate::parser::ast::{
    AgentStatement, Attribute, AttributeTarget, BlockStatement, BreakStatement, CatchBlock,
    CompilationTargetInfo, ContinueStatement, EventDeclaration, EventStatement, Expression,
    FieldVisibility, ForInStatement, FunctionCall, FunctionStatement, IfStatement, ImportStatement,
    LetStatement, LoopStatement, MatchCase, MatchPattern, MatchStatement, MessageStatement,
    Parameter, Program, ReturnStatement, ServiceField, ServiceStatement, Span, SpawnStatement,
    Statement, TryStatement, WhileStatement,
};
use crate::parser::error::{ErrorContext, ErrorRecovery, ParserError};
use std::collections::{HashMap, HashSet};

pub struct Parser {
    tokens: Vec<Token>,
    token_positions: Vec<(usize, usize)>, // (line, column) for each token
    /// Set by caller before recover_from_error; used by skip_to_synchronization_point to know where to skip from.
    pub(crate) recovery_skip_from: Option<usize>,
    /// Set by skip_to_synchronization_point; caller reads this to continue parsing after recovery.
    pub(crate) recovery_continue_at: Option<usize>,
}

impl Parser {
    /// Limit recursion to prevent stack overflow from malicious/deeply nested input (e.g. fuzzer-generated).
    /// 100 is safe for production while preventing DoS attacks via deeply nested structures.
    /// Phase 2: Increased from 25 to 100 for production use while maintaining safety.
    const MAX_RECURSION_DEPTH: usize = 100;

    /// Creates a parser from tokens. Token positions are empty, so parse errors will use fallback line/column (e.g. 1, 1).
    /// For user-facing parse paths (CLI, IDE) prefer [`Self::new_with_positions`] so errors have accurate line/column.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            token_positions: Vec::new(),
            recovery_skip_from: None,
            recovery_continue_at: None,
        }
    }

    pub fn new_with_positions(
        tokens_with_pos: Vec<crate::lexer::tokens::TokenWithPosition>,
    ) -> Self {
        let mut tokens = Vec::new();
        let mut positions = Vec::new();
        for twp in tokens_with_pos {
            positions.push((twp.line, twp.column));
            tokens.push(twp.token);
        }
        Self {
            tokens,
            token_positions: positions,
            recovery_skip_from: None,
            recovery_continue_at: None,
        }
    }

    /// Set the position to skip from when recover_from_error is called (used for multi-error recovery).
    pub fn set_recovery_skip_from(&mut self, position: usize) {
        self.recovery_skip_from = Some(position);
    }

    /// Take the position to continue parsing at after recovery; call after recover_from_error succeeds.
    pub fn get_recovery_continue_at(&mut self) -> Option<usize> {
        self.recovery_continue_at.take()
    }

    /// Advances from `start` to the next synchronization point (`;`, `}`, statement-start keyword, etc.)
    /// and sets `recovery_continue_at`. Used by ErrorRecovery; keeps token stream private.
    pub(crate) fn skip_to_sync_point_from(&mut self, start: usize) -> bool {
        // Start from start + 1 to ensure we always advance past the error position
        // This prevents infinite loops if the error position itself is a sync point
        let mut pos = start.saturating_add(1);

        // If start + 1 is already beyond bounds, set continue_at to end and exit
        if pos >= self.tokens.len() {
            self.recovery_continue_at = Some(self.tokens.len());
            return true;
        }

        while pos < self.tokens.len() {
            let token = &self.tokens[pos];
            let is_sync = match token {
                Token::Punctuation(Punctuation::Semicolon)
                | Token::Punctuation(Punctuation::RightBrace) => true,
                Token::EOF => true,
                Token::Keyword(k) => matches!(
                    k,
                    Keyword::Let
                        | Keyword::Fn
                        | Keyword::If
                        | Keyword::While
                        | Keyword::Try
                        | Keyword::For
                        | Keyword::Return
                        | Keyword::Service
                        | Keyword::Agent
                        | Keyword::Spawn
                        | Keyword::Event
                        | Keyword::Msg
                        | Keyword::Async
                ),
                Token::Punctuation(Punctuation::LeftBrace) => true,
                Token::Punctuation(Punctuation::At) => true,
                _ => false,
            };
            if is_sync {
                self.recovery_continue_at = Some(pos);
                return true;
            }
            pos += 1;
        }
        // If we reach the end, set continue_at to end position (which will exit the loop)
        self.recovery_continue_at = Some(pos);
        true
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

        // Phase 2: Statement count limit - prevent DoS via excessive statements
        const MAX_STATEMENTS: usize = 100_000; // 100K statements
        let mut statement_count = 0;

        while position < self.tokens.len() {
            // Skip EOF without adding a statement (empty input should produce empty program)
            if matches!(self.tokens.get(position), Some(Token::EOF)) {
                position += 1;
                continue;
            }

            // Check statement count limit
            statement_count += 1;
            if statement_count > MAX_STATEMENTS {
                let (line, _column) = self.get_token_position(position);
                return Err(ParserError::SemanticError {
                    message: format!(
                        "Too many statements: {} (max: {})",
                        statement_count, MAX_STATEMENTS
                    ),
                    line,
                    context: ErrorContext::new(),
                });
            }

            let (new_position, statement) = self.parse_statement(position, 0)?;
            let (line, column) = self.get_token_position(position);
            program.add_statement_with_span(statement, Some(Span { line, column }));
            position = new_position;
        }

        Ok(program)
    }

    /// Parse and collect multiple errors by recovering at statement boundaries.
    /// Returns (program with successfully parsed statements, list of parse errors).
    pub fn parse_with_recovery(&mut self) -> (Program, Vec<ParserError>) {
        let mut program = Program::new();
        let mut errors = Vec::new();
        let mut position = 0;

        while position < self.tokens.len() {
            if matches!(self.tokens.get(position), Some(Token::EOF)) {
                position += 1;
                continue;
            }
            match self.parse_statement(position, 0) {
                Ok((new_position, statement)) => {
                    let (line, column) = self.get_token_position(position);
                    program.add_statement_with_span(statement, Some(Span { line, column }));
                    position = new_position;
                }
                Err(e) => {
                    errors.push(e.clone());
                    self.set_recovery_skip_from(position);
                    if self.recover_from_error(&e).is_ok() {
                        if let Some(next_pos) = self.get_recovery_continue_at() {
                            // skip_to_sync_point_from now always starts from start+1, so next_pos should always be > position
                            // But add defensive check just in case
                            if next_pos > position {
                                position = next_pos;
                            } else {
                                // This shouldn't happen with the fix above, but if it does, advance to prevent infinite loop
                                position += 1;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        (program, errors)
    }

    fn parse_statement(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Statement), ParserError> {
        if depth > Self::MAX_RECURSION_DEPTH {
            let (line, _column) = self.get_token_position(position);
            return Err(ParserError::SemanticError {
                message: format!(
                    "Maximum recursion depth ({}) exceeded in statement parsing",
                    Self::MAX_RECURSION_DEPTH
                ),
                line,
                context: ErrorContext::new(),
            });
        }
        // M5: Explicit export at top level: export fn ... / export service ...
        if depth == 0
            && matches!(
                self.tokens.get(position),
                Some(Token::Keyword(Keyword::Export))
            )
        {
            let mut current_position = position + 1; // consume 'export'
            let mut attributes = Vec::new();
            while current_position < self.tokens.len() {
                if let Some(Token::Punctuation(Punctuation::At)) = self.tokens.get(current_position)
                {
                    let (new_pos, attr) = self.parse_attribute(current_position)?;
                    attributes.push(attr);
                    current_position = new_pos;
                } else {
                    break;
                }
            }
            if let Some(Token::Keyword(Keyword::Fn)) = self.tokens.get(current_position) {
                let (new_pos, mut func_stmt) = self.parse_function_statement(current_position)?;
                if let Statement::Function(ref mut func) = func_stmt {
                    func.attributes = attributes;
                    func.exported = true;
                }
                return Ok((new_pos, func_stmt));
            } else if let Some(Token::Keyword(Keyword::Async)) = self.tokens.get(current_position) {
                let (new_pos, mut func_stmt) =
                    self.parse_async_function_statement(current_position)?;
                if let Statement::Function(ref mut func) = func_stmt {
                    func.attributes = attributes;
                    func.exported = true;
                }
                return Ok((new_pos, func_stmt));
            } else if let Some(Token::Keyword(Keyword::Service)) = self.tokens.get(current_position)
            {
                let (new_pos, service_stmt) = self.parse_service_statement_with_attributes(
                    current_position,
                    attributes,
                    true,
                )?;
                return Ok((new_pos, service_stmt));
            } else {
                let expected = vec!["function declaration", "service declaration"];
                return Err(self.error_unexpected_token(current_position, &expected));
            }
        }

        // Check for attributes first
        if let Some(Token::Punctuation(Punctuation::At)) = self.tokens.get(position) {
            // Attributes can appear before function or service declarations
            let mut current_position = position;
            let mut attributes = Vec::new();

            // Collect all attributes
            while current_position < self.tokens.len() {
                if let Some(Token::Punctuation(Punctuation::At)) = self.tokens.get(current_position)
                {
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
                let (new_pos, mut func_stmt) =
                    self.parse_async_function_statement(current_position)?;
                // Set the collected attributes
                if let Statement::Function(ref mut func) = func_stmt {
                    func.attributes = attributes;
                }
                return Ok((new_pos, func_stmt));
            } else if let Some(Token::Keyword(Keyword::Service)) = self.tokens.get(current_position)
            {
                let (new_pos, service_stmt) = self.parse_service_statement_with_attributes(
                    current_position,
                    attributes,
                    false,
                )?;
                return Ok((new_pos, service_stmt));
            } else {
                // Get the actual token for better error reporting
                let expected = vec!["function declaration", "service declaration"];
                return Err(self.error_unexpected_token(current_position, &expected));
            }
        }

        // Import is top-level only (M1: module system)
        if depth == 0
            && matches!(
                self.tokens.get(position),
                Some(Token::Keyword(Keyword::Import))
            )
        {
            return self.parse_import_statement(position);
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
        } else if let Some(Token::Keyword(Keyword::While)) = self.tokens.get(position) {
            self.parse_while_statement(position)
        } else if let Some(Token::Keyword(Keyword::Try)) = self.tokens.get(position) {
            self.parse_try_statement(position)
        } else if let Some(Token::Keyword(Keyword::For)) = self.tokens.get(position) {
            self.parse_for_in_statement(position)
        } else if let Some(Token::Keyword(Keyword::Break)) = self.tokens.get(position) {
            self.parse_break_statement(position)
        } else if let Some(Token::Keyword(Keyword::Continue)) = self.tokens.get(position) {
            self.parse_continue_statement(position)
        } else if let Some(Token::Keyword(Keyword::Loop)) = self.tokens.get(position) {
            self.parse_loop_statement(position)
        } else if let Some(Token::Keyword(Keyword::Match)) = self.tokens.get(position) {
            self.parse_match_statement(position)
        } else if let Some(Token::Keyword(Keyword::Service)) = self.tokens.get(position) {
            self.parse_service_statement(position)
        } else if let Some(Token::Keyword(Keyword::Return)) = self.tokens.get(position) {
            self.parse_return_statement(position)
        } else if let Some(Token::Punctuation(Punctuation::LeftBrace)) = self.tokens.get(position) {
            let (new_position, block) = self.parse_block_statement(position, depth + 1)?;
            Ok((new_position, Statement::Block(block)))
        } else if let Some(Token::Punctuation(Punctuation::Semicolon)) = self.tokens.get(position) {
            // Skip semicolons (empty statements)
            Ok((
                position + 1,
                Statement::Expression(Expression::Literal(Literal::Null)),
            ))
        } else if let Some(Token::EOF) = self.tokens.get(position) {
            // Skip EOF tokens
            Ok((
                position + 1,
                Statement::Expression(Expression::Literal(Literal::Null)),
            ))
        } else {
            // Defensive: service is already handled above; if we ever reach here with Service token, parse as service.
            if let Some(Token::Keyword(Keyword::Service)) = self.tokens.get(position) {
                return self.parse_service_statement(position);
            }
            let (new_position, expr) = self.parse_expression(position)?;
            Ok((new_position, Statement::Expression(expr)))
        }
    }

    fn parse_let_statement(&mut self, position: usize) -> Result<(usize, Statement), ParserError> {
        let (line, _) = self.get_token_position(position);
        let mut current_position = position + 1; // consume 'let'

        // Check for 'mut' keyword
        let _is_mutable =
            if let Some(Token::Keyword(Keyword::Mut)) = self.tokens.get(current_position) {
                current_position += 1;
                true
            } else {
                false
            };

        // Use expect_identifier_or_keyword to allow keywords as variable names (e.g., "agent", "ai", "chain")
        let (new_position, name) = self.expect_identifier_or_keyword(current_position)?;
        current_position = new_position;

        let (new_position, _) =
            self.expect_token(current_position, &Token::Operator(Operator::Assign))?;
        current_position = new_position;

        let (new_position, value) = self.parse_expression(current_position)?;
        current_position = new_position;

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::Semicolon),
        )?;
        current_position = new_position;

        Ok((
            current_position,
            Statement::Let(LetStatement {
                name,
                value,
                line: Some(line),
            }),
        ))
    }

    fn parse_return_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'return'

        let value = if let Some(Token::Punctuation(Punctuation::Semicolon)) =
            self.tokens.get(current_position)
        {
            None
        } else {
            let (new_position, expr) = self.parse_expression(current_position)?;
            current_position = new_position;
            Some(expr)
        };

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::Semicolon),
        )?;
        Ok((new_position, Statement::Return(ReturnStatement { value })))
    }

    /// Parse top-level import: `import <path>;` or `import <path> as <alias>;`
    /// Path is either an identifier path (e.g. stdlib::chain) or a string literal (e.g. "./mymod.dal", "pkg").
    fn parse_import_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
        use crate::lexer::tokens::Literal;
        let mut current_position = position + 1; // consume 'import'

        let path = if let Some(Token::Literal(Literal::String(s))) =
            self.tokens.get(current_position)
        {
            current_position += 1;
            s.clone()
        } else if let Some(Token::Identifier(first)) = self.tokens.get(current_position) {
            let mut parts = vec![first.clone()];
            current_position += 1;
            while current_position + 1 < self.tokens.len() {
                if !matches!(
                    self.tokens.get(current_position),
                    Some(Token::Punctuation(Punctuation::DoubleColon))
                ) {
                    break;
                }
                let seg = match self.get_identifier_or_keyword_string_at(current_position + 1) {
                    Some(s) => s,
                    None => break,
                };
                parts.push(seg);
                current_position += 2;
            }
            parts.join("::")
        } else {
            let (line, _) = self.get_token_position(current_position);
            return Err(ParserError::SemanticError {
                message: "import expects a path: string literal (e.g. \"./mymod.dal\") or identifier path (e.g. stdlib::chain)".to_string(),
                line,
                context: ErrorContext::new(),
            });
        };

        let alias = if let Some(Token::Keyword(Keyword::As)) = self.tokens.get(current_position) {
            current_position += 1;
            let (new_position, name) = self.expect_identifier_or_keyword(current_position)?;
            current_position = new_position;
            Some(name)
        } else {
            None
        };

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::Semicolon),
        )?;

        Ok((
            new_position,
            Statement::Import(ImportStatement { path, alias }),
        ))
    }

    fn parse_block_statement(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, BlockStatement), ParserError> {
        if depth > Self::MAX_RECURSION_DEPTH {
            let (line, _column) = self.get_token_position(position);
            return Err(ParserError::SemanticError {
                message: format!(
                    "Maximum recursion depth ({}) exceeded in block statement parsing",
                    Self::MAX_RECURSION_DEPTH
                ),
                line,
                context: ErrorContext::new(),
            });
        }

        let (position, _) =
            self.expect_token(position, &Token::Punctuation(Punctuation::LeftBrace))?;

        let mut block = BlockStatement::new();
        let mut current_position = position;

        while current_position < self.tokens.len() {
            if let Some(Token::Punctuation(Punctuation::RightBrace)) =
                self.tokens.get(current_position)
            {
                let (new_position, _) = self.expect_token(
                    current_position,
                    &Token::Punctuation(Punctuation::RightBrace),
                )?;
                return Ok((new_position, block));
            }

            let (new_position, statement) = self.parse_statement(current_position, depth + 1)?;
            block.add_statement(statement);
            current_position = new_position;
        }

        Err(ParserError::unexpected_eof("}"))
    }

    fn parse_expression(&mut self, position: usize) -> Result<(usize, Expression), ParserError> {
        self.parse_assignment(position, 0) // Start with depth 0 for top-level expressions
    }

    fn parse_expression_with_depth(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Expression), ParserError> {
        self.parse_assignment(position, depth)
    }

    fn parse_assignment(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Expression), ParserError> {
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
                    return Ok((
                        position,
                        Expression::FieldAssignment(object_expr, field_name, Box::new(value)),
                    ));
                }
                Expression::IndexAccess(container, index_expr) => {
                    // Index assignment: arr[index] = value, map_var[key] = value, or self.field[key] = value
                    let mut args = vec![*container.clone(), *index_expr.clone(), value];
                    match container.as_ref() {
                        Expression::Identifier(var_name) => {
                            args.push(Expression::Literal(Literal::String(var_name.clone())));
                        }
                        Expression::FieldAccess(_, field_name) => {
                            args.push(Expression::Literal(Literal::String(String::new()))); // 4th: not a var
                            args.push(Expression::Literal(Literal::String(field_name.clone())));
                            // 5th: field name
                        }
                        _ => {}
                    }
                    return Ok((
                        position,
                        Expression::FunctionCall(FunctionCall {
                            name: "__index_assign__".to_string(),
                            arguments: args,
                        }),
                    ));
                }
                Expression::FunctionCall(call)
                    if call.name == "__index__" && call.arguments.len() == 2 =>
                {
                    // Legacy: __index__ as assignment target (e.g. from older AST)
                    let mut args =
                        vec![call.arguments[0].clone(), call.arguments[1].clone(), value];
                    match &call.arguments[0] {
                        Expression::Identifier(var_name) => {
                            args.push(Expression::Literal(Literal::String(var_name.clone())));
                        }
                        Expression::FieldAccess(_, field_name) => {
                            args.push(Expression::Literal(Literal::String(String::new())));
                            args.push(Expression::Literal(Literal::String(field_name.clone())));
                        }
                        _ => {}
                    }
                    return Ok((
                        position,
                        Expression::FunctionCall(FunctionCall {
                            name: "__index_assign__".to_string(),
                            arguments: args,
                        }),
                    ));
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

    fn parse_or(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Expression), ParserError> {
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_and(current_position, depth)?;
        current_position = new_position;

        while current_position < self.tokens.len() {
            if let Some(Token::Operator(Operator::Or)) = self.tokens.get(current_position) {
                let (new_pos, _) =
                    self.expect_token(current_position, &Token::Operator(Operator::Or))?;
                let (new_pos, right) = self.parse_and(new_pos, depth)?;
                expr = Expression::BinaryOp(Box::new(expr), Operator::Or, Box::new(right));
                current_position = new_pos;
            } else {
                break;
            }
        }

        Ok((current_position, expr))
    }

    fn parse_and(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Expression), ParserError> {
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_equality(current_position, depth)?;
        current_position = new_position;

        while current_position < self.tokens.len() {
            if let Some(Token::Operator(Operator::And)) = self.tokens.get(current_position) {
                let (new_pos, _) =
                    self.expect_token(current_position, &Token::Operator(Operator::And))?;
                let (new_pos, right) = self.parse_equality(new_pos, depth)?;
                expr = Expression::BinaryOp(Box::new(expr), Operator::And, Box::new(right));
                current_position = new_pos;
            } else {
                break;
            }
        }

        Ok((current_position, expr))
    }

    fn parse_equality(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Expression), ParserError> {
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_comparison(current_position, depth)?;
        current_position = new_position;

        while current_position < self.tokens.len() {
            if let Some(Token::Operator(Operator::Equal)) = self.tokens.get(current_position) {
                let (new_pos, _) =
                    self.expect_token(current_position, &Token::Operator(Operator::Equal))?;
                let (new_pos, right) = self.parse_comparison(new_pos, depth)?;
                expr = Expression::BinaryOp(Box::new(expr), Operator::Equal, Box::new(right));
                current_position = new_pos;
            } else if let Some(Token::Operator(Operator::NotEqual)) =
                self.tokens.get(current_position)
            {
                let (new_pos, _) =
                    self.expect_token(current_position, &Token::Operator(Operator::NotEqual))?;
                let (new_pos, right) = self.parse_comparison(new_pos, depth)?;
                expr = Expression::BinaryOp(Box::new(expr), Operator::NotEqual, Box::new(right));
                current_position = new_pos;
            } else {
                break;
            }
        }

        Ok((current_position, expr))
    }

    fn parse_comparison(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Expression), ParserError> {
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_range(current_position, depth)?;
        current_position = new_position;

        while current_position < self.tokens.len() {
            let op = self.tokens.get(current_position).and_then(|t| {
                if let Token::Operator(o) = t {
                    Some(o.clone())
                } else {
                    None
                }
            });
            if let Some(op) = op {
                match &op {
                    Operator::Less
                    | Operator::LessEqual
                    | Operator::Greater
                    | Operator::GreaterEqual => {
                        let (new_pos, _) =
                            self.expect_token(current_position, &Token::Operator(op.clone()))?;
                        let (new_pos, right) = self.parse_range(new_pos, depth)?;
                        expr = Expression::BinaryOp(Box::new(expr), op, Box::new(right));
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

    fn parse_range(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Expression), ParserError> {
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_term(current_position, depth)?;
        current_position = new_position;

        // Check for range operator (..)
        if let Some(Token::Punctuation(Punctuation::DotDot)) = self.tokens.get(current_position) {
            current_position += 1; // consume ..
            let (new_position, right) = self.parse_term(current_position, depth)?;
            expr = Expression::Range(Box::new(expr), Box::new(right));
            current_position = new_position;
        }

        Ok((current_position, expr))
    }

    fn parse_term(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Expression), ParserError> {
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_factor(current_position, depth)?;
        current_position = new_position;

        while current_position < self.tokens.len() {
            let op = self.tokens.get(current_position).and_then(|t| {
                if let Token::Operator(o) = t {
                    Some(o.clone())
                } else {
                    None
                }
            });
            if let Some(op) = op {
                match &op {
                    Operator::Plus | Operator::Minus => {
                        let (new_pos, _) =
                            self.expect_token(current_position, &Token::Operator(op.clone()))?;
                        let (new_pos, right) = self.parse_factor(new_pos, depth)?;
                        expr = Expression::BinaryOp(Box::new(expr), op, Box::new(right));
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

    fn parse_factor(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Expression), ParserError> {
        let mut current_position = position;
        let (new_position, mut expr) = self.parse_unary(current_position, depth)?;
        current_position = new_position;

        while current_position < self.tokens.len() {
            let op = self.tokens.get(current_position).and_then(|t| {
                if let Token::Operator(o) = t {
                    Some(o.clone())
                } else {
                    None
                }
            });
            if let Some(op) = op {
                match &op {
                    Operator::Star | Operator::Slash | Operator::Percent => {
                        let (new_pos, _) =
                            self.expect_token(current_position, &Token::Operator(op.clone()))?;
                        let (new_pos, right) = self.parse_unary(new_pos, depth)?;
                        expr = Expression::BinaryOp(Box::new(expr), op, Box::new(right));
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

    fn parse_unary(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Expression), ParserError> {
        // spawn <expression> e.g. spawn worker_process(i)
        if let Some(Token::Keyword(Keyword::Spawn)) = self.tokens.get(position) {
            let (position, _) = self.expect_token(position, &Token::Keyword(Keyword::Spawn))?;
            let (position, expr) = self.parse_unary(position, depth)?;
            return Ok((position, Expression::Spawn(Box::new(expr))));
        }
        let op = self.tokens.get(position).and_then(|t| {
            if let Token::Operator(o) = t {
                Some(o.clone())
            } else {
                None
            }
        });
        if let Some(op) = op {
            match &op {
                Operator::Minus | Operator::Not => {
                    let (position, _) =
                        self.expect_token(position, &Token::Operator(op.clone()))?;
                    let (position, right) = self.parse_unary(position, depth)?;
                    return Ok((position, Expression::UnaryOp(op, Box::new(right))));
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
            if let Some(Token::Punctuation(Punctuation::LeftBracket)) =
                self.tokens.get(current_position)
            {
                let (new_pos, _) = self.expect_token(
                    current_position,
                    &Token::Punctuation(Punctuation::LeftBracket),
                )?;
                current_position = new_pos;

                // Parse index expression
                let (new_pos, index_expr) =
                    self.parse_expression_with_depth(current_position, depth + 1)?;
                current_position = new_pos;

                // Expect closing bracket
                let (new_pos, _) = self.expect_token(
                    current_position,
                    &Token::Punctuation(Punctuation::RightBracket),
                )?;
                current_position = new_pos;

                expr = Expression::IndexAccess(Box::new(expr), Box::new(index_expr));
                continue;
            }

            // Chained field access: expr.field (for cases like self.balances[key])
            if let Some(Token::Punctuation(Punctuation::Dot)) = self.tokens.get(current_position) {
                let (new_pos, _) =
                    self.expect_token(current_position, &Token::Punctuation(Punctuation::Dot))?;
                current_position = new_pos;

                let (new_pos, field_name) = self.expect_identifier_or_keyword(current_position)?;
                current_position = new_pos;

                // Check if this is a method call: expr.field()
                if let Some(Token::Punctuation(Punctuation::LeftParen)) =
                    self.tokens.get(current_position)
                {
                    let (new_pos, arguments) =
                        self.parse_function_arguments(current_position, depth)?;
                    current_position = new_pos;
                    // Create a function call with the field access as the name
                    expr = Expression::FunctionCall(FunctionCall {
                        name: format!(
                            "{}.{}",
                            match &expr {
                                Expression::Identifier(name) => name.clone(),
                                Expression::FieldAccess(obj, field) => format!(
                                    "{}.{}",
                                    match obj.as_ref() {
                                        Expression::Identifier(n) => n.clone(),
                                        _ => "self".to_string(),
                                    },
                                    field
                                ),
                                _ => "self".to_string(),
                            },
                            field_name
                        ),
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

    fn parse_primary(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Expression), ParserError> {
        if depth > Self::MAX_RECURSION_DEPTH {
            let (line, _column) = self.get_token_position(position);
            return Err(ParserError::SemanticError {
                message: format!(
                    "Maximum recursion depth ({}) exceeded in expression. This may indicate malformed input.",
                    Self::MAX_RECURSION_DEPTH
                ),
                line,
                context: ErrorContext::new(),
            });
        }
        if let Some(token) = self.tokens.get(position) {
            match token {
                Token::Literal(Literal::Int(value)) => {
                    return Ok((position + 1, Expression::Literal(Literal::Int(*value))));
                }
                Token::Literal(Literal::Float(value)) => {
                    return Ok((position + 1, Expression::Literal(Literal::Float(*value))));
                }
                Token::Literal(Literal::String(value)) => {
                    return Ok((
                        position + 1,
                        Expression::Literal(Literal::String(value.clone())),
                    ));
                }
                Token::Literal(Literal::Bool(value)) => {
                    return Ok((position + 1, Expression::Literal(Literal::Bool(*value))));
                }
                Token::Literal(Literal::Null) => {
                    return Ok((position + 1, Expression::Literal(Literal::Null)));
                }
                Token::Identifier(name) => {
                    let namespace_name = name.clone();

                    // Check if this is a macro call (identifier!(...)) e.g. vec!(a,b), map!("k",v)
                    if let Some(Token::Operator(Operator::Not)) = self.tokens.get(position + 1) {
                        if let Some(Token::Punctuation(Punctuation::LeftParen)) =
                            self.tokens.get(position + 2)
                        {
                            let (new_position, _) =
                                self.expect_token(position + 1, &Token::Operator(Operator::Not))?;
                            let (new_position, _) = self.expect_token(
                                new_position,
                                &Token::Punctuation(Punctuation::LeftParen),
                            )?;
                            let (new_position, arguments) =
                                self.parse_function_arguments(new_position, depth)?;
                            let (new_position, _) = self.expect_token(
                                new_position,
                                &Token::Punctuation(Punctuation::RightParen),
                            )?;
                            let macro_name = namespace_name.to_lowercase();
                            return Ok((
                                new_position,
                                match macro_name.as_str() {
                                    "vec" => Expression::ArrayLiteral(arguments),
                                    "map" => {
                                        if arguments.len() % 2 != 0 {
                                            let (line, col) = self.get_token_position(position);
                                            return Err(ParserError::unexpected_token(
                                            self.tokens.get(position).unwrap_or(&Token::EOF),
                                            &["map! requires even number of arguments (key, value pairs)"],
                                            line, col
                                        ));
                                        }
                                        let mut obj = HashMap::new();
                                        for i in (0..arguments.len()).step_by(2) {
                                            let key_expr = &arguments[i];
                                            let key_str = match key_expr {
                                                Expression::Literal(Literal::String(s)) => {
                                                    s.clone()
                                                }
                                                _ => {
                                                    let (line, col) =
                                                        self.get_token_position(position);
                                                    return Err(ParserError::unexpected_token(
                                                        self.tokens
                                                            .get(position)
                                                            .unwrap_or(&Token::EOF),
                                                        &["map! keys must be string literals"],
                                                        line,
                                                        col,
                                                    ));
                                                }
                                            };
                                            obj.insert(key_str, arguments[i + 1].clone());
                                        }
                                        Expression::ObjectLiteral(obj)
                                    }
                                    _ => Expression::FunctionCall(FunctionCall {
                                        name: format!("{}!", namespace_name),
                                        arguments,
                                    }),
                                },
                            ));
                        }
                    }

                    // Check if this is a namespace call (identifier::identifier)
                    if let Some(Token::Punctuation(Punctuation::DoubleColon)) =
                        self.tokens.get(position + 1)
                    {
                        let (new_position, _) = self.expect_token(
                            position + 1,
                            &Token::Punctuation(Punctuation::DoubleColon),
                        )?;
                        let (new_position, method_name) =
                            self.expect_identifier_or_keyword(new_position)?;

                        // Check if this is a function call
                        if let Some(Token::Punctuation(Punctuation::LeftParen)) =
                            self.tokens.get(new_position)
                        {
                            let (new_position, arguments) =
                                self.parse_function_arguments(new_position, depth)?;
                            return Ok((
                                new_position,
                                Expression::FunctionCall(FunctionCall {
                                    name: format!("{}::{}", namespace_name, method_name),
                                    arguments,
                                }),
                            ));
                        } else {
                            return Ok((
                                new_position,
                                Expression::Identifier(format!(
                                    "{}::{}",
                                    namespace_name, method_name
                                )),
                            ));
                        }
                    }
                    // Check if this is a function call
                    else if let Some(Token::Punctuation(Punctuation::LeftParen)) =
                        self.tokens.get(position + 1)
                    {
                        let (new_position, arguments) =
                            self.parse_function_arguments(position + 1, depth)?;
                        return Ok((
                            new_position,
                            Expression::FunctionCall(FunctionCall {
                                name: namespace_name.clone(),
                                arguments,
                            }),
                        ));
                    }
                    // Check if this is a field access (identifier.field) or method call (identifier.field())
                    else if let Some(Token::Punctuation(Punctuation::Dot)) =
                        self.tokens.get(position + 1)
                    {
                        let (new_position, _) =
                            self.expect_token(position + 1, &Token::Punctuation(Punctuation::Dot))?;
                        let (new_position, field_name) =
                            self.expect_identifier_or_keyword(new_position)?;
                        // Check if this is a method call: identifier.field()
                        if let Some(Token::Punctuation(Punctuation::LeftParen)) =
                            self.tokens.get(new_position)
                        {
                            let (new_pos, arguments) =
                                self.parse_function_arguments(new_position, depth)?;
                            return Ok((
                                new_pos,
                                Expression::FunctionCall(FunctionCall {
                                    name: format!("{}.{}", namespace_name, field_name),
                                    arguments,
                                }),
                            ));
                        } else {
                            return Ok((
                                new_position,
                                Expression::FieldAccess(
                                    Box::new(Expression::Identifier(namespace_name)),
                                    field_name,
                                ),
                            ));
                        }
                    } else {
                        return Ok((position + 1, Expression::Identifier(namespace_name.clone())));
                    }
                }
                Token::Keyword(Keyword::Service) => {
                    let namespace_name = "service".to_string();

                    // Check if this is a namespace call (service::identifier)
                    if let Some(Token::Punctuation(Punctuation::DoubleColon)) =
                        self.tokens.get(position + 1)
                    {
                        let (new_position, _) = self.expect_token(
                            position + 1,
                            &Token::Punctuation(Punctuation::DoubleColon),
                        )?;
                        let (new_position, method_name) =
                            self.expect_identifier_or_keyword(new_position)?;
                        // Check if this is a function call
                        if let Some(Token::Punctuation(Punctuation::LeftParen)) =
                            self.tokens.get(new_position)
                        {
                            let (new_position, arguments) =
                                self.parse_function_arguments(new_position, depth)?;
                            return Ok((
                                new_position,
                                Expression::FunctionCall(FunctionCall {
                                    name: format!("{}::{}", namespace_name, method_name),
                                    arguments,
                                }),
                            ));
                        } else {
                            return Ok((
                                new_position,
                                Expression::Identifier(format!(
                                    "{}::{}",
                                    namespace_name, method_name
                                )),
                            ));
                        }
                    } else {
                        return Ok((position + 1, Expression::Identifier(namespace_name.clone())));
                    }
                }
                Token::Keyword(Keyword::Ai) => {
                    let namespace_name = "ai".to_string();

                    // Check if this is a namespace call (ai::identifier)
                    if let Some(Token::Punctuation(Punctuation::DoubleColon)) =
                        self.tokens.get(position + 1)
                    {
                        let (new_position, _) = self.expect_token(
                            position + 1,
                            &Token::Punctuation(Punctuation::DoubleColon),
                        )?;
                        let (new_position, method_name) =
                            self.expect_identifier_or_keyword(new_position)?;

                        // Check if this is a function call
                        if let Some(Token::Punctuation(Punctuation::LeftParen)) =
                            self.tokens.get(new_position)
                        {
                            let (new_position, arguments) =
                                self.parse_function_arguments(new_position, depth)?;
                            return Ok((
                                new_position,
                                Expression::FunctionCall(FunctionCall {
                                    name: format!("{}::{}", namespace_name, method_name),
                                    arguments,
                                }),
                            ));
                        } else {
                            return Ok((
                                new_position,
                                Expression::Identifier(format!(
                                    "{}::{}",
                                    namespace_name, method_name
                                )),
                            ));
                        }
                    } else {
                        return Ok((position + 1, Expression::Identifier(namespace_name.clone())));
                    }
                }
                Token::Keyword(Keyword::Await) => {
                    let (position, _) =
                        self.expect_token(position, &Token::Keyword(Keyword::Await))?;
                    let (position, expr) = self.parse_expression_with_depth(position, depth + 1)?;
                    return Ok((position, Expression::Await(Box::new(expr))));
                }
                Token::Keyword(Keyword::Throw) => {
                    let (position, _) =
                        self.expect_token(position, &Token::Keyword(Keyword::Throw))?;
                    let (position, expr) = self.parse_expression_with_depth(position, depth + 1)?;
                    return Ok((position, Expression::Throw(Box::new(expr))));
                }
                Token::Keyword(_) => {
                    // Allow keywords to be used as identifiers in expressions (e.g., "chain" as a variable name)
                    let (new_position, name) = self.expect_identifier_or_keyword(position)?;
                    return Ok((new_position, Expression::Identifier(name)));
                }

                Token::Punctuation(Punctuation::LeftParen) => {
                    let (position, _) =
                        self.expect_token(position, &Token::Punctuation(Punctuation::LeftParen))?;
                    let (position, expr) = self.parse_expression_with_depth(position, depth + 1)?;
                    let (position, _) =
                        self.expect_token(position, &Token::Punctuation(Punctuation::RightParen))?;
                    return Ok((position, expr));
                }
                Token::Punctuation(Punctuation::LeftBrace) => {
                    let (position, object_literal) = self.parse_object_literal(position, depth)?;
                    return Ok((position, Expression::ObjectLiteral(object_literal)));
                }
                Token::Punctuation(Punctuation::LeftBracket) => {
                    // Array literal: [expr1, expr2, ...]
                    // Note: Array access expr[index] is handled in parse_unary postfix operations
                    let (position, array_literal) = self.parse_array_literal(position, depth)?;
                    return Ok((position, Expression::ArrayLiteral(array_literal)));
                }
                Token::Operator(Operator::Not) => {
                    // !(...) without identifier (e.g. from tokenization edge case); vec!(...) and map!(...) are handled in Identifier branch
                    let (position, _) =
                        self.expect_token(position, &Token::Operator(Operator::Not))?;
                    let (position, _) =
                        self.expect_token(position, &Token::Punctuation(Punctuation::LeftParen))?;
                    let (position, arguments) = self.parse_function_arguments(position, depth)?;
                    let (position, _) =
                        self.expect_token(position, &Token::Punctuation(Punctuation::RightParen))?;
                    return Ok((
                        position,
                        Expression::FunctionCall(FunctionCall {
                            name: "macro".to_string(),
                            arguments,
                        }),
                    ));
                }
                _ => {}
            }
        }

        let (line, column) = self.get_token_position(position);
        Err(ParserError::unexpected_token(
            self.tokens.get(position).unwrap_or(&Token::EOF),
            &["expression"],
            line,
            column,
        ))
    }

    fn parse_function_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'fn'

        let (new_position, name) = self.expect_identifier(current_position)?;
        current_position = new_position;

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::LeftParen),
        )?;
        current_position = new_position;

        let (new_position, parameters) = self.parse_parameters(current_position)?;
        current_position = new_position;

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::RightParen),
        )?;
        current_position = new_position;

        // Parse return type if present
        let return_type = if let Some(Token::Punctuation(Punctuation::Arrow)) =
            self.tokens.get(current_position)
        {
            let (new_position, _) =
                self.expect_token(current_position, &Token::Punctuation(Punctuation::Arrow))?;
            current_position = new_position;
            let (new_position, return_type) = self.parse_type_expression(current_position)?;
            current_position = new_position;
            Some(return_type)
        } else {
            None
        };

        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;

        Ok((
            current_position,
            Statement::Function(FunctionStatement {
                name,
                parameters,
                return_type,
                body,
                attributes: Vec::new(),
                is_async: false,
                exported: false,
            }),
        ))
    }

    fn parse_async_function_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'async'

        // Must be followed by 'fn'
        let (new_position, _) =
            self.expect_token(current_position, &Token::Keyword(Keyword::Fn))?;
        current_position = new_position;

        let (new_position, name) = self.expect_identifier(current_position)?;
        current_position = new_position;

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::LeftParen),
        )?;
        current_position = new_position;

        let (new_position, parameters) = self.parse_parameters(current_position)?;
        current_position = new_position;

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::RightParen),
        )?;
        current_position = new_position;

        // Parse return type if present
        let return_type = if let Some(Token::Punctuation(Punctuation::Arrow)) =
            self.tokens.get(current_position)
        {
            let (new_position, _) =
                self.expect_token(current_position, &Token::Punctuation(Punctuation::Arrow))?;
            current_position = new_position;
            let (new_position, return_type) = self.parse_type_expression(current_position)?;
            current_position = new_position;
            Some(return_type)
        } else {
            None
        };

        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;

        Ok((
            current_position,
            Statement::Function(FunctionStatement {
                name,
                parameters,
                return_type,
                body,
                attributes: Vec::new(),
                is_async: true,
                exported: false,
            }),
        ))
    }

    fn parse_parameters(&self, position: usize) -> Result<(usize, Vec<Parameter>), ParserError> {
        let mut current_position = position;
        let mut parameters = Vec::new();

        // Check if parameters list is empty
        if let Some(Token::Punctuation(Punctuation::RightParen)) = self.tokens.get(current_position)
        {
            return Ok((current_position, parameters));
        }

        loop {
            // Allow keywords as parameter names (e.g., "chain" can be a parameter name)
            let (new_position, name) = self.expect_identifier_or_keyword(current_position)?;
            current_position = new_position;

            // Parse type annotation if present (supports generics and keywords like list<int>)
            let param_type = if let Some(Token::Punctuation(Punctuation::Colon)) =
                self.tokens.get(current_position)
            {
                let (new_position, _) =
                    self.expect_token(current_position, &Token::Punctuation(Punctuation::Colon))?;
                current_position = new_position;
                let (new_position, param_type) = self.parse_type_expression(current_position)?;
                current_position = new_position;
                Some(param_type)
            } else {
                None
            };

            parameters.push(Parameter { name, param_type });

            // Check for comma or end of parameters
            if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position)
            {
                let (new_position, _) =
                    self.expect_token(current_position, &Token::Punctuation(Punctuation::Comma))?;
                current_position = new_position;
            } else {
                break;
            }
        }

        Ok((current_position, parameters))
    }

    fn parse_spawn_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'spawn'

        let (new_position, agent_name) = self.expect_identifier(current_position)?;
        current_position = new_position;

        // Check for optional type specification (agent_name:type), e.g. spawn my_assistant:ai
        let mut agent_type = None;
        if let Some(Token::Punctuation(Punctuation::Colon)) = self.tokens.get(current_position) {
            current_position += 1; // consume ':'
            let (new_position, type_name) = self.expect_identifier_or_keyword(current_position)?;
            current_position = new_position;
            agent_type = Some(type_name);
        }

        // Check for optional configuration block
        let mut config = None;
        if let Some(Token::Punctuation(Punctuation::LeftBrace)) = self.tokens.get(current_position)
        {
            if agent_type.is_some() {
                // Only allow config if we have a type
                let (new_position, config_map) = self.parse_object_literal(current_position, 0)?;
                current_position = new_position;
                config = Some(config_map);
            }
        }

        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;

        Ok((
            current_position,
            Statement::Spawn(SpawnStatement {
                agent_name,
                agent_type,
                config,
                body,
            }),
        ))
    }

    fn parse_agent_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'agent'

        let (new_position, name) = self.expect_identifier(current_position)?;
        current_position = new_position;

        // Parse agent type (required for agent statements)
        let agent_type = if let Some(Token::Punctuation(Punctuation::Colon)) =
            self.tokens.get(current_position)
        {
            current_position += 1; // consume ':'
            let (new_position, type_str) = self.expect_identifier_or_keyword(current_position)?;
            current_position = new_position;

            match type_str.as_str() {
                "ai" => crate::parser::ast::AgentType::AI,
                "system" => crate::parser::ast::AgentType::System,
                "worker" => crate::parser::ast::AgentType::Worker,
                _ => crate::parser::ast::AgentType::Custom(type_str),
            }
        } else {
            let (line, column) = self.get_token_position(current_position);
            return Err(ParserError::unexpected_token(
                &self.tokens[current_position],
                &[":"],
                line,
                column,
            ));
        };

        // Parse configuration block
        let (new_position, config) = self.parse_object_literal(current_position, 0)?;
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

        Ok((
            current_position,
            Statement::Agent(AgentStatement {
                name,
                agent_type,
                config,
                capabilities,
                body,
            }),
        ))
    }

    fn parse_object_literal(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, HashMap<String, Expression>), ParserError> {
        if depth > Self::MAX_RECURSION_DEPTH {
            let (line, _column) = self.get_token_position(position);
            return Err(ParserError::SemanticError {
                message: format!(
                    "Maximum recursion depth ({}) exceeded in object literal. This may indicate malformed input.",
                    Self::MAX_RECURSION_DEPTH
                ),
                line,
                context: ErrorContext::new(),
            });
        }

        let mut current_position = position;

        // Expect opening brace
        if let Some(Token::Punctuation(Punctuation::LeftBrace)) = self.tokens.get(current_position)
        {
            current_position += 1;
        } else {
            return Err(ParserError::unexpected_token(
                &self.tokens[current_position],
                &["{"],
                current_position,
                0,
            ));
        }

        let mut properties = HashMap::new();

        // Parse properties until closing brace (clone token so we don't hold &self across parse_expression)
        loop {
            let token = self.tokens.get(current_position).cloned();
            match token {
                Some(Token::Punctuation(Punctuation::RightBrace)) => {
                    current_position += 1;
                    break;
                }
                Some(Token::Identifier(key)) => {
                    current_position += 1; // consume key

                    // Expect colon (allow missing colon when value starts with "this" for compatibility)
                    if let Some(Token::Punctuation(Punctuation::Colon)) =
                        self.tokens.get(current_position)
                    {
                        current_position += 1; // consume ':'
                    } else if let Some(Token::Identifier(id)) = self.tokens.get(current_position) {
                        if id == "this" {
                            // Missing colon before "this.xxx" - parse value anyway
                        } else {
                            let (line, column) = self.get_token_position(current_position);
                            return Err(ParserError::unexpected_token(
                                &self.tokens[current_position],
                                &[":"],
                                line,
                                column,
                            ));
                        }
                    } else {
                        let (line, column) = self.get_token_position(current_position);
                        return Err(ParserError::unexpected_token(
                            &self.tokens[current_position],
                            &[":"],
                            line,
                            column,
                        ));
                    }

                    // Parse value expression (no borrow of self.tokens held here)
                    let (new_position, value) =
                        self.parse_expression_with_depth(current_position, depth + 1)?;
                    current_position = new_position;

                    properties.insert(key, value);

                    // Check for comma (optional for last property)
                    if let Some(Token::Punctuation(Punctuation::Comma)) =
                        self.tokens.get(current_position)
                    {
                        current_position += 1; // consume ','
                    }
                }
                Some(Token::Literal(Literal::String(key))) => {
                    current_position += 1; // consume key

                    // Expect colon (allow missing colon when value starts with "this" for compatibility)
                    if let Some(Token::Punctuation(Punctuation::Colon)) =
                        self.tokens.get(current_position)
                    {
                        current_position += 1; // consume ':'
                    } else if let Some(Token::Identifier(id)) = self.tokens.get(current_position) {
                        if id == "this" {
                            // Missing colon before "this.xxx" - parse value anyway
                        } else {
                            let (line, column) = self.get_token_position(current_position);
                            return Err(ParserError::unexpected_token(
                                &self.tokens[current_position],
                                &[":"],
                                line,
                                column,
                            ));
                        }
                    } else {
                        let (line, column) = self.get_token_position(current_position);
                        return Err(ParserError::unexpected_token(
                            &self.tokens[current_position],
                            &[":"],
                            line,
                            column,
                        ));
                    }

                    // Parse value expression (no borrow of self.tokens held here)
                    let (new_position, value) =
                        self.parse_expression_with_depth(current_position, depth + 1)?;
                    current_position = new_position;

                    properties.insert(key, value);

                    // Check for comma (optional for last property)
                    if let Some(Token::Punctuation(Punctuation::Comma)) =
                        self.tokens.get(current_position)
                    {
                        current_position += 1; // consume ','
                    }
                }
                Some(_) => {
                    let (line, column) = self.get_token_position(current_position);
                    return Err(ParserError::unexpected_token(
                        &self.tokens[current_position],
                        &["property key", "}"],
                        line,
                        column,
                    ));
                }
                None => break,
            }
        }

        Ok((current_position, properties))
    }

    fn parse_array_literal(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Vec<Expression>), ParserError> {
        if depth > Self::MAX_RECURSION_DEPTH {
            let (line, _column) = self.get_token_position(position);
            return Err(ParserError::SemanticError {
                message: format!(
                    "Maximum recursion depth ({}) exceeded in array literal. This may indicate malformed input.",
                    Self::MAX_RECURSION_DEPTH
                ),
                line,
                context: ErrorContext::new(),
            });
        }

        let mut current_position = position;

        // Expect opening bracket
        if let Some(Token::Punctuation(Punctuation::LeftBracket)) =
            self.tokens.get(current_position)
        {
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
                    let (new_position, expr) =
                        self.parse_expression_with_depth(current_position, depth + 1)?;
                    current_position = new_position;
                    elements.push(expr);

                    // Check for comma (optional for last element)
                    if let Some(Token::Punctuation(Punctuation::Comma)) =
                        self.tokens.get(current_position)
                    {
                        current_position += 1; // consume ','
                    } else if let Some(Token::Punctuation(Punctuation::RightBracket)) =
                        self.tokens.get(current_position)
                    {
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

    fn parse_capabilities_list(
        &self,
        position: usize,
    ) -> Result<(usize, Vec<String>), ParserError> {
        let mut current_position = position;
        let mut capabilities = Vec::new();

        // Expect opening bracket
        if let Some(Token::Punctuation(Punctuation::LeftBracket)) =
            self.tokens.get(current_position)
        {
            current_position += 1;
        } else {
            return Err(ParserError::unexpected_token(
                &self.tokens[current_position],
                &["["],
                current_position,
                0,
            ));
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
                    if let Some(Token::Punctuation(Punctuation::Comma)) =
                        self.tokens.get(current_position)
                    {
                        current_position += 1; // consume ','
                    }
                }
                _ => {
                    return Err(ParserError::unexpected_token(
                        &self.tokens[current_position],
                        &["string literal", "]"],
                        current_position,
                        0,
                    ));
                }
            }
        }

        Ok((current_position, capabilities))
    }

    fn parse_message_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'msg'

        let (new_position, recipient) = self.expect_identifier(current_position)?;
        current_position = new_position;

        let (new_position, _) =
            self.expect_token(current_position, &Token::Keyword(Keyword::With))?;
        current_position = new_position;

        let (new_position, data) = self.parse_message_data(current_position)?;
        current_position = new_position;

        Ok((
            current_position,
            Statement::Message(MessageStatement { recipient, data }),
        ))
    }

    fn parse_event_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'event'

        let (new_position, event_name) = self.expect_identifier(current_position)?;
        current_position = new_position;

        let (new_position, data) = self.parse_message_data(current_position)?;
        current_position = new_position;

        Ok((
            current_position,
            Statement::Event(EventStatement { event_name, data }),
        ))
    }

    fn parse_message_data(
        &mut self,
        position: usize,
    ) -> Result<(usize, HashMap<String, Expression>), ParserError> {
        let mut current_position = position;
        let mut data = HashMap::new();

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::LeftBrace),
        )?;
        current_position = new_position;

        // Check if data is empty
        if let Some(Token::Punctuation(Punctuation::RightBrace)) = self.tokens.get(current_position)
        {
            let (new_position, _) = self.expect_token(
                current_position,
                &Token::Punctuation(Punctuation::RightBrace),
            )?;
            return Ok((new_position, data));
        }

        loop {
            let (new_position, key) = self.expect_identifier(current_position)?;
            current_position = new_position;

            let (new_position, _) =
                self.expect_token(current_position, &Token::Punctuation(Punctuation::Colon))?;
            current_position = new_position;

            let (new_position, value) = self.parse_expression(current_position)?;
            current_position = new_position;

            data.insert(key, value);

            // Check for comma or end of data
            if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position)
            {
                let (new_position, _) =
                    self.expect_token(current_position, &Token::Punctuation(Punctuation::Comma))?;
                current_position = new_position;
            } else {
                break;
            }
        }

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::RightBrace),
        )?;
        Ok((new_position, data))
    }

    fn parse_if_statement(&mut self, position: usize) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'if'

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::LeftParen),
        )?;
        current_position = new_position;

        let (new_position, condition) = self.parse_expression(current_position)?;
        current_position = new_position;

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::RightParen),
        )?;
        current_position = new_position;

        let (new_position, consequence) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;

        // Parse else block if present (supports "else if (cond) { block }" as well as "else { block }")
        let alternative =
            if let Some(Token::Keyword(Keyword::Else)) = self.tokens.get(current_position) {
                let (new_pos, _) =
                    self.expect_token(current_position, &Token::Keyword(Keyword::Else))?;
                current_position = new_pos;
                // If we see "if (" then parse as "else if (cond) { block }" and wrap in a single-statement block
                let else_block = if let (
                    Some(Token::Keyword(Keyword::If)),
                    Some(Token::Punctuation(Punctuation::LeftParen)),
                ) = (
                    self.tokens.get(current_position),
                    self.tokens.get(current_position + 1),
                ) {
                    let (new_pos, if_stmt) = self.parse_if_statement(current_position)?;
                    current_position = new_pos;
                    BlockStatement {
                        statements: vec![if_stmt],
                    }
                } else {
                    let (new_pos, block) = self.parse_block_statement(current_position, 0)?;
                    current_position = new_pos;
                    block
                };
                Some(else_block)
            } else {
                None
            };

        Ok((
            current_position,
            Statement::If(IfStatement {
                condition,
                consequence,
                alternative,
            }),
        ))
    }

    fn parse_while_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'while'

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::LeftParen),
        )?;
        current_position = new_position;

        let (new_position, condition) = self.parse_expression(current_position)?;
        current_position = new_position;

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::RightParen),
        )?;
        current_position = new_position;

        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;

        Ok((
            current_position,
            Statement::While(WhileStatement { condition, body }),
        ))
    }

    fn parse_function_arguments(
        &mut self,
        position: usize,
        depth: usize,
    ) -> Result<(usize, Vec<Expression>), ParserError> {
        let mut current_position = position;
        let mut arguments = Vec::new();

        // Skip the opening parenthesis
        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::LeftParen),
        )?;
        current_position = new_position;

        // Check if arguments list is empty
        if let Some(Token::Punctuation(Punctuation::RightParen)) = self.tokens.get(current_position)
        {
            let (new_position, _) = self.expect_token(
                current_position,
                &Token::Punctuation(Punctuation::RightParen),
            )?;
            return Ok((new_position, arguments));
        }

        loop {
            let (new_position, argument) =
                self.parse_expression_with_depth(current_position, depth + 1)?;
            current_position = new_position;
            arguments.push(argument);

            // Arrow function: (param => { body }) — if last arg is single identifier and next is =>
            let param_name = arguments.last().and_then(|a| {
                if let Expression::Identifier(p) = a {
                    Some(p.clone())
                } else {
                    None
                }
            });
            if let Some(param) = param_name {
                if let Some(Token::Punctuation(Punctuation::FatArrow)) =
                    self.tokens.get(current_position)
                {
                    let (pos2, _) = self.expect_token(
                        current_position,
                        &Token::Punctuation(Punctuation::FatArrow),
                    )?;
                    let (pos3, body) = self.parse_block_statement(pos2, depth)?;
                    arguments.pop();
                    arguments.push(Expression::ArrowFunction { param, body });
                    current_position = pos3;
                    let (new_pos, _) = self.expect_token(
                        current_position,
                        &Token::Punctuation(Punctuation::RightParen),
                    )?;
                    return Ok((new_pos, arguments));
                }
            }

            // Check for comma or end of arguments
            if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position)
            {
                let (new_position, _) =
                    self.expect_token(current_position, &Token::Punctuation(Punctuation::Comma))?;
                current_position = new_position;
            } else {
                break;
            }
        }

        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::RightParen),
        )?;
        Ok((new_position, arguments))
    }

    fn parse_attribute(&mut self, position: usize) -> Result<(usize, Attribute), ParserError> {
        let mut current_position = position + 1; // consume '@'

        // Parse attribute name (can be identifier or keyword). Store with leading @ so required_attributes match.
        let name = if let Some(Token::Identifier(n)) = self.tokens.get(current_position) {
            let (new_position, _) = self.expect_identifier(current_position)?;
            current_position = new_position;
            format!("@{}", n)
        } else if let Some(Token::Keyword(keyword)) = self.tokens.get(current_position) {
            let (new_position, _) =
                self.expect_token(current_position, &Token::Keyword(keyword.clone()))?;
            current_position = new_position;
            let raw = match keyword {
                Keyword::Txn => "txn".to_string(),
                Keyword::Secure => "secure".to_string(),
                Keyword::Limit => "limit".to_string(),
                Keyword::Trust => "trust".to_string(),
                Keyword::Chain => "chain".to_string(),
                _ => format!("{:?}", keyword).to_lowercase(),
            };
            format!("@{}", raw)
        } else {
            let (line, column) = self.get_token_position(current_position);
            return Err(ParserError::unexpected_token(
                self.tokens.get(current_position).unwrap_or(&Token::EOF),
                &["identifier", "keyword"],
                line,
                column,
            ));
        };

        let mut parameters = Vec::new();

        // Check if parameters are present
        if let Some(Token::Punctuation(Punctuation::LeftParen)) = self.tokens.get(current_position)
        {
            let (new_pos, _) = self.expect_token(
                current_position,
                &Token::Punctuation(Punctuation::LeftParen),
            )?;
            current_position = new_pos;

            // Check if parameters list is empty
            if let Some(Token::Punctuation(Punctuation::RightParen)) =
                self.tokens.get(current_position)
            {
                let (new_pos, _) = self.expect_token(
                    current_position,
                    &Token::Punctuation(Punctuation::RightParen),
                )?;
                return Ok((
                    new_pos,
                    Attribute {
                        name,
                        parameters,
                        target: AttributeTarget::Function, // Default target
                    },
                ));
            }

            loop {
                let (new_pos, param) = self.parse_expression(current_position)?; // Top-level, depth 0
                current_position = new_pos;
                parameters.push(param);

                // Check for comma or end of parameters
                if let Some(Token::Punctuation(Punctuation::Comma)) =
                    self.tokens.get(current_position)
                {
                    let (new_pos, _) = self
                        .expect_token(current_position, &Token::Punctuation(Punctuation::Comma))?;
                    current_position = new_pos;
                } else {
                    break;
                }
            }

            let (new_pos, _) = self.expect_token(
                current_position,
                &Token::Punctuation(Punctuation::RightParen),
            )?;
            current_position = new_pos;
        }

        // Target is set by caller when attaching to function (Function) or service (Module)
        let target = AttributeTarget::Function;

        Ok((
            current_position,
            Attribute {
                name,
                parameters,
                target,
            },
        ))
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

        Ok((
            current_position,
            Statement::Try(TryStatement {
                try_block,
                catch_blocks,
                finally_block,
            }),
        ))
    }

    fn parse_catch_block(&mut self, position: usize) -> Result<(usize, CatchBlock), ParserError> {
        let mut current_position = position + 1; // consume 'catch'

        let mut error_type = None;
        let mut error_variable = None;

        // Check if catch has parameters: catch (ErrorType error_var)
        if let Some(Token::Punctuation(Punctuation::LeftParen)) = self.tokens.get(current_position)
        {
            let (new_position, _) = self.expect_token(
                current_position,
                &Token::Punctuation(Punctuation::LeftParen),
            )?;
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

            let (new_position, _) = self.expect_token(
                current_position,
                &Token::Punctuation(Punctuation::RightParen),
            )?;
            current_position = new_position;
        }

        // Parse catch body
        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;

        Ok((
            current_position,
            CatchBlock {
                error_type,
                error_variable,
                body,
            },
        ))
    }

    fn parse_finally_block(
        &mut self,
        position: usize,
    ) -> Result<(usize, BlockStatement), ParserError> {
        let mut current_position = position + 1; // consume 'finally'

        // Parse finally body
        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;

        Ok((current_position, body))
    }

    /// Parse `for <variable> in <iterable> { body }`
    fn parse_for_in_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'for'
        let (new_position, variable) = self.expect_identifier_or_keyword(current_position)?;
        current_position = new_position;
        let (new_position, _) =
            self.expect_token(current_position, &Token::Keyword(Keyword::In))?;
        current_position = new_position;
        let (new_position, iterable) = self.parse_expression(current_position)?;
        current_position = new_position;
        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;
        Ok((
            current_position,
            Statement::ForIn(ForInStatement {
                variable,
                iterable,
                body,
            }),
        ))
    }

    /// Parse `break;` or `break expr;`
    fn parse_break_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'break'

        // Check if there's a value expression
        let value = if let Some(Token::Punctuation(Punctuation::Semicolon)) =
            self.tokens.get(current_position)
        {
            None
        } else {
            let (new_position, expr) = self.parse_expression(current_position)?;
            current_position = new_position;
            Some(expr)
        };

        // Expect semicolon
        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::Semicolon),
        )?;
        current_position = new_position;

        Ok((current_position, Statement::Break(BreakStatement { value })))
    }

    /// Parse `continue;`
    fn parse_continue_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'continue'

        // Expect semicolon
        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::Semicolon),
        )?;
        current_position = new_position;

        Ok((current_position, Statement::Continue(ContinueStatement)))
    }

    /// Parse `loop { body }`
    fn parse_loop_statement(&mut self, position: usize) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'loop'

        // Parse loop body
        let (new_position, body) = self.parse_block_statement(current_position, 0)?;
        current_position = new_position;

        Ok((current_position, Statement::Loop(LoopStatement { body })))
    }

    /// Parse `match expr { case1 => body1, case2 => body2, default => body }`
    fn parse_match_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
        let mut current_position = position + 1; // consume 'match'

        // Parse expression to match against
        let (new_position, expression) = self.parse_expression(current_position)?;
        current_position = new_position;

        // Expect opening brace
        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::LeftBrace),
        )?;
        current_position = new_position;

        let mut cases = Vec::new();
        let mut default_case = None;

        // Parse cases
        while current_position < self.tokens.len() {
            // Check for closing brace
            if let Some(Token::Punctuation(Punctuation::RightBrace)) =
                self.tokens.get(current_position)
            {
                let (new_position, _) = self.expect_token(
                    current_position,
                    &Token::Punctuation(Punctuation::RightBrace),
                )?;
                current_position = new_position;
                break;
            }

            // Check for default case
            if let Some(Token::Keyword(Keyword::Default)) = self.tokens.get(current_position) {
                let (new_position, _) =
                    self.expect_token(current_position, &Token::Keyword(Keyword::Default))?;
                current_position = new_position;

                // Expect fat arrow
                let (new_position, _) = self
                    .expect_token(current_position, &Token::Punctuation(Punctuation::FatArrow))?;
                current_position = new_position;

                // Parse default body - can be a block, a statement (break/continue), or a simple expression
                let body = if let Some(Token::Punctuation(Punctuation::LeftBrace)) =
                    self.tokens.get(current_position)
                {
                    // It's a block
                    let (new_position, block) = self.parse_block_statement(current_position, 0)?;
                    current_position = new_position;
                    block
                } else if let Some(Token::Keyword(Keyword::Break)) =
                    self.tokens.get(current_position)
                {
                    // It's a break statement - parse without expecting semicolon (match uses commas)
                    let mut pos = current_position + 1; // consume 'break'
                    let value = if matches!(
                        self.tokens.get(pos),
                        Some(Token::Punctuation(Punctuation::Comma))
                            | Some(Token::Punctuation(Punctuation::RightBrace))
                    ) {
                        None
                    } else {
                        let (new_position, expr) = self.parse_expression(pos)?;
                        pos = new_position;
                        Some(expr)
                    };
                    current_position = pos;
                    let mut block = BlockStatement::new();
                    block.add_statement(Statement::Break(BreakStatement { value }));
                    block
                } else if let Some(Token::Keyword(Keyword::Continue)) =
                    self.tokens.get(current_position)
                {
                    // It's a continue statement - no semicolon needed in match context
                    current_position += 1; // consume 'continue'
                    let mut block = BlockStatement::new();
                    block.add_statement(Statement::Continue(ContinueStatement));
                    block
                } else {
                    // It's a simple expression - convert to single-statement block
                    let (new_position, expr) = self.parse_expression(current_position)?;
                    current_position = new_position;
                    let mut block = BlockStatement::new();
                    block.add_statement(Statement::Expression(expr));
                    block
                };
                default_case = Some(body);

                // Check for comma (optional)
                if let Some(Token::Punctuation(Punctuation::Comma)) =
                    self.tokens.get(current_position)
                {
                    current_position += 1;
                }
                continue;
            }

            // Parse pattern
            let pattern = self.parse_match_pattern(current_position)?;
            let (new_position, pattern) = pattern;
            current_position = new_position;

            // Expect fat arrow
            let (new_position, _) =
                self.expect_token(current_position, &Token::Punctuation(Punctuation::FatArrow))?;
            current_position = new_position;

            // Parse case body - can be a block, a statement (break/continue), or a simple expression
            let body = if let Some(Token::Punctuation(Punctuation::LeftBrace)) =
                self.tokens.get(current_position)
            {
                // It's a block
                let (new_position, block) = self.parse_block_statement(current_position, 0)?;
                current_position = new_position;
                block
            } else if let Some(Token::Keyword(Keyword::Break)) = self.tokens.get(current_position) {
                // It's a break statement - parse without expecting semicolon (match uses commas)
                let mut pos = current_position + 1; // consume 'break'
                let value = if matches!(
                    self.tokens.get(pos),
                    Some(Token::Punctuation(Punctuation::Comma))
                        | Some(Token::Punctuation(Punctuation::RightBrace))
                ) {
                    None
                } else {
                    let (new_position, expr) = self.parse_expression(pos)?;
                    pos = new_position;
                    Some(expr)
                };
                current_position = pos;
                let mut block = BlockStatement::new();
                block.add_statement(Statement::Break(BreakStatement { value }));
                block
            } else if let Some(Token::Keyword(Keyword::Continue)) =
                self.tokens.get(current_position)
            {
                // It's a continue statement - no semicolon needed in match context
                current_position += 1; // consume 'continue'
                let mut block = BlockStatement::new();
                block.add_statement(Statement::Continue(ContinueStatement));
                block
            } else {
                // It's a simple expression - convert to single-statement block
                let (new_position, expr) = self.parse_expression(current_position)?;
                current_position = new_position;
                let mut block = BlockStatement::new();
                block.add_statement(Statement::Expression(expr));
                block
            };

            cases.push(MatchCase { pattern, body });

            // Check for comma (optional)
            if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position)
            {
                current_position += 1;
            }
        }

        Ok((
            current_position,
            Statement::Match(MatchStatement {
                expression,
                cases,
                default_case,
            }),
        ))
    }

    /// Parse a match pattern: literal, identifier, wildcard, or range
    fn parse_match_pattern(
        &mut self,
        position: usize,
    ) -> Result<(usize, MatchPattern), ParserError> {
        let current_position = position;

        // Check for wildcard
        if let Some(Token::Identifier(ref id)) = self.tokens.get(current_position) {
            if id == "_" {
                return Ok((current_position + 1, MatchPattern::Wildcard));
            }
        }

        // Check for range pattern (start..end)
        // Peek ahead: if we have a literal followed by DotDot, it's a range
        if let Some(Token::Literal(ref lit)) = self.tokens.get(current_position) {
            if let Some(Token::Punctuation(Punctuation::DotDot)) =
                self.tokens.get(current_position + 1)
            {
                // It's a range pattern - parse literals directly, don't use parse_expression
                let start = Expression::Literal(lit.clone());
                let mut new_pos = current_position + 1; // Skip first literal

                // Expect DotDot
                let (new_pos2, _) =
                    self.expect_token(new_pos, &Token::Punctuation(Punctuation::DotDot))?;
                new_pos = new_pos2;

                // Parse end literal
                if let Some(Token::Literal(ref end_lit)) = self.tokens.get(new_pos) {
                    let end = Expression::Literal(end_lit.clone());
                    return Ok((
                        new_pos + 1,
                        MatchPattern::Range(Box::new(start), Box::new(end)),
                    ));
                } else {
                    let (line, column) = self.get_token_position(new_pos);
                    return Err(ParserError::UnexpectedToken {
                        token: format!("{:?}", self.tokens.get(new_pos)),
                        expected: "Literal (range end)".to_string(),
                        line,
                        column,
                        context: ErrorContext::new(),
                    });
                }
            }

            // Not a range, parse as literal
            return Ok((current_position + 1, MatchPattern::Literal(lit.clone())));
        }

        // Otherwise parse as identifier (binds the value)
        let (new_position, identifier) = self.expect_identifier_or_keyword(current_position)?;
        Ok((new_position, MatchPattern::Identifier(identifier)))
    }

    // NEW: Service statement parsing with pre-parsed attributes (M5: exported flag)
    fn parse_service_statement_with_attributes(
        &mut self,
        position: usize,
        pre_parsed_attributes: Vec<Attribute>,
        exported: bool,
    ) -> Result<(usize, Statement), ParserError> {
        // Expect 'service' keyword
        let (new_position, _) = self.expect_token(position, &Token::Keyword(Keyword::Service))?;
        let mut current_position = new_position;

        // Parse service name
        let (new_position, name) = self.expect_identifier(current_position)?;
        current_position = new_position;

        // Use the pre-parsed attributes (target already set to Module by caller); set Module for any added later
        let mut attributes = pre_parsed_attributes;
        let mut compilation_target = None;

        // Parse any additional attributes that might come after the service name
        while let Some(Token::Punctuation(Punctuation::At)) = self.tokens.get(current_position) {
            // Check if this is a @compile_target attribute
            if let Some(Token::Keyword(Keyword::CompileTarget)) =
                self.tokens.get(current_position + 1)
            {
                let (new_position, target_info) =
                    self.parse_compile_target_attribute(current_position)?;
                compilation_target = Some(target_info);
                current_position = new_position;
            } else {
                let (new_position, attr) = self.parse_attribute(current_position)?;
                attributes.push(attr);
                current_position = new_position;
            }
        }

        // Expect opening brace
        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::LeftBrace),
        )?;
        current_position = new_position;

        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut events = Vec::new();

        // Parse service body
        while current_position < self.tokens.len() {
            match self.tokens.get(current_position) {
                Some(Token::Punctuation(Punctuation::At)) => {
                    // Check if this is attributes before a function
                    let mut attr_position = current_position;
                    let mut method_attributes = Vec::new();

                    // Collect all attributes
                    while attr_position < self.tokens.len() {
                        if let Some(Token::Punctuation(Punctuation::At)) =
                            self.tokens.get(attr_position)
                        {
                            let (new_pos, attr) = self.parse_attribute(attr_position)?;
                            method_attributes.push(attr);
                            attr_position = new_pos;
                        } else {
                            break;
                        }
                    }

                    // Check if attributes are followed by function declaration
                    // After parsing attributes, attr_position points to the token after the last attribute
                    if attr_position < self.tokens.len() {
                        match self.tokens.get(attr_position) {
                            Some(Token::Keyword(Keyword::Fn)) => {
                                // Found function - parse it with attributes
                                let (new_position, mut method) =
                                    self.parse_function_statement(attr_position)?;
                                if let Statement::Function(ref mut func) = method {
                                    func.attributes = method_attributes;
                                }
                                methods.push(if let Statement::Function(func) = method {
                                    func
                                } else {
                                    unreachable!()
                                });
                                current_position = new_position;
                            }
                            Some(Token::Identifier(_)) => {
                                // Attributes followed by identifier - this is a field with visibility attribute
                                let (new_position, field) =
                                    self.parse_service_field(current_position)?;
                                fields.push(field);
                                current_position = new_position;
                            }
                            _ => {
                                // Attributes not followed by function or field name - error
                                let (line, _col) = self.get_token_position(attr_position);
                                return Err(ParserError::SemanticError {
                                    message: format!(
                                        "Attributes must be followed by 'fn' (function) or field name, found: {:?}",
                                        self.tokens.get(attr_position)
                                    ),
                                    line,
                                    context: ErrorContext::new(),
                                });
                            }
                        }
                    } else {
                        // End of tokens after attributes - error
                        let (line, _col) = self.get_token_position(attr_position.saturating_sub(1));
                        return Err(ParserError::SemanticError {
                            message: "Attributes must be followed by 'fn' (function) or field name"
                                .to_string(),
                            line,
                            context: ErrorContext::new(),
                        });
                    }
                }
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

        // All attributes on this service apply to the module
        for attr in &mut attributes {
            attr.target = AttributeTarget::Module;
        }
        let service_stmt = ServiceStatement {
            name,
            attributes,
            fields,
            methods,
            events,
            compilation_target,
            exported,
        };

        // Validate target constraints if compilation target is specified
        if let Some(_) = &service_stmt.compilation_target {
            self.validate_target_constraints(&service_stmt)?;
        }

        // Validate security-critical attribute rules
        self.validate_service_security(&service_stmt)?;

        Ok((current_position, Statement::Service(service_stmt)))
    }

    // NEW: Service statement parsing
    fn parse_service_statement(
        &mut self,
        position: usize,
    ) -> Result<(usize, Statement), ParserError> {
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
            if let Some(Token::Keyword(Keyword::CompileTarget)) =
                self.tokens.get(current_position + 1)
            {
                let (new_position, target_info) =
                    self.parse_compile_target_attribute(current_position)?;
                compilation_target = Some(target_info);
                current_position = new_position;
            } else {
                let (new_position, attr) = self.parse_attribute(current_position)?;
                attributes.push(attr);
                current_position = new_position;
            }
        }

        // Expect opening brace
        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::LeftBrace),
        )?;
        current_position = new_position;

        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut events = Vec::new();

        // Parse service body
        while current_position < self.tokens.len() {
            match self.tokens.get(current_position) {
                Some(Token::Punctuation(Punctuation::At)) => {
                    // Check if this is attributes before a function
                    let mut attr_position = current_position;
                    let mut method_attributes = Vec::new();

                    // Collect all attributes
                    while attr_position < self.tokens.len() {
                        if let Some(Token::Punctuation(Punctuation::At)) =
                            self.tokens.get(attr_position)
                        {
                            let (new_pos, attr) = self.parse_attribute(attr_position)?;
                            method_attributes.push(attr);
                            attr_position = new_pos;
                        } else {
                            break;
                        }
                    }

                    // Check if attributes are followed by function declaration
                    // After parsing attributes, attr_position points to the token after the last attribute
                    if attr_position < self.tokens.len() {
                        match self.tokens.get(attr_position) {
                            Some(Token::Keyword(Keyword::Fn)) => {
                                // Found function - parse it with attributes
                                let (new_position, mut method) =
                                    self.parse_function_statement(attr_position)?;
                                if let Statement::Function(ref mut func) = method {
                                    func.attributes = method_attributes;
                                }
                                methods.push(if let Statement::Function(func) = method {
                                    func
                                } else {
                                    unreachable!()
                                });
                                current_position = new_position;
                            }
                            Some(Token::Identifier(_)) => {
                                // Attributes followed by identifier - this is a field with visibility attribute
                                let (new_position, field) =
                                    self.parse_service_field(current_position)?;
                                fields.push(field);
                                current_position = new_position;
                            }
                            _ => {
                                // Attributes not followed by function or field name - error
                                let (line, _col) = self.get_token_position(attr_position);
                                return Err(ParserError::SemanticError {
                                    message: format!(
                                        "Attributes must be followed by 'fn' (function) or field name, found: {:?}",
                                        self.tokens.get(attr_position)
                                    ),
                                    line,
                                    context: ErrorContext::new(),
                                });
                            }
                        }
                    } else {
                        // End of tokens after attributes - error
                        let (line, _col) = self.get_token_position(attr_position.saturating_sub(1));
                        return Err(ParserError::SemanticError {
                            message: "Attributes must be followed by 'fn' (function) or field name"
                                .to_string(),
                            line,
                            context: ErrorContext::new(),
                        });
                    }
                }
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

        for attr in &mut attributes {
            attr.target = AttributeTarget::Module;
        }
        let service_stmt = ServiceStatement {
            name,
            attributes,
            fields,
            methods,
            events,
            compilation_target,
            exported: false,
        };

        // Validate target constraints if compilation target is specified
        if let Some(_) = &service_stmt.compilation_target {
            self.validate_target_constraints(&service_stmt)?;
        }

        // Validate security-critical attribute rules
        self.validate_service_security(&service_stmt)?;

        Ok((current_position, Statement::Service(service_stmt)))
    }

    fn parse_service_field(
        &mut self,
        position: usize,
    ) -> Result<(usize, ServiceField), ParserError> {
        let mut current_position = position;
        let mut visibility = FieldVisibility::Public;

        // Optional visibility: @public / @private / @internal or keyword private before field name
        if let Some(Token::Punctuation(Punctuation::At)) = self.tokens.get(current_position) {
            let (new_position, _) =
                self.expect_token(current_position, &Token::Punctuation(Punctuation::At))?;
            current_position = new_position;
            let (new_position, vis_name) = self.expect_identifier_or_keyword(current_position)?;
            current_position = new_position;
            let vis_name = vis_name.to_lowercase();
            visibility = match vis_name.as_str() {
                "public" => FieldVisibility::Public,
                "private" => FieldVisibility::Private,
                "internal" => FieldVisibility::Internal,
                _ => {
                    let (line, col) = self.get_token_position(current_position.saturating_sub(1));
                    return Err(ParserError::unexpected_token(
                        self.tokens
                            .get(current_position.saturating_sub(1))
                            .unwrap_or(&Token::EOF),
                        &["@public", "@private", "@internal"],
                        line,
                        col,
                    ));
                }
            };
        } else if let Some(Token::Keyword(Keyword::Private)) = self.tokens.get(current_position) {
            let (new_position, _) =
                self.expect_token(current_position, &Token::Keyword(Keyword::Private))?;
            current_position = new_position;
            visibility = FieldVisibility::Private;
        }

        // Parse field name
        let (new_position, name) = self.expect_identifier(current_position)?;
        current_position = new_position;

        // Expect colon
        let (new_position, _) =
            self.expect_token(current_position, &Token::Punctuation(Punctuation::Colon))?;
        current_position = new_position;

        // Parse field type (supports generics like map<string, int>)
        let (new_position, field_type) = self.parse_type_expression(current_position)?;
        current_position = new_position;

        // Parse initial value if present
        let initial_value =
            if let Some(Token::Operator(Operator::Assign)) = self.tokens.get(current_position) {
                let (new_position, _) =
                    self.expect_token(current_position, &Token::Operator(Operator::Assign))?;
                current_position = new_position;
                let (new_position, value) = self.parse_expression(current_position)?;
                current_position = new_position;
                Some(value)
            } else {
                None
            };

        // Expect semicolon
        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::Semicolon),
        )?;
        current_position = new_position;

        let field = ServiceField {
            name,
            field_type,
            initial_value,
            visibility,
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
            let (new_position, _) =
                self.expect_token(current_position, &Token::Keyword(keyword.clone()))?;
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
            let (line, column) = self.get_token_position(current_position);
            return Err(ParserError::unexpected_token(
                self.tokens.get(current_position).unwrap_or(&Token::EOF),
                &["identifier", "keyword"],
                line,
                column,
            ));
        };

        // Check if this is a generic type (has <)
        if let Some(Token::Operator(Operator::Less)) = self.tokens.get(current_position) {
            // Parse generic parameters
            let (new_pos, _) =
                self.expect_token(current_position, &Token::Operator(Operator::Less))?;
            current_position = new_pos;

            let mut type_params = Vec::new();

            // Parse type parameters
            loop {
                let (new_pos, param_type) = self.parse_type_expression(current_position)?;
                current_position = new_pos;
                type_params.push(param_type);

                // Check for comma or closing bracket
                if let Some(Token::Punctuation(Punctuation::Comma)) =
                    self.tokens.get(current_position)
                {
                    let (new_pos, _) = self
                        .expect_token(current_position, &Token::Punctuation(Punctuation::Comma))?;
                    current_position = new_pos;
                } else if let Some(Token::Operator(Operator::Greater)) =
                    self.tokens.get(current_position)
                {
                    let (new_pos, _) =
                        self.expect_token(current_position, &Token::Operator(Operator::Greater))?;
                    current_position = new_pos;
                    break;
                } else {
                    return Err(ParserError::unexpected_token(
                        self.tokens.get(current_position).unwrap_or(&Token::EOF),
                        &[",", ">"],
                        1,
                        1,
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

    fn parse_event_declaration(
        &self,
        position: usize,
    ) -> Result<(usize, EventDeclaration), ParserError> {
        let mut current_position = position + 1; // consume 'event'

        // Parse event name
        let (new_position, name) = self.expect_identifier(current_position)?;
        current_position = new_position;

        // Parse parameters
        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::LeftParen),
        )?;
        current_position = new_position;

        let mut parameters = Vec::new();
        while current_position < self.tokens.len() {
            if let Some(Token::Punctuation(Punctuation::RightParen)) =
                self.tokens.get(current_position)
            {
                current_position += 1;
                break;
            }

            // Parse single parameter
            let (new_position, param_name) = self.expect_identifier(current_position)?;
            current_position = new_position;

            // Check for type annotation (supports generics and keywords like list<int>)
            let param_type = if let Some(Token::Punctuation(Punctuation::Colon)) =
                self.tokens.get(current_position)
            {
                let (new_position, _) =
                    self.expect_token(current_position, &Token::Punctuation(Punctuation::Colon))?;
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
            if let Some(Token::Punctuation(Punctuation::Comma)) = self.tokens.get(current_position)
            {
                current_position += 1;
            }
        }

        // Expect semicolon
        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::Semicolon),
        )?;
        current_position = new_position;

        Ok((current_position, EventDeclaration { name, parameters }))
    }

    // NEW: Compilation Target Parsing
    fn parse_compile_target_attribute(
        &mut self,
        position: usize,
    ) -> Result<(usize, CompilationTargetInfo), ParserError> {
        let mut current_position = position + 1; // consume '@'
        let (new_position, _) =
            self.expect_token(current_position, &Token::Keyword(Keyword::CompileTarget))?;
        current_position = new_position; // consume 'compile_target'

        // Expect opening parenthesis
        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::LeftParen),
        )?;
        current_position = new_position;

        // Parse target name (string literal)
        let target_name = if let Some(Token::Literal(Literal::String(name))) =
            self.tokens.get(current_position)
        {
            let (new_position, _) = self.expect_token(
                current_position,
                &Token::Literal(Literal::String(name.clone())),
            )?;
            current_position = new_position;
            name.clone()
        } else {
            return Err(ParserError::unexpected_token(
                self.tokens.get(current_position).unwrap_or(&Token::EOF),
                &["string literal"],
                1,
                1,
            ));
        };

        // Expect closing parenthesis
        let (new_position, _) = self.expect_token(
            current_position,
            &Token::Punctuation(Punctuation::RightParen),
        )?;
        current_position = new_position;

        // Parse target from string
        let target = crate::lexer::tokens::CompilationTarget::from_string(&target_name)
            .ok_or_else(|| {
                ParserError::unexpected_token(
                    self.tokens.get(position).unwrap_or(&Token::EOF),
                    &["valid compilation target"],
                    1,
                    1,
                )
            })?;

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

    /// Collect namespaces used in method body (e.g. "chain" from chain::deploy, "web" from web::fetch).
    fn collect_namespaces_from_expression(&self, expr: &Expression) -> HashSet<String> {
        let mut out = HashSet::new();
        match expr {
            Expression::FunctionCall(call) => {
                if let Some(ns) = call.name.split("::").next() {
                    if call.name.contains("::") && !ns.is_empty() {
                        out.insert(ns.to_string());
                    }
                }
                for arg in &call.arguments {
                    out.extend(self.collect_namespaces_from_expression(arg));
                }
            }
            Expression::BinaryOp(l, _, r) => {
                out.extend(self.collect_namespaces_from_expression(l));
                out.extend(self.collect_namespaces_from_expression(r));
            }
            Expression::UnaryOp(_, e) => {
                out.extend(self.collect_namespaces_from_expression(e));
            }
            Expression::Assignment(_, v) => {
                out.extend(self.collect_namespaces_from_expression(v));
            }
            Expression::FieldAccess(obj, _) => {
                out.extend(self.collect_namespaces_from_expression(obj));
            }
            Expression::FieldAssignment(obj, _, v) => {
                out.extend(self.collect_namespaces_from_expression(obj));
                out.extend(self.collect_namespaces_from_expression(v));
            }
            Expression::Await(e) | Expression::Spawn(e) | Expression::Throw(e) => {
                out.extend(self.collect_namespaces_from_expression(e));
            }
            Expression::IndexAccess(c, i) => {
                out.extend(self.collect_namespaces_from_expression(c));
                out.extend(self.collect_namespaces_from_expression(i));
            }
            Expression::ObjectLiteral(props) => {
                for (_, e) in props {
                    out.extend(self.collect_namespaces_from_expression(e));
                }
            }
            Expression::ArrayLiteral(elems) => {
                for e in elems {
                    out.extend(self.collect_namespaces_from_expression(e));
                }
            }
            Expression::ArrowFunction { body, .. } => {
                out.extend(self.collect_namespaces_from_block(&body));
            }
            Expression::Range(start, end) => {
                out.extend(self.collect_namespaces_from_expression(start));
                out.extend(self.collect_namespaces_from_expression(end));
            }
            _ => {}
        }
        out
    }

    fn collect_namespaces_from_statement(&self, stmt: &Statement) -> HashSet<String> {
        let mut out = HashSet::new();
        match stmt {
            Statement::Expression(expr) => {
                out.extend(self.collect_namespaces_from_expression(expr));
            }
            Statement::Let(let_stmt) => {
                out.extend(self.collect_namespaces_from_expression(&let_stmt.value));
            }
            Statement::Return(ret) => {
                if let Some(ref e) = ret.value {
                    out.extend(self.collect_namespaces_from_expression(e));
                }
            }
            Statement::Block(block) => {
                out.extend(self.collect_namespaces_from_block(block));
            }
            Statement::If(if_stmt) => {
                out.extend(self.collect_namespaces_from_expression(&if_stmt.condition));
                out.extend(self.collect_namespaces_from_block(&if_stmt.consequence));
                if let Some(ref alt) = if_stmt.alternative {
                    out.extend(self.collect_namespaces_from_block(alt));
                }
            }
            Statement::While(while_stmt) => {
                out.extend(self.collect_namespaces_from_expression(&while_stmt.condition));
                out.extend(self.collect_namespaces_from_block(&while_stmt.body));
            }
            Statement::Try(try_stmt) => {
                out.extend(self.collect_namespaces_from_block(&try_stmt.try_block));
                for cb in &try_stmt.catch_blocks {
                    out.extend(self.collect_namespaces_from_block(&cb.body));
                }
                if let Some(ref fb) = try_stmt.finally_block {
                    out.extend(self.collect_namespaces_from_block(fb));
                }
            }
            Statement::ForIn(for_stmt) => {
                out.extend(self.collect_namespaces_from_expression(&for_stmt.iterable));
                out.extend(self.collect_namespaces_from_block(&for_stmt.body));
            }
            Statement::Function(func) => {
                out.extend(self.collect_namespaces_from_block(&func.body));
            }
            Statement::Spawn(spawn) => {
                out.extend(self.collect_namespaces_from_block(&spawn.body));
            }
            Statement::Agent(agent) => {
                out.extend(self.collect_namespaces_from_block(&agent.body));
            }
            Statement::Message(msg) => {
                for (_, e) in &msg.data {
                    out.extend(self.collect_namespaces_from_expression(e));
                }
            }
            Statement::Event(ev) => {
                for (_, e) in &ev.data {
                    out.extend(self.collect_namespaces_from_expression(e));
                }
            }
            Statement::Service(service) => {
                for method in &service.methods {
                    out.extend(self.collect_namespaces_from_block(&method.body));
                }
            }
            Statement::Break(break_stmt) => {
                if let Some(ref expr) = break_stmt.value {
                    out.extend(self.collect_namespaces_from_expression(expr));
                }
            }
            Statement::Continue(_) => {
                // No expressions to check
            }
            Statement::Loop(loop_stmt) => {
                out.extend(self.collect_namespaces_from_block(&loop_stmt.body));
            }
            Statement::Match(match_stmt) => {
                out.extend(self.collect_namespaces_from_expression(&match_stmt.expression));
                for case in &match_stmt.cases {
                    out.extend(self.collect_namespaces_from_block(&case.body));
                    // Check range patterns for expressions
                    if let crate::parser::ast::MatchPattern::Range(start, end) = &case.pattern {
                        out.extend(self.collect_namespaces_from_expression(start));
                        out.extend(self.collect_namespaces_from_expression(end));
                    }
                }
                if let Some(ref default_body) = match_stmt.default_case {
                    out.extend(self.collect_namespaces_from_block(default_body));
                }
            }
            Statement::Import(_) => {
                // Import path does not contribute namespace calls in this pass
            }
        }
        out
    }

    fn collect_namespaces_from_block(&self, block: &BlockStatement) -> HashSet<String> {
        let mut out = HashSet::new();
        for stmt in &block.statements {
            out.extend(self.collect_namespaces_from_statement(stmt));
        }
        out
    }

    fn validate_target_constraints(&self, service: &ServiceStatement) -> Result<(), ParserError> {
        if let Some(ref target_info) = service.compilation_target {
            let constraint = &target_info.constraints;

            // Check required attributes (attr.name stored with @ e.g. "@native")
            let mut found_required_attrs = Vec::new();
            for attr in &service.attributes {
                if constraint.required_attributes.contains(&attr.name) {
                    found_required_attrs.push(attr.name.clone());
                }
            }

            if found_required_attrs.len() < constraint.required_attributes.len() {
                let missing: Vec<String> = constraint
                    .required_attributes
                    .iter()
                    .filter(|attr| !found_required_attrs.contains(attr))
                    .cloned()
                    .collect();

                return Err(ParserError::unexpected_token(
                    self.tokens.get(0).unwrap_or(&Token::EOF),
                    &[&format!("Missing required attributes: {:?}", missing)],
                    1,
                    1,
                ));
            }

            // Forbidden namespaces for this target (from constraint.forbidden_operations, e.g. "web::http_request" -> "web")
            let forbidden_namespaces: HashSet<String> = constraint
                .forbidden_operations
                .iter()
                .filter_map(|op| op.split("::").next().map(|s| s.to_string()))
                .collect();

            for method in &service.methods {
                let used_namespaces = self.collect_namespaces_from_block(&method.body);
                let violating: Vec<String> = used_namespaces
                    .intersection(&forbidden_namespaces)
                    .cloned()
                    .collect();
                if !violating.is_empty() {
                    return Err(ParserError::unexpected_token(
                        self.tokens.get(0).unwrap_or(&Token::EOF),
                        &[&format!(
                            "Method '{}' uses forbidden namespace(s) for target {:?}: {:?}",
                            method.name, target_info.target, violating
                        )],
                        1,
                        1,
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate security-critical attribute rules for services.
    /// This enforces security rules at parse time to prevent invalid code from being parsed.
    fn validate_service_security(&self, service: &ServiceStatement) -> Result<(), ParserError> {
        use crate::lexer::tokens::Literal;
        use crate::parser::ast::Expression;

        // Collect attribute names for compatibility checks
        let attr_names: Vec<&str> = service.attributes.iter().map(|a| a.name.as_str()).collect();

        let has_trust = attr_names.contains(&"@trust");
        let has_chain = attr_names.contains(&"@chain");
        let has_secure = attr_names.contains(&"@secure");
        let has_public = attr_names.contains(&"@public");

        // Rule 1: @trust requires @chain (security-critical)
        if has_trust && !has_chain {
            let (line, _column) = self.get_service_line_column(service);
            return Err(ParserError::SemanticError {
                message: format!(
                    "Service '{}' with @trust attribute must also have @chain attribute",
                    service.name
                ),
                line,
                context: ErrorContext::new(),
            });
        }

        // Rule 2: @secure and @public are mutually exclusive (security-critical)
        if has_secure && has_public {
            let (line, _column) = self.get_service_line_column(service);
            return Err(ParserError::SemanticError {
                message: format!(
                    "Service '{}' cannot have both @secure and @public attributes (mutually exclusive)",
                    service.name
                ),
                line,
                context: ErrorContext::new(),
            });
        }

        // Rule 3: Validate trust model values
        for attr in &service.attributes {
            if attr.name == "@trust" {
                if let Some(Expression::Literal(Literal::String(model))) = attr.parameters.first() {
                    let valid_models = ["hybrid", "centralized", "decentralized", "trustless"];
                    if !valid_models.contains(&model.as_str()) {
                        let (line, _column) = self.get_service_line_column(service);
                        return Err(ParserError::SemanticError {
                            message: format!(
                                "Service '{}' has invalid trust model '{}'. Valid options: {:?}",
                                service.name, model, valid_models
                            ),
                            line,
                            context: ErrorContext::new(),
                        });
                    }
                }
            }

            // Rule 4: Validate chain identifiers
            if attr.name == "@chain" {
                for param in &attr.parameters {
                    if let Expression::Literal(Literal::String(chain)) = param {
                        let valid_chains = [
                            "ethereum",
                            "polygon",
                            "bsc",
                            "solana",
                            "bitcoin",
                            "avalanche",
                            "arbitrum",
                            "optimism",
                            "base",
                            "near",
                            "eth", // Common shorthand
                        ];
                        let chain_lower = chain.to_lowercase();
                        if !valid_chains.contains(&chain_lower.as_str()) {
                            let (line, _column) = self.get_service_line_column(service);
                            return Err(ParserError::SemanticError {
                                message: format!(
                                    "Service '{}' has invalid chain identifier '{}'. Valid options: {:?}",
                                    service.name, chain, valid_chains
                                ),
                                line,
                                context: ErrorContext::new(),
                            });
                        }
                    }
                }
            }
        }

        // Rule 5: Validate function-level attributes (@secure and @public are mutually exclusive)
        for method in &service.methods {
            let func_attr_names: Vec<&str> =
                method.attributes.iter().map(|a| a.name.as_str()).collect();

            let func_has_secure = func_attr_names.contains(&"@secure");
            let func_has_public = func_attr_names.contains(&"@public");

            if func_has_secure && func_has_public {
                // Try to get line number for the function
                let line =
                    if let Some(pos) = self.find_function_position(&service.name, &method.name) {
                        let (line, _column) = self.get_token_position(pos);
                        line
                    } else {
                        0 // Fallback if position not found
                    };

                return Err(ParserError::SemanticError {
                    message: format!(
                        "Function '{}' in service '{}' cannot have both @secure and @public attributes (mutually exclusive)",
                        method.name, service.name
                    ),
                    line,
                    context: ErrorContext::new(),
                });
            }
        }

        Ok(())
    }

    /// Helper to find the position of a function within a service
    fn find_function_position(&self, _service_name: &str, function_name: &str) -> Option<usize> {
        // This is a simplified implementation - in practice, you'd track positions during parsing
        // For now, we'll search through tokens to find the function
        for (pos, tokens_window) in self.tokens.windows(3).enumerate() {
            if let [Token::Keyword(Keyword::Fn), Token::Identifier(name), ..] = tokens_window {
                if name == function_name {
                    return Some(pos);
                }
            }
        }
        None
    }

    /// Get the line and column for a service statement (for error reporting)
    fn get_service_line_column(&self, _service: &ServiceStatement) -> (usize, usize) {
        // Try to find the service keyword token
        for (i, token) in self.tokens.iter().enumerate() {
            if let Token::Keyword(Keyword::Service) = token {
                if i < self.token_positions.len() {
                    return self.token_positions[i];
                }
            }
        }
        // Fallback to first token position or (1, 1)
        if !self.token_positions.is_empty() {
            self.token_positions[0]
        } else {
            (1, 1)
        }
    }

    // Helper methods
    fn expect_token(
        &self,
        position: usize,
        expected: &Token,
    ) -> Result<(usize, &Token), ParserError> {
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

    /// Returns the string value of the token at position if it is an Identifier or Keyword (for import path segments).
    fn get_identifier_or_keyword_string_at(&self, position: usize) -> Option<String> {
        match self.tokens.get(position) {
            Some(Token::Identifier(s)) => Some(s.clone()),
            Some(Token::Keyword(k)) => Some(self.keyword_to_string(k)),
            _ => None,
        }
    }

    fn keyword_to_string(&self, k: &Keyword) -> String {
        match k {
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
            _ => format!("{:?}", k).to_lowercase(),
        }
    }

    fn expect_identifier_or_keyword(
        &self,
        position: usize,
    ) -> Result<(usize, String), ParserError> {
        if let Some(Token::Identifier(name)) = self.tokens.get(position) {
            Ok((position + 1, name.clone()))
        } else if let Some(Token::Keyword(keyword)) = self.tokens.get(position) {
            let name = self.keyword_to_string(keyword);
            Ok((position + 1, name))
        } else {
            let (line, column) = self.get_token_position(position);
            Err(ParserError::unexpected_token(
                self.tokens.get(position).unwrap_or(&Token::EOF),
                &["identifier", "keyword"],
                line,
                column,
            ))
        }
    }
}
