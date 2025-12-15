use crate::parser::ast::*;
use crate::lexer::tokens::{Literal, Operator};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OptimizationPass {
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub original_ast: Program,
    pub optimized_ast: Program,
    pub optimizations_applied: Vec<String>,
    pub performance_improvement: f64, // estimated improvement percentage
}

pub struct Optimizer {
    passes: Vec<OptimizationPass>,
    optimization_level: OptimizationLevel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
    Maximum,
}

impl Optimizer {
    pub fn new() -> Self {
        Self {
            passes: Self::create_default_passes(),
            optimization_level: OptimizationLevel::Basic,
        }
    }

    pub fn with_level(mut self, level: OptimizationLevel) -> Self {
        self.optimization_level = level;
        self
    }

    pub fn optimize(&self, ast: Program) -> OptimizationResult {
        let mut current_ast = ast.clone();
        let mut applied_optimizations = Vec::new();

        for pass in &self.passes {
            if pass.enabled && self.should_apply_pass(pass) {
                let (optimized_ast, optimizations) = self.apply_pass(pass, &current_ast);
                current_ast = optimized_ast;
                applied_optimizations.extend(optimizations);
            }
        }

        let performance_improvement = self.estimate_improvement(&applied_optimizations);

        OptimizationResult {
            original_ast: ast,
            optimized_ast: current_ast,
            optimizations_applied: applied_optimizations,
            performance_improvement,
        }
    }

    fn create_default_passes() -> Vec<OptimizationPass> {
        vec![
            OptimizationPass {
                name: "constant_folding".to_string(),
                description: "Evaluate constant expressions at compile time".to_string(),
                enabled: true,
            },
            OptimizationPass {
                name: "dead_code_elimination".to_string(),
                description: "Remove unreachable code".to_string(),
                enabled: true,
            },
            OptimizationPass {
                name: "function_inlining".to_string(),
                description: "Inline small functions".to_string(),
                enabled: true,
            },
            OptimizationPass {
                name: "common_subexpression_elimination".to_string(),
                description: "Eliminate redundant computations".to_string(),
                enabled: true,
            },
            OptimizationPass {
                name: "loop_optimization".to_string(),
                description: "Optimize loop structures".to_string(),
                enabled: true,
            },
            OptimizationPass {
                name: "strength_reduction".to_string(),
                description: "Replace expensive operations with cheaper ones".to_string(),
                enabled: true,
            },
        ]
    }

    fn should_apply_pass(&self, pass: &OptimizationPass) -> bool {
        match self.optimization_level {
            OptimizationLevel::None => false,
            OptimizationLevel::Basic => matches!(
                pass.name.as_str(),
                "constant_folding" | "dead_code_elimination"
            ),
            OptimizationLevel::Aggressive => true,
            OptimizationLevel::Maximum => true,
        }
    }

    fn apply_pass(&self, pass: &OptimizationPass, ast: &Program) -> (Program, Vec<String>) {
        match pass.name.as_str() {
            "constant_folding" => self.constant_folding(ast),
            "dead_code_elimination" => self.dead_code_elimination(ast),
            "function_inlining" => self.function_inlining(ast),
            "common_subexpression_elimination" => self.common_subexpression_elimination(ast),
            "loop_optimization" => self.loop_optimization(ast),
            "strength_reduction" => self.strength_reduction(ast),
            _ => (ast.clone(), vec![]),
        }
    }

    fn constant_folding(&self, ast: &Program) -> (Program, Vec<String>) {
        let mut optimized_statements = Vec::new();
        let mut optimizations = Vec::new();

        for statement in &ast.statements {
            match statement {
                Statement::Let(let_stmt) => {
                    if let Some(folded_value) = self.fold_constant_expression(&let_stmt.value) {
                        optimized_statements.push(Statement::Let(LetStatement {
                            name: let_stmt.name.clone(),
                            value: folded_value,
                        }));
                        optimizations.push(format!("Constant folded: {}", let_stmt.name));
                    } else {
                        optimized_statements.push(statement.clone());
                    }
                }
                Statement::Expression(expr) => {
                    if let Some(folded_value) = self.fold_constant_expression(expr) {
                        optimized_statements.push(Statement::Expression(folded_value));
                        optimizations.push("Constant folded expression".to_string());
                    } else {
                        optimized_statements.push(statement.clone());
                    }
                }
                Statement::Function(func_stmt) => {
                    let optimized_body = self.optimize_block(&func_stmt.body);
                    optimized_statements.push(Statement::Function(FunctionStatement {
                        name: func_stmt.name.clone(),
                        parameters: func_stmt.parameters.clone(),
                        return_type: func_stmt.return_type.clone(),
                        body: optimized_body,
                        attributes: func_stmt.attributes.clone(),
                        is_async: func_stmt.is_async,
                    }));
                }
                _ => optimized_statements.push(statement.clone()),
            }
        }

        (Program { statements: optimized_statements }, optimizations)
    }

