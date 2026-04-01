//! CT2: Blockchain (EVM) backend — transpile DAL services to Solidity and invoke solc.
//! See docs/development/implementation/COMPILE_TARGET_IMPLEMENTATION_PLAN.md.

use crate::compile::{CompileArtifacts, CompileBackend, CompileError, CompileOptions};
use crate::lexer::tokens::{Literal, Operator};
use crate::parser::ast::{
    BlockStatement, Expression, FunctionStatement, Program, ServiceStatement, Statement,
};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::process::Command;

/// Map DAL type name to Solidity type for state vars and function params.
fn dal_type_to_solidity(dal_type: &str) -> String {
    let t = dal_type.trim();
    if t.is_empty() {
        return "uint256".to_string();
    }
    if t == "int" {
        return "int256".to_string();
    }
    if t == "string" || t == "bool" {
        return t.to_string();
    }
    if t.starts_with("list<") || t.starts_with("vector<") {
        let inner = t
            .strip_prefix("list<")
            .or_else(|| t.strip_prefix("vector<"))
            .and_then(|s| s.strip_suffix('>'))
            .unwrap_or("uint256");
        return format!("{}[]", dal_type_to_solidity(inner));
    }
    if t.starts_with("map<") {
        if let Some(rest) = t.strip_prefix("map<").and_then(|s| s.strip_suffix('>')) {
            let mut parts = rest.splitn(2, ',');
            let k = parts.next().unwrap_or("string").trim();
            let v = parts.next().unwrap_or("uint256").trim();
            let k_sol = dal_type_to_solidity(k);
            let v_sol = dal_type_to_solidity(v);
            return format!("mapping({} => {})", k_sol, v_sol);
        }
    }
    if t == "any" {
        return "bytes".to_string();
    }
    t.to_string()
}

fn service_declares_decentralized(service: &ServiceStatement) -> bool {
    service.attributes.iter().any(|attr| {
        attr.name == "@trust"
            && attr
                .parameters
                .first()
                .and_then(expression_to_string_literal)
                == Some("decentralized")
    })
}

fn solidity_binary_operator(op: &Operator) -> Option<&'static str> {
    match op {
        Operator::Plus => Some("+"),
        Operator::Minus => Some("-"),
        Operator::Star => Some("*"),
        Operator::Slash => Some("/"),
        Operator::Percent => Some("%"),
        Operator::Equal | Operator::EqualEqual => Some("=="),
        Operator::BangEqual | Operator::NotEqual => Some("!="),
        Operator::Less => Some("<"),
        Operator::LessEqual => Some("<="),
        Operator::Greater => Some(">"),
        Operator::GreaterEqual => Some(">="),
        Operator::And => Some("&&"),
        Operator::Or => Some("||"),
        _ => None,
    }
}

fn normalized_value_type(sol_type: &str) -> String {
    match sol_type.trim() {
        "string memory" | "string calldata" | "string storage" => "string".to_string(),
        other => other.to_string(),
    }
}

/// Extract the value type from a DAL `map<K,V>` type string.
/// Returns `None` if the type is not a map.
fn extract_map_value_type(dal_type: &str) -> Option<String> {
    let t = dal_type.trim();
    let rest = t.strip_prefix("map<")?.strip_suffix('>')?;
    let mut parts = rest.splitn(2, ',');
    let _key = parts.next()?;
    let value = parts.next()?.trim();
    Some(value.to_string())
}

fn infer_expr_solidity_type(
    service: &ServiceStatement,
    params: &HashMap<String, String>,
    locals: &HashMap<String, String>,
    expr: &Expression,
) -> Option<String> {
    match expr {
        Expression::Literal(Literal::Int(_)) => Some("int256".to_string()),
        Expression::Literal(Literal::Bool(_)) => Some("bool".to_string()),
        Expression::Literal(Literal::String(_)) => Some("string memory".to_string()),
        Expression::Identifier(name) => locals
            .get(name)
            .cloned()
            .or_else(|| params.get(name).cloned())
            .map(|t| normalized_value_type(&t)),
        Expression::BinaryOp(left, op, right) => {
            let left_ty = infer_expr_solidity_type(service, params, locals, left)?;
            let right_ty = infer_expr_solidity_type(service, params, locals, right)?;
            match op {
                Operator::Plus
                | Operator::Minus
                | Operator::Star
                | Operator::Slash
                | Operator::Percent => {
                    if left_ty == "int256" && right_ty == "int256" {
                        Some("int256".to_string())
                    } else {
                        None
                    }
                }
                Operator::Equal
                | Operator::EqualEqual
                | Operator::BangEqual
                | Operator::NotEqual
                | Operator::Less
                | Operator::LessEqual
                | Operator::Greater
                | Operator::GreaterEqual
                | Operator::And
                | Operator::Or => Some("bool".to_string()),
                _ => None,
            }
        }
        Expression::UnaryOp(op, inner) => {
            let inner_ty = infer_expr_solidity_type(service, params, locals, inner)?;
            match op {
                Operator::Minus if inner_ty == "int256" => Some("int256".to_string()),
                Operator::Bang | Operator::Not if inner_ty == "bool" => Some("bool".to_string()),
                _ => None,
            }
        }
        Expression::FieldAccess(owner, field) => {
            if matches!(owner.as_ref(), Expression::Identifier(id) if id == "self") {
                service
                    .fields
                    .iter()
                    .find(|f| f.name == *field)
                    .map(|f| normalized_value_type(&dal_type_to_solidity(&f.field_type)))
            } else {
                None
            }
        }
        Expression::IndexAccess(container, _index) => {
            if let Expression::FieldAccess(owner, field) = container.as_ref() {
                if matches!(owner.as_ref(), Expression::Identifier(id) if id == "self") {
                    return service
                        .fields
                        .iter()
                        .find(|f| f.name == *field)
                        .and_then(|f| extract_map_value_type(&f.field_type))
                        .map(|vt| normalized_value_type(&dal_type_to_solidity(&vt)));
                }
            }
            None
        }
        Expression::FunctionCall(call) => {
            if call.name.contains("::") {
                return None;
            }
            service
                .methods
                .iter()
                .find(|m| m.name == call.name)
                .and_then(|m| m.return_type.as_deref())
                .map(|rt| normalized_value_type(&dal_type_to_solidity(rt)))
        }
        _ => None,
    }
}

