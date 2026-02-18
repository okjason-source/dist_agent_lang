# dist_agent_lang

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg)](https://github.com/dist_agent_lang/dist_agent_lang)

**A hybrid compiled programming language for AI agents, blockchain, and distributed systems**

---

## 📖 **PUBLIC DOCUMENTATION**

### ➡️ **[Complete Documentation (AI/LLM Ready)](docs/PUBLIC_DOCUMENTATION_INDEX.md)**

**Quick Links:**
- [Installation](docs/PUBLIC_DOCUMENTATION_INDEX.md#installation)
- [Getting Started](docs/PUBLIC_DOCUMENTATION_INDEX.md#your-first-dal-program)
- [Standard Library Reference](docs/PUBLIC_DOCUMENTATION_INDEX.md#standard-library-stdlib)
- [Package Management](docs/PUBLIC_DOCUMENTATION_INDEX.md#package-management)
- [CLI Commands](docs/PUBLIC_DOCUMENTATION_INDEX.md#cli-reference)
- [Examples & Tutorials](docs/PUBLIC_DOCUMENTATION_INDEX.md#examples)
- [Integration](docs/PUBLIC_DOCUMENTATION_INDEX.md#aillm-integration-guide)

**Optimized for both developers and AI assistants.**

---

## ⚠️ Beta Release Notice

**Current Version: v1.0.5 (Beta Release) — Actively Developed**

dist_agent_lang is an **actively maintained beta release** with consistent updates and improvements. The language includes extensive security features, comprehensive documentation, and has passed all tests (140+ passing).

**🙏 Beta Testing Contributions Appreciated!**  
We welcome feedback, bug reports, and contributions from the developer community to help us reach production readiness (v1.1.0+).

### ✅ Safe For:
- **Development & Prototyping** - Building and testing applications
- **Learning & Experimentation** - Educational purposes and learning
- **Non-Critical Applications** - Applications not handling significant value
- **Testing & Validation** - Validating concepts and workflows
- **Beta Testing** - Help us improve with your feedback!

### ⚠️ Use With Caution For:
- **Production Financial Applications** - Applications handling real money (wait for v1.1.0+)
- **High-Value Smart Contracts** - Contracts managing significant assets (third-party audit recommended)
- **Critical Infrastructure** - Systems requiring high reliability (additional validation needed)
- **Sensitive Data Applications** - Additional security audits strongly recommended

### 🔒 Security Features (v1.0.5):
- ✅ Reentrancy protection
- ✅ Safe math (overflow/underflow protection)
- ✅ State isolation
- ✅ Cross-chain security
- ✅ Oracle security (signed feeds, multi-source validation)
- ✅ Transaction atomicity (ACID guarantees)
- ✅ Enhanced security logging with source tracking
- ✅ 140+ tests passing (100%)
- ✅ Zero compilation errors
- ✅ Dependency security audit passed (0 vulnerabilities)

### 🚀 What's New in v1.0.5:
- **Enhanced Log Functions** - Optional source parameter for all log functions (info, warning, error, debug, audit)
- **HTTP Timeout Standardization** - Standardized to 30000ms (30 seconds) for consistency with HTTP libraries
- **Comprehensive Security Integration Tests** - End-to-end security workflow tests using actual DAL language code
- **Improved Logging** - Custom source identifiers for better log filtering and debugging
- **Enhanced Testing** - 140+ tests covering all standard library modules

### 🔄 Active Development:
This project receives **consistent updates** with improvements to security, performance, documentation, and features. We're working toward v1.1.0 (production release) with third-party security audits, real-world validation, and community feedback.

**Target for Production (v1.1.0+):** ~14 weeks with community validation

### 📋 Recommendations:
- **For Production Use**: Wait for v1.1.0+ with third-party security audit and real-world validation
- **For Critical Applications**: Conduct independent security audit before deployment
- **For Financial Applications**: Additional formal verification strongly recommended
- **Always**: Test thoroughly in development/testnet environments first
- **Join Beta Testing**: Help us improve by testing and providing feedback!

### 🤝 How to Contribute to Beta:
- **🧪 Testing** - Use the language, run examples, report bugs (no coding required!)
- **📝 Documentation** - Improve docs, write tutorials, fix typos
- **💻 Code** - Fix bugs, implement features, add tests
- **💡 Ideas** - Share use cases, suggest improvements, join discussions

**👉 New to contributing?** Check out [GOOD_FIRST_ISSUES.md](GOOD_FIRST_ISSUES.md) for beginner-friendly tasks!

**📖 Full Guide:** See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed contribution guidelines.

**We value your feedback and contributions as we work toward production readiness!**

## 🚀 Features

### 🤖 AI Agent Framework
- **Multi-Agent Coordination**: Create and orchestrate AI agents with built-in coordination
- **Workflow Management**: Define complex workflows with step dependencies
- **Task Execution**: Distributed task processing with status tracking
- **Memory Management**: Persistent agent memory and context awareness
- **Communication Protocols**: Inter-agent messaging and event-driven communication

### ⛓️ Blockchain Integration
- **Multi-Chain Support**: Ethereum, Polygon, Binance, Solana, Avalanche, Arbitrum, Optimism
- **Smart Contract Development**: Native smart contract creation and deployment
- **Cross-Chain Operations**: Seamless asset transfers across different blockchains
- **Oracle Integration**: Real-world data feeds and external API connectivity
- **Gas Optimization**: Chain-specific gas estimation and transaction management

### 🔒 Security & Compliance
- **KYC/AML Integration**: Built-in Know Your Customer and Anti-Money Laundering checks
- **Trust Model System**: Configurable trust levels and security profiles
- **Audit Trails**: Comprehensive logging and compliance tracking
- **Capability-Based Security**: Fine-grained permission system
- **Cryptographic Operations**: AES-256, SHA-256, ECDSA, multi-signature support

### 🎯 Multi-Target Compilation
- **Blockchain**: Smart contract compilation for multiple chains
- **WebAssembly**: Web-based applications and browser integration
- **Native**: High-performance desktop applications
- **Mobile**: iOS and Android app development
- **Edge**: IoT and edge computing devices

## 📚 Documentation

### Getting Started (5 minutes)
- **[Quick Start Guide](../docs/QUICK_START.md)** - Get up and running in 5 minutes
- **[Installation Guide](#-installation)** - Detailed installation instructions
- **[Your First Contract](../docs/QUICK_START.md#-your-first-contract-3-minutes)** - Build and deploy your first smart contract

### Production Deployment
- **[Deployment Guide](../docs/DEPLOYMENT_GUIDE.md)** - Complete production deployment guide
- **[Docker Deployment](../docs/DEPLOYMENT_GUIDE.md#-docker-deployment)** - Containerized deployment
- **[CI/CD Integration](../docs/DEPLOYMENT_GUIDE.md#-cicd-integration)** - Automated deployment pipelines

### Developer Guides
- **[Best Practices](../docs/BEST_PRACTICES.md)** - Security, performance, and code organization
- **[API Reference](../docs/API_REFERENCE.md)** - Complete standard library documentation (22 modules)
- **[Testing Guide](../docs/BEST_PRACTICES.md#-testing-strategies)** - Comprehensive testing strategies

### Tutorials
- **[Tutorial 1: DeFi Token](../docs/tutorials/01_defi_token.md)** - Build a token with oracle integration
- **[Tutorial 2: AI Trading Agent](../docs/tutorials/02_ai_trading_agent.md)** - Build an AI-powered trading bot
- **[Tutorial 3: NFT Marketplace](#)** - Complete NFT marketplace
- **[Tutorial 4: Cross-Chain Bridge](#)** - Multi-chain operations
- **[More Tutorials...](../examples/)** - Real-world examples

### AI Features
- **[AI Features Guide](../docs/AI_FEATURES_GUIDE.md)** - Complete AI capabilities overview
- **[AI Best Practices](../docs/AI_BEST_PRACTICES.md)** - Security and optimization for AI
- **[AI+Blockchain Integration](#)** - Combine AI with DeFi, NFTs, and more
- **[AI API Reference](../docs/API_REFERENCE.md#6-ai-aiml-integration)** - Complete AI API documentation

### CloudAdmin & Hybrid Trust
- **[CloudAdmin Guide](../docs/CLOUDADMIN_GUIDE.md)** - Hybrid trust and admin control architecture
- **[Hybrid Marketplace Tutorial](../docs/tutorials/03_hybrid_marketplace_cloudadmin.md)** - Build moderated marketplace
- **[CloudAdmin API Reference](../docs/API_REFERENCE.md#21-cloudadmin-cloudadmin-security)** - Complete CloudAdmin API
- **[Process Management](../docs/API_REFERENCE.md#22-admin-process-management)** - Admin process control

### Migration Guides
- **[From Solidity](../docs/migration/FROM_SOLIDITY.md)** - Migrate Solidity contracts to DAL
- **[From Rust](#)** - Migrate Rust contracts
- **[From Move](#)** - Migrate Move contracts

### Architecture & Reference
- **[Language Reference](../Documentation.md)** - Complete language specification
- **[Architecture Overview](../docs/ARCHITECTURE_SEPARATION.md)** - System architecture
- **[Phase 3 Plan](PHASE3_PLAN.md)** - Current development phase
- **[Production Roadmap](PRODUCTION_ROADMAP.md)** - Journey to v1.1.0

---

## 📦 Installation

### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Node.js 18+ (for build scripts)

### Quick Install

**From crates.io (recommended):**
```bash
cargo install dist_agent_lang
```

**From source:**
```bash
git clone https://github.com/okjason-source/dist_agent_lang.git
cd dist_agent_lang
cargo install --path
```

**From release binary:** Download from [GitHub Releases](https://github.com/okjason-source/dist_agent_lang/releases) and extract the binary to your PATH.

## 🎮 Quick Start

### Hello World
```rust
// hello_world.dal
@trust("hybrid")
@chain("ethereum")
service HelloWorld {
    fn main() {
        print("Hello, dist_agent_lang!");
        
        // Create an AI agent
        let agent = ai::create_agent("greeter", {
            "role": "greeting_specialist",
            "capabilities": ["greeting", "conversation"]
        });
        
        // Deploy to blockchain
        let contract = chain::deploy_contract("HelloWorld", {
            "name": "Hello World Contract",
            "version": "1.0.5"
        });
        
        log::info("main", "Hello World deployed successfully!");
    }
}
```

### AI Agent Example
```rust
// ai_agent_example.dal
@trust("hybrid")
@ai
@chain("ethereum")
service AIAgentDemo {
    fn create_ai_system() {
        // Create coordinator
        let coordinator = ai::create_coordinator("project_coordinator");
        
        // Spawn specialized agents
        spawn data_analyzer:ai {
            role: "data_analysis_specialist",
            capabilities: ["data_analysis", "statistics", "visualization"]
        } {
            log::info("agent", "Data Analyzer agent ready");
        }
        
        spawn blockchain_expert:ai {
            role: "blockchain_specialist", 
            capabilities: ["smart_contracts", "defi", "nft"]
        } {
            log::info("agent", "Blockchain Expert agent ready");
        }
        
        // Add agents to coordinator
        ai::add_agent_to_coordinator(coordinator, data_analyzer);
        ai::add_agent_to_coordinator(coordinator, blockchain_expert);
        
        // Create workflow
        let workflow = ai::create_workflow(coordinator, "data_analysis", [
            {
                "step_id": "analyze_data",
                "agent_id": data_analyzer.id,
                "task_type": "data_analysis"
            },
            {
                "step_id": "deploy_contract",
                "agent_id": blockchain_expert.id,
                "task_type": "contract_deployment",
                "dependencies": ["analyze_data"]
            }
        ]);
        
        // Execute workflow
        ai::execute_workflow(coordinator, workflow.workflow_id);
    }
}
```

### Smart Contract Example
```rust
// smart_contract_example.dal
@trust("hybrid")
@secure
@chain("ethereum")
service TokenContract {
    name: string = "MyToken";
    symbol: string = "MTK";
    total_supply: int = 1000000;
    balances: map<string, int> = {};
    
    fn initialize(owner: string) {
        self.balances[owner] = self.total_supply;
        log::info("contract", "Token contract initialized");
    }
    
    fn transfer(to: string, amount: int) -> bool {
        let from = auth::session().user_id;
        
        if self.balances[from] < amount {
            return false;
        }
        
        self.balances[from] = self.balances[from] - amount;
        self.balances[to] = self.balances[to] + amount;
        
        log::info("transfer", "Transfer completed: " + amount + " tokens");
        return true;
    }
}

// Create and use service instances
let token = TokenContract::new();
token.initialize("0x1234567890123456789012345678901234567890");

// Alternative instantiation syntax
let token2 = service::new("TokenContract");
token2.initialize("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd");

// Call methods on instances
token.transfer("0x456...", 100);
```

## 📚 Documentation

### Getting Started
- [Installation Guide](INSTALLATION.md) - Step-by-step installation instructions
- [Usage Guide](USAGE.md) - Quick start and common commands
- [Comprehensive Documentation](docs/Documentation.md) - Complete language overview

### Tutorials
- [Complete Tutorial Series](docs/tutorials.md) - 12 comprehensive tutorials from beginner to advanced
  - Tutorial 1: Getting Started
  - Tutorial 2: AI Agents
  - Tutorial 3: Blockchain Integration
  - Tutorial 4: Multi-Chain Operations
  - Tutorial 5: Security & Compliance
  - Tutorial 6: AI Integration
  - Tutorial 7: Database Operations
  - Tutorial 8: Web API Operations
  - Tutorial 9: Compliance Features
  - Tutorial 10: Error Handling
  - Tutorial 11: Performance Optimization
  - Tutorial 12: Testing & Debugging

### Reference Documentation
- [API Reference](docs/api_reference.md) - Complete standard library API
- [Language Syntax](docs/syntax.md) - Syntax reference guide
- [Attributes Reference](docs/attributes.md) - All available attributes
- [Usage Guide](docs/USAGE_GUIDE.md) - Detailed usage instructions
- [Configuration Guide](docs/CONFIGURATION_GUIDE.md) - Configuration management

### Feature Guides
- [KYC/AML Features](docs/KYC_AML_FEATURES.md) - Compliance features guide
- [Oracle Development](docs/ORACLE_DEVELOPMENT_README.md) - Oracle integration guide
- [XNFT & Dynamic RWA Guide](docs/XNFT_DYNAMIC_RWA_GUIDE.md) - NFT and RWA tokenization

### Examples
- [Example Programs](examples/) - 27+ example programs demonstrating features

## 🛠️ Development

### Building from Source
```bash
git clone https://github.com/dist_agent_lang/dist_agent_lang.git
cd dist_agent_lang

# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Project Structure
```
dist_agent_lang/
├── src/
│   ├── lexer/          # Lexical analysis
│   ├── parser/          # Syntax parsing
│   ├── runtime/         # Runtime environment
│   ├── stdlib/          # Standard library modules
│   ├── testing/         # Testing framework
│   └── performance/     # Performance optimization
├── examples/            # Example code
├── docs/               # Documentation
├── scripts/            # Build and deployment scripts
├── templates/          # Code generation templates
└── tests/              # Test suites
```

## 🧪 Examples

The project includes 11 comprehensive examples:

1. **AI Agent System Demo** - Multi-agent coordination and workflow management
2. **Backend Connectivity Patterns** - Database and API integration patterns
3. **Chain Selection Example** - Multi-chain interaction and selection
4. **Cross-Chain Patterns** - Asset management across different blockchains
5. **DeFi NFT RWA Contract** - Real World Asset tokenization (Arbitrum)
6. **Dynamic NFT Examples** - Dynamic NFTs with real-world data (Ethereum)
7. **Dynamic RWA Examples** - Real World Asset tokenization (Ethereum)
8. **Enhanced Language Features** - Advanced language capabilities
9. **General Purpose Demo** - General programming examples
10. **Integrated Spawn AI Examples** - Spawn and AI agent integration
11. **KEYS Token Implementation** - Complete token system (Ethereum)

## 🧪 Testing

DAL uses a **three-layer testing strategy** for comprehensive validation:

### Layer 1: Rust Unit Tests (Syntax Validation)
Fast syntax and parse-time validation for all DAL code:

```bash
cargo test                        # Run all tests
cargo test --test example_tests   # Run example tests only
cargo test -- --nocapture         # Run with output
```

### Layer 2: Semantic Validators (Attribute & Type Validation)
Validation helpers for semantic correctness:

```rust
// Used within tests for semantic validation
test::expect_valid_trust_model("hybrid");
test::expect_valid_chain("ethereum");
test::expect_compatible_attributes(["trust", "chain"]);
test::expect_type(&value, "number");
test::expect_in_range(value, 0.0, 100.0);
```

### Layer 3: DAL Test Files (Runtime Behavior)
Hardhat-style testing framework for DAL (`.test.dal` files):

```bash
# Run all DAL test files
./scripts/run_dal_tests.sh

# Run specific test file
cargo run --release -- run examples/token_contract.test.dal
```

**Example DAL test:**
```dal
describe("TokenContract", fn() {
    let contract;
    
    beforeEach(fn() {
        contract = deploy_service("TokenContract", {});
    });
    
    it("should transfer tokens", fn() {
        contract.transfer("bob", 100.0);
        expect(contract.balance_of("bob")).to_equal(100.0);
    });
});
```

### Testing Documentation

For complete testing guides:
- [`docs/THREE_LAYER_TESTING.md`](docs/THREE_LAYER_TESTING.md) - Complete strategy overview
- [`docs/guides/TESTING_GUIDE.md`](docs/guides/TESTING_GUIDE.md) - Comprehensive testing guide
- [`docs/TESTING_ATTRIBUTES.md`](docs/TESTING_ATTRIBUTES.md) - Attribute testing deep dive
- [`docs/WHY_RUST_UNIT_TESTS.md`](docs/WHY_RUST_UNIT_TESTS.md) - Rationale for Rust tests

## 🔧 Configuration

### Environment Variables
```bash
# Blockchain configuration
DIST_AGENT_RPC_URL_ETHEREUM=https://mainnet.infura.io/v3/YOUR_KEY
DIST_AGENT_RPC_URL_POLYGON=https://polygon-rpc.com
DIST_AGENT_PRIVATE_KEY=your_private_key

# AI configuration
DIST_AGENT_AI_API_KEY=your_openai_key
DIST_AGENT_AI_MODEL=gpt-4

# Database configuration
DIST_AGENT_DB_URL=postgresql://user:pass@localhost/db
```

### Configuration File
```toml
# config.toml
[blockchain]
ethereum_rpc = "https://mainnet.infura.io/v3/YOUR_KEY"
polygon_rpc = "https://polygon-rpc.com"
private_key = "your_private_key"

[ai]
api_key = "your_openai_key"
model = "gpt-4"
max_tokens = 4096

[database]
url = "postgresql://user:pass@localhost/db"
pool_size = 10
```

## 🤝 Contributing

**Contributors Welcome!** 🎉 We're actively seeking help to improve and test dist_agent_lang. Every contribution, no matter how small, helps!

### 🚀 Quick Start Contributing

**No coding required?** You can still help!
- 🧪 **Test the language** - Run examples, report bugs
- 📝 **Improve documentation** - Fix typos, clarify instructions
- 💡 **Share feedback** - Tell us what works and what doesn't

**Ready to code?** Great!
- 🟢 **Beginners**: Check out [GOOD_FIRST_ISSUES.md](GOOD_FIRST_ISSUES.md)
- 📖 **Full Guide**: See [CONTRIBUTING.md](CONTRIBUTING.md)
- 🔧 **Specific Tasks**: See [CODEBASE_TODOS.md](../CODEBASE_TODOS.md)

### Development Setup
```bash
# Fork and clone
git clone https://github.com/your-username/dist_agent_lang.git
cd dist_agent_lang

# Install dependencies
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Check for issues
cargo clippy
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/dist_agent_lang/dist_agent_lang/issues)
- **Discussions**: [GitHub Discussions](https://github.com/dist_agent_lang/dist_agent_lang/discussions)
- **Email**: jason.dinh.developer@gmail.com

## 🙏 Acknowledgments

- Rust community for the excellent language and ecosystem
- Ethereum community for blockchain standards and tools
- AI/ML community for inspiration and best practices
- Open source contributors who made this project possible

---

**Made with ❤️ by OK Jason**
