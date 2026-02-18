# Installation and Usage Guide

## ⚠️ Production Readiness Notice

**dist_agent_lang v1.0.5 is currently in beta/early release.** While it includes comprehensive security features, it has not yet undergone extensive real-world production testing. 

**Recommended for**: Development, prototyping, learning, and non-critical applications.  
**Use caution for**: Production financial applications, high-value smart contracts, and critical infrastructure.

See [SECURITY_DISCLAIMER.md](SECURITY_DISCLAIMER.md) for detailed information.

## 📦 Installation

### Prerequisites
- **crates.io / binary install:** Linux, macOS, or Windows; [Rust](https://rustup.rs) required for `cargo install`.
- **Release binary:** No Rust needed; download pre-built binary for your OS.
- **Build from source:** Rust (e.g. [rustup](https://rustup.rs)), and optionally Node.js 18+ and system build tools; the install script can install these.

---

### Option A: Install from crates.io (recommended)

If you have [Rust](https://rustup.rs) installed:

```bash
cargo install dist_agent_lang
```

This downloads the package from [crates.io](https://crates.io/crates/dist_agent_lang), builds it, and installs the `dist_agent_lang` binary to `~/.cargo/bin/` (which is in PATH if you use rustup).

**Use as a Rust library** in your project:
```toml
[dependencies]
dist_agent_lang = "1.0.5"
```

Or: `cargo add dist_agent_lang`

---

### Option B: Install from release binary

**Latest version:** Go to [GitHub Releases → Latest](https://github.com/okjason-source/dist_agent_lang/releases/latest) and download the archive for your OS. Or use a specific version from the [Releases](https://github.com/okjason-source/dist_agent_lang/releases) list.

#### Linux (x64)
```bash
# Option 1: Install latest (uses GitHub API to get current release)
V=$(curl -sL https://api.github.com/repos/okjason-source/dist_agent_lang/releases/latest | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
wget "https://github.com/okjason-source/dist_agent_lang/releases/download/v${V}/dist_agent_lang-v${V}-linux-x64.tar.gz"
tar -xzf "dist_agent_lang-v${V}-linux-x64.tar.gz"
sudo cp target/x86_64-unknown-linux-gnu/release/dist_agent_lang /usr/local/bin/
```

```bash
# Option 2: Install a specific version (replace v1.0.5 with the release you want)
wget https://github.com/okjason-source/dist_agent_lang/releases/download/v1.0.5/dist_agent_lang-v1.0.5-linux-x64.tar.gz
tar -xzf dist_agent_lang-v1.0.5-linux-x64.tar.gz
sudo cp target/x86_64-unknown-linux-gnu/release/dist_agent_lang /usr/local/bin/
# OR user install (no sudo):
mkdir -p ~/.local/bin
cp target/x86_64-unknown-linux-gnu/release/dist_agent_lang ~/.local/bin/
chmod +x ~/.local/bin/dist_agent_lang
export PATH="$HOME/.local/bin:$PATH"   # add to ~/.bashrc or ~/.zshrc
```

#### macOS (x64 or ARM64)
```bash
# Download the right one: ...-macos-x64.tar.gz or ...-macos-arm64.tar.gz
# Extract, then copy the binary from the path inside the tarball to /usr/local/bin or ~/.local/bin
# Example (ARM64):
tar -xzf dist_agent_lang-v1.0.5-macos-arm64.tar.gz
sudo cp target/aarch64-apple-darwin/release/dist_agent_lang /usr/local/bin/
```

#### Windows
```bash
# Download dist_agent_lang-v1.0.5-windows-x64.zip from Releases
# Extract, then copy dist_agent_lang.exe to a folder in your PATH (e.g. C:\Program Files\dist_agent_lang\)
# Add that folder to the system PATH environment variable.
```

---

### Option C: Build from source (developers)

Use this if you clone the repo or need to build from source.

1. **Clone the repository**
   ```bash
   git clone https://github.com/okjason-source/dist_agent_lang.git
   cd dist_agent_lang
   ```

2. **Run the install script from the repository root** (the directory that contains `Cargo.toml`):
   ```bash
   ./scripts/install.sh
   ```
   The script will install Rust (if needed), system dependencies, build the project with `cargo build --release`, run tests, copy the binary to `/usr/local/bin` or `~/.local/bin`, create a default config under `~/.config/dist_agent_lang`, and copy examples to `~/dist_agent_lang_examples`.

3. **If you prefer to build manually:**
   ```bash
   cargo build --release
   sudo cp target/release/dist_agent_lang /usr/local/bin/
   # OR: cp target/release/dist_agent_lang ~/.local/bin/  and add ~/.local/bin to PATH
   ```

---

### Verify installation
```bash
dist_agent_lang --version
# Should output: dist_agent_lang v1.0.5 (or the version you installed)
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
# If you built from source, examples are in the repo and in ~/dist_agent_lang_examples
cd examples   # or: cd ~/dist_agent_lang_examples

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
dist_agent_lang run examples/defi_nft_rwa_contract.dal
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
# Make binary executable (if you have the binary locally)
chmod +x dist_agent_lang

# For build-from-source, install with sudo if needed
sudo ./scripts/install.sh
```

### install.sh not found or "Cargo.toml not found"
```bash
# The install script must be run from the repository root (the directory that contains Cargo.toml)
cd /path/to/dist_agent_lang
./scripts/install.sh
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
3. **Check CHANGELOG.md** - See what's new in the current version
4. **Visit Documentation** - See docs/ directory for detailed guides

## 🆘 Getting Help

- **Documentation**: See README.md and CHANGELOG.md in the package
- **Examples**: Check the examples/ directory
- **Issues**: Report at https://github.com/okjason-source/dist_agent_lang/issues
- **Email**: team@distagentlang.com

---

**Happy coding with dist_agent_lang! 🚀**

