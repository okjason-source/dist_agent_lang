// Integration tests for Solidity contract integration

use dist_agent_lang::stdlib::solidity_adapter;

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

    let result = solidity_adapter::parse_abi(abi_json.to_string());
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

    let result = solidity_adapter::parse_events(abi_json.to_string());
    assert!(result.is_ok());
    
    let events = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].name, "Transfer");
    assert_eq!(events[0].inputs.len(), 3);
}

#[test]
fn test_type_conversion() {
    assert_eq!(solidity_adapter::solidity_to_dal_type("uint256"), "int");
    assert_eq!(solidity_adapter::solidity_to_dal_type("address"), "string");
    assert_eq!(solidity_adapter::solidity_to_dal_type("bool"), "bool");
    assert_eq!(solidity_adapter::solidity_to_dal_type("string"), "string");
}

#[test]
fn test_contract_registration() {
    let contract = solidity_adapter::register_contract(
        "TestContract".to_string(),
        "0x1234".to_string(),
        1,
        None
    );

    assert_eq!(contract.name, "TestContract");
    assert_eq!(contract.address, "0x1234");
    assert_eq!(contract.chain_id, 1);
}

