# Installation and Usage Guide

## ⚠️ Beta Release Notice

**Current Version: v1.0.3 (Beta Release)**

dist_agent_lang is an **actively developed beta release** that is updated consistently with new features and improvements. While the language includes comprehensive security features and has passed all tests (187+ passing), it should be used with appropriate caution based on your use case.

**🙏 Beta Testing Contributions Appreciated!**  
We welcome feedback, bug reports, and contributions from early adopters to help us reach production readiness.

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

### 🔒 Security Features (v1.0.3):
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

### 🚀 What's New in v1.0.3:
- **Enhanced Log Functions** - Optional source parameter for all log functions (info, warning, error, debug, audit)
- **HTTP Timeout Standardization** - Standardized to 30000ms (30 seconds) for consistency with HTTP libraries
- **Comprehensive Security Integration Tests** - End-to-end security workflow tests using actual DAL language code
- **Improved Logging** - Custom source identifiers for better log filtering and debugging
- **Enhanced Testing** - 140+ tests covering all standard library modules

### 🔄 Active Development:
This project is updated consistently with improvements to security, performance, and features. Check the repository regularly for updates and join our community to contribute!


## 📦 Installation

### Prerequisites
- Linux, macOS, or Windows
- No additional dependencies required (binary is self-contained)

### Quick Install

#### Linux/macOS
```bash
# Download the release package
wget https://github.com/distagentlang/dist_agent_lang/releases/latest/download/dist_agent_lang-latest.tar.gz

# Extract the package
tar -xzf dist_agent_lang-latest.tar.gz
cd dist_agent_lang-*

# Run the installation script
./install.sh
```

#### Windows
```bash
# Download the release package
# Extract dist_agent_lang-1.0.0.zip
# Open Command Prompt in the extracted folder
# Run install.bat (if available) or manually copy bin/dist_agent_lang.exe to your PATH
```

### Manual Installation

#### Linux/macOS
```bash
# Extract the package
tar -xzf dist_agent_lang-1.0.0.tar.gz
cd dist_agent_lang-1.0.0

# Copy binary to a directory in your PATH
sudo cp bin/dist_agent_lang /usr/local/bin/
# OR for user installation:
mkdir -p ~/.local/bin
cp bin/dist_agent_lang ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"  # Add to ~/.bashrc or ~/.zshrc
```

#### Windows
```bash
# Extract dist_agent_lang-1.0.0.zip
# Copy bin/dist_agent_lang.exe to a directory in your PATH
# For example: C:\Program Files\dist_agent_lang\
# Add that directory to your system PATH environment variable
```

### Verify Installation
```bash
dist_agent_lang --version
# Should output: dist_agent_lang 1.0.3
```

## 🚀 Quick Start

### 1. Run a dist_agent_lang File

```bash
# Run a simple example
dist_agent_lang run examples/hello_world_demo.dal

# Run with specific file
dist_agent_lang run path/to/your/file.dal
```

### 2. Create Your First dist_agent_lang Program

Create a file `hello.dal`:

```rust
@trust("hybrid")
service HelloWorld {
    fn main() {
        print("Hello, dist_agent_lang!");
        
        // Create an AI agent
        let agent = ai::create_agent("greeter", {
            "role": "greeting_specialist",
            "capabilities": ["greeting", "conversation"]
        });
        
        log::info("Hello World program executed successfully!", {}, None);
    }
}
```

Run it:
```bash
dist_agent_lang run hello.dal
```

### 3. Available Commands

```bash
# Run a dist_agent_lang file
dist_agent_lang run <file.dal>

# Parse and validate syntax (without executing)
dist_agent_lang parse <file.dal>

# Show help
dist_agent_lang --help

# Show version
dist_agent_lang --version
```

## 📚 Examples

The package includes example files in the `dist_agent_lang/examples/` directory:

### Basic Examples
- `hello_world_demo.dal` - Simple hello world program
- `general_purpose_demo.dal` - General programming examples
- `simple_chain_examples.dal` - Basic blockchain operations

### AI Agent Examples
- `agent_system_demo.dal` - Multi-agent coordination
- `integrated_spawn_ai_examples.dal` - Spawn and AI integration
- `llm_integration_examples.dal` - LLM integration patterns
- `llm_motivations_demo.dal` - LLM use cases and motivations

### Blockchain Examples
- `smart_contract.dal` - Smart contract development
- `cross_chain_patterns.dal` - Cross-chain operations
- `multi_chain_operations.dal` - Multi-chain management
- `chain_selection_example.dal` - Chain selection patterns
- `keys_token_implementation.dal` - Token implementation
- `defi_nft_rwa_contract.dal` - DeFi, NFT, and RWA contracts

