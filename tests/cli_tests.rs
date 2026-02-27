// CLI mutation tests: assert on chain_subcommand_to_args return value so mutations
// (e.g. replace with vec![], vec![String::new()], vec!["xyzzy"]) are caught.

use dist_agent_lang::cli::{chain_subcommand_to_args, ChainSubcommand};

#[test]
fn test_chain_subcommand_to_args_list() {
    let args = chain_subcommand_to_args(&ChainSubcommand::List);
    assert_eq!(args.len(), 1, "List should produce exactly one arg");
    assert_eq!(args[0], "list");
}

#[test]
fn test_chain_subcommand_to_args_config() {
    let args = chain_subcommand_to_args(&ChainSubcommand::Config { chain_id: 42 });
    assert_eq!(args[0], "config");
    assert_eq!(args.len(), 2);
    assert_eq!(args[1], "42");
}

#[test]
fn test_chain_subcommand_to_args_balance() {
    let args = chain_subcommand_to_args(&ChainSubcommand::Balance {
        chain_id: 1,
        address: "0xabc".to_string(),
    });
    assert_eq!(args[0], "balance");
    assert_eq!(args.len(), 3);
    assert_eq!(args[1], "1");
    assert_eq!(args[2], "0xabc");
}

#[test]
fn test_chain_subcommand_to_args_asset() {
    let args = chain_subcommand_to_args(&ChainSubcommand::Asset {
        id: "my-id".to_string(),
    });
    assert_eq!(args.len(), 2);
    assert_eq!(args[0], "asset");
    assert_eq!(args[1], "my-id");
}

#[test]
fn test_chain_subcommand_to_args_deploy_without_args() {
    let args = chain_subcommand_to_args(&ChainSubcommand::Deploy {
        chain_id: 137,
        contract: "MyContract".to_string(),
        args: None,
    });
    assert_eq!(args[0], "deploy");
    assert_eq!(args[1], "137");
    assert_eq!(args[2], "MyContract");
    assert_eq!(args.len(), 3);
}

#[test]
fn test_chain_subcommand_to_args_deploy_with_args() {
    let args = chain_subcommand_to_args(&ChainSubcommand::Deploy {
        chain_id: 137,
        contract: "C".to_string(),
        args: Some("a,b".to_string()),
    });
    assert_eq!(args[0], "deploy");
    assert_eq!(args[1], "137");
    assert_eq!(args[2], "C");
    assert_eq!(args[3], "--args");
    assert_eq!(args[4], "a,b");
    assert_eq!(args.len(), 5);
}
