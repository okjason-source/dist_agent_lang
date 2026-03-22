use crate::runtime::values::Value;
use serde_json;
use std::collections::HashMap;
use std::sync::Mutex;

/// Add Solidity — utilities for integrating with Solidity contracts (ABI parse, register, call, events).

#[derive(Debug, Clone)]
pub struct SolidityContract {
    pub name: String,
    pub address: String,
    pub chain_id: i64,
    pub abi: Option<String>, // JSON ABI string
}

#[derive(Debug, Clone)]
pub struct ContractFunction {
    pub name: String,
    pub inputs: Vec<FunctionInput>,
    pub outputs: Vec<FunctionOutput>,
    pub state_mutability: String, // "view", "nonpayable", "payable"
}

#[derive(Debug, Clone)]
pub struct FunctionInput {
    pub name: String,
    pub param_type: String, // "uint256", "address", "string", etc.
    pub indexed: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionOutput {
    pub name: String,
    pub param_type: String,
}

#[derive(Debug, Clone)]
pub struct ContractEvent {
    pub name: String,
    pub inputs: Vec<FunctionInput>,
    pub anonymous: bool,
}

lazy_static::lazy_static! {
    static ref ABI_REGISTRY: Mutex<HashMap<(i64, String), Vec<ContractFunction>>> =
        Mutex::new(HashMap::new());
    static ref ABI_SELECTOR_REGISTRY: Mutex<HashMap<(i64, String), HashMap<String, Vec<ContractFunction>>>> =
        Mutex::new(HashMap::new());
}

fn normalize_registry_address(address: &str) -> String {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        return "0x".to_string();
    }
    if trimmed.starts_with("0x") {
        trimmed.to_lowercase()
    } else {
        format!("0x{}", trimmed.to_lowercase())
    }
}

fn normalize_selector(selector: &str) -> Option<String> {
    let s = selector.trim().trim_start_matches("0x").to_lowercase();
    if s.len() != 8 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(s)
}

#[derive(Debug, Clone)]
pub struct AbiTypedCallResult {
    pub result_hex: Option<String>,
    pub decoded: Value,
    pub decode_error: Option<String>,
    pub tx_hash: Option<String>,
    pub receipt_status: Option<String>,
    pub revert_data: Option<String>,
    pub error_code: Option<String>,
    pub message: String,
}

/// Parse Solidity ABI JSON string
pub fn parse_abi(abi_json: String) -> Result<Vec<ContractFunction>, String> {
    let abi: Vec<serde_json::Value> =
        serde_json::from_str(&abi_json).map_err(|e| format!("Failed to parse ABI: {}", e))?;

    let mut functions = Vec::new();

    for item in abi {
        if let Some(item_type) = item.get("type").and_then(|v| v.as_str()) {
            if item_type == "function" {
                if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                    let inputs = parse_inputs(item.get("inputs"));
                    let outputs = parse_outputs(item.get("outputs"));
                    let state_mutability = item
                        .get("stateMutability")
                        .and_then(|v| v.as_str())
                        .unwrap_or("nonpayable")
                        .to_string();

                    functions.push(ContractFunction {
                        name: name.to_string(),
                        inputs,
                        outputs,
                        state_mutability: state_mutability.to_string(),
                    });
                }
            }
        }
    }

    Ok(functions)
}

/// Parse event definitions from ABI
pub fn parse_events(abi_json: String) -> Result<Vec<ContractEvent>, String> {
    let abi: Vec<serde_json::Value> =
        serde_json::from_str(&abi_json).map_err(|e| format!("Failed to parse ABI: {}", e))?;

    let mut events = Vec::new();

    for item in abi {
        if let Some(item_type) = item.get("type").and_then(|v| v.as_str()) {
            if item_type == "event" {
                if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                    let inputs = parse_inputs(item.get("inputs"));
                    let anonymous = item
                        .get("anonymous")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    events.push(ContractEvent {
                        name: name.to_string(),
                        inputs,
                        anonymous,
                    });
                }
            }
        }
    }

    Ok(events)
}