fn expression_to_solidity(expr: &Expression) -> Result<String, String> {
    match expr {
        Expression::Literal(Literal::Int(v)) => Ok(v.to_string()),
        Expression::Literal(Literal::Float(_)) => {
            Err("float literals are unsupported in decentralized-v1 blockchain codegen".to_string())
        }
        Expression::Literal(Literal::String(s)) => Ok(format!("{:?}", s)),
        Expression::Literal(Literal::Bool(v)) => Ok(v.to_string()),
        Expression::Literal(Literal::Null) => {
            Err("null literal is unsupported in decentralized-v1 blockchain codegen".to_string())
        }
        Expression::Identifier(name) => Ok(name.clone()),
        Expression::BinaryOp(left, op, right) => {
            let op = solidity_binary_operator(op).ok_or_else(|| {
                format!(
                    "unsupported binary operator in decentralized-v1 blockchain codegen: {:?}",
                    op
                )
            })?;
            let l = expression_to_solidity(left)?;
            let r = expression_to_solidity(right)?;
            Ok(format!("({} {} {})", l, op, r))
        }
        Expression::UnaryOp(op, e) => {
            let inner = expression_to_solidity(e)?;
            match op {
                Operator::Minus => Ok(format!("(-{})", inner)),
                Operator::Bang | Operator::Not => Ok(format!("(!{})", inner)),
                _ => Err(format!(
                    "unsupported unary operator in decentralized-v1 blockchain codegen: {:?}",
                    op
                )),
            }
        }
        Expression::Assignment(name, value) => {
            let rhs = expression_to_solidity(value)?;
            Ok(format!("{} = {}", name, rhs))
        }
        Expression::FieldAccess(owner, field) => {
            if matches!(owner.as_ref(), Expression::Identifier(id) if id == "self") {
                Ok(field.clone())
            } else {
                Err("field access is only supported for self.<field> in decentralized-v1 blockchain codegen".to_string())
            }
        }
        Expression::FieldAssignment(owner, field, value) => {
            if matches!(owner.as_ref(), Expression::Identifier(id) if id == "self") {
                let rhs = expression_to_solidity(value)?;
                Ok(format!("{} = {}", field, rhs))
            } else {
                Err("field assignment is only supported for self.<field> in decentralized-v1 blockchain codegen".to_string())
            }
        }
        Expression::IndexAccess(container, index) => {
            if let Expression::FieldAccess(owner, field) = container.as_ref() {
                if matches!(owner.as_ref(), Expression::Identifier(id) if id == "self") {
                    let key = expression_to_solidity(index)?;
                    return Ok(format!("{}[{}]", field, key));
                }
            }
            Err("index access is only supported for self.<map_field>[key] in decentralized-v1 blockchain codegen".to_string())
        }
        Expression::FunctionCall(call) => {
            if call.name.contains("::") {
                return Err(format!(
                    "namespace call '{}' is not allowed in decentralized-v1 blockchain codegen; move to hybrid/centralized",
                    call.name
                ));
            }
            let args: Result<Vec<String>, String> =
                call.arguments.iter().map(expression_to_solidity).collect();
            Ok(format!("{}({})", call.name, args?.join(", ")))
        }
        _ => Err("unsupported expression in decentralized-v1 blockchain codegen".to_string()),
    }
}

fn block_to_solidity_v1(
    service: &ServiceStatement,
    method: &FunctionStatement,
    block: &BlockStatement,
    indent: usize,
    params: &HashMap<String, String>,
    locals: &mut HashMap<String, String>,
) -> Result<String, String> {
    let pad = " ".repeat(indent);
    let mut out = String::new();
    for stmt in &block.statements {
        match stmt {
            Statement::Let(let_stmt) => {
                let ty =
                    infer_expr_solidity_type(service, params, locals, &let_stmt.value).ok_or_else(
                        || {
                            format!(
                                "unsupported let binding '{}' type inference in decentralized-v1 blockchain codegen",
                                let_stmt.name
                            )
                        },
                    )?;
                let let_ty = if ty == "string" {
                    "string memory".to_string()
                } else {
                    ty.clone()
                };
                locals.insert(let_stmt.name.clone(), ty);
                let value = expression_to_solidity(&let_stmt.value)?;
                out.push_str(&format!(
                    "{}{} {} = {};\n",
                    pad, let_ty, let_stmt.name, value
                ));
            }
            Statement::Return(ret) => {
                if let Some(value) = &ret.value {
                    out.push_str(&format!(
                        "{}return {};\n",
                        pad,
                        expression_to_solidity(value)?
                    ));
                } else {
                    out.push_str(&format!("{}return;\n", pad));
                }
            }
            Statement::Expression(Expression::FunctionCall(call))
                if call.name == "__index_assign__" && call.arguments.len() >= 3 =>
            {
                if let Expression::FieldAccess(owner, field) = &call.arguments[0] {
                    if matches!(owner.as_ref(), Expression::Identifier(id) if id == "self") {
                        let key = expression_to_solidity(&call.arguments[1])?;
                        let val = expression_to_solidity(&call.arguments[2])?;
                        out.push_str(&format!("{}{}[{}] = {};\n", pad, field, key, val));
                    } else {
                        return Err(
                            "map index assignment is only supported for self.<map_field>[key] in decentralized-v1 blockchain codegen"
                                .to_string(),
                        );
                    }
                } else {
                    return Err(
                        "map index assignment is only supported for self.<map_field>[key] in decentralized-v1 blockchain codegen"
                            .to_string(),
                    );
                }
            }
            Statement::Expression(Expression::Literal(Literal::Null)) => {
                // Parser artifact from semicolons — skip.
            }
            Statement::Expression(expr) => {
                out.push_str(&format!("{}{};\n", pad, expression_to_solidity(expr)?));
            }
            Statement::Event(ev) => {
                let decl = service
                    .events
                    .iter()
                    .find(|e| e.name == ev.event_name)
                    .ok_or_else(|| {
                        format!(
                            "event '{}' used in method '{}' is not declared on service '{}'",
                            ev.event_name, method.name, service.name
                        )
                    })?;
                let mut args = Vec::new();
                for p in &decl.parameters {
                    let value = ev.data.get(&p.name).ok_or_else(|| {
                        format!(
                            "event '{}' missing data for parameter '{}' in method '{}'",
                            ev.event_name, p.name, method.name
                        )
                    })?;
                    args.push(expression_to_solidity(value)?);
                }
                out.push_str(&format!(
                    "{}emit {}({});\n",
                    pad,
                    ev.event_name,
                    args.join(", ")
                ));
            }
            Statement::If(if_stmt) => {
                let condition = expression_to_solidity(&if_stmt.condition)?;
                out.push_str(&format!("{}if ({}) {{\n", pad, condition));
                let mut branch_locals = locals.clone();
                out.push_str(&block_to_solidity_v1(
                    service,
                    method,
                    &if_stmt.consequence,
                    indent + 4,
                    params,
                    &mut branch_locals,
                )?);
                out.push_str(&format!("{}}}", pad));
                if let Some(alt) = &if_stmt.alternative {
                    out.push_str(" else {\n");
                    let mut alt_locals = locals.clone();
                    out.push_str(&block_to_solidity_v1(
                        service,
                        method,
                        alt,
                        indent + 4,
                        params,
                        &mut alt_locals,
                    )?);
                    out.push_str(&format!("{}}}", pad));
                }
                out.push('\n');
            }
            Statement::Block(block_stmt) => {
                out.push_str(&format!("{}{{\n", pad));
                let mut block_locals = locals.clone();
                out.push_str(&block_to_solidity_v1(
                    service,
                    method,
                    block_stmt,
                    indent + 4,
                    params,
                    &mut block_locals,
                )?);
                out.push_str(&format!("{}}}\n", pad));
            }
            _ => {
                return Err(format!(
                    "unsupported statement in decentralized-v1 blockchain codegen: {:?}",
                    stmt
                ));
            }
        }
    }
    Ok(out)
}