### Web & Backend Examples
- `simple_web_api_example.dal` - Web API creation
- `backend_connectivity_patterns.dal` - Database and API patterns
- `real_time_backend_example.dal` - Real-time backend
- `practical_backend_example.dal` - Practical backend patterns

### Advanced Examples
- `dynamic_rwa_examples.dal` - Real World Asset tokenization
- `enhanced_language_features.dal` - Advanced language features
- `secure_configuration_example.dal` - Security configuration
- `oracle_quick_start.dal` - Oracle integration
- `oracle_development_setup.dal` - Oracle development
- `xnft_implementation.dal` - XNFT implementation
- `solidity_abi_integration.dal` - Solidity ABI integration
- `solidity_orchestration.dal` - Solidity orchestration


### Running Examples
```bash
# Navigate to the dist_agent_lang directory
cd dist_agent_lang

# Run any example
dist_agent_lang run examples/hello_world_demo.dal
dist_agent_lang run examples/agent_system_demo.dal
dist_agent_lang run examples/smart_contract.dal
```

## 🔧 Configuration

### Environment Variables

Set these environment variables for full functionality:

```bash
# Blockchain configuration
export DIST_AGENT_RPC_URL_ETHEREUM="https://mainnet.infura.io/v3/YOUR_KEY"
export DIST_AGENT_RPC_URL_POLYGON="https://polygon-rpc.com"
export DIST_AGENT_PRIVATE_KEY="your_private_key"

# AI configuration
export DIST_AGENT_AI_API_KEY="your_openai_key"
export DIST_AGENT_AI_MODEL="gpt-4"

# Database configuration
export DIST_AGENT_DB_URL="postgresql://user:pass@localhost/db"

# Logging
export DIST_AGENT_LOG_LEVEL="info"
export RUST_LOG="info"
```

### Configuration File

Create a `config.toml` file:

```toml
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

## 💡 Common Use Cases

### 1. Create a Smart Contract
```bash
dist_agent_lang run examples/smart_contract.dal
```

### 2. Deploy a Token
```bash
dist_agent_lang run examples/keys_token_implementation.dal
```

### 3. Create an AI Agent System
```bash
dist_agent_lang run examples/agent_system_demo.dal
```

### 4. Build a Web API
```bash
dist_agent_lang run examples/simple_web_api_example.dal
```

## 🐛 Troubleshooting

### Binary not found
```bash
# Check if binary is in PATH
which dist_agent_lang

# If not found, add installation directory to PATH
export PATH="$HOME/.local/bin:$PATH"  # For Linux/macOS
```

### Permission denied
```bash
# Make binary executable
chmod +x bin/dist_agent_lang

# Or install with sudo
sudo ./install.sh
```

### File not found
```bash
# Make sure you're in the correct directory
pwd

# Check if file exists
ls -la examples/hello_world_demo.dal
```

## 📖 Next Steps

1. **Read the Documentation** - See [dist_agent_lang/README.md](dist_agent_lang/README.md) for complete overview
2. **Explore Examples** - Run through the example files in `dist_agent_lang/examples/`
3. **Check CHANGELOG.md** - See what's new in [dist_agent_lang/CHANGELOG.md](dist_agent_lang/CHANGELOG.md)
4. **Visit Documentation** - See `dist_agent_lang/docs/` directory for detailed guides
5. **Contribute** - Check [dist_agent_lang/CONTRIBUTING.md](dist_agent_lang/CONTRIBUTING.md) and [dist_agent_lang/GOOD_FIRST_ISSUES.md](dist_agent_lang/GOOD_FIRST_ISSUES.md)

## 🆘 Getting Help

- **Documentation**: See [dist_agent_lang/README.md](dist_agent_lang/README.md) and [dist_agent_lang/CHANGELOG.md](dist_agent_lang/CHANGELOG.md)
- **Examples**: Check the `dist_agent_lang/examples/` directory
- **GitHub**: [Repository](https://github.com/okjason-source/dist_agent_lang) | [Issues](https://github.com/okjason-source/dist_agent_lang/issues) | [Discussions](https://github.com/okjason-source/dist_agent_lang/discussions)
- **Wiki**: [GitHub Wiki](https://github.com/okjason-source/dist_agent_lang/wiki)
- **Email**: jason.dinh.developer@gmail.com

## 🤝 Contributing

We welcome contributions! See:
- [CONTRIBUTING.md](dist_agent_lang/CONTRIBUTING.md) - Contribution guidelines
- [GOOD_FIRST_ISSUES.md](dist_agent_lang/GOOD_FIRST_ISSUES.md) - Beginner-friendly tasks
- [CONTRIBUTORS.md](dist_agent_lang/CONTRIBUTORS.md) - Contributor recognition

---

**Happy coding with dist_agent_lang! 🚀**
