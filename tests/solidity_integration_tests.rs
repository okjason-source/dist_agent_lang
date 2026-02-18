// Comprehensive Solidity Integration Tests
// Tests for Solidity contract integration using actual language code
// Aligned with PRODUCTION_ROADMAP.md goals for production readiness

use dist_agent_lang::parse_source;
use dist_agent_lang::stdlib::add_sol;

#[test]
fn test_abi_parsing() {
    let abi_json = r#"[
        {
            "type": "function",
            "name": "transfer",
            "inputs": [
                {"name": "to", "type": "address"},
                {"name": "amount", "type": "uint256"}
            ],
            "outputs": [{"name": "", "type": "bool"}],
            "stateMutability": "nonpayable"
        }
    ]"#;

    let result = add_sol::parse_abi(abi_json.to_string());
    assert!(result.is_ok());

    let functions = result.unwrap();
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].name, "transfer");
    assert_eq!(functions[0].inputs.len(), 2);
}

#[test]
fn test_event_parsing() {
    let abi_json = r#"[
        {
            "type": "event",
            "name": "Transfer",
            "inputs": [
                {"name": "from", "type": "address", "indexed": true},
                {"name": "to", "type": "address", "indexed": true},
                {"name": "amount", "type": "uint256", "indexed": false}
            ],
            "anonymous": false
        }
    ]"#;

    let result = add_sol::parse_events(abi_json.to_string());
    assert!(result.is_ok());

    let events = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].name, "Transfer");
    assert_eq!(events[0].inputs.len(), 3);
}

#[test]
fn test_type_conversion() {
    assert_eq!(add_sol::solidity_to_dal_type("uint256"), "int");
    assert_eq!(add_sol::solidity_to_dal_type("address"), "string");
    assert_eq!(add_sol::solidity_to_dal_type("bool"), "bool");
    assert_eq!(add_sol::solidity_to_dal_type("string"), "string");
}

#[test]
fn test_contract_registration() {
    let contract =
        add_sol::register_contract("TestContract".to_string(), "0x1234".to_string(), 1, None);

    assert_eq!(contract.name, "TestContract");
    assert_eq!(contract.address, "0x1234");
    assert_eq!(contract.chain_id, 1);
}

#[test]
fn test_solidity_contract_orchestration() {
    // Test orchestrating Solidity contracts from dist_agent_lang
    let code = r#"
    @chain("ethereum")
    service SolidityOrchestrator {
        contract_address: string;
        
        fn deploy_and_call() {
            // Deploy Solidity contract
            self.contract_address = chain::deploy(1, "ERC20", {
                "name": "TestToken",
                "symbol": "TST"
            });
            
            // Call Solidity function
            chain::call(1, self.contract_address, "transfer", {
                "to": "0x5678",
                "amount": 1000
            });
        }
        
        event SolidityCallExecuted(contract: string, function: string, result: string);
    }
    "#;

    let program = parse_source(code).unwrap();
    assert!(!program.statements.is_empty());
}

#[test]
fn test_solidity_event_listening() {
    // Test setting up event listeners for Solidity contracts
    let code = r#"
    @chain("ethereum")
    service EventListener {
        contract_address: string = "0x1234";
        
        fn setup_listener() {
            // Register event listener (simulated - would use add_sol)
            let listener_config = {
                "contract": self.contract_address,
                "event": "Transfer",
                "handler": "handle_transfer"
            };
        }
        
        fn handle_transfer(event_data: map<string, any>) -> string {
            let from = event_data["from"];
            let to = event_data["to"];
            let amount = event_data["amount"];
            
            return "transfer_processed";
        }
        
        event EventReceived(event_name: string, data: map<string, any>);
    }
    "#;

    let program = parse_source(code).unwrap();
    assert!(!program.statements.is_empty());
}

#[test]
fn test_solidity_abi_integration() {
    // Test using parsed ABI with dist_agent_lang code
    let code = r#"
    @chain("ethereum")
    service ABIIntegration {
        abi_functions: map<string, any> = {};
        
        fn register_abi(abi_json: string) {
            // Parse and register ABI (simulated)
            self.abi_functions["transfer"] = {
                "name": "transfer",
                "inputs": ["to", "amount"]
            };
        }
        
        fn call_with_abi(function_name: string, args: map<string, any>) -> string {
            let function_info = self.abi_functions[function_name];
            return "called";
        }
        
        event ABIFunctionCalled(function_name: string, args: map<string, any>);
    }
    "#;

    let program = parse_source(code).unwrap();
    assert!(!program.statements.is_empty());
}
