//! CT2: Blockchain (EVM) backend — transpile DAL services to Solidity and invoke solc.
//! See docs/development/implementation/COMPILE_TARGET_IMPLEMENTATION_PLAN.md.

use crate::compile::{CompileArtifacts, CompileError, CompileOptions, CompileBackend};
use crate::parser::ast::{Program, ServiceStatement};
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

/// Emit Solidity source for a single DAL service (contract name, state vars, functions, events).
fn service_to_solidity(service: &ServiceStatement) -> String {
    let mut out = String::new();
    out.push_str("// SPDX-License-Identifier: MIT\n");
    out.push_str("pragma solidity ^0.8.0;\n\n");
    out.push_str(&format!("contract {} {{\n", service.name));

    for field in &service.fields {
        let sol_type = dal_type_to_solidity(&field.field_type);
        out.push_str(&format!("    {} {};\n", sol_type, field.name));
    }
    if !service.events.is_empty() {
        out.push('\n');
        for ev in &service.events {
            let params: Vec<String> = ev
                .parameters
                .iter()
                .map(|p| {
                    let ty = p
                        .param_type
                        .as_deref()
                        .unwrap_or("uint256");
                    format!("{} {}", dal_type_to_solidity(ty), p.name)
                })
                .collect();
            out.push_str(&format!("    event {}({});\n", ev.name, params.join(", ")));
        }
    }
    if !service.methods.is_empty() {
        out.push('\n');
        for method in &service.methods {
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
                "    function {}({}) public{} {{\n        revert(\"DAL transpiled; implement in Solidity\");\n    }}\n",
                method.name,
                params.join(", "),
                ret
            ));
        }
    }

    out.push_str("}\n");
    out
}

/// Check that solc is available.
fn check_solc_available() -> bool {
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
        if !check_solc_available() {
            return Err(CompileError::CompilerNotFound {
                target: "blockchain".to_string(),
                hint: "Install solc: https://github.com/ethereum/solidity/releases".to_string(),
            });
        }

        let mut artifact_paths = Vec::new();
        let mut service_names = Vec::new();

        for service in services {
            let solidity = service_to_solidity(service);
            let sol_name = format!("{}.sol", service.name);
            let sol_path = opts.output_dir.join(&sol_name);
            std::fs::create_dir_all(&opts.output_dir).map_err(CompileError::Io)?;
            std::fs::write(&sol_path, solidity).map_err(CompileError::Io)?;

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
            service_names.push(service.name.clone());
        }

        Ok(CompileArtifacts {
            target: "blockchain".to_string(),
            service_names,
            artifact_paths,
            stub: false,
        })
    }
}
