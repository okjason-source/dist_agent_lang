# 🚀 dist_agent_lang Usage Guide

## Overview

`dist_agent_lang` is a comprehensive library designed for blockchain development, AI integration, and multi-chain operations. This guide explains how to properly use the language and its features.

---

## 🎯 **Core Language Features**

### **1. Service Declarations with Attributes**

The `@` attributes are **core features** of the language that define how services behave:

```rust
// This is valid dist_agent_lang syntax
@compile_target("blockchain")
@trust("decentralized")
@chain("ethereum")
@interface("typescript")
service DeFiService {
    field balance: int = 1000;
    field owner: address;
    
    fn transfer(to: address, amount: int) -> string {
        if amount > balance {
            return "insufficient_funds";
        }
        balance = balance - amount;
        return "success";
    }
    
    event Transfer(from: address, to: address, amount: int);
}
```

### **2. Available Attributes**

#### **Compilation Targets**
```rust
@compile_target("blockchain")    // Deploy to blockchain
@compile_target("wasm")          // Compile to WebAssembly
@compile_target("native")        // Compile to native code
@compile_target("mobile")        // Compile for mobile
@compile_target("edge")          // Compile for edge computing
```

#### **Trust Models**
```rust
@trust("decentralized")          // Fully decentralized
@trust("hybrid")                 // Hybrid trust model
@trust("centralized")            // Centralized trust
```

#### **Chain Support**
```rust
@chain("ethereum")               // Ethereum mainnet
@chain("polygon")                // Polygon network
@chain("bsc")                // Binance Smart Chain
@chain("solana")                 // Solana network
@chain("avalanche")              // Avalanche network
@chain("arbitrum")               // Arbitrum network
@chain("optimism")               // Optimism network
```

#### **Interface Generation**
```rust
@interface("typescript")         // Generate TypeScript interface
@interface("python")             // Generate Python interface
@interface("rust")               // Generate Rust interface
@interface("javascript")         // Generate JavaScript interface
@interface("java")               // Generate Java interface
@interface("go")                 // Generate Go interface
```

#### **Security & Performance**
```rust
@secure                          // Enable security features
@limit(10000)                    // Set operation limits
@audit                           // Enable audit features
@persistent                      // Enable persistence
@cached                          // Enable caching
```

---

## 🔧 **How to Use the Language**

### **1. Writing dist_agent_lang Code**

Create a file with `.dal` extension (dist_agent_lang):

```rust
// example.dal
@compile_target("blockchain")
@trust("decentralized")
@chain("ethereum")
@interface("typescript")

service TokenService {
    field total_supply: int = 1000000;
    field owner: address;
    
    fn mint(to: address, amount: int) {
        total_supply = total_supply + amount;
        emit Mint(to, amount);
    }
    
    fn burn(from: address, amount: int) {
        total_supply = total_supply - amount;
        emit Burn(from, amount);
    }
    
    event Mint(to: address, amount: int);
    event Burn(from: address, amount: int);
}
```

### **2. Using the Runtime**

```rust
use dist_agent_lang::runtime::Runtime;

fn main() {
    let mut runtime = Runtime::new();
    
    // Execute dist_agent_lang code
    let code = r#"
        @compile_target("blockchain")
        service TestService {
            field balance: int = 1000;
            
            fn transfer(to: address, amount: int) -> string {
                if amount > balance {
                    return "insufficient_funds";
                }
                balance = balance - amount;
                return "success";
            }
        }
    "#;
    
    let result = runtime.execute(code);
    match result {
        Ok(_) => println!("Service executed successfully"),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

### **3. Multi-Chain Services**

```rust
@compile_target("blockchain")
@trust("hybrid")
@chain("ethereum")
@chain("polygon")
@interface("typescript")
@interface("python")