/// Emit Solidity source for a single DAL service (contract name, state vars, functions, events).
fn service_to_solidity(
    service: &ServiceStatement,
    methods: &[FunctionStatement],
) -> Result<String, CompileError> {
    let mut out = String::new();
    out.push_str("// SPDX-License-Identifier: MIT\n");
    out.push_str("pragma solidity ^0.8.0;\n\n");
    out.push_str(&format!("contract {} {{\n", service.name));

    let is_decentralized = service_declares_decentralized(service);
    for field in &service.fields {
        let sol_type = dal_type_to_solidity(&field.field_type);
        if is_decentralized {
            if let Some(initial) = &field.initial_value {
                if sol_type.starts_with("mapping(") {
                    out.push_str(&format!("    {} {};\n", sol_type, field.name));
                    continue;
                }
                let init_expr = expression_to_solidity(initial).map_err(|e| {
                    CompileError::Policy(format!(
                        "Service '{}' field '{}' has unsupported decentralized-v1 initializer: {}",
                        service.name, field.name, e
                    ))
                })?;
                out.push_str(&format!(
                    "    {} {} = {};\n",
                    sol_type, field.name, init_expr
                ));
                continue;
            }
        }
        out.push_str(&format!("    {} {};\n", sol_type, field.name));
    }
    if !service.events.is_empty() {
        out.push('\n');
        for ev in &service.events {
            let params: Vec<String> = ev
                .parameters
                .iter()
                .map(|p| {
                    let ty = p.param_type.as_deref().unwrap_or("uint256");
                    format!("{} {}", dal_type_to_solidity(ty), p.name)
                })
                .collect();
            out.push_str(&format!("    event {}({});\n", ev.name, params.join(", ")));
        }
    }
    if !methods.is_empty() {
        out.push('\n');
        for method in methods {
            let params: Vec<String> = method
                .parameters
                .iter()
                .map(|p| {
                    let ty = p.param_type.as_deref().unwrap_or("uint256");
                    format!("{} {}", dal_type_to_solidity(ty), p.name)
                })
                .collect();
            let ret = method
                .return_type
                .as_deref()
                .map(|r| format!(" returns ({} )", dal_type_to_solidity(r)))
                .unwrap_or_default();
            out.push_str(&format!(
                "    function {}({}) public{} {{\n",
                method.name,
                params.join(", "),
                ret
            ));
            if is_decentralized {
                let param_types = method
                    .parameters
                    .iter()
                    .map(|p| {
                        let ty = p.param_type.as_deref().unwrap_or("uint256");
                        (
                            p.name.clone(),
                            normalized_value_type(&dal_type_to_solidity(ty)),
                        )
                    })
                    .collect::<HashMap<_, _>>();
                let mut local_types = HashMap::new();
                let lowered = block_to_solidity_v1(
                    service,
                    method,
                    &method.body,
                    8,
                    &param_types,
                    &mut local_types,
                )
                .map_err(|e| {
                    CompileError::Policy(format!(
                        "Service '{}' method '{}' cannot be codegened in decentralized-v1: {}",
                        service.name, method.name, e
                    ))
                })?;
                if lowered.trim().is_empty() {
                    out.push_str("        // no-op\n");
                } else {
                    out.push_str(&lowered);
                }
            } else {
                out.push_str("        revert(\"DAL transpiled; implement in Solidity\");\n");
            }
            out.push_str("    }\n");
        }
    }

    out.push_str("}\n");
    Ok(out)
}

fn expression_to_string_literal(expr: &Expression) -> Option<&str> {
    use crate::lexer::tokens::Literal;
    if let Expression::Literal(Literal::String(s)) = expr {
        Some(s.as_str())
    } else {
        None
    }
}

fn service_declares_hybrid(service: &ServiceStatement) -> bool {
    service.attributes.iter().any(|attr| {
        attr.name == "@trust"
            && attr
                .parameters
                .first()
                .and_then(expression_to_string_literal)
                == Some("hybrid")
    })
}

fn collect_namespaces_from_block(block: &BlockStatement) -> HashSet<String> {
    block
        .statements
        .iter()
        .flat_map(collect_namespaces_from_statement)
        .collect()
}

fn collect_namespaces_from_statement(stmt: &Statement) -> HashSet<String> {
    use crate::parser::ast::Statement::*;
    match stmt {
        Let(s) => collect_namespaces_from_expression(&s.value),
        Return(s) => s
            .value
            .as_ref()
            .map(collect_namespaces_from_expression)
            .unwrap_or_default(),
        Block(b) => collect_namespaces_from_block(b),
        Expression(e) => collect_namespaces_from_expression(e),
        If(s) => {
            let mut set = collect_namespaces_from_expression(&s.condition);
            set.extend(collect_namespaces_from_block(&s.consequence));
            if let Some(alt) = &s.alternative {
                set.extend(collect_namespaces_from_block(alt));
            }
            set
        }
        While(s) => {
            let mut set = collect_namespaces_from_expression(&s.condition);
            set.extend(collect_namespaces_from_block(&s.body));
            set
        }
        ForIn(s) => {
            let mut set = collect_namespaces_from_expression(&s.iterable);
            set.extend(collect_namespaces_from_block(&s.body));
            set
        }
        Try(s) => {
            let mut set = collect_namespaces_from_block(&s.try_block);
            for catch in &s.catch_blocks {
                set.extend(collect_namespaces_from_block(&catch.body));
            }
            if let Some(fin) = &s.finally_block {
                set.extend(collect_namespaces_from_block(fin));
            }
            set
        }
        Event(e) => e
            .data
            .values()
            .flat_map(collect_namespaces_from_expression)
            .collect(),
        Function(f) => collect_namespaces_from_block(&f.body),
        Service(s) => s
            .methods
            .iter()
            .flat_map(|m| collect_namespaces_from_block(&m.body))
            .collect(),
        Agent(a) => collect_namespaces_from_block(&a.body),
        Spawn(s) => collect_namespaces_from_block(&s.body),
        Import(_) | Break(_) | Continue(_) | Message(_) => HashSet::new(),
        Loop(loop_stmt) => collect_namespaces_from_block(&loop_stmt.body),
        Match(match_stmt) => {
            let mut set = collect_namespaces_from_expression(&match_stmt.expression);
            for case in &match_stmt.cases {
                set.extend(collect_namespaces_from_block(&case.body));
                if let crate::parser::ast::MatchPattern::Range(start, end) = &case.pattern {
                    set.extend(collect_namespaces_from_expression(start.as_ref()));
                    set.extend(collect_namespaces_from_expression(end.as_ref()));
                }
            }
            if let Some(default_body) = &match_stmt.default_case {
                set.extend(collect_namespaces_from_block(default_body));
            }
            set
        }
    }
}

