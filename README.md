# 🌿 dist_agent_lang

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Version](https://img.shields.io/badge/version-1.0.5-blue.svg)](https://github.com/okjason-source/dist_agent_lang)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg)](https://github.com/okjason-source/dist_agent_lang)

**A hybrid compiled programming language built with simplicity for a sophisticated future.**

> 🌱 **v1.0.5 Beta Release** — Actively maintained with consistent updates. Test thoroughly before production. **Beta testing contributions appreciated!** 🙏

---

## 🚀 Quick Start

### Installation

**From Source (Become a Contributor):**
```bash
git clone https://github.com/okjason-source/dist_agent_lang.git
cd dist_agent_lang
cargo build --release
./target/release/dal --version
```

**From Binary:**
Download from [GitHub Releases](https://github.com/okjason-source/dist_agent_lang/releases/latest)

**From Cargo:**
```bash
cargo install --git https://github.com/okjason-source/dist_agent_lang.git --bin dal
```

### Your First Program

Create `hello.dal`:
```dal
@trust("hybrid")
service HelloWorld {
    fn main() {
        print("Hello, dist_agent_lang!");
    }
}
```

Run it:
```bash
dal run hello.dal
# Output: Hello, dist_agent_lang!
```

---

## 🌳 Features

### 🤖 AI Agent Framework
Native support for AI agents with multi-agent coordination, workflow management, and task execution.

### ⛓️ Multi-Chain Support
Deploy to Ethereum, Polygon, Solana, Arbitrum, and more with a single codebase.

### 🔒 Built-in Security
Automatic reentrancy protection, safe math, and comprehensive security features.

### 🌐 Hybrid Trust
Seamlessly combine onchain and offchain systems with hybrid trust models.

### 📚 Rich Standard Library
22 modules covering blockchain, AI, database, web, and more.

### 🔄 Solidity Converter
Automatically convert Solidity contracts to DAL format.

### ⚡ High Performance
Compiled to native code with optimized execution for speed and efficiency.

### 🛠️ Developer Tools
Comprehensive tooling including debugger, formatter, and package manager.

### ⌨️ CLI Commands
Powerful command-line interface with commands for running, testing, formatting, and managing your DAL projects.

### 🧬 Agent Mold System
Reusable configurations linked to agents for rapid development and deployment.

---

## 📖 Documentation

### Getting Started
- **[Quick Start Guide](docs/guides/QUICK_START.md)** - Get up and running in 5 minutes
- **[Complete Documentation Index](docs/PUBLIC_DOCUMENTATION_INDEX.md)** - Comprehensive guide optimized for developers and AI assistants

### Guides
- **[AI Features Guide](docs/guides/AI_FEATURES_GUIDE.md)** - Complete AI capabilities overview
- **[AI Best Practices](docs/guides/AI_BEST_PRACTICES.md)** - Security and optimization for AI
- **[CloudAdmin Guide](docs/guides/CLOUDADMIN_GUIDE.md)** - Hybrid trust and admin control
- **[API Reference](docs/guides/API_REFERENCE.md)** - Complete standard library documentation

### Reference
- **[Language Syntax](docs/syntax.md)** - DAL syntax reference
- **[Standard Library](docs/STDLIB_REFERENCE.md)** - Machine-readable API reference
- **[Attributes Reference](docs/attributes.md)** - Service and function attributes

---

## 🎮 CLI Commands

```bash
# Run a DAL program
dal run program.dal

# Test your code
dal test

# Check syntax
dal check program.dal

# Format code
dal fmt program.dal

# Lint code
dal lint program.dal

# Parse and show AST
dal parse program.dal

# Start REPL
dal repl

# Watch for changes
dal watch program.dal

# Create new project
dal new my_project
```

---

## 🧬 Agent Molds

Create reusable agent templates:

```bash
# List available molds
dal mold list

# Load a mold
dal mold load verify_mold

# Spawn agent from mold
dal mold spawn verify_mold MyAgent

# Get mold info (requires web3)
dal mold info mold_id
```

---

## 💡 Example: AI-Powered Trading Agent

```dal
@ai
@chain("ethereum")
service TradingAgent {
    fn analyze_market(data: string) -> string {
        // Analyze market sentiment
        let analysis = ai::analyze_text(data);
        
        if analysis.sentiment > 0.7 {
            return "bullish";
        } else if analysis.sentiment < 0.3 {
            return "bearish";
        }
        return "neutral";
    }
    
    fn execute_trade(signal: string, amount: int) {
        if signal == "bullish" {
            // Execute buy order
            chain::call(1, dex_address, "swap", json::stringify({
                "token_in": usdc_address,
                "token_out": weth_address,
                "amount": amount
            }));
        }
    }
}
```

---

## 🔒 Security Features

- ✅ Reentrancy protection
- ✅ Safe math (overflow/underflow protection)
- ✅ State isolation
- ✅ Cross-chain security
- ✅ Oracle security (signed feeds, multi-source validation)
- ✅ Transaction atomicity (ACID guarantees)
- ✅ Enhanced security logging
- ✅ 140+ tests passing

---

## ⚠️ Beta Release Notice

**Current Version: v1.0.5 (Beta Release) — Actively Developed**

dist_agent_lang is an **actively maintained beta release** with consistent updates and improvements.

### ✅ Safe For:
- Development & Prototyping
- Learning & Experimentation
- Non-Critical Applications
- Testing & Validation
- Beta Testing

### ⚠️ Use With Caution For:
- Production Financial Applications (wait for v1.1.0+)
- High-Value Smart Contracts (third-party audit recommended)
- Critical Infrastructure (additional validation needed)
- Sensitive Data Applications (additional security audits recommended)

**Target for Production (v1.1.0+):** ~community validation requested

---

## 🤝 Contributing

We welcome contributions! Every contribution helps us reach production readiness.

### Quick Start Contributing

**No coding required?** You can still help!
- 🧪 **Test the language** - Run examples, report bugs
- 📝 **Improve documentation** - Fix typos, clarify instructions
- 💡 **Share feedback** - Tell us what works and what doesn't

**Ready to code?**
- 🟢 **Beginners**: Check out [GOOD_FIRST_ISSUES.md](GOOD_FIRST_ISSUES.md)
- 📖 **Full Guide**: See [CONTRIBUTING.md](CONTRIBUTING.md)

---

## 📦 Installation Methods

### From Source
```bash
git clone https://github.com/okjason-source/dist_agent_lang.git
cd dist_agent_lang
cargo build --release
```

### From Binary
Download from [GitHub Releases](https://github.com/okjason-source/dist_agent_lang/releases/latest)

### From Cargo
```bash
cargo install --git https://github.com/okjason-source/dist_agent_lang.git --bin dal
```

**Requirements:** Rust 1.70+ ([Install Rust](https://rustup.rs/))

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test --test example_tests
```

**140+ tests passing** covering all standard library modules.

---

## 📚 Learn More

- **[Documentation](docs/PUBLIC_DOCUMENTATION_INDEX.md)** - Complete documentation index
- **[Examples](examples/)** - 27+ example programs
- **[GitHub](https://github.com/okjason-source/dist_agent_lang)** - Source code and issues
- **[Discord](https://discord.gg/tu7tg9eN)** - Join our community

---

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

---

**Made with 💚 by OK Jason**

**🌿 Growing the future of decentralized development**
