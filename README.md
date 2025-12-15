# Installation and Usage Guide

## ⚠️ Production Readiness Disclaimer

**Current Status: Beta Release (v1.0.0)**

dist_agent_lang is currently in **beta/early release** stage. While the language includes comprehensive security features and has passed dependency audits, it has not yet undergone extensive real-world production testing.

### ✅ Safe For:
- **Development & Prototyping** - Building and testing applications
- **Learning & Experimentation** - Educational purposes
- **Non-Critical Applications** - Applications not handling significant value
- **Testing & Validation** - Validating concepts and workflows

### ⚠️ Use Caution For:
- **Production Financial Applications** - Applications handling real money
- **High-Value Smart Contracts** - Contracts managing significant assets
- **Critical Infrastructure** - Systems requiring high reliability
- **Applications Handling Sensitive Data** - Without additional security audits


## 📦 Installation

### Prerequisites
- Linux, macOS, or Windows
- No additional dependencies required (binary is self-contained)

### Quick Install

#### Linux/macOS
```bash
# Download the release package
wget https://github.com/distagentlang/dist_agent_lang/releases/download/v1.0.0/dist_agent_lang-1.0.0.tar.gz

# Extract the package
tar -xzf dist_agent_lang-1.0.0.tar.gz
cd dist_agent_lang-1.0.0

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
# Should output: dist_agent_lang 1.0.0
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
        
        log::info("main", "Hello World program executed successfully!");
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

The package includes 27 example files in the `examples/` directory:

### Basic Examples
- `hello_world_demo.dal` - Simple hello world program
- `general_purpose_demo.dal` - General programming examples
- `simple_chain_examples.dal` - Basic blockchain operations

### AI Agent Examples
- `agent_system_demo.dal` - Multi-agent coordination
- `integrated_spawn_ai_examples.dal` - Spawn and AI integration
- `llm_integration_examples.dal` - LLM integration patterns
- `phase4_ai_agent_examples.dal` - Advanced AI agent features

### Blockchain Examples
- `smart_contract.dal` - Smart contract development
- `cross_chain_patterns.dal` - Cross-chain operations
- `multi_chain_operations.dal` - Multi-chain management
- `chain_selection_example.dal` - Chain selection patterns
- `keys_token_implementation.dal` - Token implementation

### Web & Backend Examples
- `simple_web_api_example.dal` - Web API creation
- `backend_connectivity_patterns.dal` - Database and API patterns
- `real_time_backend_example.dal` - Real-time backend
- `phase2_web_framework_examples.dal` - Web framework features

### Database Examples
- `phase3_database_examples.dal` - Database operations

### Desktop & Mobile Examples
- `phase5_desktop_examples.dal` - Desktop application development
- `phase5_mobile_examples.dal` - Mobile application development

### Edge & IoT Examples
- `phase6_edge_examples.dal` - Edge computing examples

### Advanced Examples
- `dynamic_nft_examples.dal` - Dynamic NFT creation
- `dynamic_rwa_examples.dal` - Real World Asset tokenization
- `enhanced_language_features.dal` - Advanced language features
- `secure_configuration_example.dal` - Security configuration
- `oracle_quick_start.dal` - Oracle integration
- `oracle_development_setup.dal` - Oracle development

### Running Examples
```bash
# Navigate to the examples directory
cd examples

# Run any example
dist_agent_lang run hello_world_demo.dal
dist_agent_lang run agent_system_demo.dal
dist_agent_lang run smart_contract.dal
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

1. **Read the README.md** - Overview of features and capabilities
2. **Explore Examples** - Run through the example files
3. **Check CHANGELOG.md** - See what's new in version 1.0.0
4. **Visit Documentation** - See docs/ directory for detailed guides

## 🆘 Getting Help

- **Documentation**: See README.md and CHANGELOG.md in the package
- **Examples**: Check the examples/ directory
- **Email**: jason.dinh.developer@gmail.com

---

**Happy coding with dist_agent_lang! 🚀**
