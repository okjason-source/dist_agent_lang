use crate::lexer::tokens::Token;
use std::collections::VecDeque;
use thiserror::Error;

/// Enhanced error context for better debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub file_path: Option<String>,
    pub source_code: Option<String>,
    pub call_stack: VecDeque<String>,
    pub variable_scope: Vec<String>,
    pub suggestions: Vec<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            file_path: None,
            source_code: None,
            call_stack: VecDeque::new(),
            variable_scope: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn with_file_path(mut self, file_path: String) -> Self {
        self.file_path = Some(file_path);
        self
    }

    pub fn with_source_code(mut self, source_code: String) -> Self {
        self.source_code = Some(source_code);
        self
    }

    pub fn add_call_stack(mut self, function: String) -> Self {
        self.call_stack.push_front(function);
        self
    }

    pub fn add_variable_scope(mut self, variable: String) -> Self {
        self.variable_scope.push(variable);
        self
    }

    pub fn add_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Set suggestions (replaces any existing). Used when attaching generated suggestions to parse errors.
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }
}

/// Comprehensive error types with context
#[derive(Error, Debug, Clone)]
pub enum ParserError {
    #[error("Syntax Error: Unexpected token '{token}' at line {line}, column {column}. Expected one of: {expected}")]
    UnexpectedToken {
        token: String,
        expected: String,
        line: usize,
        column: usize,
        context: ErrorContext,
    },

    #[error("Syntax Error: Unexpected end of file. Expected: {expected}")]
    UnexpectedEOF {
        expected: String,
        context: ErrorContext,
    },

    #[error("Attribute Error: Invalid attribute '{attribute}' at line {line}")]
    InvalidAttribute {
        attribute: String,
        line: usize,
        context: ErrorContext,
    },

    #[error("Syntax Error: Missing closing brace for function '{function}' at line {line}")]
    MissingClosingBrace {
        function: String,
        line: usize,
        context: ErrorContext,
    },

    #[error("Function Error: Invalid function call '{function}' at line {line}: {reason}")]
    InvalidFunctionCall {
        function: String,
        reason: String,
        line: usize,
        context: ErrorContext,
    },

    #[error("Type Error: Type mismatch at line {line}: expected {expected}, got {got}")]
    TypeMismatch {
        expected: String,
        got: String,
        line: usize,
        context: ErrorContext,
    },

    #[error("Semantic Error: {message} at line {line}")]
    SemanticError {
        message: String,
        line: usize,
        context: ErrorContext,
    },

    #[error("Recovery Error: Failed to recover from previous error")]
    RecoveryError {
        original_error: Box<ParserError>,
        context: ErrorContext,
    },
}

impl ParserError {
    pub fn unexpected_token(token: &Token, expected: &[&str], line: usize, column: usize) -> Self {
        let expected_str = expected.join(", ");
        let token_str = format!("{:?}", token);

        ParserError::UnexpectedToken {
            token: token_str,
            expected: expected_str,
            line,
            column,
            context: ErrorContext::new(),
        }
    }

    pub fn unexpected_eof(expected: &str) -> Self {
        ParserError::UnexpectedEOF {
            expected: expected.to_string(),
            context: ErrorContext::new(),
        }
    }

    pub fn invalid_attribute(attribute: &str, line: usize) -> Self {
        ParserError::InvalidAttribute {
            attribute: attribute.to_string(),
            line,
            context: ErrorContext::new(),
        }
    }

    pub fn missing_closing_brace(function: &str, line: usize) -> Self {
        ParserError::MissingClosingBrace {
            function: function.to_string(),
            line,
            context: ErrorContext::new(),
        }
    }

    pub fn invalid_function_call(function: &str, reason: &str, line: usize) -> Self {
        ParserError::InvalidFunctionCall {
            function: function.to_string(),
            reason: reason.to_string(),
            line,
            context: ErrorContext::new(),
        }
    }

    pub fn type_mismatch(expected: &str, got: &str, line: usize) -> Self {
        ParserError::TypeMismatch {
            expected: expected.to_string(),
            got: got.to_string(),
            line,
            context: ErrorContext::new(),
        }
    }

    pub fn semantic_error(message: &str, line: usize) -> Self {
        ParserError::SemanticError {
            message: message.to_string(),
            line,
            context: ErrorContext::new(),
        }
    }

