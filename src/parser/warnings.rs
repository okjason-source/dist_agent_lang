//! Parse-time warnings (unused variables and similar).
//!
//! Collected by walking the AST after a successful parse. Does not fail the parse;
//! expose via CLI (e.g. after run/check/lint) and API.

use crate::parser::ast::*;
use std::collections::HashMap;

/// A non-fatal parse warning (e.g. unused variable).
#[derive(Debug, Clone)]
pub struct ParseWarning {
    pub message: String,
    /// 1-based line number; 0 if unknown.
    pub line: usize,
}

/// Collect unused-variable (and similar) warnings by walking the AST.
pub fn collect_warnings(program: &Program) -> Vec<ParseWarning> {
    let mut pass = WarningPass {
        warnings: Vec::new(),
        scopes: Vec::new(),
    };
    pass.visit_program(program);
    pass.warnings
}

struct WarningPass {
    warnings: Vec<ParseWarning>,
    /// Stack of scopes: each scope maps bound name -> (line, used).
    scopes: Vec<HashMap<String, (usize, bool)>>,
}

impl WarningPass {
    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            for (name, (line, used)) in scope {
                if !used {
                    self.warnings.push(ParseWarning {
                        message: format!("unused variable '{}'", name),
                        line,
                    });
                }
            }
        }
    }

    fn bind(&mut self, name: String, line: usize) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, (line, false));
        }
    }

    fn mark_used(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some((_line, used)) = scope.get_mut(name) {
                *used = true;
                return;
            }
        }
    }

    fn visit_program(&mut self, program: &Program) {
        self.push_scope();
        for stmt in &program.statements {
            self.visit_statement(stmt);
        }
        self.pop_scope();
    }

    fn visit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let(LetStatement { name, value, line }) => {
                let line = line.unwrap_or(0);
                self.bind(name.clone(), line);
                self.visit_expression(value);
            }
            Statement::Expression(expr) => self.visit_expression(expr),
            Statement::Return(ReturnStatement { value }) => {
                if let Some(expr) = value {
                    self.visit_expression(expr);
                }
            }
            Statement::Block(block) => {
                self.push_scope();
                for s in &block.statements {
                    self.visit_statement(s);
                }
                self.pop_scope();
            }
            Statement::Function(FunctionStatement {
                parameters, body, ..
            }) => {
                self.push_scope();
                for param in parameters {
                    self.bind(param.name.clone(), 0);
                }
                for s in &body.statements {
                    self.visit_statement(s);
                }
                self.pop_scope();
            }
            Statement::Service(ServiceStatement { methods, .. }) => {
                for method in methods {
                    self.push_scope();
                    for param in &method.parameters {
                        self.bind(param.name.clone(), 0);
                    }
                    for s in &method.body.statements {
                        self.visit_statement(s);
                    }
                    self.pop_scope();
                }
            }
            Statement::Spawn(SpawnStatement { body, .. })
            | Statement::Agent(AgentStatement { body, .. }) => {
                self.push_scope();
                for s in &body.statements {
                    self.visit_statement(s);
                }
                self.pop_scope();
            }
            Statement::If(IfStatement {
                condition,
                consequence,
                alternative,
            }) => {
                self.visit_expression(condition);
                self.push_scope();
                for s in &consequence.statements {
                    self.visit_statement(s);
                }
                self.pop_scope();
                if let Some(alt) = alternative {
                    self.push_scope();
                    for s in &alt.statements {
                        self.visit_statement(s);
                    }
                    self.pop_scope();
                }
            }
            Statement::While(WhileStatement { condition, body }) => {
                self.visit_expression(condition);
                self.push_scope();
                for s in &body.statements {
                    self.visit_statement(s);
                }
                self.pop_scope();
            }
            Statement::Loop(LoopStatement { body }) => {
                self.push_scope();
                for s in &body.statements {
                    self.visit_statement(s);
                }
                self.pop_scope();
            }
            Statement::ForIn(ForInStatement {
                variable,
                iterable,
                body,
            }) => {
                self.visit_expression(iterable);
                self.push_scope();
                self.bind(variable.clone(), 0);
                for s in &body.statements {
                    self.visit_statement(s);
                }
                self.pop_scope();
            }
            Statement::Try(TryStatement {
                try_block,
                catch_blocks,
                finally_block,
            }) => {
                self.push_scope();
                for s in &try_block.statements {
                    self.visit_statement(s);
                }
                self.pop_scope();
                for catch in catch_blocks {
                    self.push_scope();
                    if let Some(var) = &catch.error_variable {
                        self.bind(var.clone(), 0);
                    }
                    for s in &catch.body.statements {
                        self.visit_statement(s);
                    }
                    self.pop_scope();
                }
                if let Some(finally) = finally_block {
                    self.push_scope();
                    for s in &finally.statements {
                        self.visit_statement(s);
                    }
                    self.pop_scope();
                }
            }
            Statement::Match(MatchStatement {
                expression,
                cases,
                default_case,
            }) => {
                self.visit_expression(expression);
                for case in cases {
                    self.push_scope();
                    if let MatchPattern::Identifier(name) = &case.pattern {
                        self.bind(name.clone(), 0);
                    }
                    for s in &case.body.statements {
                        self.visit_statement(s);
                    }
                    self.pop_scope();
                }
                if let Some(default) = default_case {
                    self.push_scope();
                    for s in &default.statements {
                        self.visit_statement(s);
                    }
                    self.pop_scope();
                }
            }
            Statement::Message(MessageStatement { data, .. })
            | Statement::Event(EventStatement { data, .. }) => {
                for expr in data.values() {
                    self.visit_expression(expr);
                }
            }
            Statement::Break(BreakStatement { value }) => {
                if let Some(expr) = value {
                    self.visit_expression(expr);
                }
            }
            Statement::Continue(ContinueStatement) => {}
            Statement::Import(_) => {}
        }
    }

    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Identifier(name) => {
                self.mark_used(name);
            }
            Expression::Assignment(name, value) => {
                self.mark_used(name);
                self.visit_expression(value);
            }
            Expression::Literal(_) => {}
            Expression::BinaryOp(l, _op, r) => {
                self.visit_expression(l);
                self.visit_expression(r);
            }
            Expression::UnaryOp(_op, e) => self.visit_expression(e),
            Expression::FunctionCall(FunctionCall { arguments, .. }) => {
                for arg in arguments {
                    self.visit_expression(arg);
                }
            }
            Expression::FieldAccess(e, _) | Expression::Await(e) | Expression::Spawn(e) => {
                self.visit_expression(e);
            }
            Expression::FieldAssignment(e1, _f, e2) => {
                self.visit_expression(e1);
                self.visit_expression(e2);
            }
            Expression::Throw(e) => self.visit_expression(e),
            Expression::ObjectLiteral(map) => {
                for e in map.values() {
                    self.visit_expression(e);
                }
            }
            Expression::ArrayLiteral(list) => {
                for e in list {
                    self.visit_expression(e);
                }
            }
            Expression::IndexAccess(e1, e2) => {
                self.visit_expression(e1);
                self.visit_expression(e2);
            }
            Expression::ArrowFunction { param, body } => {
                self.push_scope();
                self.bind(param.clone(), 0);
                for s in &body.statements {
                    self.visit_statement(s);
                }
                self.pop_scope();
            }
            Expression::Range(e1, e2) => {
                self.visit_expression(e1);
                self.visit_expression(e2);
            }
        }
    }
}
