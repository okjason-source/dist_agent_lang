# 🚀 Quick Start Guide - Get Started in 5 Minutes

> **📢 Beta Release v1.0.5:** dist_agent_lang is actively developed with consistent updates. This is a beta release — test thoroughly before production use. **Beta testing contributions appreciated!** 🙏

Welcome to **dist_agent_lang** (DAL) - the hybrid language for decentralized smart contracts and centralized services!

This guide will get you from zero to your first DAL program in **5 minutes**.

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
Download from [GitHub Releases](https://github.com/okjason-source/dist_agent_lang/releases) and extract the binary (`dal` or `dal.exe`) to your PATH.

### Verify Installation
```bash
dal --version
# Expected: dist_agent_lang v1.0.5
```

---

## 🎯 Your First DAL Program (3 minutes)

### Step 1: Create a New File
```bash
# Create your first DAL file
touch hello.dal
```

### Step 2: Write Your Program
Copy this into `hello.dal`:

```dal
// hello.dal - Your first DAL program!

fn main() {
    let greeting = "Hello, dist_agent_lang!";
    log::info("hello", greeting);
    
    let result = add_numbers(10, 20);
    log::info("result", { "sum": result });
}

fn add_numbers(a: int, b: int) -> int {
    return a + b;
}
```

### Step 3: Run It
```bash
dal run hello.dal

# Output:
# 🚀 Running dist_agent_lang file: hello.dal
# ✅ Tokenization successful! Generated 25 tokens
# ✅ Parsing successful! Generated 2 statements
# [INFO] "hello": "Hello, dist_agent_lang!"
# [INFO] "result": {"sum": 30}
# ✅ Execution successful!
```

---

## 🌐 Your First Service (2 minutes)

### Create a Service
Create `greeting.dal`:

```dal
// greeting.dal - A simple service example

@trust("hybrid")
service GreetingService {
    message: string = "Hello";
    
    fn initialize(msg: string) {
        self.message = msg;
    }
    
    fn greet(name: string) -> string {
        return self.message + ", " + name + "!";
    }
    
    fn get_message() -> string {
        return self.message;
    }
}

// Use the service
fn main() {
    let service = GreetingService::new();
    service.initialize("Welcome");
    
    let greeting = service.greet("Alice");
    log::info("greeting", greeting);
    // Output: "Welcome, Alice!"
}
```

### Run the Service
```bash
dal run greeting.dal
```

---

## 🧪 Test Your Code

### Run Tests
```bash
# Run all tests in current directory
dal test

# Run tests for a specific file
dal test hello.dal
```

### Create a Test File
Create `hello.test.dal`:

```dal
// hello.test.dal - Test file

fn test_add_numbers() {
    let result = add_numbers(5, 3);
    assert(result == 8, "5 + 3 should equal 8");
}

fn add_numbers(a: int, b: int) -> int {
    return a + b;
}
```

Run it:
```bash
dal test hello.test.dal
```

---

## 🔍 Code Quality Tools

### Check Syntax
```bash
dal check hello.dal
# Validates syntax and type checking
```

### Parse (Syntax Validation)
```bash
dal parse hello.dal
# Shows parse tree and validates syntax
```

### Format Code
```bash
dal fmt hello.dal
# Formats your DAL code consistently
```

### Lint Code
```bash
dal lint hello.dal
# Checks for common issues and best practices
```

---

## 📚 Next Steps

Congratulations! You've just:
- ✅ Installed dist_agent_lang
- ✅ Written your first DAL program
- ✅ Created a service
- ✅ Run tests
- ✅ Used code quality tools

### Continue Learning

1. **[Syntax Reference](../syntax.md)** - Learn DAL syntax in detail
2. **[Attributes Reference](../attributes.md)** - Understand `@trust`, `@secure`, `@chain`, etc.
3. **[Standard Library](../STDLIB_REFERENCE.md)** - Explore 22 stdlib modules
4. **[CLI Reference](../CLI_QUICK_REFERENCE.md)** - Complete command reference
5. **[Examples](../../examples/)** - See real-world code examples

### Try More Examples

```bash
# View available examples
ls examples/

# Run example programs
dal run examples/hello_world_demo.dal
dal run examples/smart_contract.dal
dal run examples/ai_agent_examples.dal
```

---

## 🆘 Troubleshooting

### Build Errors
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

### Syntax Errors
```bash
# Check syntax
dal check hello.dal

# Parse with verbose output
dal parse hello.dal
```

### Runtime Errors
```bash
# Run with verbose logging
RUST_LOG=debug dal run hello.dal
```

---

## 💡 Quick Tips

### 1. Use the REPL
```bash
dal repl
# Interactive shell for testing code snippets
```

### 2. Watch Mode
```bash
dal watch hello.dal
# Auto-runs on file save
```

### 3. Create New Projects
```bash
dal new my-project
# Creates a new project structure

dal new my-ai --type ai
# Creates an AI-focused project
```

### 4. Format Before Committing
```bash
dal fmt hello.dal
# Ensures consistent code style
```

---

## 🎓 Key Concepts (30-second overview)

### Services
Services are the main building blocks in DAL:

```dal
@trust("hybrid")
@chain("ethereum")
service MyService {
    count: int = 0;
    
    fn increment() {
        self.count = self.count + 1;
    }
}
```

### Attributes
Use attributes to control behavior:
- `@trust("hybrid")` - Hybrid trust model
- `@trust("decentralized")` - Fully decentralized
- `@trust("centralized")` - Centralized trust
- `@chain("ethereum")` - Target blockchain
- `@secure` - Enable security features
- `@txn` - Transaction context
- `@limit(n)` - Resource limits

### Standard Library Namespaces
DAL provides 22 standard library modules:

```dal
// Logging
log::info("category", "message");
log::error("category", "error");

// Authentication
auth::create_user("alice", "password", "email");
auth::authenticate("alice", "password");

// Blockchain
chain::deploy(1, "Contract", []);
chain::call(1, "0x...", "function", []);

// Crypto
crypto::hash("data", "SHA256");
crypto::sign("data", "private_key", SignatureAlgorithm::ECDSA);

// And many more: oracle::, service::, database::, web::, ai::, etc.
```

### Multi-Chain Support
```dal
@trust("hybrid")
@chain("ethereum", "polygon", "arbitrum")
service MultiChainService {
    // Automatically works across all specified chains!
}
```

### Built-in Security
```dal
@trust("hybrid")
@secure
service SecureService {
    // Automatic:
    // - Reentrancy protection
    // - Safe math (overflow protection)
    // - Audit logging
    // - Capability checks
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
| **AI Integration** | ✅ Native | ❌ | ⚠️ External |
| **Hybrid Trust** | ✅ Native | ❌ | ❌ |
| **Learning Curve** | 🟢 Moderate | 🟢 Easy | 🔴 Steep |
| **Agent Support** | ✅ First-class | ❌ | ⚠️ External |

---

## 🤝 Get Help

- 📖 **Documentation**: [Full Docs](../Documentation.md)
- 📖 **Public Index**: [Public Documentation Index](../PUBLIC_DOCUMENTATION_INDEX.md)
- 💬 **Discord**: [Join Community](https://discord.gg/tu7tg9eN)
- 🐛 **Issues**: [GitHub Issues](https://github.com/okjason-source/dist_agent_lang/issues)
- 📧 **Email**: jason.dinh.developer@gmail.com

---

## 📝 Example Projects

Check out complete examples in the `examples/` directory:

```bash
cd examples/

# Core language features
dal run hello_world_demo.dal
dal run enhanced_language_features.dal

# Services and smart contracts
dal run smart_contract.dal
dal run defi_nft_rwa_contract.dal

# AI and agents
dal run ai_agent_examples.dal
dal run agent_system_demo.dal

# Multi-chain
dal run multi_chain_operations.dal
dal run cross_chain_patterns.dal

# Web and APIs
dal run simple_web_api_example.dal
dal run web_framework_examples.dal
```

---

**🎉 Welcome to the DAL community! Let's build the future of decentralized applications together.**

**Next:** [Syntax Reference →](../syntax.md) | [CLI Reference →](../CLI_QUICK_REFERENCE.md) | [Standard Library →](../STDLIB_REFERENCE.md)
