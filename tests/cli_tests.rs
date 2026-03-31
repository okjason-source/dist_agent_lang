// CLI mutation tests: assert on chain_subcommand_to_args return value so mutations
// (e.g. replace with vec![], vec![String::new()], vec!["xyzzy"]) are caught.
// Also assert Cli::parse_from (clap) so parse entrypoint mutants are caught.

use clap::Parser;
use dist_agent_lang::cli::{chain_subcommand_to_args, ChainSubcommand, Cli, Commands};

#[test]
fn test_cli_parse_from_chain_list() {
    let cli = Cli::parse_from(["dal", "chain", "list"]);
    match cli.command {
        Some(Commands::Chain { subcommand }) => {
            assert!(matches!(subcommand, ChainSubcommand::List));
        }
        _ => panic!("expected Commands::Chain(List)"),
    }
}

#[test]
fn test_cli_parse_from_chain_balance() {
    let cli = Cli::parse_from(["dal", "chain", "balance", "1", "0xabc"]);
    match cli.command {
        Some(Commands::Chain { subcommand }) => match subcommand {
            ChainSubcommand::Balance { chain_id, address } => {
                assert_eq!(chain_id, 1);
                assert_eq!(address, "0xabc");
            }
            _ => panic!("expected Balance"),
        },
        _ => panic!("expected chain balance"),
    }
}

#[test]
fn test_cli_parse_from_global_quiet_with_chain() {
    let cli = Cli::parse_from(["dal", "--quiet", "chain", "list"]);
    assert!(cli.quiet);
    assert!(matches!(
        cli.command,
        Some(Commands::Chain {
            subcommand: ChainSubcommand::List
        })
    ));
}

#[test]
fn test_cli_parse_from_run_subcommand() {
    let cli = Cli::parse_from(["dal", "run", "app.dal"]);
    match cli.command {
        Some(Commands::Run { file }) => assert_eq!(file, "app.dal"),
        _ => panic!("expected run"),
    }
}

#[test]
fn test_cli_parse_from_serve_port_and_file() {
    let cli = Cli::parse_from(["dal", "serve", "api.dal", "--port", "9000"]);
    match cli.command {
        Some(Commands::Serve {
            file,
            port,
            frontend,
            ..
        }) => {
            assert_eq!(file, "api.dal");
            assert_eq!(port, 9000);
            assert!(frontend.is_none());
        }
        _ => panic!("expected serve"),
    }
}

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

#[test]
fn test_chain_subcommand_to_args_gas_price() {
    let args = chain_subcommand_to_args(&ChainSubcommand::GasPrice { chain_id: 99 });
    assert_eq!(args, vec!["gas-price".to_string(), "99".to_string()]);
}

#[test]
fn test_chain_subcommand_to_args_tx_status() {
    let args = chain_subcommand_to_args(&ChainSubcommand::TxStatus {
        chain_id: 5,
        tx_hash: "0xdead".to_string(),
    });
    assert_eq!(
        args,
        vec![
            "tx-status".to_string(),
            "5".to_string(),
            "0xdead".to_string()
        ]
    );
}

#[test]
fn test_chain_subcommand_to_args_block_time() {
    let args = chain_subcommand_to_args(&ChainSubcommand::BlockTime { chain_id: 1 });
    assert_eq!(args, vec!["block-time".to_string(), "1".to_string()]);
}

#[test]
fn test_chain_subcommand_to_args_call_without_extra_args() {
    let args = chain_subcommand_to_args(&ChainSubcommand::Call {
        chain_id: 10,
        address: "0x1".to_string(),
        function: "transfer".to_string(),
        args: None,
    });
    assert_eq!(
        args,
        vec![
            "call".to_string(),
            "10".to_string(),
            "0x1".to_string(),
            "transfer".to_string(),
        ]
    );
}

#[test]
fn test_chain_subcommand_to_args_call_with_extra_args() {
    let args = chain_subcommand_to_args(&ChainSubcommand::Call {
        chain_id: 10,
        address: "0x1".to_string(),
        function: "transfer".to_string(),
        args: Some("1,2".to_string()),
    });
    assert_eq!(args.len(), 6);
    assert_eq!(args[0], "call");
    assert_eq!(args[1], "10");
    assert_eq!(args[2], "0x1");
    assert_eq!(args[3], "transfer");
    assert_eq!(args[4], "--args");
    assert_eq!(args[5], "1,2");
}

#[test]
fn test_chain_subcommand_to_args_mint_without_meta() {
    let args = chain_subcommand_to_args(&ChainSubcommand::Mint {
        name: "gold".to_string(),
        meta: None,
    });
    assert_eq!(args, vec!["mint".to_string(), "gold".to_string()]);
}

#[test]
fn test_chain_subcommand_to_args_mint_with_meta() {
    let args = chain_subcommand_to_args(&ChainSubcommand::Mint {
        name: "gold".to_string(),
        meta: Some("k=v".to_string()),
    });
    assert_eq!(
        args,
        vec![
            "mint".to_string(),
            "gold".to_string(),
            "--meta".to_string(),
            "k=v".to_string(),
        ]
    );
}