service MultiChainDeFi {
    field total_supply: int = 1000000;
    field owner: address;
    
    fn deploy_to_all_chains() {
        chain::deploy("TokenContract", "bytecode");
    }
    
    fn bridge_tokens(from_chain: string, to_chain: string, amount: int) {
        bridge::transfer(from_chain, to_chain, amount);
        oracle::verify_bridge_completion(from_chain, to_chain);
    }
    
    event BridgeTransfer(from: string, to: string, amount: int);
}
```

### **4. AI Integration**

```rust
@compile_target("blockchain")
@trust("hybrid")
@ai

service AITradingBot {
    trading_strategy: any = null;
    market_data: any = null;
    
    fn initialize() {
        self.trading_strategy = ai::create_strategy({
            "type": "momentum",
            "risk_level": "medium"
        });
    }
    
    fn analyze_market() {
        let market_data = oracle::get_price_data("BTC");
        let analysis = ai::analyze_market_conditions(market_data);
        
        if analysis.opportunity_detected {
            self.execute_trade(analysis.recommendation);
        }
    }
    
    fn execute_trade(recommendation: any) {
        chain::call_contract("trading_contract", "execute", [recommendation]);
    }
}

// Create and use service instances
let bot = AITradingBot::new();
bot.initialize();
bot.analyze_market();
```

---

## 🏗️ **Language Architecture**

### **1. Lexer (Tokenization)**
The lexer recognizes `@` attributes and converts them to tokens:

```rust
// Lexer recognizes these patterns:
@compile_target("blockchain") -> Token::Punctuation(Punctuation::At) + Token::Keyword(Keyword::CompileTarget)
@trust("decentralized")      -> Token::Punctuation(Punctuation::At) + Token::Keyword(Keyword::Trust)
@chain("ethereum")           -> Token::Punctuation(Punctuation::At) + Token::Keyword(Keyword::Chain)
```

### **2. Parser (AST Generation)**
The parser builds an Abstract Syntax Tree with attribute information:

```rust
ServiceStatement {
    name: "DeFiService",
    attributes: ["@compile_target", "@trust", "@chain"],
    compilation_target: Some(CompilationTargetInfo { target: Blockchain, ... }),
    fields: [...],
    methods: [...],
    events: [...]
}
```

### **3. Runtime (Execution)**
The runtime executes services with attribute-aware behavior:

```rust
// Runtime respects compilation targets
if service.compilation_target == CompilationTarget::Blockchain {
    // Generate blockchain bytecode
    generate_solidity_contract(service);
}

// Runtime respects trust models
if service.trust_level == TrustLevel::Decentralized {
    // Enforce decentralized constraints
    validate_decentralized_operations(service);
}
```

---

## 📚 **Example Use Cases**

### **1. DeFi Application**
```rust
@compile_target("blockchain")
@trust("decentralized")
@chain("ethereum")
@chain("polygon")
@interface("typescript")

service DeFiProtocol {
    field total_liquidity: int = 0;
    field fee_rate: float = 0.003;
    
    fn add_liquidity(amount: int) {
        total_liquidity = total_liquidity + amount;
        emit LiquidityAdded(msg.sender, amount);
    }
    
    fn swap(from_token: address, to_token: address, amount: int) {
        let fee = amount * fee_rate;
        let swap_amount = amount - fee;
        
        // Execute swap logic
        chain::call_contract("router", "swap", [from_token, to_token, swap_amount]);
        emit Swap(msg.sender, from_token, to_token, amount);
    }
    
    event LiquidityAdded(user: address, amount: int);
    event Swap(user: address, from: address, to: address, amount: int);
}
```

### **2. AI-Powered NFT**
```rust
@compile_target("blockchain")
@trust("hybrid")
@ai
@interface("typescript")