fn collect_namespaces_from_expression(expr: &Expression) -> HashSet<String> {
    use crate::parser::ast::Expression::*;
    let mut out = HashSet::new();
    match expr {
        FunctionCall(call) => {
            if let Some((ns, _)) = call.name.split_once("::") {
                out.insert(ns.to_string());
            }
            for arg in &call.arguments {
                out.extend(collect_namespaces_from_expression(arg));
            }
        }
        BinaryOp(l, _, r) => {
            out.extend(collect_namespaces_from_expression(l));
            out.extend(collect_namespaces_from_expression(r));
        }
        UnaryOp(_, e) => {
            out.extend(collect_namespaces_from_expression(e));
        }
        Assignment(_, e) => {
            out.extend(collect_namespaces_from_expression(e));
        }
        FieldAccess(obj, _) => {
            out.extend(collect_namespaces_from_expression(obj));
        }
        FieldAssignment(obj, _, v) => {
            out.extend(collect_namespaces_from_expression(obj));
            out.extend(collect_namespaces_from_expression(v));
        }
        Await(e) | Spawn(e) | Throw(e) => {
            out.extend(collect_namespaces_from_expression(e));
        }
        ObjectLiteral(map) => {
            for e in map.values() {
                out.extend(collect_namespaces_from_expression(e));
            }
        }
        ArrayLiteral(items) => {
            for e in items {
                out.extend(collect_namespaces_from_expression(e));
            }
        }
        IndexAccess(obj, idx) => {
            out.extend(collect_namespaces_from_expression(obj));
            out.extend(collect_namespaces_from_expression(idx));
        }
        ArrowFunction { body, .. } => {
            out.extend(collect_namespaces_from_block(body));
        }
        Range(start, end) => {
            out.extend(collect_namespaces_from_expression(start));
            out.extend(collect_namespaces_from_expression(end));
        }
        MethodCall {
            receiver,
            arguments,
            ..
        } => {
            out.extend(collect_namespaces_from_expression(receiver));
            for arg in arguments {
                out.extend(collect_namespaces_from_expression(arg));
            }
        }
        Literal(_) | Identifier(_) => {}
    }
    out
}

