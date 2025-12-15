# dist_agent_lang

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg)](https://github.com/distagentlang/dist_agent_lang)

**A hybrid compiled programming language for AI agents, blockchain, and distributed systems**

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

## 📦 Installation

### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Node.js 18+ (for build scripts)

### Quick Install
```bash
# Clone the repository
git clone https://github.com/distagentlang/dist_agent_lang.git
cd dist_agent_lang

# Build and install
cargo install --path .
```

### From Release Package
```bash
# Download latest release
wget https://github.com/distagentlang/dist_agent_lang/releases/latest/download/dist_agent_lang-1.0.0.tar.gz

# Extract and install
tar -xzf dist_agent_lang-1.0.0.tar.gz
cd dist_agent_lang-1.0.0
./install.sh
```

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
            "version": "1.0.0"
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
    name: string = "MyToken",
    symbol: string = "MTK",
    total_supply: int = 1000000,
    balances: map<string, int>,
    
    fn initialize() {
        let owner = auth::session().user_id;
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
git clone https://github.com/distagentlang/dist_agent_lang.git
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

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

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
- **Issues**: [GitHub Issues](https://github.com/distagentlang/dist_agent_lang/issues)
- **Discussions**: [GitHub Discussions](https://github.com/distagentlang/dist_agent_lang/discussions)
- **Email**: team@distagentlang.com

## 🙏 Acknowledgments

- Rust community for the excellent language and ecosystem
- Ethereum community for blockchain standards and tools
- AI/ML community for inspiration and best practices
- Open source contributors who made this project possible

---

**Made with ❤️ by the dist_agent_lang team**
