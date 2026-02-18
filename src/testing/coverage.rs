use crate::parser::ast::*;
use crate::testing::framework::TestCoverage;
use std::collections::HashSet;

/// Coverage tracker for monitoring code execution
#[derive(Debug, Clone)]
pub struct CoverageTracker {
    pub executed_lines: HashSet<usize>,
    pub executed_functions: HashSet<String>,
    pub executed_branches: HashSet<(usize, String)>, // (line, condition)
    pub total_lines: usize,
    pub total_functions: usize,
    pub total_branches: usize,
    pub source_code: Option<String>,
}

impl CoverageTracker {
    pub fn new() -> Self {
        Self {
            executed_lines: HashSet::new(),
            executed_functions: HashSet::new(),
            executed_branches: HashSet::new(),
            total_lines: 0,
            total_functions: 0,
            total_branches: 0,
            source_code: None,
        }
    }

    pub fn with_source_code(mut self, source_code: String) -> Self {
        self.total_lines = source_code.lines().count();
        self.source_code = Some(source_code);
        self
    }

    pub fn mark_line_executed(&mut self, line: usize) {
        self.executed_lines.insert(line);
    }

    pub fn mark_function_executed(&mut self, function_name: &str) {
        self.executed_functions.insert(function_name.to_string());
    }

    pub fn mark_branch_executed(&mut self, line: usize, condition: &str) {
        self.executed_branches.insert((line, condition.to_string()));
    }

    pub fn get_coverage(&self) -> TestCoverage {
        TestCoverage {
            lines_covered: self.executed_lines.len(),
            total_lines: self.total_lines,
            functions_called: self.executed_functions.iter().cloned().collect(),
            branches_covered: self.executed_branches.len(),
            total_branches: self.total_branches,
        }
    }

    pub fn line_coverage_percentage(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            (self.executed_lines.len() as f64 / self.total_lines as f64) * 100.0
        }
    }

    pub fn function_coverage_percentage(&self) -> f64 {
        if self.total_functions == 0 {
            0.0
        } else {
            (self.executed_functions.len() as f64 / self.total_functions as f64) * 100.0
        }
    }

    pub fn branch_coverage_percentage(&self) -> f64 {
        if self.total_branches == 0 {
            0.0
        } else {
            (self.executed_branches.len() as f64 / self.total_branches as f64) * 100.0
        }
    }

    pub fn generate_coverage_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Code Coverage Report\n");
        report.push_str("===================\n\n");

        report.push_str(&format!(
            "Line Coverage: {:.1}% ({}/{})\n",
            self.line_coverage_percentage(),
            self.executed_lines.len(),
            self.total_lines
        ));

        report.push_str(&format!(
            "Function Coverage: {:.1}% ({}/{})\n",
            self.function_coverage_percentage(),
            self.executed_functions.len(),
            self.total_functions
        ));

        report.push_str(&format!(
            "Branch Coverage: {:.1}% ({}/{})\n",
            self.branch_coverage_percentage(),
            self.executed_branches.len(),
            self.total_branches
        ));

        // Show uncovered lines
        if let Some(source) = &self.source_code {
            let lines: Vec<&str> = source.lines().collect();
            let uncovered_lines: Vec<usize> = (1..=lines.len())
                .filter(|&line| !self.executed_lines.contains(&line))
                .collect();

            if !uncovered_lines.is_empty() {
                report.push_str("\nUncovered Lines:\n");
                for line_num in uncovered_lines {
                    if line_num <= lines.len() {
                        report.push_str(&format!("  Line {}: {}\n", line_num, lines[line_num - 1]));
                    }
                }
            }
        }

        report
    }
}

/// Coverage analyzer that analyzes AST for coverage information
pub struct CoverageAnalyzer {
    pub tracker: CoverageTracker,
}

impl CoverageAnalyzer {
    pub fn new() -> Self {
        Self {
            tracker: CoverageTracker::new(),
        }
    }

    pub fn analyze_ast(&mut self, ast: &Program) {
        // Count total functions and branches
        self.count_functions_and_branches(ast);

        // Analyze each statement
        for statement in &ast.statements {
            self.analyze_statement(statement);
        }
    }

    fn count_functions_and_branches(&mut self, ast: &Program) {
        for statement in &ast.statements {
            match statement {
                Statement::Function(func) => {
                    self.tracker.total_functions += 1;
                    self.count_branches_in_block(&func.body);
                }
                Statement::Block(block) => {
                    self.count_branches_in_block(block);
                }
                _ => {}
            }
        }
    }

    fn count_branches_in_block(&mut self, block: &BlockStatement) {
        for statement in &block.statements {
            match statement {
                Statement::If(if_stmt) => {
                    self.tracker.total_branches += 2; // if and else branches
                    self.count_branches_in_block(&if_stmt.consequence);
                    if let Some(alternative) = &if_stmt.alternative {
                        self.count_branches_in_block(alternative);
                    }
                }
                Statement::While(while_stmt) => {
                    self.tracker.total_branches += 1; // loop vs exit
                    self.count_branches_in_block(&while_stmt.body);
                }
                Statement::Block(nested_block) => {
                    self.count_branches_in_block(nested_block);
                }
                _ => {}
            }
        }
    }

