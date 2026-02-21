use crate::lexer::tokens::*;

#[derive(Debug)]
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

#[allow(dead_code)]
impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        self.tokenize_immutable()
    }

    pub fn tokenize_immutable(&self) -> Result<Vec<Token>, LexerError> {
        self.tokenize_with_positions_immutable()
            .map(|tokens_with_pos| tokens_with_pos.into_iter().map(|twp| twp.token).collect())
    }

    pub fn tokenize_with_positions_immutable(
        &self,
    ) -> Result<Vec<crate::lexer::tokens::TokenWithPosition>, LexerError> {
        let mut tokens = Vec::new();
        let mut position = self.position;
        let mut line = self.line;
        let mut column = self.column;

        // Phase 2: Token count limit - prevent DoS via excessive tokens
        const MAX_TOKENS: usize = 1_000_000; // 1M tokens

        // Safety: Prevent infinite loops from mutations that cause position to not advance
        let max_iterations = self.input.len() * 2; // Allow up to 2x input length iterations
        let mut iterations = 0;

        while position < self.input.len() {
            // Safety check: prevent infinite loops
            iterations += 1;
            if iterations > max_iterations {
                return Err(LexerError::UnexpectedCharacter('\0', line, column));
            }

            // Phase 2: Check token count limit
            if tokens.len() >= MAX_TOKENS {
                return Err(LexerError::TooManyTokens(
                    line,
                    column,
                    tokens.len(),
                    MAX_TOKENS,
                ));
            }

            // Skip whitespace first
            let (new_pos, new_line, new_col) =
                self.skip_whitespace_immutable(position, line, column);
            position = new_pos;
            line = new_line;
            column = new_col;

            // Check if we've reached the end
            if position >= self.input.len() {
                break;
            }

            // Get the next token
            let (new_pos, new_line, new_col, token) =
                self.next_token_immutable(position, line, column)?;

            // Safety check: ensure position advances (prevents infinite loops from mutations)
            if new_pos <= position {
                return Err(LexerError::UnexpectedCharacter('\0', line, column));
            }

            // Store the token with its position (before advancing)
            tokens.push(crate::lexer::tokens::TokenWithPosition::new(
                token, line, column,
            ));
            position = new_pos;
            line = new_line;
            column = new_col.max(1); // Ensure column is always >= 1
        }

        tokens.push(crate::lexer::tokens::TokenWithPosition::new(
            Token::EOF,
            line,
            column,
        ));
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        let (new_pos, new_line, new_col, token) =
            self.next_token_immutable(self.position, self.line, self.column)?;
        self.position = new_pos;
        self.line = new_line;
        self.column = new_col;
        Ok(token)
    }

    fn next_token_immutable(
        &self,
        position: usize,
        line: usize,
        column: usize,
    ) -> Result<(usize, usize, usize, Token), LexerError> {
        if position >= self.input.len() {
            return Err(LexerError::UnexpectedCharacter('\0', line, column));
        }

        let ch = self.input[position];
        let mut new_position = position;
        let new_line = line;
        let mut new_column = column.max(1); // Ensure column is always >= 1

        match ch {
            // Identifiers and keywords (include $ for SQL placeholders in strings, identifiers)
            'a'..='z' | 'A'..='Z' | '_' | '$' => {
                let (pos, identifier) = self.read_identifier_immutable(position);
                new_position = pos;

                // Handle boolean and null literals
                match identifier.as_str() {
                    "true" => Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Literal(Literal::Bool(true)),
                    )),
                    "false" => Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Literal(Literal::Bool(false)),
                    )),
                    "null" => Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Literal(Literal::Null),
                    )),
                    _ => {
                        // Check if this might be a namespace call (identifier::)
                        if new_position < self.input.len()
                            && self.input[new_position] == ':'
                            && new_position + 1 < self.input.len()
                            && self.input[new_position + 1] == ':'
                        {
                            // Treat as identifier for namespace calls
                            Ok((
                                new_position,
                                new_line,
                                new_column,
                                Token::Identifier(identifier),
                            ))
                        } else if let Some(keyword) = self.is_keyword(&identifier) {
                            Ok((new_position, new_line, new_column, Token::Keyword(keyword)))
                        } else {
                            Ok((
                                new_position,
                                new_line,
                                new_column,
                                Token::Identifier(identifier),
                            ))
                        }
                    }
                }
            }

            // Numbers
            '0'..='9' => {
                let (pos, literal) = self.read_number_immutable(position)?;
                new_position = pos;
                Ok((new_position, new_line, new_column, Token::Literal(literal)))
            }

            // Strings
            '"' => {
                let (pos, final_line, final_col, string) =
                    self.read_string_immutable_with_positions(position, new_line, new_column)?;
                new_position = pos;
                Ok((
                    new_position,
                    final_line,
                    final_col,
                    Token::Literal(Literal::String(string)),
                ))
            }

            // Operators and punctuation
            '+' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Operator(Operator::Plus),
                ))
            }
            '-' => {
                if position + 1 < self.input.len() && self.input[position + 1] == '>' {
                    new_position += 2;
                    new_column += 2;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Punctuation(Punctuation::Arrow),
                    ))
                } else {
                    new_position += 1;
                    new_column += 1;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Operator(Operator::Minus),
                    ))
                }
            }
            '*' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Operator(Operator::Star),
                ))
            }
            '/' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Operator(Operator::Slash),
                ))
            }
            '%' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Operator(Operator::Percent),
                ))
            }

            '@' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Punctuation(Punctuation::At),
                ))
            }

            '=' => {
                if position + 1 < self.input.len() && self.input[position + 1] == '>' {
                    new_position += 2;
                    new_column += 2;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Punctuation(Punctuation::FatArrow),
                    ))
                } else if position + 1 < self.input.len() && self.input[position + 1] == '=' {
                    new_position += 2;
                    new_column += 2;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Operator(Operator::Equal),
                    ))
                } else {
                    new_position += 1;
                    new_column += 1;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Operator(Operator::Assign),
                    ))
                }
            }

            '!' => {
                if position + 1 < self.input.len() && self.input[position + 1] == '=' {
                    new_position += 2;
                    new_column += 2;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Operator(Operator::NotEqual),
                    ))
                } else {
                    new_position += 1;
                    new_column += 1;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Operator(Operator::Not),
                    ))
                }
            }

            '<' => {
                if position + 1 < self.input.len() && self.input[position + 1] == '=' {
                    new_position += 2;
                    new_column += 2;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Operator(Operator::LessEqual),
                    ))
                } else {
                    new_position += 1;
                    new_column += 1;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Operator(Operator::Less),
                    ))
                }
            }

            '>' => {
                if position + 1 < self.input.len() && self.input[position + 1] == '=' {
                    new_position += 2;
                    new_column += 2;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Operator(Operator::GreaterEqual),
                    ))
                } else {
                    new_position += 1;
                    new_column += 1;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Operator(Operator::Greater),
                    ))
                }
            }

            '&' => {
                if position + 1 < self.input.len() && self.input[position + 1] == '&' {
                    new_position += 2;
                    new_column += 2;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Operator(Operator::And),
                    ))
                } else {
                    Err(LexerError::UnexpectedCharacter(ch, line, column))
                }
            }

            '|' => {
                if position + 1 < self.input.len() && self.input[position + 1] == '|' {
                    new_position += 2;
                    new_column += 2;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Operator(Operator::Or),
                    ))
                } else {
                    Err(LexerError::UnexpectedCharacter(ch, line, column))
                }
            }

            ':' => {
                if position + 1 < self.input.len() && self.input[position + 1] == ':' {
                    new_position += 2;
                    new_column += 2;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Punctuation(Punctuation::DoubleColon),
                    ))
                } else {
                    new_position += 1;
                    new_column += 1;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Punctuation(Punctuation::Colon),
                    ))
                }
            }

            '.' => {
                // Check for range operator (..)
                if position + 1 < self.input.len() && self.input[position + 1] == '.' {
                    new_position += 2;
                    new_column += 2;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Punctuation(Punctuation::DotDot),
                    ))
                } else {
                    new_position += 1;
                    new_column += 1;
                    Ok((
                        new_position,
                        new_line,
                        new_column,
                        Token::Punctuation(Punctuation::Dot),
                    ))
                }
            }
            '?' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Punctuation(Punctuation::Question),
                ))
            }
            ',' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Punctuation(Punctuation::Comma),
                ))
            }
            ';' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Punctuation(Punctuation::Semicolon),
                ))
            }
            '[' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Punctuation(Punctuation::LeftBracket),
                ))
            }
            ']' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Punctuation(Punctuation::RightBracket),
                ))
            }
            '{' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Punctuation(Punctuation::LeftBrace),
                ))
            }
            '}' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Punctuation(Punctuation::RightBrace),
                ))
            }
            '(' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Punctuation(Punctuation::LeftParen),
                ))
            }
            ')' => {
                new_position += 1;
                new_column += 1;
                Ok((
                    new_position,
                    new_line,
                    new_column,
                    Token::Punctuation(Punctuation::RightParen),
                ))
            }

            _ => Err(LexerError::UnexpectedCharacter(ch, line, column)),
        }
        .map(|(pos, ln, col, token)| {
            // Safety: Ensure position always advances and column is always >= 1
            // This prevents infinite loops from mutations that cause position to not advance
            if pos <= position {
                Err(LexerError::UnexpectedCharacter('\0', line, column))
            } else {
                Ok((pos, ln, col.max(1), token))
            }
        })
        .and_then(|x| x)
    }

    fn read_identifier(&mut self) -> String {
        let start = self.position;

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        self.input[start..self.position].iter().collect()
    }

    fn read_number(&mut self) -> Result<i64, LexerError> {
        let start = self.position;

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        let number_str: String = self.input[start..self.position].iter().collect();
        number_str
            .parse()
            .map_err(|_| LexerError::InvalidNumber(number_str, self.line, self.column))
    }

    fn read_string(&mut self) -> Result<String, LexerError> {
        self.advance(); // Skip opening quote

        let mut string = String::new();
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch == '"' {
                self.advance(); // Skip closing quote
                return Ok(string);
            } else if ch == '\\' {
                self.advance();
                if self.position < self.input.len() {
                    let decoded = decode_escape(self.current_char());
                    string.push(decoded);
                    self.advance();
                }
            } else {
                string.push(ch);
                self.advance();
            }
        }

        Err(LexerError::UnterminatedString(self.line, self.column))
    }

    fn skip_whitespace(&mut self) {
        let (new_pos, new_line, new_col) =
            self.skip_whitespace_immutable(self.position, self.line, self.column);
        self.position = new_pos;
        self.line = new_line;
        self.column = new_col;
    }

    fn skip_whitespace_immutable(
        &self,
        mut position: usize,
        mut line: usize,
        mut column: usize,
    ) -> (usize, usize, usize) {
        while position < self.input.len() {
            let ch = self.input[position];
            match ch {
                ' ' | '\t' => {
                    position += 1;
                    column += 1;
                }
                '\n' => {
                    line += 1;
                    column = 1;
                    position += 1;
                }
                '\r' => {
                    if position + 1 < self.input.len() && self.input[position + 1] == '\n' {
                        position += 1;
                    }
                    line += 1;
                    column = 1;
                    position += 1;
                }
                '/' => {
                    // Check for comment: // or /*
                    if position + 1 < self.input.len() {
                        let next_ch = self.input[position + 1];
                        if next_ch == '/' {
                            // Single-line comment: skip until end of line
                            position += 2; // Skip //
                            while position < self.input.len() {
                                let comment_ch = self.input[position];
                                if comment_ch == '\n' {
                                    line += 1;
                                    column = 1;
                                    position += 1;
                                    break;
                                } else if comment_ch == '\r' {
                                    if position + 1 < self.input.len()
                                        && self.input[position + 1] == '\n'
                                    {
                                        position += 1;
                                    }
                                    line += 1;
                                    column = 1;
                                    position += 1;
                                    break;
                                } else {
                                    position += 1;
                                }
                            }
                        } else if next_ch == '*' {
                            // Multi-line comment: skip until */
                            position += 2; // Skip /*
                            while position + 1 < self.input.len() {
                                if self.input[position] == '*' && self.input[position + 1] == '/' {
                                    position += 2; // Skip */
                                    break;
                                } else if self.input[position] == '\n' {
                                    line += 1;
                                    column = 1;
                                    position += 1;
                                } else if self.input[position] == '\r' {
                                    if position + 1 < self.input.len()
                                        && self.input[position + 1] == '\n'
                                    {
                                        position += 1;
                                    }
                                    line += 1;
                                    column = 1;
                                    position += 1;
                                } else {
                                    position += 1;
                                }
                            }
                        } else {
                            // Not a comment, just a division operator
                            break;
                        }
                    } else {
                        // Single / at end of input, treat as operator
                        break;
                    }
                }
                _ => break,
            }
        }
        (position, line, column)
    }

    fn is_keyword(&self, identifier: &str) -> Option<Keyword> {
        let keywords: std::collections::HashMap<&str, Keyword> = [
            ("fn", Keyword::Fn),
            ("let", Keyword::Let),
            ("mut", Keyword::Mut),
            ("if", Keyword::If),
            ("else", Keyword::Else),
            ("match", Keyword::Match),
            ("default", Keyword::Default),
            ("for", Keyword::For),
            ("in", Keyword::In),
            ("while", Keyword::While),
            ("loop", Keyword::Loop),
            ("break", Keyword::Break),
            ("continue", Keyword::Continue),
            ("return", Keyword::Return),
            ("struct", Keyword::Struct),
            ("enum", Keyword::Enum),
            ("import", Keyword::Import),
            ("export", Keyword::Export),
            ("as", Keyword::As),
            ("pub", Keyword::Pub),
            ("private", Keyword::Private),
            ("spawn", Keyword::Spawn),
            ("await", Keyword::Await),
            ("agent", Keyword::Agent),
            ("msg", Keyword::Msg),
            ("event", Keyword::Event),
            ("service", Keyword::Service),
            ("with", Keyword::With),
            ("async", Keyword::Async),
            ("try", Keyword::Try),
            ("catch", Keyword::Catch),
            ("throw", Keyword::Throw),
            ("finally", Keyword::Finally),
            // Attributes
            ("secure", Keyword::Secure),
            ("audit", Keyword::Audit),
            ("txn", Keyword::Txn),
            ("limit", Keyword::Limit),
            ("trust", Keyword::Trust),
            // ("web", Keyword::Web), // Treat as identifier for namespace calls
            ("mobile", Keyword::Mobile),
            ("desktop", Keyword::Desktop),
            ("iot", Keyword::Iot),
            ("ai", Keyword::Ai),
            ("persistent", Keyword::Persistent),
            ("cached", Keyword::Cached),
            ("versioned", Keyword::Versioned),
            ("deprecated", Keyword::Deprecated),
            ("compile_target", Keyword::CompileTarget), // NEW: Compilation target
            ("chain", Keyword::Chain),                  // NEW: Chain keyword
            ("interface", Keyword::Interface),          // NEW: Interface keyword
        ]
        .iter()
        .cloned()
        .collect();

        keywords.get(identifier).cloned()
    }

    fn current_char(&self) -> char {
        self.input[self.position]
    }

    fn peek_next(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.position += 1;
            self.column += 1;
        }
    }

    // Immutable helper methods
    fn read_identifier_immutable(&self, mut position: usize) -> (usize, String) {
        let start = position;
        while position < self.input.len() {
            let ch = self.input[position];
            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                position += 1;
            } else {
                break;
            }
        }
        let identifier = self.input[start..position].iter().collect();
        (position, identifier)
    }

    fn read_number_immutable(&self, mut position: usize) -> Result<(usize, Literal), LexerError> {
        let start = position;
        let mut has_decimal = false;

        // Read integer part
        while position < self.input.len() {
            let ch = self.input[position];
            if ch.is_ascii_digit() {
                position += 1;
            } else {
                break;
            }
        }

        // Check for decimal point
        if position < self.input.len() && self.input[position] == '.' {
            // Look ahead to ensure there's a digit after the dot (not a method call like `0.toString()`)
            if position + 1 < self.input.len() && self.input[position + 1].is_ascii_digit() {
                has_decimal = true;
                position += 1; // consume '.'

                // Read fractional part
                while position < self.input.len() {
                    let ch = self.input[position];
                    if ch.is_ascii_digit() {
                        position += 1;
                    } else {
                        break;
                    }
                }
            }
        }

        let number_str: String = self.input[start..position].iter().collect();

        if has_decimal {
            match number_str.parse::<f64>() {
                Ok(num) => Ok((position, Literal::Float(num))),
                Err(_) => Err(LexerError::InvalidNumber(number_str, 0, 0)),
            }
        } else {
            match number_str.parse::<i64>() {
                Ok(num) => Ok((position, Literal::Int(num))),
                Err(_) => Err(LexerError::InvalidNumber(number_str, 0, 0)),
            }
        }
    }

    fn read_string_immutable(&self, mut position: usize) -> Result<(usize, String), LexerError> {
        position += 1; // Skip opening quote
        let mut string = String::new();
        while position < self.input.len() {
            let ch = self.input[position];
            if ch == '"' {
                position += 1; // Skip closing quote
                return Ok((position, string));
            } else if ch == '\\' {
                position += 1;
                if position < self.input.len() {
                    let decoded = decode_escape(self.input[position]);
                    string.push(decoded);
                    position += 1;
                }
            } else {
                string.push(ch);
                position += 1;
            }
        }
        Err(LexerError::UnterminatedString(0, 0))
    }

    fn read_string_immutable_with_positions(
        &self,
        mut position: usize,
        mut line: usize,
        mut column: usize,
    ) -> Result<(usize, usize, usize, String), LexerError> {
        position += 1; // Skip opening quote
        column += 1;
        let mut string = String::new();
        while position < self.input.len() {
            let ch = self.input[position];
            if ch == '"' {
                position += 1; // Skip closing quote
                column += 1;
                return Ok((position, line, column, string));
            } else if ch == '\\' {
                position += 1;
                column += 1;
                if position < self.input.len() {
                    let next_ch = self.input[position];
                    if next_ch == '\n' {
                        line += 1;
                        column = 1;
                    } else {
                        column += 1;
                    }
                    string.push(decode_escape(next_ch));
                    position += 1;
                }
            } else if ch == '\n' {
                position += 1;
                line += 1;
                column = 1;
                string.push(ch);
            } else if ch == '\r' {
                position += 1;
                if position < self.input.len() && self.input[position] == '\n' {
                    position += 1;
                }
                line += 1;
                column = 1;
                string.push('\r');
            } else {
                position += 1;
                column += 1;
                string.push(ch);
            }
        }
        Err(LexerError::UnterminatedString(line, column))
    }
}

/// Decode a single escape sequence character (the char after the backslash).
fn decode_escape(ch: char) -> char {
    match ch {
        '"' => '"',
        '\\' => '\\',
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        _ => ch, // Unknown escape: pass through (e.g. \x -> x)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LexerError {
    #[error("Unexpected character '{0}' at line {1}, column {2}")]
    UnexpectedCharacter(char, usize, usize),

    #[error("Invalid number '{0}' at line {1}, column {2}")]
    InvalidNumber(String, usize, usize),

    #[error("Unterminated string at line {0}, column {1}")]
    UnterminatedString(usize, usize),

    #[error("Too many tokens: {2} (max: {3}) at line {0}, column {1}")]
    TooManyTokens(usize, usize, usize, usize),
}

impl LexerError {
    /// Line and column for source snippet display (1-based). Returns None for errors without location.
    pub fn line_column(&self) -> Option<(usize, usize)> {
        match self {
            LexerError::UnexpectedCharacter(_, line, col) => Some((*line, *col)),
            LexerError::InvalidNumber(_, line, col) => Some((*line, *col)),
            LexerError::UnterminatedString(line, col) => Some((*line, *col)),
            LexerError::TooManyTokens(line, col, _, _) => Some((*line, *col)),
        }
    }
}
