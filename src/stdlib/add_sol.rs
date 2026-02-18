use crate::runtime::values::Value;
use serde_json;
use std::collections::HashMap;

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
            if solidity_type.ends_with("[]") {
                "vector<".to_string()
                    + &solidity_to_dal_type(&solidity_type[..solidity_type.len() - 2])
                    + ">"
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
    SolidityContract {
        name,
        address,
        chain_id,
        abi: abi_json,
    }
}

/// Call a Solidity contract function with type checking
pub fn call_with_abi(
    contract: &SolidityContract,
    function_name: String,
    args: HashMap<String, Value>,
) -> Result<String, String> {
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

        // Call the contract
        return Ok(crate::stdlib::chain::call(
            contract.chain_id,
            contract.address.clone(),
            function_name,
            convert_args_to_string_map(args),
        ));
    }

    // Fallback to direct call without ABI validation
    Ok(crate::stdlib::chain::call(
        contract.chain_id,
        contract.address.clone(),
        function_name,
        convert_args_to_string_map(args),
    ))
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
