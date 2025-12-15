# Solidity Contract Integration Guide

## Overview

This guide shows how to easily integrate **dist_agent_lang orchestration** with existing **Solidity contracts**. You can use dist_agent_lang to orchestrate, coordinate, and enhance Solidity contracts without rewriting them.

---

## 🎯 Why Integrate dist_agent_lang with Solidity?

### Benefits:
- ✅ **Keep Your Solidity Contracts** - No need to rewrite proven contracts
- ✅ **Add Orchestration** - Coordinate multiple contracts easily
- ✅ **Multi-Chain Support** - Manage contracts across chains
- ✅ **AI Integration** - Add AI-powered decision making
- ✅ **Simplified Logic** - Complex workflows in one language
- ✅ **Type Safety** - ABI parsing provides type checking
- ✅ **Event Listening** - Listen to Solidity contract events
- ✅ **Easy Testing** - Built-in testing utilities

---

## 🚀 Quick Start

### 1. Basic Solidity Contract Call

```rust
@trust("hybrid")
service SimpleOrchestrator {
    fn call_uniswap() {
        // Call Uniswap Router (Solidity contract)
        let result = chain::call(
            1,  // Ethereum chain ID
            "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D",  // Uniswap Router address
            "swapExactTokensForTokens",  // Solidity function name
            {
                "amountIn": "1000000000000000000",
                "amountOutMin": "950000000000000000",
                "path": ["0xTokenA", "0xTokenB"],
                "to": "0xYourAddress",
                "deadline": "1234567890"
            }
        );
        
        log::info("swap", "Result: " + result);
    }
}
```

### 2. Register Solidity Contracts with ABI

```rust
@trust("hybrid")
service ContractRegistry {
    fn initialize() {
        // Register Uniswap with ABI for type safety
        solidity_adapter::register_contract(
            "UniswapRouter",
            "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D",
            1,  // Ethereum
            uniswap_abi_json  // ABI JSON string
        );
    }
    
    fn call_with_type_safety() {
        // Type-safe call with ABI validation
        let result = solidity_adapter::call_with_abi(
            "UniswapRouter",
            "swapExactTokensForTokens",
            {
                "amountIn": 1000000,
                "amountOutMin": 950000,
                "path": ["0xTokenA", "0xTokenB"],
                "to": "0xRecipient",
                "deadline": 1234567890
            }
        );
    }
}
```

---

## 📋 Advanced Features

### 1. ABI Parsing & Type Safety

```rust
@trust("hybrid")
service TypeSafeIntegration {
    // Parse ABI to get function signatures
    fn parse_contract_abi(abi_json: string) {
        let functions = solidity_adapter::parse_abi(abi_json);
        
        for function in functions {
            log::info("abi", "Function: " + function.name);
            log::info("abi", "  Inputs: " + function.inputs.size());
            log::info("abi", "  Outputs: " + function.outputs.size());
        }
    }
    
    // Generate wrapper code from ABI
    fn generate_wrapper(contract_address: string, abi_json: string) -> string {
        let contract = solidity_adapter::register_contract(
            "MyContract",
            contract_address,
            1,
            abi_json
        );
        
        return solidity_adapter::generate_wrapper_code(contract);
    }
}
```

### 2. Event Listening

```rust
@trust("hybrid")
service EventListener {
    fn setup_listeners() {
        // Listen to Swap events from Uniswap
        solidity_adapter::listen_to_event(
            "UniswapRouter",
            "Swap",
            "handle_swap_event",  // Callback function
            null,  // from_block (null = latest)
            null   // to_block (null = latest)
        );
    }
    
    fn handle_swap_event(event_data: map<string, any>) {
        log::info("swap", "Swap detected:");
        log::info("swap", "  Sender: " + event_data["sender"]);
        log::info("swap", "  Amount: " + event_data["amount0In"]);
    }
}
```

### 3. Testing Utilities

```rust
@trust("hybrid")
service ContractTesting {
    fn test_contract_integration() {
        // Test without deploying to blockchain
        let test_result = solidity_testing::test_contract_call(
            "MockERC20",
            "balanceOf",
            {"account": "0xTestAccount"}
        );
        
        solidity_testing::assert(
            test_result["success"],
            "Contract call test failed"
        );
    }
    
    fn test_multi_step_operation() {
        // Test complex multi-step operations
        let result = solidity_testing::test_multi_step_operation();
        
        solidity_testing::assert(
            result["success"],
            "Multi-step operation failed"
        );
    }
}
```

---

## 📚 Common Patterns