fn parse_inputs(inputs_value: Option<&serde_json::Value>) -> Vec<FunctionInput> {
    let mut inputs = Vec::new();

    if let Some(inputs_array) = inputs_value.and_then(|v| v.as_array()) {
        for input in inputs_array {
            if let (Some(name), Some(param_type)) = (
                input.get("name").and_then(|v| v.as_str()),
                input.get("type").and_then(|v| v.as_str()),
            ) {
                let indexed = input
                    .get("indexed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                inputs.push(FunctionInput {
                    name: name.to_string(),
                    param_type: param_type.to_string(),
                    indexed,
                });
            }
        }
    }

    inputs
}

fn parse_outputs(outputs_value: Option<&serde_json::Value>) -> Vec<FunctionOutput> {
    let mut outputs = Vec::new();

    if let Some(outputs_array) = outputs_value.and_then(|v| v.as_array()) {
        for output in outputs_array {
            if let (Some(name), Some(param_type)) = (
                output.get("name").and_then(|v| v.as_str()),
                output.get("type").and_then(|v| v.as_str()),
            ) {
                outputs.push(FunctionOutput {
                    name: name.to_string(),
                    param_type: param_type.to_string(),
                });
            }
        }
    }

    outputs
}

/// Convert Solidity type to dist_agent_lang type
pub fn solidity_to_dal_type(solidity_type: &str) -> String {
    match solidity_type {
        "uint256" | "uint128" | "uint64" | "uint32" | "uint8" | "uint" => "int".to_string(),
        "int256" | "int128" | "int64" | "int32" | "int8" | "int" => "int".to_string(),
        "address" => "string".to_string(),
        "bool" => "bool".to_string(),
        "string" | "bytes" | "bytes32" => "string".to_string(),
        _ => {
            // Handle arrays
            if let Some(stripped) = solidity_type.strip_suffix("[]") {
                "vector<".to_string() + &solidity_to_dal_type(stripped) + ">"
            } else {
                "any".to_string()
            }
        }
    }
}

/// Generate type-safe wrapper function signature
pub fn generate_wrapper_signature(function: &ContractFunction) -> String {
    let mut signature = format!("fn {}(&self", function.name);

    for input in &function.inputs {
        signature.push_str(&format!(
            ", {}: {}",
            input.name,
            solidity_to_dal_type(&input.param_type)
        ));
    }

    signature.push_str(") -> ");

    if function.outputs.is_empty() {
        signature.push_str("bool");
    } else if function.outputs.len() == 1 {
        signature.push_str(&solidity_to_dal_type(&function.outputs[0].param_type));
    } else {
        signature.push_str("map<string, any>");
    }

    signature.push_str(";");
    signature
}

/// Register a Solidity contract with ABI
pub fn register_contract(
    name: String,
    address: String,
    chain_id: i64,
    abi_json: Option<String>,
) -> SolidityContract {
    if let Some(ref abi_json) = abi_json {
        if let Ok(functions) = parse_abi(abi_json.clone()) {
            let key = (chain_id, normalize_registry_address(&address));
            if let Ok(mut registry) = ABI_REGISTRY.lock() {
                registry.insert(key.clone(), functions.clone());
            }
            if let Ok(mut selector_registry) = ABI_SELECTOR_REGISTRY.lock() {
                let mut by_selector: HashMap<String, Vec<ContractFunction>> = HashMap::new();
                for function in &functions {
                    let signature = function_signature(function);
                    let selector = crate::stdlib::abi::selector_from_signature(&signature);
                    by_selector
                        .entry(selector)
                        .or_default()
                        .push(function.clone());
                }
                selector_registry.insert(key, by_selector);
            }
        }
    }
    SolidityContract {
        name,
        address,
        chain_id,
        abi: abi_json,
    }
}