    fn analyze_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Let(let_stmt) => {
                // Analyze the expression
                self.analyze_expression(&let_stmt.value);
            }
            Statement::Function(func) => {
                self.tracker.mark_function_executed(&func.name);
                self.analyze_block(&func.body);
            }
            Statement::If(if_stmt) => {
                self.analyze_expression(&if_stmt.condition);
                self.analyze_block(&if_stmt.consequence);
                if let Some(alternative) = &if_stmt.alternative {
                    self.analyze_block(alternative);
                }
            }
            Statement::While(while_stmt) => {
                self.analyze_expression(&while_stmt.condition);
                self.analyze_block(&while_stmt.body);
            }
            Statement::Block(block) => {
                self.analyze_block(block);
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr);
            }
            _ => {}
        }
    }

    fn analyze_block(&mut self, block: &BlockStatement) {
        for statement in &block.statements {
            self.analyze_statement(statement);
        }
    }

    fn analyze_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Literal(_) => {}
            Expression::Identifier(_) => {}
            Expression::BinaryOp(left, _, right) => {
                self.analyze_expression(left);
                self.analyze_expression(right);
            }
            Expression::FunctionCall(func_call) => {
                self.tracker.mark_function_executed(&func_call.name);
                for arg in &func_call.arguments {
                    self.analyze_expression(arg);
                }
            }
            Expression::Await(await_expr) => {
                self.analyze_expression(await_expr);
            }
            Expression::Spawn(expr) => {
                self.analyze_expression(expr);
            }
            Expression::IndexAccess(container, index_expr) => {
                self.analyze_expression(container);
                self.analyze_expression(index_expr);
            }
            Expression::ArrowFunction { body, .. } => {
                for stmt in &body.statements {
                    self.analyze_statement(stmt);
                }
            }
            _ => {}
        }
    }
}

/// Coverage instrumentation for runtime tracking
pub struct CoverageInstrumentation {
    pub tracker: CoverageTracker,
    pub enabled: bool,
}

