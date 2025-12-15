// End-to-end DeFi workflow tests

use dist_agent_lang::{Runtime, Parser, Lexer};

#[test]
fn test_complete_swap_workflow() {
        let code = r#"
        @trust("hybrid")
        @chain("ethereum")
        service SwapWorkflow {
            fn execute_swap(token_a: string, token_b: string, amount: int) {
                // Step 1: Check balance
                let balance = chain::call(1, token_a, "balanceOf", {"account": "0xUser"});
                
                // Step 2: Approve
                chain::call(1, token_a, "approve", {
                    "spender": "0xUniswap",
                    "amount": amount
                });
                
                // Step 3: Swap
                chain::call(1, "0xUniswap", "swapExactTokensForTokens", {
                    "amountIn": amount,
                    "path": [token_a, token_b],
                    "to": "0xUser",
                    "deadline": 1234567890
                });
            }
        }
        "#;

        // Parse and validate
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        // Should parse successfully
        assert!(!program.statements.is_empty());
}

#[test]
fn test_multi_chain_deployment() {
        let code = r#"
        @trust("decentralized")
        @chain("ethereum", "polygon", "arbitrum")
        service MultiChainToken {
            fn deploy_to_all_chains() {
                for chain_id in [1, 137, 42161] {
                    chain::deploy(chain_id, "Token", {"name": "Test"});
                }
            }
        }
        "#;

        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        assert!(!program.statements.is_empty());
}