### Pattern 1: Multi-Step DeFi Operations

```rust
@trust("hybrid")
service DeFiOrchestrator {
    fn execute_swap_flow(token_a: string, token_b: string, amount: int) {
        // Step 1: Check balance (ERC20 Solidity contract)
        let balance = chain::call(1, token_a, "balanceOf", {
            "account": auth::session().user_id
        });
        
        // Step 2: Approve (ERC20 Solidity contract)
        chain::call(1, token_a, "approve", {
            "spender": "0xUniswapRouter",
            "amount": amount
        });
        
        // Step 3: Swap (Uniswap Solidity contract)
        chain::call(1, "0xUniswapRouter", "swapExactTokensForTokens", {
            "amountIn": amount,
            "amountOutMin": amount * 95 / 100,
            "path": [token_a, token_b],
            "to": auth::session().user_id,
            "deadline": chain::get_block_timestamp(1) + 1800
        });
    }
}
```

### Pattern 2: Multi-Chain Price Comparison

```rust
@trust("hybrid")
service PriceArbitrage {
    fn find_best_price(token_a: string, token_b: string, amount: int) -> int {
        let chains = [1, 137, 42161];  // Ethereum, Polygon, Arbitrum
        let best_chain = null;
        let best_price = 0;
        
        for chain_id in chains {
            let quote = chain::call(
                chain_id,
                self.get_uniswap_router(chain_id),
                "getAmountsOut",
                {"amountIn": amount, "path": [token_a, token_b]}
            );
            
            if quote > best_price {
                best_price = quote;
                best_chain = chain_id;
            }
        }
        
        return best_chain;
    }
}
```

### Pattern 3: AI-Powered Orchestration

```rust
@trust("hybrid")
@ai
service AIDefiOrchestrator {
    fn ai_optimized_swap(user_request: string) {
        // AI analyzes market
        let analysis = ai::analyze_market({
            "request": user_request,
            "current_prices": self.get_prices()
        });
        
        // AI recommends strategy
        let strategy = ai::generate_strategy(analysis);
        
        // Execute using Solidity contracts
        chain::call(
            strategy.chain_id,
            strategy.contract_address,
            strategy.function_name,
            strategy.args
        );
    }
}
```

---

## 🔧 API Reference

### `solidity_adapter::register_contract()`
Register a Solidity contract with ABI for type-safe calls.

### `solidity_adapter::parse_abi()`
Parse Solidity ABI JSON to extract function and event definitions.

### `solidity_adapter::call_with_abi()`
Call a Solidity contract function with ABI validation and type checking.

### `solidity_adapter::listen_to_event()`
Listen to events emitted by Solidity contracts.

### `solidity_adapter::generate_wrapper_code()`
Auto-generate dist_agent_lang wrapper code from Solidity ABI.

### `solidity_testing::test_contract_call()`
Test contract calls without deploying to blockchain.

### `solidity_testing::assert()`
Assertion helper for testing.

---

## 📖 Examples

See the following example files:
- `examples/solidity_orchestration.dal` - Basic orchestration patterns
- `examples/solidity_abi_integration.dal` - ABI parsing and type safety
- `examples/solidity_testing.dal` - Testing utilities

---

## 🛠️ Best Practices

### 1. Always Use ABI When Available
```rust
// ✅ Good: Type-safe with ABI
solidity_adapter::call_with_abi("Contract", "function", args);

// ⚠️ Less safe: Direct call without validation
chain::call(chain_id, address, "function", args);
```

### 2. Register Contracts Once
```rust
fn initialize() {
    // Register all contracts at startup
    solidity_adapter::register_contract(...);
}
```

### 3. Use Event Listeners for Async Operations
```rust
fn setup_async_handling() {
    solidity_adapter::listen_to_event("Contract", "Event", "handler");
}
```

### 4. Test Before Deploying
```rust
fn test_before_deploy() {
    solidity_testing::test_contract_call(...);
}
```

---

## 📖 Summary

**dist_agent_lang orchestration + Solidity contracts** gives you:

- ✅ **Keep proven Solidity contracts** - No rewriting needed
- ✅ **Easy orchestration** - Coordinate multiple contracts
- ✅ **Multi-chain support** - Manage across chains
- ✅ **AI integration** - Add intelligent decision making
- ✅ **Type safety** - ABI parsing provides validation
- ✅ **Event listening** - React to contract events
- ✅ **Easy testing** - Built-in testing utilities

**Use dist_agent_lang for orchestration, keep Solidity for core contracts!**