impl CoverageInstrumentation {
    pub fn new() -> Self {
        Self {
            tracker: CoverageTracker::new(),
            enabled: false,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn instrument_line(&mut self, line: usize) {
        if self.enabled {
            self.tracker.mark_line_executed(line);
        }
    }

    pub fn instrument_function(&mut self, function_name: &str) {
        if self.enabled {
            self.tracker.mark_function_executed(function_name);
        }
    }

    pub fn instrument_branch(&mut self, line: usize, condition: &str) {
        if self.enabled {
            self.tracker.mark_branch_executed(line, condition);
        }
    }

    pub fn get_coverage(&self) -> TestCoverage {
        self.tracker.get_coverage()
    }
}

/// Coverage reporter for generating detailed reports
pub struct CoverageReporter {
    pub trackers: Vec<CoverageTracker>,
}

impl CoverageReporter {
    pub fn new() -> Self {
        Self {
            trackers: Vec::new(),
        }
    }

    pub fn add_tracker(&mut self, tracker: CoverageTracker) {
        self.trackers.push(tracker);
    }

    pub fn generate_summary_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Coverage Summary Report\n");
        report.push_str("======================\n\n");

        let mut total_lines = 0;
        let mut total_covered_lines = 0;
        let mut total_functions = 0;
        let mut total_covered_functions = 0;
        let mut total_branches = 0;
        let mut total_covered_branches = 0;

        for (i, tracker) in self.trackers.iter().enumerate() {
            report.push_str(&format!("File {}:\n", i + 1));
            report.push_str(&format!(
                "  Line Coverage: {:.1}% ({}/{})\n",
                tracker.line_coverage_percentage(),
                tracker.executed_lines.len(),
                tracker.total_lines
            ));
            report.push_str(&format!(
                "  Function Coverage: {:.1}% ({}/{})\n",
                tracker.function_coverage_percentage(),
                tracker.executed_functions.len(),
                tracker.total_functions
            ));
            report.push_str(&format!(
                "  Branch Coverage: {:.1}% ({}/{})\n",
                tracker.branch_coverage_percentage(),
                tracker.executed_branches.len(),
                tracker.total_branches
            ));
            report.push('\n');

            total_lines += tracker.total_lines;
            total_covered_lines += tracker.executed_lines.len();
            total_functions += tracker.total_functions;
            total_covered_functions += tracker.executed_functions.len();
            total_branches += tracker.total_branches;
            total_covered_branches += tracker.executed_branches.len();
        }

        // Overall summary
        let overall_line_coverage = if total_lines > 0 {
            (total_covered_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        let overall_function_coverage = if total_functions > 0 {
            (total_covered_functions as f64 / total_functions as f64) * 100.0
        } else {
            0.0
        };

        let overall_branch_coverage = if total_branches > 0 {
            (total_covered_branches as f64 / total_branches as f64) * 100.0
        } else {
            0.0
        };

        report.push_str("Overall Summary:\n");
        report.push_str(&format!(
            "  Line Coverage: {:.1}% ({}/{})\n",
            overall_line_coverage, total_covered_lines, total_lines
        ));
        report.push_str(&format!(
            "  Function Coverage: {:.1}% ({}/{})\n",
            overall_function_coverage, total_covered_functions, total_functions
        ));
        report.push_str(&format!(
            "  Branch Coverage: {:.1}% ({}/{})\n",
            overall_branch_coverage, total_covered_branches, total_branches
        ));

        report
    }

    pub fn generate_html_report(&self) -> String {
        let mut html = String::new();
        html.push_str(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Coverage Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .summary { background: #f5f5f5; padding: 15px; border-radius: 5px; margin-bottom: 20px; }
        .coverage-bar { background: #ddd; height: 20px; border-radius: 10px; overflow: hidden; }
        .coverage-fill { height: 100%; background: linear-gradient(90deg, #4CAF50, #8BC34A); }
        .low-coverage { background: linear-gradient(90deg, #FF5722, #FF9800) !important; }
        .medium-coverage { background: linear-gradient(90deg, #FF9800, #FFC107) !important; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        th, td { padding: 8px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background-color: #f2f2f2; }
    </style>
</head>
<body>
    <h1>Code Coverage Report</h1>
"#,
        );

        // Calculate overall coverage
        let mut total_lines = 0;
        let mut total_covered_lines = 0;
        let mut total_functions = 0;
        let mut total_covered_functions = 0;
        let mut total_branches = 0;
        let mut total_covered_branches = 0;

        for tracker in &self.trackers {
            total_lines += tracker.total_lines;
            total_covered_lines += tracker.executed_lines.len();
            total_functions += tracker.total_functions;
            total_covered_functions += tracker.executed_functions.len();
            total_branches += tracker.total_branches;
            total_covered_branches += tracker.executed_branches.len();
        }

        let overall_line_coverage = if total_lines > 0 {
            (total_covered_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        let overall_function_coverage = if total_functions > 0 {
            (total_covered_functions as f64 / total_functions as f64) * 100.0
        } else {
            0.0
        };

        let overall_branch_coverage = if total_branches > 0 {
            (total_covered_branches as f64 / total_branches as f64) * 100.0
        } else {
            0.0
        };

        html.push_str(&format!(
            r#"
    <div class="summary">
        <h2>Overall Coverage</h2>
        <p><strong>Line Coverage:</strong> {:.1}% ({}/{})</p>
        <div class="coverage-bar">
            <div class="coverage-fill {}" style="width: {:.1}%"></div>
        </div>
        
        <p><strong>Function Coverage:</strong> {:.1}% ({}/{})</p>
        <div class="coverage-bar">
            <div class="coverage-fill {}" style="width: {:.1}%"></div>
        </div>
        
        <p><strong>Branch Coverage:</strong> {:.1}% ({}/{})</p>
        <div class="coverage-bar">
            <div class="coverage-fill {}" style="width: {:.1}%"></div>
        </div>
    </div>
"#,
            overall_line_coverage,
            total_covered_lines,
            total_lines,
            if overall_line_coverage < 50.0 {
                "low-coverage"
            } else if overall_line_coverage < 80.0 {
                "medium-coverage"
            } else {
                ""
            },
            overall_line_coverage,
            overall_function_coverage,
            total_covered_functions,
            total_functions,
            if overall_function_coverage < 50.0 {
                "low-coverage"
            } else if overall_function_coverage < 80.0 {
                "medium-coverage"
            } else {
                ""
            },
            overall_function_coverage,
            overall_branch_coverage,
            total_covered_branches,
            total_branches,
            if overall_branch_coverage < 50.0 {
                "low-coverage"
            } else if overall_branch_coverage < 80.0 {
                "medium-coverage"
            } else {
                ""
            },
            overall_branch_coverage
        ));

        html.push_str(
            r#"
    <table>
        <thead>
            <tr>
                <th>File</th>
                <th>Line Coverage</th>
                <th>Function Coverage</th>
                <th>Branch Coverage</th>
            </tr>
        </thead>
        <tbody>
"#,
        );

        for (i, tracker) in self.trackers.iter().enumerate() {
            html.push_str(&format!(
                r#"
            <tr>
                <td>File {}</td>
                <td>{:.1}%</td>
                <td>{:.1}%</td>
                <td>{:.1}%</td>
            </tr>
"#,
                i + 1,
                tracker.line_coverage_percentage(),
                tracker.function_coverage_percentage(),
                tracker.branch_coverage_percentage()
            ));
        }

        html.push_str(
            r#"
        </tbody>
    </table>
</body>
</html>"#,
        );

        html
    }
}
