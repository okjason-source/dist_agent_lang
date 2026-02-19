# DAL (Distributed Agent Language) - Public Documentation Index

**Version:** 1.0.5  
**Last Updated:** 2026-02-18  
**Optimized for:** Human developers and AI/LLM assistants

---

## 📋 Quick Navigation

| Section | Description | Target Audience |
|---------|-------------|-----------------|
| [Getting Started](#getting-started) | Installation, first program | New users, AI assistants |
| [Language Reference](#language-reference) | Syntax, types, operators | All developers, LLMs |
| [Standard Library](#standard-library-stdlib) | All stdlib modules | Developers, AI code generation |
| [Package Management](#package-management) | Installing, dependencies | Developers, DevOps |
| [CLI Reference](#cli-reference) | Command-line tools | Developers, automation |
| [Examples](#examples) | Code samples | Learning, AI training data |
| [API Reference](#api-reference) | Programmatic access | Integrations, tools |

---

## 🚀 Getting Started

### Installation

**Quick Install - Github Releases - (Recommended):**

### Your First DAL Program

**hello.dal:**
```dal
function main() {
    log::info("Hello, DAL!");
}
```

**Run it:**
```bash
dal run hello.dal
```

**Output:**
```
🚀 Running dist_agent_lang file: hello.dal
✅ Tokenization successful! Generated 10 tokens
✅ Parsing successful! Generated 1 statements
[INFO] "Hello, DAL!"
✅ Execution successful!
```

---

## 📚 Language Reference

### Core Concepts

**DAL is:**
- **Multi-paradigm:** Services, functions, imperative
- **Strongly-typed:** Type safety at runtime
- **Blockchain-native:** Built-in chain, crypto, auth
- **AI-first:** Native agent and AI support
- **Web-friendly:** HTTP, templates, middleware

### File: [docs/syntax.md](./syntax.md)

---

## 📦 Package Management

**➡️ [Packaging & Distribution Guide (PACKAGING.md)](./PACKAGING.md)**

Learn about:
- Installation methods (curl, direct download, package managers)
- Edition comparison (Community, Professional, Enterprise)
- Updates and versioning
- System requirements
- Configuration
- Troubleshooting

---

## 📦 Standard Library (stdlib)

### Overview

DAL's standard library provides built-in functionality across multiple domains:

| Module | Purpose | Key Functions |
|--------|---------|---------------|
| **chain** | Blockchain operations | `deploy()`, `call()`, `get_balance()` |
| **crypto** | Cryptography | `hash()`, `sign()`, `verify()`, `encrypt()` |
| **auth** | Authentication | `create_user()`, `login()`, `validate_token()` |
| **db** | Database operations | `query()`, `connect()`, `migrate()` |
| **ai** | AI/ML operations | `generate_text()`, `classify()`, `embed()` |
| **agent** | Agent orchestration | `create()`, `coordinate()`, `communicate()` |
| **iot** | IoT device management | `connect_device()`, `read_sensor()` |
| **oracle** | Oracle data feeds | `fetch()`, `stream()`, `verify()` |
| **web** | HTTP operations | `get_request()`, `post_request()` |
| **log** | Logging | `info()`, `warn()`, `error()` |
| **config** | Configuration | `get_env()`, `get_database_config()` |
| **cloudadmin** | Cloud management | `authorize()`, `grant()`, `audit_log()` |
| **trust** | Trust & permissions | `validate_hybrid_trust()`, `authorize()` |
| **aml** | Anti-money laundering | `perform_check()`, `get_status()` |
| **kyc** | Know Your Customer | `verify()`, `get_verification()` |
| **mold** | Agent molds & templates | `load()`, `spawn_from()`, `list()`, `get_info()`, `use_mold()` |

### Detailed References

**📘 [Complete Standard Library Reference (STDLIB_REFERENCE.md)](./STDLIB_REFERENCE.md)**

This comprehensive, machine-readable reference includes:
- All stdlib modules with complete function signatures
- Parameter types and return types
- Usage examples for every function
- Error handling patterns
- Best practices for AI code generation

**Individual Module Guides:**
- [Chain Module](./guides/API_REFERENCE.md#chain-module)
- [Crypto Module](./guides/API_REFERENCE.md#crypto-module)
- [Auth Module](./guides/API_REFERENCE.md#auth-module)
- [Agent Module](./guides/API_REFERENCE.md#agent-module)
- [AI Module](./guides/AI_FEATURES_GUIDE.md)
- [CloudAdmin Module](./guides/CLOUDADMIN_GUIDE.md)
- [Mold Module](./STDLIB_REFERENCE.md#mold-module)

---

## 🎯 Package Management

### Binary Distribution

**Current Model:**
- DAL is distributed as compiled binaries
- Includes full standard library
- No source code access (proprietary)

### Installing DAL

**System Requirements:**
- Linux: x86_64, kernel 3.2+
- macOS: 10.15+ (Catalina or later)
- Windows: Windows 10+ (64-bit)

**Install Locations:**
```
Linux/macOS:
/usr/local/bin/dal              # Binary executable
/usr/local/lib/dal/             # Runtime libraries
/usr/local/share/dal/docs/      # Documentation

Windows:
C:\Program Files\DAL\dal.exe
C:\Program Files\DAL\lib\
C:\Program Files\DAL\docs\
```

### Updating DAL

```bash
# Check for updates
dal version --check

# Update to latest version
dal upgrade

# Update to specific version
dal upgrade --version 1.0.5
```

---

## 🔧 CLI Reference

### Core Commands

| Command | Description | Example |
|---------|-------------|---------|
| `run <file>` | Execute DAL file | `dal run app.dal` |
| `parse <file>` | Parse and validate | `dal parse app.dal` |
| `test <file>` | Run tests | `dal test app.test.dal` |
| `web <file>` | Run web app | `dal web server.dal` |
| `convert <file>` | Solidity to DAL | `dal convert Token.sol` |
| `analyze <file>` | Analyze Solidity | `dal analyze Token.sol` |
| `help` | Show help | `dal help` |
| `version` | Show version | `dal version` |

### Planned Commands (Future)

See [CLI Expansion Plan](./development/stdlib_implementation_plans/09_CLI_EXPANSION_PLAN.md) for upcoming features:
- `dal fmt` - Code formatting
- `dal lint` - Code linting
- `dal new` - Project scaffolding
- `dal agent create` - Agent creation
- `dal cloud` - Cloud management
- And many more...

---

## 🤖 Templates & Marketplace

### Agent Templates

Reusable configurations for AI agents that can be shared, sold, and instantiated at scale.

**Example:**
```bash
# Install template from marketplace
dal agent template install username/fraud_detector

# Create agent from template
dal agent create --template username/fraud_detector my_detector

# Create fleet from template
dal agent fleet create --template username/fraud_detector fraud_team --agents 50
```

**See:** [Agent Template Marketplace Guide](../dist_agent_lang/docs/development/AGENT_TEMPLATE_MARKETPLACE.md)

---

## 💡 Examples

### Complete Examples

1. **[DeFi Token](./tutorials/01_defi_token.md)**
   - ERC20-compatible token
   - Minting, burning, transfers
   - Blockchain integration

2. **[AI Trading Agent](./tutorials/02_ai_trading_agent.md)**
   - AI-powered trading
   - Market analysis
   - Risk management

3. **[Hybrid Marketplace](./tutorials/03_hybrid_marketplace_cloudadmin.md)**
   - Centralized + decentralized
   - Cloud admin integration
   - Payment processing

### Code Snippets Library

**Blockchain Operations:**
```dal
@chain("ethereum")
service Token {
    function deploy() {
        let result = chain::deploy("MyToken", "{}");
        log::info("Deployed: " + result.address);
    }
}
```

**AI Agent Creation:**
```dal
import stdlib::agent;

function create_fraud_detector() {
    let config = {
        "name": "FraudDetector",
        "type": "ai",
        "role": "Detect fraudulent transactions"
    };
    
    let agent_ctx = agent::spawn(config);
    log::info("Agent created: " + agent_ctx.agent_id);
}
```

**Database Query:**
```dal
import stdlib::db;

function get_users() {
    let conn = db::connect("postgresql://localhost/mydb");
    let result = db::query(conn, "SELECT * FROM users");
    return result;
}
```

**Cloud Admin:**
```dal
import stdlib::cloudadmin;

function check_access() {
    let authorized = cloudadmin::authorize("user_123", "write", "resource_456");
    if !authorized {
        log::error("Access denied");
    }
}
```

---

## 🔌 API Reference

### For AI/LLM Assistants

**When generating DAL code, reference:**

1. **Syntax:** [docs/syntax.md](./syntax.md)
2. **Attributes:** [docs/attributes.md](./attributes.md)
3. **Stdlib API:** [docs/guides/API_REFERENCE.md](./guides/API_REFERENCE.md)
4. **Best Practices:** [docs/guides/BEST_PRACTICES.md](./guides/BEST_PRACTICES.md)
5. **Examples:** [docs/examples/](../examples/)

### Common Patterns

**Service Definition:**
```dal
@trust("hybrid")
@chain("ethereum")
service MyService {
    var state: String = "initial";
    
    function initialize() {
        state = "ready";
    }
    
    function process(data: String) -> String {
        return "Processed: " + data;
    }
}
```

**Error Handling:**
```dal
function safe_operation() {
    try {
        let result = risky_function();
        log::info("Success");
    } catch error {
        log::error("Failed: " + error);
    } finally {
        cleanup();
    }
}
```

**Agent Communication:**
```dal
function coordinate_agents() {
    let agent1 = agent::create("ai", "Analyzer");
    let agent2 = agent::create("worker", "Processor");
    
    agent::communicate(agent1.agent_id, agent2.agent_id, {
        "message_type": "task",
        "content": "Process batch 42"
    });
}
```

---

## 📖 Comprehensive Documentation Links

### For Humans

- [Quick Start Guide](./guides/QUICK_START.md)
- [Installation Guide](./getting_started/INSTALLATION.md)
- [Usage Guide](./USAGE_GUIDE.md)
- [Best Practices](./guides/BEST_PRACTICES.md)
- [Deployment Guide](./guides/DEPLOYMENT_GUIDE.md)

### For AI/LLM Tools

**When assisting with DAL development:**

1. **Language Syntax:** Parse [syntax.md](./syntax.md) for grammar rules
2. **Type System:** Reference [attributes.md](./attributes.md) for types
3. **Stdlib Functions:** Index [API_REFERENCE.md](./guides/API_REFERENCE.md)
4. **Code Examples:** Learn from [examples/](../examples/) directory
5. **Common Patterns:** Extract from [tutorials/](./tutorials/)

**Optimization Hints:**
- DAL is blockchain-native: Prefer `chain::` functions over manual HTTP
- Services are stateful: Use `var` for mutable state
- Agent orchestration: Use `agent::` module for multi-agent systems
- Trust models: Always specify `@trust` attribute for services

---

## 🛠 Development Resources

### GitHub Repositories

- **Main Repository:** (Private - contact for access)
- **Examples:** https://github.com/dist_agent_lang/examples
- **Templates:** https://github.com/dist_agent_lang/templates
- **Documentation:** https://github.com/dist_agent_lang/docs

### Community

- **Discord:** https://discord.gg/dist-agent-lang
- **Forum:** https://forum.dist_agent_lang.org
- **Stack Overflow:** Tag `dist-agent-lang` or `dal`
- **Twitter:** @dal_lang

### Commercial Support

- **Enterprise Support:** support@dist_agent_lang.org
- **Training:** training@dist_agent_lang.org
- **Consulting:** consulting@dist_agent_lang.org

---

## 🔍 AI/LLM Integration Guide

### For Code Assistants (GitHub Copilot, Cursor, etc.)

**DAL File Extensions:**
- `.dal` - Standard DAL source files
- `.test.dal` - Test files

**Syntax Highlighting:**
- Use Rust syntax as base (similar structure)
- Add DAL-specific keywords: `service`, `@chain`, `@trust`, `@ai`

**Code Generation Tips:**

1. **Always import stdlib modules:**
   ```dal
   import stdlib::chain;
   import stdlib::agent;
   ```

2. **Use attributes for services:**
   ```dal
   @trust("hybrid")
   @chain("ethereum")
   service MyService { }
   ```

3. **Leverage built-in functions:**
   - Don't reinvent crypto: use `crypto::hash()`
   - Don't write HTTP clients: use `web::get_request()`
   - Don't implement auth: use `auth::create_user()`

4. **Agent orchestration patterns:**
   ```dal
   // Create → Configure → Communicate → Coordinate
   let agent = agent::create("ai", "name");
   let task = agent::create_agent_task("task_1", "description", "high");
   agent::coordinate(agent.agent_id, task, "task_distribution");
   ```

### For Documentation Tools

**Indexed Sections:**
- Language: [syntax.md](./syntax.md)
- Standard Library: [API_REFERENCE.md](./guides/API_REFERENCE.md)
- Tutorials: [tutorials/](./tutorials/)
- Examples: [examples/](../examples/)
- CLI: [CLI_EXPANSION_PLAN.md](./development/stdlib_implementation_plans/09_CLI_EXPANSION_PLAN.md)

**Machine-Readable Format:**
- All stdlib functions documented with signatures
- Type information included
- Return types specified
- Error conditions listed

---

## 📊 Version History

| Version | Date | Highlights |
|---------|------|------------|
| **1.0.5** | 2026-02-08 | Mutation testing hardening, HTTP middleware tests, version bump |
| **1.0.3** | 2026-02-06 | mold:: stdlib, agent molds, CLI mold commands |
| **1.0.2** | 2026-01-15 | CloudAdmin module, improved stdlib |
| **1.0.1** | 2025-12-01 | Bug fixes, performance improvements |
| **1.0.0** | 2025-11-01 | Initial release |

---

## 🚦 Status & Roadmap

### Current Status

✅ **Production Ready:**
- Core runtime
- Basic stdlib (chain, crypto, auth, db)
- CLI tools (run, parse, test, web)
- Solidity conversion

🚧 **In Development:**
- Agent development contracts
- Advanced CLI commands
- Plugin system
- LSP support

📋 **Planned:**
- Cloud-native deployment
- Advanced AI integrations
- Enterprise features
- Mobile runtime

### Release Schedule

- **1.1.0** (Q2 2026) - Agent marketplace, CLI expansion
- **1.2.0** (Q3 2026) - Plugin system, LSP
- **2.0.0** (Q4 2026) - Major stdlib expansion, enterprise features

---

## ⚖️ License & Usage

### Language License

**DAL Runtime:** Proprietary
- Binary distribution only
- No source code access
- Commercial use allowed with license

**Documentation:** CC BY 4.0
- Free to read and share
- Attribution required
- Can be used for AI training

### Usage Terms

**You can:**
- ✅ Use DAL for commercial projects
- ✅ Deploy DAL applications to production
- ✅ Generate revenue from DAL-powered services
- ✅ Use documentation for learning
- ✅ Train AI models on public documentation

**You cannot:**
- ❌ Redistribute DAL binaries without license
- ❌ Reverse engineer the runtime
- ❌ Remove attribution from documentation
- ❌ Create competing languages from DAL

### Mold System

Agent configurations in the marketplace can have individual licenses (MIT, Apache, Commercial, etc.)built purposefully on chain.

---

## 📞 Support & Contact

### Free Support

- **Documentation:** https://docs.dist_agent_lang.org
- **Community Forum:** https://forum.dist_agent_lang.org
- **GitHub Issues:** https://github.com/dist_agent_lang/issues
- **Stack Overflow:** Tag `dal` or `dist-agent-lang`

### Paid Support

- **Professional:** $99/month - Email support, 48h response
- **Enterprise:** $999/month - Priority support, SLA, dedicated engineer
- **Contact:** support@dist_agent_lang.org

---

## 🎓 Training & Certification

### Self-Paced Learning

- **Free Courses:** https://learn.dist_agent_lang.org
- **Video Tutorials:** https://youtube.com/dist-agent-lang
- **Interactive Playground:** https://play.dist_agent_lang.org

### Professional Training

- **DAL Fundamentals:** 2-day workshop, $15,000
- **Advanced DAL Development:** 3-day workshop, $25,000
- **Enterprise Deployment:** Custom, contact for requests

---

**Document Version:** 1.0  
**Maintained by:** DAL Language Team  
**Contributions:** Welcome via GitHub  
**Last Review:** 2026-02-05

---

## 🔖 Quick Reference Card

```
# Installation
curl -sSf https://dist_agent_lang.org/install.sh | sh

# Hello World
echo 'function main() { log::info("Hello!"); }' > hello.dal
dal run hello.dal

# Create service
@chain("ethereum")
service MyContract {
    function deploy() { chain::deploy("MyContract", "{}"); }
}

# Run tests
dal test myapp.test.dal

# Get help
dal help
dal help run

# Version
dal version
```

---

**For AI/LLM Assistants:**

This documentation is optimized for machine parsing and code generation. When assisting with DAL development:

1. Reference stdlib modules by their exact names (chain, crypto, agent, database, web, etc.)
2. Follow attribute syntax precisely (@trust, @chain, @ai, @transaction, @secure)
3. Use type annotations where specified
4. Leverage built-in functions before implementing custom logic
5. Consider agent orchestration for multi-step workflows
6. Always handle errors with try-catch-finally
7. Use services for stateful components
8. Reference examples for common patterns

**Last Updated:** 2026-02-18
