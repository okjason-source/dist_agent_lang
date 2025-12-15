# dist_agent_lang Usage Guide

## 🚀 Quick Start

### Installation

```bash
# Download and extract the package
tar -xzf dist_agent_lang-1.0.0.tar.gz
cd dist_agent_lang-1.0.0

# Install the binary
./install.sh

# Verify installation
dist_agent_lang --version
```

### Basic Usage

```bash
# Run a dist_agent_lang file
dist_agent_lang run examples/hello_world_demo.dal

# Run your own file
dist_agent_lang run my_program.dal

# Show help
dist_agent_lang --help
```

## 📋 Available Commands

### `dist_agent_lang run <file.dal>`
Execute a dist_agent_lang program.

**Example:**
```bash
dist_agent_lang run examples/smart_contract.dal
```

### `dist_agent_lang test`
Run the test suite (if available).

**Example:**
```bash
dist_agent_lang test
```

### `dist_agent_lang web <file.dal>`
Run a web application.

**Example:**
```bash
dist_agent_lang web examples/simple_web_api_example.dal
```

### `dist_agent_lang --help` or `dist_agent_lang -h`
Display help information.

### `dist_agent_lang --version` or `dist_agent_lang -v`
Display version information.

## 📝 Creating Your First Program

1. **Create a file** `hello.dal`:

```rust
@trust("hybrid")
service HelloWorld {
    fn main() {
        print("Hello, dist_agent_lang!");
        log::info("main", "Program executed successfully!");
    }
}
```

2. **Run it:**
```bash
dist_agent_lang run hello.dal
```

## 🎯 Common Examples

### Run Example Programs

```bash
# Hello World
dist_agent_lang run examples/hello_world_demo.dal

# Smart Contract
dist_agent_lang run examples/smart_contract.dal

# AI Agent System
dist_agent_lang run examples/agent_system_demo.dal

# Web API
dist_agent_lang run examples/simple_web_api_example.dal

# Blockchain Operations
dist_agent_lang run examples/multi_chain_operations.dal
```

## 🔧 Configuration

### Environment Variables

Set these for full functionality:

```bash
# Blockchain
export DIST_AGENT_RPC_URL_ETHEREUM="https://mainnet.infura.io/v3/YOUR_KEY"
export DIST_AGENT_PRIVATE_KEY="your_private_key"

# AI
export DIST_AGENT_AI_API_KEY="your_openai_key"

# Database
export DIST_AGENT_DB_URL="postgresql://user:pass@localhost/db"

# Logging
export DIST_AGENT_LOG_LEVEL="info"
```

## 📚 Example Files Included

The package includes 27 example files:

- **Basic**: `hello_world_demo.dal`, `general_purpose_demo.dal`
- **AI Agents**: `agent_system_demo.dal`, `llm_integration_examples.dal`
- **Blockchain**: `smart_contract.dal`, `cross_chain_patterns.dal`
- **Web**: `simple_web_api_example.dal`, `backend_connectivity_patterns.dal`
- **Database**: `phase3_database_examples.dal`
- **Mobile/Desktop**: `phase5_mobile_examples.dal`, `phase5_desktop_examples.dal`

See the `examples/` directory for the complete list.

## 🐛 Troubleshooting

### Command not found
```bash
# Add to PATH
export PATH="$HOME/.local/bin:$PATH"

# Or use full path
/path/to/dist_agent_lang run file.dal
```

### Permission denied
```bash
chmod +x bin/dist_agent_lang
```

### File not found
```bash
# Check current directory
pwd

# Use full path
dist_agent_lang run /full/path/to/file.dal
```

## 📖 Next Steps

1. Explore the `examples/` directory
2. Read `README.md` for feature overview
3. Check `CHANGELOG.md` for version details
4. Visit the documentation for advanced features

---

**For more help, see INSTALLATION.md or README.md**