service AINFT {
    field personality: any;
    field knowledge_base: any;
    
    fn initialize() {
        this.personality = ai::create_personality({
            "traits": ["creative", "helpful", "analytical"],
            "knowledge_domains": ["art", "technology", "finance"]
        });
        
        this.knowledge_base = ai::initialize_knowledge_base();
    }
    
    fn interact_with_owner(message: string) -> string {
        let response = ai::generate_response({
            "message": message,
            "personality": this.personality,
            "context": this.knowledge_base
        });
        
        return response.text;
    }
    
    fn evolve() {
        this.knowledge_base = ai::update_knowledge(this.knowledge_base);
        emit NFTEvolved(this.token_id, this.knowledge_base);
    }
    
    event NFTEvolved(token_id: int, new_knowledge: any);
}
```

### **3. Cross-Chain Bridge**
```rust
@compile_target("blockchain")
@trust("hybrid")
@chain("ethereum")
@chain("polygon")
@chain("binance")
@interface("typescript")
@interface("python")

service CrossChainBridge {
    field bridge_fees: map<string, float>;
    field supported_chains: array<string>;
    
    fn initialize() {
        this.supported_chains = ["ethereum", "polygon", "binance"];
        this.bridge_fees = {
            "ethereum": 0.001,
            "polygon": 0.0001,
            "binance": 0.0005
        };
    }
    
    fn bridge_tokens(from_chain: string, to_chain: string, amount: int) {
        // Validate chains
        if !this.supported_chains.contains(from_chain) {
            throw "Unsupported source chain";
        }
        
        if !this.supported_chains.contains(to_chain) {
            throw "Unsupported destination chain";
        }
        
        // Calculate fees
        let fee = amount * this.bridge_fees[from_chain];
        let bridge_amount = amount - fee;
        
        // Execute bridge
        bridge::transfer(from_chain, to_chain, bridge_amount);
        
        // Verify completion
        oracle::verify_bridge_completion(from_chain, to_chain);
        
        emit BridgeCompleted(from_chain, to_chain, bridge_amount, fee);
    }
    
    event BridgeCompleted(from: string, to: string, amount: int, fee: float);
}
```

---

## 🔍 **Understanding the Example Files**

### **Why Example Files Use Comments**

The example files in the `examples/` directory are **demonstrations** of the language syntax, not actual Rust code. They use comments (`// @trust("hybrid")`) because:

1. **File Extension**: They have `.rs` extensions for documentation purposes
2. **Rust Compiler**: Rust tries to parse them as Rust code
3. **Syntax Conflict**: `@` attributes aren't valid Rust syntax

### **How to Use the Examples**

1. **Read the Examples**: Understand the language syntax and patterns
2. **Extract Patterns**: Use the patterns in your actual `.dal` files
3. **Runtime Execution**: Use the runtime to execute the language code

### **Converting Examples to Actual Code**

```rust
// Example file shows:
// @trust("hybrid")
// service MyService { ... }

// Actual usage:
let code = r#"
@trust("hybrid")
service MyService {
    // Implementation
}
"#;

let mut runtime = Runtime::new();
runtime.execute(code);
```

---

## 🚀 **Getting Started**

### **1. Install the Language**
```bash
git clone <repository>
cd dist_agent_lang
cargo build
```

### **2. Create Your First Service**
```rust
// my_service.dal
@compile_target("blockchain")
@trust("decentralized")
@chain("ethereum")

service MyFirstService {
    field counter: int = 0;
    
    fn increment() {
        counter = counter + 1;
        emit CounterIncremented(counter);
    }
    
    event CounterIncremented(value: int);
}
```

### **3. Execute the Service**
```rust
use dist_agent_lang::runtime::Runtime;

fn main() {
    let mut runtime = Runtime::new();
    
    let code = std::fs::read_to_string("my_service.dal").unwrap();
    let result = runtime.execute(&code);
    
    match result {
        Ok(_) => println!("Service executed successfully!"),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

---

## 📖 **Next Steps**

1. **Explore Examples**: Study the example files for patterns
2. **Build Services**: Create your own services with attributes
3. **Multi-Chain**: Experiment with cross-chain operations
4. **AI Integration**: Try AI-powered services
5. **Interface Generation**: Generate client interfaces

The `@` attributes are **core features** of `dist_agent_lang` that make it powerful for blockchain development, AI integration, and multi-chain operations. Use them to define how your services behave and interact with different platforms!
