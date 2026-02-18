# 🚀 Quick Start Guide - Get Started in 5 Minutes

> **📢 Beta Release v1.0.5:** dist_agent_lang is actively developed with consistent updates. This is a beta release — test thoroughly before production use. **Beta testing contributions appreciated!** 🙏

Welcome to **dist_agent_lang** (DAL) - the hybrid language for decentralized smart contracts and centralized services!

This guide will get you from zero to your first deployed contract in **5 minutes**.

---

## 📦 Installation (30 seconds)

### Option 1: From crates.io (recommended)
```bash
cargo install dist_agent_lang
```

### Option 2: From source
```bash
git clone https://github.com/okjason-source/dist_agent_lang.git
cd dist_agent_lang
cargo build --release
export PATH="$PWD/target/release:$PATH"
```

### Option 3: From release binary
Download from [GitHub Releases](https://github.com/okjason-source/dist_agent_lang/releases) and extract the binary to your PATH.

### Verify Installation
```bash
dist_agent_lang --version
# Expected: dist_agent_lang v1.0.5
```

---

## 🎯 Your First Contract (3 minutes)

### Step 1: Create a New Project
```bash
# Create project directory
mkdir my-first-contract
cd my-first-contract

# Create your first contract file
touch hello.dal
```

### Step 2: Write Your Contract
Copy this into `hello.dal`:

```dal
// hello.dal - Your first smart contract!

@contract
@blockchain("ethereum")
@version("1.0.0")
contract HelloWorld {
    // State variable
    string public greeting;
    
    // Constructor
    constructor(string memory _greeting) {
        greeting = _greeting;
    }
    
    // Update greeting
    @public
    function setGreeting(string memory newGreeting) {
        greeting = newGreeting;
    }
    
    // Get greeting
    @public
    @view
    function getGreeting() -> string {
        return greeting;
    }
    
    // Greet someone
    @public
    @view
    function greet(string memory name) -> string {
        return greeting + ", " + name + "!";
    }
}
```

### Step 3: Compile Your Contract
```bash
dal compile hello.dal

# Output:
# ✅ Compiled successfully: hello.dal
# 📄 Output: build/HelloWorld.sol (Solidity)
# 📄 Output: build/HelloWorld.wasm (WASM)
# 📄 Output: build/HelloWorld.json (ABI)
```

### Step 4: Test Your Contract
```bash
dal test hello.dal

# Creates a test environment and runs basic checks
# ✅ All tests passed!
```

---

## 🌐 Deploy Your Contract (1 minute)

### Local Testnet (Fastest)
```bash
# Start local testnet
dal testnet start

# Deploy
dal deploy hello.dal --network local --constructor "Hello, World"

# Output:
# 🚀 Deploying HelloWorld to local testnet...
# ✅ Deployed at: 0x1234567890abcdef...
# 📝 Transaction: 0xabcdef1234567890...
```

### Public Testnet (Sepolia, Mumbai, etc.)
```bash
# Set up your wallet (one-time setup)
dal wallet create

# Fund your wallet (get test tokens from faucet)
dal wallet fund --network sepolia

# Deploy to testnet
dal deploy hello.dal --network sepolia --constructor "Hello, Testnet"
```

---

## 🧪 Interact with Your Contract

### Using the CLI
```bash
# Read greeting
dal call HelloWorld getGreeting --network local

# Update greeting
dal send HelloWorld setGreeting "Hello, DAL!" --network local

# Greet someone
dal call HelloWorld greet "Alice" --network local
# Output: "Hello, DAL!, Alice!"
```

### Using the Interactive Console
```bash
# Start interactive console
dal console --network local

# In console:
> let contract = await connect("HelloWorld");
> await contract.getGreeting();
"Hello, World"

> await contract.setGreeting("Bonjour");
Transaction: 0x...

> await contract.greet("Bob");
"Bonjour, Bob!"
```

---

## 📚 Next Steps

Congratulations! You've just:
- ✅ Installed dist_agent_lang
- ✅ Written your first smart contract
- ✅ Compiled it to multiple targets
- ✅ Deployed to a local testnet
- ✅ Interacted with your contract

### Continue Learning

1. **[Deployment Guide](DEPLOYMENT_GUIDE.md)** - Deploy to production networks
2. **[Best Practices](BEST_PRACTICES.md)** - Write secure, efficient contracts
3. **[Tutorials](tutorials/)** - Build real-world applications
4. **[API Reference](API_REFERENCE.md)** - Explore the full feature set

### Try More Examples

```bash
# DeFi Token
dal new token --template erc20
cd token && dal compile . && dal deploy .

# NFT Marketplace
dal new marketplace --template nft
cd marketplace && dal compile . && dal deploy .

# Cross-Chain Bridge
dal new bridge --template cross-chain
cd bridge && dal compile . && dal deploy .
```

---

## 🆘 Troubleshooting

### Build Errors
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

### Contract Compilation Fails
```bash
# Check syntax
dal check hello.dal

# Verbose output
dal compile hello.dal --verbose
```

### Deployment Fails
```bash
# Check network status
dal network status --network local

# Check wallet balance
dal wallet balance

# Increase gas limit
dal deploy hello.dal --gas-limit 5000000
```

---

## 💡 Quick Tips

### 1. Use Templates
```bash
dal new my-project --template [starter|token|nft|defi|dao]
```

### 2. Hot Reload During Development
```bash
dal watch hello.dal --auto-test
# Auto-compiles and tests on file save
```

### 3. Format Your Code
```bash
dal format hello.dal
# Applies consistent formatting
```

### 4. Lint Your Code
```bash
dal lint hello.dal
# Checks for common issues and best practices
```

### 5. Generate Documentation
```bash
dal doc hello.dal
# Generates HTML documentation from your comments
```

---

## 🎓 Key Concepts (30-second overview)

### Attributes
Use attributes to control behavior:
- `@contract` - Mark as a contract
- `@public` - Public function
- `@private` - Private function
- `@view` - Read-only (no state changes)
- `@async` - Asynchronous function
- `@blockchain("ethereum")` - Target blockchain

### Multi-Chain Support
```dal
@contract
@blockchain("ethereum")  // Deploy to Ethereum
@blockchain("solana")    // AND Solana
@blockchain("polygon")   // AND Polygon
contract MultiChain {
    // Automatically compiled for all chains!
}
```

### Built-in Security
```dal
@contract
@reentrancy_guard        // Automatic reentrancy protection
@safe_math               // Automatic overflow protection
contract Secure {
    // Your code is secure by default!
}
```

### Oracle Integration
```dal
@contract
contract PriceFeed {
    @public
    function getBTCPrice() -> int {
        // Fetch from oracle with built-in security
        let price = oracle::fetch("chainlink", "BTC/USD");
        return price;
    }
}
```

---

## 🌟 What Makes DAL Special?

| Feature | DAL | Solidity | Rust |
|---------|-----|----------|------|
| **Multi-Chain** | ✅ Native | ❌ | ⚠️ Limited |
| **Type Safety** | ✅ Strong | ⚠️ Medium | ✅ Strong |
| **Security** | ✅ Built-in | ⚠️ Manual | ⚠️ Manual |
| **Oracle Access** | ✅ Native | ❌ External | ❌ External |
| **Async/Await** | ✅ Native | ❌ | ✅ Native |
| **Learning Curve** | 🟢 Moderate | 🟢 Easy | 🔴 Steep |
| **Deployment** | ✅ Multi-target | ⚠️ EVM only | ⚠️ Single target |

---

## 🤝 Get Help

- 📖 **Documentation**: [Full Docs](../Documentation.md)
- 💬 **Discord**: [Join Community](#)
- 🐛 **Issues**: [GitHub Issues](https://github.com/okjason-source/dist_agent_lang/issues)
- 📧 **Email**: support@dist-agent-lang.org
- 🎓 **Tutorials**: [Learn by Building](tutorials/)

---

## 📝 Example Projects

Check out complete examples in the `examples/` directory:

```bash
cd examples/

# Token standards
./01_hello_world.dal         # Simple greeting contract
./02_nft_marketplace.dal     # NFT marketplace with royalties
./03_trading_bot.dal         # Automated trading bot
./04_error_handling.dal      # Error handling patterns

# Advanced examples
./defi_nft_rwa_contract.dal  # DeFi + NFT + RWA integration
./multi_chain_operations.dal # Cross-chain operations
./enhanced_language_features.dal # All language features
```

---

**🎉 Welcome to the DAL community! Let's build the future of decentralized applications together.**

**Next:** [Deployment Guide →](DEPLOYMENT_GUIDE.md)