fn partition_methods_for_targets(
    service: &ServiceStatement,
) -> (Vec<FunctionStatement>, Vec<FunctionStatement>) {
    // In hybrid mode, auth/cloudadmin blocks are orchestration-only and should compile to HTTP path.
    let split_namespaces: HashSet<String> = ["auth", "cloud", "cloudadmin"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let is_hybrid = service_declares_hybrid(service);
    let mut evm_methods = Vec::new();
    let mut http_methods = Vec::new();
    for method in &service.methods {
        if is_hybrid {
            let used = collect_namespaces_from_block(&method.body);
            let routed_to_http = used.intersection(&split_namespaces).next().is_some();
            if routed_to_http {
                http_methods.push(method.clone());
                continue;
            }
        }
        evm_methods.push(method.clone());
    }
    (evm_methods, http_methods)
}

fn emit_http_split_artifact(
    service: &ServiceStatement,
    http_methods: &[FunctionStatement],
    output_dir: &std::path::Path,
) -> Result<Option<std::path::PathBuf>, CompileError> {
    if http_methods.is_empty() {
        return Ok(None);
    }
    let path = output_dir.join(format!("{}.http.json", service.name));
    let method_names: Vec<String> = http_methods.iter().map(|m| m.name.clone()).collect();
    let payload = json!({
        "service": service.name,
        "kind": "hybrid_http_split",
        "reason": "auth/cloudadmin namespaces are routed to HTTP/orchestration target",
        "methods": method_names,
    });
    std::fs::write(&path, payload.to_string()).map_err(CompileError::Io)?;
    Ok(Some(path))
}

/// Check that solc is available.
fn check_solc_available() -> bool {
    if let Some(available) = super::get_compiler_available_override() {
        return available;
    }
    Command::new("solc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Blockchain backend: DAL → Solidity → solc → bytecode + ABI.
pub struct BlockchainBackend;

impl CompileBackend for BlockchainBackend {
    fn compile(
        &self,
        _program: &Program,
        services: &[&ServiceStatement],
        opts: &CompileOptions,
    ) -> Result<CompileArtifacts, CompileError> {
        std::fs::create_dir_all(&opts.output_dir).map_err(CompileError::Io)?;
        let mut artifact_paths = Vec::new();
        let mut service_names = Vec::new();
        let mut needs_solc = false;

        for service in services {
            let (evm_methods, http_methods) = partition_methods_for_targets(service);
            if let Some(http_artifact) =
                emit_http_split_artifact(service, &http_methods, &opts.output_dir)?
            {
                artifact_paths.push(http_artifact);
            }
            if evm_methods.is_empty() {
                service_names.push(service.name.clone());
                continue;
            }
            needs_solc = true;
            let solidity = service_to_solidity(service, &evm_methods)?;
            let sol_name = format!("{}.sol", service.name);
            let sol_path = opts.output_dir.join(&sol_name);
            std::fs::create_dir_all(&opts.output_dir).map_err(CompileError::Io)?;
            std::fs::write(&sol_path, solidity).map_err(CompileError::Io)?;
            artifact_paths.push(sol_path.clone());
            service_names.push(service.name.clone());
        }

        if needs_solc && !check_solc_available() {
            return Err(CompileError::CompilerNotFound {
                target: "blockchain".to_string(),
                hint: "Install solc: https://github.com/ethereum/solidity/releases".to_string(),
            });
        }

        for service in services {
            let (evm_methods, _http_methods) = partition_methods_for_targets(service);
            if evm_methods.is_empty() {
                continue;
            }
            let sol_path = opts.output_dir.join(format!("{}.sol", service.name));
            let out = Command::new("solc")
                .arg("--bin")
                .arg("--abi")
                .arg("--optimize")
                .arg("--output-dir")
                .arg(&opts.output_dir)
                .arg(&sol_path)
                .output()
                .map_err(|e| CompileError::Io(e))?;

            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                return Err(CompileError::Backend(format!(
                    "solc failed: {}",
                    stderr.trim()
                )));
            }

            let bin_path = opts.output_dir.join(format!("{}.bin", service.name));
            let abi_path = opts.output_dir.join(format!("{}.abi", service.name));
            if bin_path.exists() {
                artifact_paths.push(bin_path);
            }
            if abi_path.exists() {
                artifact_paths.push(abi_path);
            }
        }

        Ok(CompileArtifacts {
            target: "blockchain".to_string(),
            service_names,
            artifact_paths,
            stub: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::CompileBackend;

    #[test]
    fn hybrid_partition_routes_auth_and_cloudadmin_to_http() {
        let source = r#"
@secure
@trust("hybrid")
@chain("ethereum")
service Token @compile_target("blockchain") {
    fn onchain_transfer(to: string, amount: int) {
        chain::call(1, "0x1234", "transfer", {"to": to, "amount": amount});
    }
    fn authorize_transfer(user: string) {
        let session = auth::session(user, ["user"]);
        cloudadmin::authorize("admin", "transfer", "token");
    }
}
"#;
        let program = crate::parse_source(source).expect("parse");
        let services = crate::compile::select_services_for_target(
            &program,
            &crate::lexer::tokens::CompilationTarget::Blockchain,
        );
        let service = services[0];
        let (evm, http) = partition_methods_for_targets(service);
        assert!(evm.iter().any(|m| m.name == "onchain_transfer"));
        assert!(http.iter().any(|m| m.name == "authorize_transfer"));
    }

    #[test]
    fn emits_http_artifact_when_service_is_http_only_after_split() {
        let source = r#"
@secure
@trust("hybrid")
@chain("ethereum")
service Ops @compile_target("blockchain") {
    fn admin_task() {
        auth::session("user", ["admin"]);
    }
}
"#;
        let program = crate::parse_source(source).expect("parse");
        let services = crate::compile::select_services_for_target(
            &program,
            &crate::lexer::tokens::CompilationTarget::Blockchain,
        );
        let out = tempfile::tempdir().expect("tempdir");
        let backend = BlockchainBackend;
        let opts = crate::compile::CompileOptions {
            entry_path: out.path().join("main.dal"),
            target: crate::lexer::tokens::CompilationTarget::Blockchain,
            output_dir: out.path().join("out"),
            trust_mode: crate::compile::TrustCompileMode::Auto,
        };
        let artifacts = backend
            .compile(&program, &services, &opts)
            .expect("compile");
        let has_http = artifacts
            .artifact_paths
            .iter()
            .any(|p| p.extension().map(|e| e == "json").unwrap_or(false));
        assert!(has_http, "expected http split artifact path");
    }

    #[test]
    fn decentralized_v1_generates_non_stub_method_body_for_supported_pattern() {
        use crate::lexer::tokens::CompilationTarget;
        use crate::parser::ast::{
            Attribute, AttributeTarget, BlockStatement, EventDeclaration, EventStatement,
            Expression, FieldVisibility, FunctionStatement, IfStatement, Parameter,
            ReturnStatement, ServiceField, ServiceStatement, Statement,
        };
        use std::collections::HashMap;

        let method = FunctionStatement {
            name: "bump".to_string(),
            parameters: vec![Parameter {
                name: "amount".to_string(),
                param_type: Some("int".to_string()),
            }],
            return_type: Some("int".to_string()),
            body: BlockStatement {
                statements: vec![
                    Statement::If(IfStatement {
                        condition: Expression::BinaryOp(
                            Box::new(Expression::Identifier("amount".to_string())),
                            crate::lexer::tokens::Operator::Greater,
                            Box::new(Expression::Literal(crate::lexer::tokens::Literal::Int(0))),
                        ),
                        consequence: BlockStatement {
                            statements: vec![
                                Statement::Expression(Expression::FieldAssignment(
                                    Box::new(Expression::Identifier("self".to_string())),
                                    "value".to_string(),
                                    Box::new(Expression::BinaryOp(
                                        Box::new(Expression::FieldAccess(
                                            Box::new(Expression::Identifier("self".to_string())),
                                            "value".to_string(),
                                        )),
                                        crate::lexer::tokens::Operator::Plus,
                                        Box::new(Expression::Identifier("amount".to_string())),
                                    )),
                                )),
                                Statement::Event(EventStatement {
                                    event_name: "Updated".to_string(),
                                    data: {
                                        let mut m = HashMap::new();
                                        m.insert(
                                            "value".to_string(),
                                            Expression::Identifier("amount".to_string()),
                                        );
                                        m
                                    },
                                }),
                                Statement::Return(ReturnStatement {
                                    value: Some(Expression::FieldAccess(
                                        Box::new(Expression::Identifier("self".to_string())),
                                        "value".to_string(),
                                    )),
                                }),
                            ],
                        },
                        alternative: None,
                    }),
                    Statement::Return(ReturnStatement {
                        value: Some(Expression::FieldAccess(
                            Box::new(Expression::Identifier("self".to_string())),
                            "value".to_string(),
                        )),
                    }),
                ],
            },
            attributes: vec![],
            is_async: false,
            exported: false,
        };

        let service = ServiceStatement {
            name: "Counter".to_string(),
            attributes: vec![
                Attribute {
                    name: "@trust".to_string(),
                    parameters: vec![Expression::Literal(crate::lexer::tokens::Literal::String(
                        "decentralized".to_string(),
                    ))],
                    target: AttributeTarget::Module,
                },
                Attribute {
                    name: "@chain".to_string(),
                    parameters: vec![Expression::Literal(crate::lexer::tokens::Literal::String(
                        "ethereum".to_string(),
                    ))],
                    target: AttributeTarget::Module,
                },
            ],
            fields: vec![ServiceField {
                name: "value".to_string(),
                field_type: "int".to_string(),
                initial_value: None,
                visibility: FieldVisibility::Private,
            }],
            methods: vec![method.clone()],
            events: vec![EventDeclaration {
                name: "Updated".to_string(),
                parameters: vec![Parameter {
                    name: "value".to_string(),
                    param_type: Some("int".to_string()),
                }],
            }],
            compilation_target: Some(crate::parser::ast::CompilationTargetInfo {
                target: CompilationTarget::Blockchain,
                constraints: crate::lexer::tokens::TargetConstraint::new(
                    CompilationTarget::Blockchain,
                ),
                validation_errors: vec![],
            }),
            exported: false,
        };

        let solidity = service_to_solidity(&service, &[method]).expect("solidity generation");
        assert!(
            solidity.contains("value = (value + amount);"),
            "expected lowered assignment, got:\n{}",
            solidity
        );
        assert!(
            solidity.contains("emit Updated(amount);"),
            "expected lowered event emit, got:\n{}",
            solidity
        );
        assert!(
            !solidity.contains("DAL transpiled; implement in Solidity"),
            "decentralized supported pattern should not emit revert stubs"
        );
    }

    #[test]
    fn decentralized_v1_rejects_codegen_for_unsupported_method_pattern() {
        let source = r#"
@secure
@trust("decentralized")
@chain("ethereum")
service Counter @compile_target("blockchain") {
    fn bad() {
        chain::get_gas_price(1);
    }
}
"#;
        let program = crate::parse_source(source).expect("parse");
        let services = crate::compile::select_services_for_target(
            &program,
            &crate::lexer::tokens::CompilationTarget::Blockchain,
        );
        let out = tempfile::tempdir().expect("tempdir");
        let backend = BlockchainBackend;
        let opts = crate::compile::CompileOptions {
            entry_path: out.path().join("main.dal"),
            target: crate::lexer::tokens::CompilationTarget::Blockchain,
            output_dir: out.path().join("out"),
            trust_mode: crate::compile::TrustCompileMode::Auto,
        };
        let err = backend
            .compile(&program, &services, &opts)
            .expect_err("expected unsupported decentralized-v1 codegen rejection");
        assert!(
            format!("{}", err).contains("cannot be codegened in decentralized-v1"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn decentralized_v1_infers_local_types_for_binary_and_identifier_flows() {
        use crate::parser::ast::{
            Attribute, AttributeTarget, BlockStatement, Expression, FieldVisibility,
            FunctionStatement, IfStatement, LetStatement, Parameter, ReturnStatement, ServiceField,
            ServiceStatement, Statement,
        };

        let method = FunctionStatement {
            name: "calc".to_string(),
            parameters: vec![Parameter {
                name: "amount".to_string(),
                param_type: Some("int".to_string()),
            }],
            return_type: Some("int".to_string()),
            body: BlockStatement {
                statements: vec![
                    Statement::Let(LetStatement {
                        name: "next".to_string(),
                        value: Expression::BinaryOp(
                            Box::new(Expression::Identifier("amount".to_string())),
                            crate::lexer::tokens::Operator::Plus,
                            Box::new(Expression::Literal(crate::lexer::tokens::Literal::Int(1))),
                        ),
                        line: None,
                    }),
                    Statement::Let(LetStatement {
                        name: "is_large".to_string(),
                        value: Expression::BinaryOp(
                            Box::new(Expression::Identifier("next".to_string())),
                            crate::lexer::tokens::Operator::Greater,
                            Box::new(Expression::Literal(crate::lexer::tokens::Literal::Int(10))),
                        ),
                        line: None,
                    }),
                    Statement::If(IfStatement {
                        condition: Expression::Identifier("is_large".to_string()),
                        consequence: BlockStatement {
                            statements: vec![
                                Statement::Expression(Expression::FieldAssignment(
                                    Box::new(Expression::Identifier("self".to_string())),
                                    "value".to_string(),
                                    Box::new(Expression::Identifier("next".to_string())),
                                )),
                                Statement::Return(ReturnStatement {
                                    value: Some(Expression::FieldAccess(
                                        Box::new(Expression::Identifier("self".to_string())),
                                        "value".to_string(),
                                    )),
                                }),
                            ],
                        },
                        alternative: None,
                    }),
                    Statement::Return(ReturnStatement {
                        value: Some(Expression::Identifier("next".to_string())),
                    }),
                ],
            },
            attributes: vec![],
            is_async: false,
            exported: false,
        };

        let service = ServiceStatement {
            name: "Calc".to_string(),
            attributes: vec![Attribute {
                name: "@trust".to_string(),
                parameters: vec![Expression::Literal(crate::lexer::tokens::Literal::String(
                    "decentralized".to_string(),
                ))],
                target: AttributeTarget::Module,
            }],
            fields: vec![ServiceField {
                name: "value".to_string(),
                field_type: "int".to_string(),
                initial_value: None,
                visibility: FieldVisibility::Private,
            }],
            methods: vec![method],
            events: vec![],
            compilation_target: None,
            exported: false,
        };

        let solidity = service_to_solidity(&service, service.methods.as_slice());
        let solidity = solidity.expect("solidity generation");
        assert!(
            solidity.contains("int256 next = (amount + 1);"),
            "expected inferred int local binding, got:\n{}",
            solidity
        );
        assert!(
            solidity.contains("bool is_large = (next > 10);"),
            "expected inferred bool local binding, got:\n{}",
            solidity
        );
    }

    #[test]
    fn decentralized_v1_emits_field_initializers_when_supported() {
        use crate::parser::ast::{
            Attribute, AttributeTarget, Expression, FieldVisibility, ServiceField, ServiceStatement,
        };

        let service = ServiceStatement {
            name: "InitCounter".to_string(),
            attributes: vec![Attribute {
                name: "@trust".to_string(),
                parameters: vec![Expression::Literal(crate::lexer::tokens::Literal::String(
                    "decentralized".to_string(),
                ))],
                target: AttributeTarget::Module,
            }],
            fields: vec![ServiceField {
                name: "value".to_string(),
                field_type: "int".to_string(),
                initial_value: Some(Expression::Literal(crate::lexer::tokens::Literal::Int(7))),
                visibility: FieldVisibility::Private,
            }],
            methods: vec![],
            events: vec![],
            compilation_target: None,
            exported: false,
        };

        let solidity = service_to_solidity(&service, &[]).expect("solidity generation");
        assert!(
            solidity.contains("int256 value = 7;"),
            "expected deterministic field initializer emission, got:\n{}",
            solidity
        );
    }

    #[test]
    fn decentralized_v1_lowers_unary_and_boolean_expression_family() {
        use crate::parser::ast::{
            Attribute, AttributeTarget, BlockStatement, Expression, FieldVisibility,
            FunctionStatement, LetStatement, Parameter, ReturnStatement, ServiceField,
            ServiceStatement, Statement,
        };

        let method = FunctionStatement {
            name: "guarded".to_string(),
            parameters: vec![Parameter {
                name: "amount".to_string(),
                param_type: Some("int".to_string()),
            }],
            return_type: Some("bool".to_string()),
            body: BlockStatement {
                statements: vec![
                    Statement::Let(LetStatement {
                        name: "negative".to_string(),
                        value: Expression::UnaryOp(
                            crate::lexer::tokens::Operator::Minus,
                            Box::new(Expression::Identifier("amount".to_string())),
                        ),
                        line: None,
                    }),
                    Statement::Let(LetStatement {
                        name: "is_valid".to_string(),
                        value: Expression::BinaryOp(
                            Box::new(Expression::BinaryOp(
                                Box::new(Expression::Identifier("amount".to_string())),
                                crate::lexer::tokens::Operator::Greater,
                                Box::new(Expression::Literal(crate::lexer::tokens::Literal::Int(
                                    0,
                                ))),
                            )),
                            crate::lexer::tokens::Operator::And,
                            Box::new(Expression::BinaryOp(
                                Box::new(Expression::Identifier("negative".to_string())),
                                crate::lexer::tokens::Operator::Less,
                                Box::new(Expression::Literal(crate::lexer::tokens::Literal::Int(
                                    0,
                                ))),
                            )),
                        ),
                        line: None,
                    }),
                    Statement::Return(ReturnStatement {
                        value: Some(Expression::Identifier("is_valid".to_string())),
                    }),
                ],
            },
            attributes: vec![],
            is_async: false,
            exported: false,
        };

        let service = ServiceStatement {
            name: "Logic".to_string(),
            attributes: vec![Attribute {
                name: "@trust".to_string(),
                parameters: vec![Expression::Literal(crate::lexer::tokens::Literal::String(
                    "decentralized".to_string(),
                ))],
                target: AttributeTarget::Module,
            }],
            fields: vec![ServiceField {
                name: "value".to_string(),
                field_type: "int".to_string(),
                initial_value: None,
                visibility: FieldVisibility::Private,
            }],
            methods: vec![method],
            events: vec![],
            compilation_target: None,
            exported: false,
        };

        let solidity =
            service_to_solidity(&service, service.methods.as_slice()).expect("solidity generation");
        assert!(
            solidity.contains("int256 negative = (-amount);"),
            "expected unary minus lowering, got:\n{}",
            solidity
        );
        assert!(
            solidity.contains("bool is_valid = ((amount > 0) && (negative < 0));"),
            "expected boolean conjunction lowering, got:\n{}",
            solidity
        );
        assert!(
            solidity.contains("return is_valid;"),
            "expected return lowering for bool identifier, got:\n{}",
            solidity
        );
    }

    #[test]
    fn decentralized_v1_equivalence_shape_for_state_update_path() {
        use crate::parser::ast::{
            Attribute, AttributeTarget, BlockStatement, Expression, FieldVisibility,
            FunctionStatement, IfStatement, Parameter, ReturnStatement, ServiceField,
            ServiceStatement, Statement,
        };

        let method = FunctionStatement {
            name: "apply_delta".to_string(),
            parameters: vec![Parameter {
                name: "delta".to_string(),
                param_type: Some("int".to_string()),
            }],
            return_type: Some("int".to_string()),
            body: BlockStatement {
                statements: vec![Statement::If(IfStatement {
                    condition: Expression::BinaryOp(
                        Box::new(Expression::Identifier("delta".to_string())),
                        crate::lexer::tokens::Operator::Greater,
                        Box::new(Expression::Literal(crate::lexer::tokens::Literal::Int(0))),
                    ),
                    consequence: BlockStatement {
                        statements: vec![
                            Statement::Expression(Expression::FieldAssignment(
                                Box::new(Expression::Identifier("self".to_string())),
                                "total".to_string(),
                                Box::new(Expression::BinaryOp(
                                    Box::new(Expression::FieldAccess(
                                        Box::new(Expression::Identifier("self".to_string())),
                                        "total".to_string(),
                                    )),
                                    crate::lexer::tokens::Operator::Plus,
                                    Box::new(Expression::Identifier("delta".to_string())),
                                )),
                            )),
                            Statement::Return(ReturnStatement {
                                value: Some(Expression::FieldAccess(
                                    Box::new(Expression::Identifier("self".to_string())),
                                    "total".to_string(),
                                )),
                            }),
                        ],
                    },
                    alternative: Some(BlockStatement {
                        statements: vec![Statement::Return(ReturnStatement {
                            value: Some(Expression::FieldAccess(
                                Box::new(Expression::Identifier("self".to_string())),
                                "total".to_string(),
                            )),
                        })],
                    }),
                })],
            },
            attributes: vec![],
            is_async: false,
            exported: false,
        };

        let service = ServiceStatement {
            name: "Ledger".to_string(),
            attributes: vec![Attribute {
                name: "@trust".to_string(),
                parameters: vec![Expression::Literal(crate::lexer::tokens::Literal::String(
                    "decentralized".to_string(),
                ))],
                target: AttributeTarget::Module,
            }],
            fields: vec![ServiceField {
                name: "total".to_string(),
                field_type: "int".to_string(),
                initial_value: Some(Expression::Literal(crate::lexer::tokens::Literal::Int(0))),
                visibility: FieldVisibility::Private,
            }],
            methods: vec![method],
            events: vec![],
            compilation_target: None,
            exported: false,
        };

        let solidity =
            service_to_solidity(&service, service.methods.as_slice()).expect("solidity generation");
        let expected = [
            "int256 total = 0;",
            "function apply_delta(int256 delta) public returns (int256 ) {",
            "if ((delta > 0)) {",
            "total = (total + delta);",
            "return total;",
            "} else {",
            "return total;",
            "}",
        ];
        let mut cursor = 0usize;
        for needle in expected {
            let slice = &solidity[cursor..];
            let pos = slice.find(needle).unwrap_or_else(|| {
                panic!(
                    "expected ordered fragment '{}' not found in generated Solidity:\n{}",
                    needle, solidity
                )
            });
            cursor += pos + needle.len();
        }
    }

    #[test]
    fn decentralized_v1_lowers_map_index_read_and_write_for_self_fields() {
        use crate::parser::ast::{
            Attribute, AttributeTarget, BlockStatement, Expression, FieldVisibility,
            FunctionCall as AstFunctionCall, FunctionStatement, LetStatement, Parameter,
            ReturnStatement, ServiceField, ServiceStatement, Statement,
        };

        // Method: fn transfer(to: address, amount: int) -> int
        //   self.balances[to] = self.balances[to] + amount
        //   let bal = self.balances[to]
        //   return bal
        let to_id = || Expression::Identifier("to".to_string());
        let self_balances = || {
            Expression::FieldAccess(
                Box::new(Expression::Identifier("self".to_string())),
                "balances".to_string(),
            )
        };
        let map_read = || Expression::IndexAccess(Box::new(self_balances()), Box::new(to_id()));

        let method = FunctionStatement {
            name: "transfer".to_string(),
            parameters: vec![
                Parameter {
                    name: "to".to_string(),
                    param_type: Some("address".to_string()),
                },
                Parameter {
                    name: "amount".to_string(),
                    param_type: Some("int".to_string()),
                },
            ],
            return_type: Some("int".to_string()),
            body: BlockStatement {
                statements: vec![
                    // self.balances[to] = self.balances[to] + amount
                    Statement::Expression(Expression::FunctionCall(AstFunctionCall {
                        name: "__index_assign__".to_string(),
                        arguments: vec![
                            self_balances(),
                            to_id(),
                            Expression::BinaryOp(
                                Box::new(map_read()),
                                crate::lexer::tokens::Operator::Plus,
                                Box::new(Expression::Identifier("amount".to_string())),
                            ),
                            Expression::Literal(crate::lexer::tokens::Literal::String(
                                String::new(),
                            )),
                            Expression::Literal(crate::lexer::tokens::Literal::String(
                                "balances".to_string(),
                            )),
                        ],
                    })),
                    // let bal = self.balances[to]
                    Statement::Let(LetStatement {
                        name: "bal".to_string(),
                        value: map_read(),
                        line: None,
                    }),
                    Statement::Return(ReturnStatement {
                        value: Some(Expression::Identifier("bal".to_string())),
                    }),
                ],
            },
            attributes: vec![],
            is_async: false,
            exported: false,
        };

        let service = ServiceStatement {
            name: "Token".to_string(),
            attributes: vec![Attribute {
                name: "@trust".to_string(),
                parameters: vec![Expression::Literal(crate::lexer::tokens::Literal::String(
                    "decentralized".to_string(),
                ))],
                target: AttributeTarget::Module,
            }],
            fields: vec![ServiceField {
                name: "balances".to_string(),
                field_type: "map<address,int>".to_string(),
                initial_value: None,
                visibility: FieldVisibility::Private,
            }],
            methods: vec![method],
            events: vec![],
            compilation_target: None,
            exported: false,
        };

        let solidity =
            service_to_solidity(&service, service.methods.as_slice()).expect("solidity generation");

        let expected = [
            "mapping(address => int256) balances;",
            "function transfer(address to, int256 amount) public returns (int256 ) {",
            "balances[to] = (balances[to] + amount);",
            "int256 bal = balances[to];",
            "return bal;",
        ];
        let mut cursor = 0usize;
        for needle in expected {
            let slice = &solidity[cursor..];
            let pos = slice.find(needle).unwrap_or_else(|| {
                panic!(
                    "expected ordered fragment '{}' not found in generated Solidity:\n{}",
                    needle, solidity
                )
            });
            cursor += pos + needle.len();
        }

        assert!(
            !solidity.contains("DAL transpiled; implement in Solidity"),
            "map index codegen should not emit revert stubs"
        );
    }

    #[test]
    fn decentralized_v1_rejects_non_self_index_access() {
        let expr = Expression::IndexAccess(
            Box::new(Expression::Identifier("localvar".to_string())),
            Box::new(Expression::Literal(crate::lexer::tokens::Literal::Int(0))),
        );
        let result = expression_to_solidity(&expr);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("only supported for self.<map_field>[key]"),);
    }

    #[test]
    fn extract_map_value_type_parses_supported_map_types() {
        assert_eq!(
            extract_map_value_type("map<address,int>"),
            Some("int".to_string())
        );
        assert_eq!(
            extract_map_value_type("map<string,int>"),
            Some("int".to_string())
        );
        assert_eq!(extract_map_value_type("int"), None);
        assert_eq!(extract_map_value_type("list<int>"), None);
    }

    #[test]
    fn decentralized_v1_lowers_internal_function_calls() {
        use crate::parser::ast::{
            Attribute, AttributeTarget, BlockStatement, Expression,
            FunctionCall as AstFunctionCall, FunctionStatement, LetStatement, Parameter,
            ReturnStatement, ServiceStatement, Statement,
        };

        let helper = FunctionStatement {
            name: "double".to_string(),
            parameters: vec![Parameter {
                name: "x".to_string(),
                param_type: Some("int".to_string()),
            }],
            return_type: Some("int".to_string()),
            body: BlockStatement {
                statements: vec![Statement::Return(ReturnStatement {
                    value: Some(Expression::BinaryOp(
                        Box::new(Expression::Identifier("x".to_string())),
                        crate::lexer::tokens::Operator::Star,
                        Box::new(Expression::Literal(crate::lexer::tokens::Literal::Int(2))),
                    )),
                })],
            },
            attributes: vec![],
            is_async: false,
            exported: false,
        };

        let main_method = FunctionStatement {
            name: "quadruple".to_string(),
            parameters: vec![Parameter {
                name: "n".to_string(),
                param_type: Some("int".to_string()),
            }],
            return_type: Some("int".to_string()),
            body: BlockStatement {
                statements: vec![
                    Statement::Let(LetStatement {
                        name: "d".to_string(),
                        value: Expression::FunctionCall(AstFunctionCall {
                            name: "double".to_string(),
                            arguments: vec![Expression::Identifier("n".to_string())],
                        }),
                        line: None,
                    }),
                    Statement::Return(ReturnStatement {
                        value: Some(Expression::FunctionCall(AstFunctionCall {
                            name: "double".to_string(),
                            arguments: vec![Expression::Identifier("d".to_string())],
                        })),
                    }),
                ],
            },
            attributes: vec![],
            is_async: false,
            exported: false,
        };

        let service = ServiceStatement {
            name: "Math".to_string(),
            attributes: vec![Attribute {
                name: "@trust".to_string(),
                parameters: vec![Expression::Literal(crate::lexer::tokens::Literal::String(
                    "decentralized".to_string(),
                ))],
                target: AttributeTarget::Module,
            }],
            fields: vec![],
            methods: vec![helper.clone(), main_method.clone()],
            events: vec![],
            compilation_target: None,
            exported: false,
        };

        let solidity =
            service_to_solidity(&service, &[helper, main_method]).expect("solidity generation");

        let expected = [
            "function double(int256 x) public returns (int256 )",
            "return (x * 2);",
            "function quadruple(int256 n) public returns (int256 )",
            "int256 d = double(n);",
            "return double(d);",
        ];
        let mut cursor = 0usize;
        for needle in expected {
            let slice = &solidity[cursor..];
            let pos = slice.find(needle).unwrap_or_else(|| {
                panic!(
                    "expected ordered fragment '{}' not found in generated Solidity:\n{}",
                    needle, solidity
                )
            });
            cursor += pos + needle.len();
        }

        assert!(
            !solidity.contains("DAL transpiled; implement in Solidity"),
            "internal function calls should not emit revert stubs"
        );
    }

    #[test]
    fn decentralized_v1_rejects_namespace_function_calls() {
        use crate::parser::ast::{Expression, FunctionCall as AstFunctionCall};

        let expr = Expression::FunctionCall(AstFunctionCall {
            name: "log::info".to_string(),
            arguments: vec![
                Expression::Literal(crate::lexer::tokens::Literal::String("tag".to_string())),
                Expression::Literal(crate::lexer::tokens::Literal::String("msg".to_string())),
            ],
        });
        let result = expression_to_solidity(&expr);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("namespace call"),
            "namespace calls should be rejected with clear diagnostic"
        );
    }

    #[test]
    fn decentralized_v1_e2e_source_parse_compile_produces_real_solidity() {
        let source = r#"
@secure
@trust("decentralized")
@chain("ethereum")
service Vault @compile_target("blockchain") {
    balance: int = 0;
    locked: bool = false;
    deposits: map<address,int> = {};

    event Deposited(who: address, amount: int);
    event Withdrawn(who: address, amount: int);

    fn deposit(sender: address, amount: int) -> int {
        if (amount > 0) {
            self.deposits[sender] = self.deposits[sender] + amount;
            self.balance = self.balance + amount;
            event Deposited { who: sender, amount: amount };
        }
        return self.balance;
    }

    fn withdraw(sender: address, amount: int) -> int {
        if (self.locked == false) {
            if (self.deposits[sender] >= amount) {
                self.deposits[sender] = self.deposits[sender] - amount;
                self.balance = self.balance - amount;
                event Withdrawn { who: sender, amount: amount };
            }
        }
        return self.balance;
    }

    fn is_locked() -> bool {
        return self.locked;
    }
}
"#;
        let program = crate::parse_source(source).expect("parse DAL source");
        let services = crate::compile::select_services_for_target(
            &program,
            &crate::lexer::tokens::CompilationTarget::Blockchain,
        );
        assert!(
            !services.is_empty(),
            "should find at least one blockchain service"
        );
        let service = services[0];

        let solidity = service_to_solidity(service, service.methods.as_slice())
            .expect("e2e solidity generation from parsed source");

        let expected_fragments = [
            "pragma solidity ^0.8.0;",
            "contract Vault {",
            "int256 balance = 0;",
            "bool locked = false;",
            "mapping(address => int256) deposits;",
            "event Deposited(address who, int256 amount);",
            "event Withdrawn(address who, int256 amount);",
            "function deposit(address sender, int256 amount) public returns (int256 )",
            "if ((amount > 0)) {",
            "deposits[sender] = (deposits[sender] + amount);",
            "balance = (balance + amount);",
            "emit Deposited(sender, amount);",
            "return balance;",
            "function withdraw(address sender, int256 amount) public returns (int256 )",
            "if ((locked == false)) {",
            "if ((deposits[sender] >= amount)) {",
            "deposits[sender] = (deposits[sender] - amount);",
            "balance = (balance - amount);",
            "emit Withdrawn(sender, amount);",
            "function is_locked() public returns (bool )",
            "return locked;",
        ];

        let mut cursor = 0usize;
        for needle in expected_fragments {
            let slice = &solidity[cursor..];
            let pos = slice.find(needle).unwrap_or_else(|| {
                panic!(
                    "expected ordered fragment '{}' not found in generated Solidity (from pos {}):\n{}",
                    needle, cursor, solidity
                )
            });
            cursor += pos + needle.len();
        }

        assert!(
            !solidity.contains("DAL transpiled; implement in Solidity"),
            "e2e decentralized codegen should produce no stubs:\n{}",
            solidity
        );
    }
}