    /// Add context to an error
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        match &mut self {
            ParserError::UnexpectedToken { context: ctx, .. } => *ctx = context,
            ParserError::UnexpectedEOF { context: ctx, .. } => *ctx = context,
            ParserError::InvalidAttribute { context: ctx, .. } => *ctx = context,
            ParserError::MissingClosingBrace { context: ctx, .. } => *ctx = context,
            ParserError::InvalidFunctionCall { context: ctx, .. } => *ctx = context,
            ParserError::TypeMismatch { context: ctx, .. } => *ctx = context,
            ParserError::SemanticError { context: ctx, .. } => *ctx = context,
            ParserError::RecoveryError { context: ctx, .. } => *ctx = context,
        }
        self
    }

    /// Get line number from error
    pub fn line_number(&self) -> Option<usize> {
        match self {
            ParserError::UnexpectedToken { line, .. } => Some(*line),
            ParserError::InvalidAttribute { line, .. } => Some(*line),
            ParserError::MissingClosingBrace { line, .. } => Some(*line),
            ParserError::InvalidFunctionCall { line, .. } => Some(*line),
            ParserError::TypeMismatch { line, .. } => Some(*line),
            ParserError::SemanticError { line, .. } => Some(*line),
            _ => None,
        }
    }

    /// Get column number from error
    pub fn column_number(&self) -> Option<usize> {
        match self {
            ParserError::UnexpectedToken { column, .. } => Some(*column),
            _ => None,
        }
    }

    /// Get error context
    pub fn context(&self) -> &ErrorContext {
        match self {
            ParserError::UnexpectedToken { context, .. } => context,
            ParserError::UnexpectedEOF { context, .. } => context,
            ParserError::InvalidAttribute { context, .. } => context,
            ParserError::MissingClosingBrace { context, .. } => context,
            ParserError::InvalidFunctionCall { context, .. } => context,
            ParserError::TypeMismatch { context, .. } => context,
            ParserError::SemanticError { context, .. } => context,
            ParserError::RecoveryError { context, .. } => context,
        }
    }

    /// Generate actionable suggestions for this error (used by CLI/IDE to show "what to do").
    /// Call at the point of display; no parser instance required.
    pub fn generate_suggestions(&self) -> Vec<String> {
        match self {
            ParserError::UnexpectedToken { expected, .. } => vec![
                format!("Check if you meant one of: {}", expected),
                "Make sure all parentheses and braces are properly closed".to_string(),
                "Verify that all keywords are spelled correctly".to_string(),
            ],
            ParserError::MissingClosingBrace { .. } => vec![
                "Add a closing brace '}' to match the opening brace".to_string(),
                "Check for nested braces and ensure they're properly paired".to_string(),
            ],
            ParserError::InvalidFunctionCall { .. } => vec![
                "Check that the function name is correct".to_string(),
                "Verify that all required arguments are provided".to_string(),
                "Ensure function arguments match the expected types".to_string(),
            ],
            ParserError::RecoveryError { original_error, .. } => {
                original_error.generate_suggestions()
            }
            _ => vec![
                "Review the syntax and ensure it follows the language specification".to_string(),
            ],
        }
    }

    /// Format error with source code context (file path, source line, caret, suggestions).
    pub fn format_with_source(&self) -> String {
        let mut output = String::new();

        // File path first (production-style: user sees which file immediately)
        if let Some(path) = &self.context().file_path {
            if !path.is_empty() {
                output.push_str(&format!("  --> {}\n", path));
            }
        }

        output.push_str(&format!("{}\n", self));

        if let Some(line_num) = self.line_number() {
            if let Some(source) = &self.context().source_code {
                let lines: Vec<&str> = source.lines().collect();
                if line_num > 0 && line_num <= lines.len() {
                    let line_content = lines[line_num - 1];
                    let prefix = format!("  --> Line {}: ", line_num);
                    output.push_str(&format!("{}{}\n", prefix, line_content));

                    if let Some(col) = self.column_number() {
                        // Caret under the offending column (column is 1-based)
                        let pad = prefix.len() + col.saturating_sub(1);
                        let pointer = " ".repeat(pad) + "^";
                        output.push_str(&format!("  {}\n", pointer));
                    }
                }
            }
        }

        // Add call stack if available
        if !self.context().call_stack.is_empty() {
            output.push_str("\nCall Stack:\n");
            for (i, call) in self.context().call_stack.iter().enumerate() {
                output.push_str(&format!("  {}: {}\n", i, call));
            }
        }

        // Add suggestions if available
        if !self.context().suggestions.is_empty() {
            output.push_str("\nSuggestions:\n");
            for suggestion in &self.context().suggestions {
                output.push_str(&format!("  • {}\n", suggestion));
            }
        }

        output
    }
}

/// Error recovery strategies
pub trait ErrorRecovery {
    fn recover_from_error(&mut self, error: &ParserError) -> Result<(), ParserError>;
    fn skip_to_synchronization_point(&mut self) -> bool;
    fn insert_missing_token(&mut self, token: Token) -> bool;
    fn generate_suggestions(&self, error: &ParserError) -> Vec<String>;
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Warning,
    Error,
    Fatal,
}

/// Error reporting interface
pub trait ErrorReporter {
    fn report_error(&mut self, error: ParserError);
    fn report_warning(&mut self, message: String, line: usize);
    fn has_errors(&self) -> bool;
    fn get_errors(&self) -> Vec<ParserError>;
    fn clear_errors(&mut self);
}

/// Simple error reporter implementation
pub struct SimpleErrorReporter {
    errors: Vec<ParserError>,
    warnings: Vec<(String, usize)>,
}

impl SimpleErrorReporter {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl ErrorReporter for SimpleErrorReporter {
    fn report_error(&mut self, error: ParserError) {
        self.errors.push(error);
    }

    fn report_warning(&mut self, message: String, line: usize) {
        self.warnings.push((message, line));
    }

    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    fn get_errors(&self) -> Vec<ParserError> {
        self.errors.clone()
    }

    fn clear_errors(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }
}

impl ErrorRecovery for crate::parser::Parser {
    fn recover_from_error(&mut self, error: &ParserError) -> Result<(), ParserError> {
        if self.skip_to_synchronization_point() {
            Ok(())
        } else {
            Err(ParserError::RecoveryError {
                original_error: Box::new(error.clone()),
                context: ErrorContext::new(),
            })
        }
    }

    fn skip_to_synchronization_point(&mut self) -> bool {
        let start = match self.recovery_skip_from.take() {
            Some(p) => p,
            None => return false,
        };
        self.skip_to_sync_point_from(start)
    }

    /// No-op: recovery uses skip-to-sync only (no token insertion). Returns true for API compatibility.
    fn insert_missing_token(&mut self, _token: Token) -> bool {
        true
    }

    fn generate_suggestions(&self, error: &ParserError) -> Vec<String> {
        error.generate_suggestions()
    }
}