pub fn resolve_registered_function_abi(
    chain_id: i64,
    contract_address: &str,
    function_name: &str,
) -> Vec<ContractFunction> {
    let key = (chain_id, normalize_registry_address(contract_address));
    let Ok(registry) = ABI_REGISTRY.lock() else {
        return Vec::new();
    };
    registry
        .get(&key)
        .map(|functions| {
            functions
                .iter()
                .filter(|f| f.name == function_name)
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn resolve_registered_function_abi_by_selector(
    chain_id: i64,
    contract_address: &str,
    selector: &str,
) -> Vec<ContractFunction> {
    let Some(selector_norm) = normalize_selector(selector) else {
        return Vec::new();
    };
    let key = (chain_id, normalize_registry_address(contract_address));
    let Ok(registry) = ABI_SELECTOR_REGISTRY.lock() else {
        return Vec::new();
    };
    registry
        .get(&key)
        .and_then(|by_selector| by_selector.get(&selector_norm))
        .cloned()
        .unwrap_or_default()
}

fn function_signature(function: &ContractFunction) -> String {
    let inputs = function
        .inputs
        .iter()
        .map(|i| i.param_type.clone())
        .collect::<Vec<_>>()
        .join(",");
    format!("{}({})", function.name, inputs)
}

fn is_valid_function_signature_hint(signature: &str, function_name: &str) -> bool {
    let trimmed = signature.trim();
    if trimmed.is_empty() {
        return false;
    }
    if !trimmed.starts_with(function_name) {
        return false;
    }
    let open = trimmed.find('(');
    let close = trimmed.rfind(')');
    match (open, close) {
        (Some(o), Some(c)) => o < c,
        _ => false,
    }
}

pub fn decode_registered_result_as_string(
    chain_id: i64,
    contract_address: &str,
    function_name: &str,
    function_selector_hint: Option<&str>,
    function_signature_hint: Option<&str>,
    result_hex: &Option<String>,
) -> (Option<String>, Option<String>) {
    let selector_candidates = function_selector_hint
        .map(|selector| {
            resolve_registered_function_abi_by_selector(chain_id, contract_address, selector)
        })
        .unwrap_or_default();
    let name_candidates =
        resolve_registered_function_abi(chain_id, contract_address, function_name);
    let candidates = if !selector_candidates.is_empty() {
        selector_candidates
    } else {
        name_candidates
    };

    let available_signatures = candidates
        .iter()
        .map(function_signature)
        .collect::<Vec<_>>();
    let function = if let Some(signature) = function_signature_hint {
        let signature = signature.trim();
        if !is_valid_function_signature_hint(signature, function_name) {
            return (
                None,
                Some(format!(
                    "ABI decode skipped: invalid function_signature '{}'. Expected format like '{}(type1,type2)'.",
                    signature, function_name
                )),
            );
        }
        match candidates
            .into_iter()
            .find(|f| function_signature(f).eq_ignore_ascii_case(signature))
        {
            Some(f) => f,
            None => {
                return (
                    None,
                    Some(format!(
                        "ABI decode skipped: function signature '{}' not found for {}::{}. Available: [{}]",
                        signature,
                        chain_id,
                        function_name,
                        available_signatures.join(", ")
                    )),
                );
            }
        }
    } else if candidates.len() == 1 {
        candidates[0].clone()
    } else if candidates.len() > 1 {
        return (
            None,
            Some(format!(
                "ABI decode skipped: overloaded function '{}' requires function_signature hint. Available: [{}]",
                function_name,
                available_signatures.join(", ")
            )),
        );
    } else if function_selector_hint.is_some() {
        return (
            None,
            Some(format!(
                "ABI decode skipped: selector hint '{}' not found for contract {} on chain {}",
                function_selector_hint.unwrap_or_default(),
                contract_address,
                chain_id
            )),
        );
    } else {
        return (None, None);
    };
    let (decoded, decode_error) = decode_result_with_function_abi(&function, result_hex);
    if decode_error.is_some() {
        return (None, decode_error);
    }
    match decoded {
        Value::Null => (None, None),
        other => (Some(value_to_string(&other)), None),
    }
}

/// Call a Solidity contract function with type checking
pub fn call_with_abi(
    contract: &SolidityContract,
    function_name: String,
    args: HashMap<String, Value>,
) -> Result<String, String> {
    let typed = call_with_abi_typed(contract, function_name, args)?;
    Ok(typed.result_hex.unwrap_or(typed.message))
}

/// Typed variant of ABI call that preserves chain evidence and decoded output.
pub fn call_with_abi_typed(
    contract: &SolidityContract,
    function_name: String,
    args: HashMap<String, Value>,
) -> Result<AbiTypedCallResult, String> {
    // Parse ABI if available
    if let Some(ref abi_json) = contract.abi {
        let functions = parse_abi(abi_json.clone())?;

        // Find the function
        let function = functions
            .iter()
            .find(|f| f.name == function_name)
            .ok_or_else(|| format!("Function {} not found in ABI", function_name))?;

        // Validate arguments
        validate_args(function, &args)?;

        // Call through typed chain API and decode output when ABI metadata allows.
        let result = crate::stdlib::chain::call_typed(
            contract.chain_id,
            contract.address.clone(),
            function_name,
            convert_args_to_string_map(args),
        );
        log_typed_call_evidence(contract, &result);
        let (decoded, decode_error) = decode_result_with_function_abi(function, &result.result_hex);
        return Ok(AbiTypedCallResult {
            result_hex: result.result_hex,
            decoded,
            decode_error,
            tx_hash: result.tx_hash,
            receipt_status: result.receipt_status,
            revert_data: result.revert_data,
            error_code: result.error_code,
            message: result.message,
        });
    }

    // Fallback to direct call without ABI validation
    let result = crate::stdlib::chain::call_typed(
        contract.chain_id,
        contract.address.clone(),
        function_name,
        convert_args_to_string_map(args),
    );
    log_typed_call_evidence(contract, &result);
    Ok(AbiTypedCallResult {
        result_hex: result.result_hex,
        decoded: Value::Null,
        decode_error: None,
        tx_hash: result.tx_hash,
        receipt_status: result.receipt_status,
        revert_data: result.revert_data,
        error_code: result.error_code,
        message: result.message,
    })
}

fn log_typed_call_evidence(
    contract: &SolidityContract,
    result: &crate::stdlib::chain::ChainCallResult,
) {
    if result.tx_hash.is_none()
        && result.receipt_status.is_none()
        && result.revert_data.is_none()
        && result.error_code.is_none()
        && result.decode_error.is_none()
    {
        return;
    }

    crate::stdlib::log::audit(
        "add_sol_call_typed_result",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("chain_id".to_string(), Value::Int(contract.chain_id));
            data.insert(
                "contract_address".to_string(),
                Value::String(contract.address.clone()),
            );
            if let Some(code) = &result.error_code {
                data.insert("error_code".to_string(), Value::String(code.clone()));
            }
            if let Some(tx_hash) = &result.tx_hash {
                data.insert("tx_hash".to_string(), Value::String(tx_hash.clone()));
            }
            if let Some(status) = &result.receipt_status {
                data.insert("receipt_status".to_string(), Value::String(status.clone()));
            }
            if let Some(revert_data) = &result.revert_data {
                data.insert(
                    "revert_data".to_string(),
                    Value::String(revert_data.clone()),
                );
            }
            if let Some(decoded) = &result.decoded {
                data.insert("decoded".to_string(), Value::String(decoded.clone()));
            }
            if let Some(decode_err) = &result.decode_error {
                data.insert(
                    "decode_error".to_string(),
                    Value::String(decode_err.clone()),
                );
            }
            data
        },
        Some("add_sol"),
    );
}

fn validate_args(function: &ContractFunction, args: &HashMap<String, Value>) -> Result<(), String> {
    if function.inputs.len() != args.len() {
        return Err(format!(
            "Argument count mismatch: expected {}, got {}",
            function.inputs.len(),
            args.len()
        ));
    }

    for input in &function.inputs {
        if !args.contains_key(&input.name) {
            return Err(format!("Missing argument: {}", input.name));
        }
    }

    Ok(())
}

fn convert_args_to_string_map(args: HashMap<String, Value>) -> HashMap<String, String> {
    args.iter()
        .map(|(k, v)| (k.clone(), value_to_string(v)))
        .collect()
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(arr) => format!("{:?}", arr),
        Value::Map(map) => format!("{:?}", map),
        Value::Set(set) => format!("{:?}", set),
        Value::Struct(name, fields) => format!("{} {:?}", name, fields),
        Value::List(list) => format!("{:?}", list),
        Value::Result(ok_val, err_val) => {
            // Check if it's Ok or Err by checking if err_val is Null
            if matches!(err_val.as_ref(), Value::Null) {
                format!("Ok({})", value_to_string(ok_val))
            } else {
                format!("Err({})", value_to_string(err_val))
            }
        }
        Value::Option(opt_val) => match opt_val {
            Some(val) => value_to_string(val),
            None => "null".to_string(),
        },
        Value::Closure(id) => format!("<closure {}>", id),
    }
}

fn decode_result_with_function_abi(
    function: &ContractFunction,
    result_hex: &Option<String>,
) -> (Value, Option<String>) {
    let Some(payload) = result_hex else {
        return (Value::Null, None);
    };
    let payload_hex = payload.trim().trim_start_matches("0x").to_lowercase();
    if payload_hex.is_empty() {
        return (Value::Null, None);
    }

    if function.outputs.is_empty() {
        return (Value::Null, None);
    }

    if function.outputs.len() == 1 {
        match decode_output_value(&function.outputs[0].param_type, &payload_hex) {
            Ok(v) => return (v, None),
            Err(e) => return (Value::Null, Some(e)),
        }
    }

    let mut out = HashMap::new();
    for (idx, output) in function.outputs.iter().enumerate() {
        let key = if output.name.is_empty() {
            format!("out_{}", idx)
        } else {
            output.name.clone()
        };
        let start = idx * 64;
        let end = start + 64;
        if payload_hex.len() < end {
            return (
                Value::Null,
                Some(format!(
                    "insufficient ABI payload length for output {} ({})",
                    idx, output.param_type
                )),
            );
        }
        match decode_output_value(&output.param_type, &payload_hex[start..end]) {
            Ok(v) => {
                out.insert(key, v);
            }
            Err(e) => return (Value::Null, Some(e)),
        }
    }
    (Value::Map(out), None)
}

fn decode_output_value(param_type: &str, payload_hex: &str) -> Result<Value, String> {
    let t = param_type.trim().to_ascii_lowercase();
    match t.as_str() {
        "bool" => crate::stdlib::abi_codec::decode_bool_word(payload_hex).map(Value::Bool),
        "address" => crate::stdlib::abi_codec::decode_address_word(payload_hex).map(Value::String),
        "string" => {
            crate::stdlib::abi_codec::decode_abi_string_data(payload_hex).map(Value::String)
        }
        "bytes" => crate::stdlib::abi_codec::decode_abi_bytes_data(payload_hex)
            .map(|b| Value::String(format!("0x{}", hex::encode(b)))),
        "bytes32" => {
            let normalized = payload_hex.trim().trim_start_matches("0x").to_lowercase();
            if normalized.len() != 64 {
                return Err(format!(
                    "bytes32 output expects 64 hex chars, got {}",
                    normalized.len()
                ));
            }
            Ok(Value::String(format!("0x{}", normalized)))
        }
        _ if t.starts_with("uint") || t == "uint" || t.starts_with("int") || t == "int" => {
            crate::stdlib::abi_codec::decode_uint256_word(payload_hex)
                .map(|n| Value::String(n.to_string()))
        }
        _ => Ok(Value::String(format!(
            "0x{}",
            payload_hex.trim().trim_start_matches("0x")
        ))),
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::{
        decode_result_with_function_abi, ContractFunction, FunctionOutput, SolidityContract,
    };
    use crate::runtime::values::Value;
    use std::collections::HashMap;

    #[test]
    fn decode_result_with_abi_decodes_single_bool_output() {
        let function = ContractFunction {
            name: "transfer".to_string(),
            inputs: vec![],
            outputs: vec![FunctionOutput {
                name: "ok".to_string(),
                param_type: "bool".to_string(),
            }],
            state_mutability: "nonpayable".to_string(),
        };
        let (decoded, decode_error) = decode_result_with_function_abi(
            &function,
            &Some("0x0000000000000000000000000000000000000000000000000000000000000001".to_string()),
        );
        assert_eq!(decoded, Value::Bool(true));
        assert_eq!(decode_error, None);
    }

    #[test]
    fn call_with_abi_typed_preserves_error_evidence_on_chain_unsupported() {
        let contract = SolidityContract {
            name: "Token".to_string(),
            address: "0x1234".to_string(),
            chain_id: 999_999,
            abi: Some(
                r#"[
                    {
                        "type":"function",
                        "name":"transfer",
                        "inputs":[{"name":"to","type":"address"},{"name":"amount","type":"uint256"}],
                        "outputs":[{"name":"ok","type":"bool"}]
                    }
                ]"#
                .to_string(),
            ),
        };
        let mut args = HashMap::new();
        args.insert("to".to_string(), Value::String("0x5678".to_string()));
        args.insert("amount".to_string(), Value::String("1000".to_string()));
        let result = super::call_with_abi_typed(&contract, "transfer".to_string(), args)
            .expect("typed call");
        assert_eq!(result.error_code, Some("CHAIN_UNSUPPORTED".to_string()));
        assert_eq!(result.result_hex, None);
        assert_eq!(result.decoded, Value::Null);
    }

    #[test]
    fn registry_decodes_call_result_by_contract_and_function_identity() {
        let _contract = super::register_contract(
            "Token".to_string(),
            "0x1234".to_string(),
            1,
            Some(
                r#"[
                    {
                        "type":"function",
                        "name":"transfer",
                        "inputs":[{"name":"to","type":"address"},{"name":"amount","type":"uint256"}],
                        "outputs":[{"name":"ok","type":"bool"}]
                    }
                ]"#
                .to_string(),
            ),
        );
        let (decoded, decode_error) = super::decode_registered_result_as_string(
            1,
            "0x1234",
            "transfer",
            None,
            None,
            &Some("0x0000000000000000000000000000000000000000000000000000000000000001".to_string()),
        );
        assert_eq!(decoded, Some("true".to_string()));
        assert_eq!(decode_error, None);
    }

    #[test]
    fn registry_requires_function_signature_for_overloaded_name() {
        let _contract = super::register_contract(
            "Overloaded".to_string(),
            "0x7777".to_string(),
            1,
            Some(
                r#"[
                    {
                        "type":"function",
                        "name":"foo",
                        "inputs":[{"name":"a","type":"uint256"}],
                        "outputs":[{"name":"ok","type":"bool"}]
                    },
                    {
                        "type":"function",
                        "name":"foo",
                        "inputs":[{"name":"a","type":"address"}],
                        "outputs":[{"name":"ok","type":"bool"}]
                    }
                ]"#
                .to_string(),
            ),
        );

        let (decoded, decode_error) = super::decode_registered_result_as_string(
            1,
            "0x7777",
            "foo",
            None,
            None,
            &Some("0x0000000000000000000000000000000000000000000000000000000000000001".to_string()),
        );
        assert_eq!(decoded, None);
        assert!(decode_error
            .unwrap_or_default()
            .contains("requires function_signature hint"));

        let (decoded_with_sig, decode_error_with_sig) = super::decode_registered_result_as_string(
            1,
            "0x7777",
            "foo",
            None,
            Some("foo(uint256)"),
            &Some("0x0000000000000000000000000000000000000000000000000000000000000001".to_string()),
        );
        assert_eq!(decoded_with_sig, Some("true".to_string()));
        assert_eq!(decode_error_with_sig, None);
    }

    #[test]
    fn registry_reports_signature_format_and_available_overloads() {
        let _contract = super::register_contract(
            "Overloaded".to_string(),
            "0x8888".to_string(),
            1,
            Some(
                r#"[
                    {
                        "type":"function",
                        "name":"bar",
                        "inputs":[{"name":"a","type":"uint256"}],
                        "outputs":[{"name":"ok","type":"bool"}]
                    },
                    {
                        "type":"function",
                        "name":"bar",
                        "inputs":[{"name":"a","type":"address"}],
                        "outputs":[{"name":"ok","type":"bool"}]
                    }
                ]"#
                .to_string(),
            ),
        );
        let (_decoded, err) = super::decode_registered_result_as_string(
            1,
            "0x8888",
            "bar",
            None,
            Some("bar"),
            &Some("0x0000000000000000000000000000000000000000000000000000000000000001".to_string()),
        );
        assert!(err
            .unwrap_or_default()
            .contains("invalid function_signature"));

        let (_decoded, err2) = super::decode_registered_result_as_string(
            1,
            "0x8888",
            "bar",
            None,
            Some("bar(bytes32)"),
            &Some("0x0000000000000000000000000000000000000000000000000000000000000001".to_string()),
        );
        let msg = err2.unwrap_or_default();
        assert!(msg.contains("Available: ["));
        assert!(msg.contains("bar(uint256)"));
        assert!(msg.contains("bar(address)"));
    }

    #[test]
    fn registry_selector_hint_decodes_without_function_name_match() {
        let _contract = super::register_contract(
            "Token".to_string(),
            "0x9999".to_string(),
            1,
            Some(
                r#"[
                    {
                        "type":"function",
                        "name":"transfer",
                        "inputs":[{"name":"to","type":"address"},{"name":"amount","type":"uint256"}],
                        "outputs":[{"name":"ok","type":"bool"}]
                    }
                ]"#
                .to_string(),
            ),
        );

        let (decoded, decode_error) = super::decode_registered_result_as_string(
            1,
            "0x9999",
            "not_transfer",
            Some("a9059cbb"),
            None,
            &Some("0x0000000000000000000000000000000000000000000000000000000000000001".to_string()),
        );
        assert_eq!(decoded, Some("true".to_string()));
        assert_eq!(decode_error, None);
    }
}