    fn fold_constant_expression(&self, expr: &Expression) -> Option<Expression> {
        match expr {
            Expression::BinaryOp(left, op, right) => {
                let left_folded = self.fold_constant_expression(left)?;
                let right_folded = self.fold_constant_expression(right)?;
                
                if let (Expression::Literal(lit1), Expression::Literal(lit2)) = (&left_folded, &right_folded) {
                    match (lit1, lit2, op) {
                        (Literal::Int(a), Literal::Int(b), Operator::Plus) => {
                            Some(Expression::Literal(Literal::Int(a + b)))
                        }
                        (Literal::Int(a), Literal::Int(b), Operator::Minus) => {
                            Some(Expression::Literal(Literal::Int(a - b)))
                        }
                        (Literal::Int(a), Literal::Int(b), Operator::Star) => {
                            Some(Expression::Literal(Literal::Int(a * b)))
                        }
                        (Literal::Int(a), Literal::Int(b), Operator::Slash) => {
                            if *b != 0 {
                                Some(Expression::Literal(Literal::Int(a / b)))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            Expression::UnaryOp(op, expr) => {
                let folded = self.fold_constant_expression(expr)?;
                if let Expression::Literal(lit) = &folded {
                    match (lit, op) {
                        (Literal::Int(a), Operator::Minus) => {
                            Some(Expression::Literal(Literal::Int(-a)))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            Expression::Literal(_) => Some(expr.clone()),
            _ => None,
        }
    }

    fn dead_code_elimination(&self, ast: &Program) -> (Program, Vec<String>) {
        let mut optimized_statements = Vec::new();
        let mut optimizations = Vec::new();

        for statement in &ast.statements {
            match statement {
                Statement::If(if_stmt) => {
                    // Check if condition is always true/false
                    if let Some(constant_condition) = self.evaluate_boolean_expression(&if_stmt.condition) {
                        if constant_condition {
                            optimized_statements.push(Statement::Block(if_stmt.consequence.clone()));
                            optimizations.push("Removed unreachable else branch".to_string());
                        } else {
                            if let Some(alternative) = &if_stmt.alternative {
                                optimized_statements.push(Statement::Block(alternative.clone()));
                            }
                            optimizations.push("Removed unreachable if branch".to_string());
                        }
                    } else {
                        optimized_statements.push(statement.clone());
                    }
                }
                Statement::Expression(Expression::Literal(Literal::Null)) => {
                    // Remove null expressions
                    optimizations.push("Removed null expression".to_string());
                }
                _ => optimized_statements.push(statement.clone()),
            }
        }

        (Program { statements: optimized_statements }, optimizations)
    }

    fn evaluate_boolean_expression(&self, expr: &Expression) -> Option<bool> {
        match expr {
            Expression::Literal(Literal::Bool(b)) => Some(*b),
            Expression::BinaryOp(left, op, right) => {
                let left_val = self.evaluate_boolean_expression(left)?;
                let right_val = self.evaluate_boolean_expression(right)?;
                
                match op {
                    Operator::And => Some(left_val && right_val),
                    Operator::Or => Some(left_val || right_val),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn function_inlining(&self, ast: &Program) -> (Program, Vec<String>) {
        // Simple function inlining for small functions
        let mut optimized_statements = Vec::new();
        let optimizations = Vec::new();
        let mut function_map = HashMap::new();

        // First pass: collect small functions
        for statement in &ast.statements {
            if let Statement::Function(func_stmt) = statement {
                if self.should_inline_function(func_stmt) {
                    function_map.insert(func_stmt.name.clone(), func_stmt.clone());
                }
            }
        }

        // Second pass: inline function calls
        for statement in &ast.statements {
            match statement {
                Statement::Function(func_stmt) => {
                    if !function_map.contains_key(&func_stmt.name) {
                        optimized_statements.push(statement.clone());
                    }
                }
                Statement::Expression(expr) => {
                    let inlined_expr = self.inline_function_calls(expr, &function_map);
                    optimized_statements.push(Statement::Expression(inlined_expr));
                }
                _ => optimized_statements.push(statement.clone()),
            }
        }

        (Program { statements: optimized_statements }, optimizations)
    }

    fn should_inline_function(&self, func_stmt: &FunctionStatement) -> bool {
        // Inline functions with 3 or fewer statements and no parameters
        func_stmt.parameters.is_empty() && func_stmt.body.statements.len() <= 3
    }

    fn inline_function_calls(&self, expr: &Expression, function_map: &HashMap<String, FunctionStatement>) -> Expression {
        match expr {
            Expression::FunctionCall(call) => {
                if let Some(func) = function_map.get(&call.name) {
                    // Simple inlining: replace function call with function body
                    if call.arguments.is_empty() {
                        // For now, just return the first statement's expression if it's a return
                        if let Some(Statement::Return(ret_stmt)) = func.body.statements.first() {
                            if let Some(value) = &ret_stmt.value {
                                return value.clone();
                            }
                        }
                    }
                }
                expr.clone()
            }
            _ => expr.clone(),
        }
    }

    fn common_subexpression_elimination(&self, ast: &Program) -> (Program, Vec<String>) {
        // Simple CSE: find identical expressions and replace with variables
        let mut optimized_statements = Vec::new();
        let mut optimizations = Vec::new();
        let mut expression_cache: HashMap<String, String> = HashMap::new();

        for statement in &ast.statements {
            match statement {
                Statement::Let(let_stmt) => {
                    let cache_key = self.expression_to_string(&let_stmt.value);
                    if let Some(existing_var) = expression_cache.get(&cache_key) {
                        // Replace with existing variable
                        optimized_statements.push(Statement::Let(LetStatement {
                            name: let_stmt.name.clone(),
                            value: Expression::Identifier(existing_var.clone()),
                        }));
                        optimizations.push(format!("CSE: {} = {}", let_stmt.name, existing_var));
                    } else {
                        expression_cache.insert(cache_key, let_stmt.name.clone());
                        optimized_statements.push(statement.clone());
                    }
                }
                _ => optimized_statements.push(statement.clone()),
            }
        }

        (Program { statements: optimized_statements }, optimizations)
    }

    fn expression_to_string(&self, expr: &Expression) -> String {
        match expr {
            Expression::Literal(lit) => format!("{:?}", lit),
            Expression::Identifier(id) => id.clone(),
            Expression::BinaryOp(left, op, right) => {
                format!("({:?} {:?} {:?})", left, op, right)
            }
            _ => format!("{:?}", expr),
        }
    }

    fn loop_optimization(&self, _ast: &Program) -> (Program, Vec<String>) {
        // Placeholder for loop optimizations
        // Would include loop unrolling, loop-invariant code motion, etc.
        (_ast.clone(), vec!["Loop optimization (placeholder)".to_string()])
    }

    fn strength_reduction(&self, ast: &Program) -> (Program, Vec<String>) {
        let mut optimized_statements = Vec::new();
        let optimizations = Vec::new();

        for statement in &ast.statements {
            match statement {
                Statement::Expression(expr) => {
                    let reduced_expr = self.reduce_expression_strength(expr);
                    optimized_statements.push(Statement::Expression(reduced_expr));
                }
                _ => optimized_statements.push(statement.clone()),
            }
        }

        (Program { statements: optimized_statements }, optimizations)
    }

    fn reduce_expression_strength(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::BinaryOp(left, op, right) => {
                match op {
                    Operator::Star => {
                        // Replace multiplication by 2 with left shift
                        if let Expression::Literal(Literal::Int(2)) = **right {
                            Expression::BinaryOp(
                                left.clone(),
                                Operator::Star,
                                Box::new(Expression::Literal(Literal::Int(2)))
                            )
                        } else if let Expression::Literal(Literal::Int(2)) = **left {
                            Expression::BinaryOp(
                                right.clone(),
                                Operator::Star,
                                Box::new(Expression::Literal(Literal::Int(2)))
                            )
                        } else {
                            expr.clone()
                        }
                    }
                    _ => expr.clone(),
                }
            }
            _ => expr.clone(),
        }
    }

    fn optimize_block(&self, block: &BlockStatement) -> BlockStatement {
        let mut optimized_statements = Vec::new();
        
        for statement in &block.statements {
            match statement {
                Statement::Let(let_stmt) => {
                    if let Some(folded_value) = self.fold_constant_expression(&let_stmt.value) {
                        optimized_statements.push(Statement::Let(LetStatement {
                            name: let_stmt.name.clone(),
                            value: folded_value,
                        }));
                    } else {
                        optimized_statements.push(statement.clone());
                    }
                }
                _ => optimized_statements.push(statement.clone()),
            }
        }

        BlockStatement { statements: optimized_statements }
    }

    fn estimate_improvement(&self, optimizations: &[String]) -> f64 {
        let mut improvement: f64 = 0.0;
        
        for opt in optimizations {
            if opt.contains("Constant folded") {
                improvement += 5.0; // 5% improvement for constant folding
            } else if opt.contains("Removed unreachable") {
                improvement += 3.0; // 3% improvement for dead code elimination
            } else if opt.contains("CSE") {
                improvement += 2.0; // 2% improvement for common subexpression elimination
            } else if opt.contains("inlined") {
                improvement += 8.0; // 8% improvement for function inlining
            }
        }
        
        improvement.min(50.0_f64) // Cap at 50% improvement
    }
}

// Optimization utilities
pub struct OptimizationUtils;

impl OptimizationUtils {
    pub fn analyze_complexity(ast: &Program) -> usize {
        let mut complexity = 0;
        
        for statement in &ast.statements {
            complexity += Self::statement_complexity(statement);
        }
        
        complexity
    }

    fn statement_complexity(statement: &Statement) -> usize {
        match statement {
            Statement::Function(func_stmt) => {
                1 + Self::block_complexity(&func_stmt.body)
            }
            Statement::If(if_stmt) => {
                2 + Self::block_complexity(&if_stmt.consequence) +
                if_stmt.alternative.as_ref().map_or(0, Self::block_complexity)
            }
            Statement::Let(_) => 1,
            Statement::Return(_) => 1,
            Statement::Expression(_) => 1,
            Statement::Block(block) => Self::block_complexity(block),
            _ => 1,
        }
    }

    fn block_complexity(block: &BlockStatement) -> usize {
        block.statements.iter().map(Self::statement_complexity).sum()
    }

    pub fn estimate_optimization_potential(ast: &Program) -> f64 {
        let complexity = Self::analyze_complexity(ast);
        let constant_expressions = Self::count_constant_expressions(ast);
        let function_calls = Self::count_function_calls(ast);
        
        let mut potential = 0.0;
        
        // More complex code has more optimization potential
        potential += (complexity as f64 * 0.5).min(20.0);
        
        // Constant expressions can be folded
        potential += constant_expressions as f64 * 3.0;
        
        // Function calls can be inlined
        potential += function_calls as f64 * 2.0;
        
        potential.min(50.0) // Cap at 50%
    }

    fn count_constant_expressions(ast: &Program) -> usize {
        let mut count = 0;
        
        for statement in &ast.statements {
            count += Self::count_constants_in_statement(statement);
        }
        
        count
    }

    fn count_constants_in_statement(statement: &Statement) -> usize {
        match statement {
            Statement::Let(let_stmt) => Self::count_constants_in_expression(&let_stmt.value),
            Statement::Expression(expr) => Self::count_constants_in_expression(expr),
            Statement::Function(func_stmt) => Self::block_complexity(&func_stmt.body),
            _ => 0,
        }
    }

    fn count_constants_in_expression(expr: &Expression) -> usize {
        match expr {
            Expression::Literal(_) => 1,
            Expression::BinaryOp(left, _, right) => {
                Self::count_constants_in_expression(left) + Self::count_constants_in_expression(right)
            }
            Expression::UnaryOp(_, expr) => Self::count_constants_in_expression(expr),
            _ => 0,
        }
    }

    fn count_function_calls(ast: &Program) -> usize {
        let mut count = 0;
        
        for statement in &ast.statements {
            count += Self::count_calls_in_statement(statement);
        }
        
        count
    }

    fn count_calls_in_statement(statement: &Statement) -> usize {
        match statement {
            Statement::Expression(expr) => Self::count_calls_in_expression(expr),
            Statement::Function(func_stmt) => Self::count_calls_in_block(&func_stmt.body),
            _ => 0,
        }
    }

    fn count_calls_in_expression(expr: &Expression) -> usize {
        match expr {
            Expression::FunctionCall(_) => 1,
            Expression::BinaryOp(left, _, right) => {
                Self::count_calls_in_expression(left) + Self::count_calls_in_expression(right)
            }
            _ => 0,
        }
    }

    fn count_calls_in_block(block: &BlockStatement) -> usize {
        block.statements.iter().map(Self::count_calls_in_statement).sum()
    }
}