/// Listen to Solidity contract events
pub fn listen_to_event(
    contract: &SolidityContract,
    event_name: String,
    callback: String, // Function name to call
    from_block: Option<i64>,
    to_block: Option<i64>,
) -> String {
    // Log event subscription
    crate::stdlib::log::info(
        "event_listener",
        {
            let mut data = std::collections::HashMap::new();
            data.insert(
                "contract".to_string(),
                Value::String(contract.address.clone()),
            );
            data.insert("event".to_string(), Value::String(event_name.clone()));
            data.insert("callback".to_string(), Value::String(callback.clone()));
            if let Some(from) = from_block {
                data.insert("from_block".to_string(), Value::Int(from));
            }
            if let Some(to) = to_block {
                data.insert("to_block".to_string(), Value::Int(to));
            }
            data
        },
        Some("add_sol"),
    );

    format!(
        "Event listener registered for {} on contract {}",
        event_name, contract.address
    )
}

/// Generate dist_agent_lang wrapper code from Solidity ABI
pub fn generate_wrapper_code(contract: &SolidityContract) -> Result<String, String> {
    let abi_json = contract
        .abi
        .as_ref()
        .ok_or_else(|| "ABI not available".to_string())?;

    let functions = parse_abi(abi_json.clone())?;
    let events = parse_events(abi_json.clone())?;

    let mut code = format!(
        "// Auto-generated wrapper for Solidity contract: {}\n",
        contract.name
    );
    code.push_str(&format!("// Address: {}\n", contract.address));
    code.push_str(&format!("// Chain ID: {}\n\n", contract.chain_id));

    code.push_str("@trust(\"hybrid\")\n");
    code.push_str(&format!("service {}Wrapper {{\n", contract.name));
    code.push_str(&format!(
        "    contract_address: string = \"{}\";\n",
        contract.address
    ));
    code.push_str(&format!("    chain_id: int = {};\n\n", contract.chain_id));

    // Generate wrapper functions
    for function in &functions {
        code.push_str("    ");
        code.push_str(&generate_wrapper_signature(function));
        code.push_str("\n");
        code.push_str("    ");
        code.push_str(&format!("        return chain::call(\n"));
        code.push_str(&format!("            self.chain_id,\n"));
        code.push_str(&format!("            self.contract_address,\n"));
        code.push_str(&format!("            \"{}\",\n", function.name));
        code.push_str("            {\n");

        for input in &function.inputs {
            code.push_str(&format!(
                "                \"{}\": {},\n",
                input.name, input.name
            ));
        }

        code.push_str("            }\n");
        code.push_str("        );\n");
        code.push_str("    }\n\n");
    }

    // Generate event handlers
    if !events.is_empty() {
        code.push_str("    // Event handlers\n");
        for event in &events {
            code.push_str(&format!(
                "    fn on_{}(callback: function) {{\n",
                event.name
            ));
            code.push_str(&format!("        add_sol::listen_to_event(\n"));
            code.push_str(&format!("            self.contract_address,\n"));
            code.push_str(&format!("            \"{}\",\n", event.name));
            code.push_str("            callback\n");
            code.push_str("        );\n");
            code.push_str("    }\n\n");
        }
    }

    code.push_str("}\n");

    Ok(code)
}
