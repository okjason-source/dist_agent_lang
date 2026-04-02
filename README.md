# Distributed Agent Language

## dist_agent_lang

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/okjason-source/dist_agent_lang)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg)](https://github.com/okjason-source/dist_agent_lang)

**A runtime programming language tooled with trust off-chain and on-chain controls for the agentic future.**


DAL is a programming language where AI agents are native citizens. Spawn agents, give them skills, persist their memory, coordinate multi-agent workflows, and serve them over HTTP. Program applications using distributed integration with off-chain services, and on-chain contracts coexisting through a unified trust model. [Try the IDE](https://d10e14gce1iuq7.cloudfront.net/).


> **v1.0.XX Beta** — Test thoroughly before production use.

---

## Quick start

```bash
# Install
cargo install --git https://github.com/okjason-source/dist_agent_lang.git dist_agent_lang --bin dal

# Create an agent project
mkdir my-agent && cd my-agent
dal init agent

# Start the agent server
dal agent serve
```

This gives you a running agent with persistent memory, an HTTP API, evolve context (conversation history), and the default skill set (development, creative, office, home). No configuration required — persistence is on by default.

**Update to latest:** Run the install command again, or clone the repo and run `./update.sh`.

---

## What makes DAL different

### Agents are built-in, not bolted on

DAL is a language where `agent::spawn`, `agent::coordinate`, `agent::communicate`, and `agent::evolve` are first-class operations. Agent memory, tasks, messages, and skills persist across restarts by default.

```dal
// Spawn an agent, give it skills, serve it over HTTP
let config = agent::create_config("assistant", "ai", "office assistant");
let agent = agent::spawn(config);
agent::set_serve_agent(agent.agent_id);
```

### Skills are user-owned and composable

Define custom skills in `.skill.dal` files. Your skills, your agents — no central registry, no marketplace dependency.

```
// .dal/office.skill.dal
skill "ms_office" {
  category "office"
  description "Use MS Office tools (Word, Excel, Outlook) via run or scripts."
  tools "run" "search"
}
```

Agents reference skills by name. The runtime resolves them at prompt-build time and tells the model what it can do.

### Molds: reusable agent configurations

Molds are reusable agent configurations (`.mold.dal`) that define type, role, skills, capabilities, and lifecycle hooks. Built by users, owned by users. Future licensing via on-chain smart registry (dynamic NFTs).

```bash
dal agent create --mold ./expert.mold.dal MyExpert
```

### Hybrid trust model

One language for both on-chain and off-chain. No context switching between Solidity and Python.

```dal
@trust("hybrid")
service PaymentService {
    fn process(user_id: string, amount: int) {
        let payment = payment::process(user_id, amount);
        let tx = chain::deploy(1, "PaymentRecord", {
            "user_id": user_id, "amount": amount
        });
    }
}
```

### Persistent by default

Agent memory (key-value store), task queue, message bus, evolution data, registered skills, and the agent registry all survive restarts. File-backed (JSON) by default; SQLite available for higher throughput. Opt out with `DAL_AGENT_RUNTIME_PERSIST=0`.

---

## Architecture

DAL is a **Rust-hosted interpreted language** with a tree-walking runtime. The interpreter executes `.dal` files directly via `dal run`. The runtime provides:

- **Lexer + Parser** producing a full AST
- **Runtime engine** with scope, closures, and module resolution
- **Over 30-module standard library**: agent, ai, chain, crypto, database, auth, cloud, iot, mobile, desktop, evolve, trust, oracle, and more
- **HTTP server** (axum-based) for serving agents with `/message` and `/task` endpoints
- **Package registry** for publishing and consuming DAL packages

**Compile targets** (blockchain, WASM, native) exist as transpilation/code-generation backends. The primary execution path is the interpreter.

---

## Features

| Category | What you get |
|----------|-------------|
| **AI agents** | Spawn, coordinate, communicate, evolve. Persistent memory, skills, molds, lifecycle hooks. HTTP serve with multi-step tool loop. |
| **Skills registry** | Built-in skills (development, creative, office, home) + user-defined `.skill.dal`. |
| **Persistent memory** | Agent state survives restarts. File or SQLite backend. Schema versioning. On by default. |
| **Blockchain** | Multi-chain (Ethereum, Polygon, Solana, Arbitrum). Deploy, call, events. Solidity converter. |
| **Security** | Reentrancy protection, safe math, cross-chain security, oracle validation, shell trust controls, ACID transactions, configurable guards/env vars. |
| **Hybrid trust** | @trust("decentralized") or ("hybrid") or ("centralized") — high level attribute for on-chain and off-chain service contracts. |
| **CLI toolchain** | `dal run`, `dal check`, `dal fmt`, `dal lint`, `dal test`, `dal repl`, `dal watch`, `dal new`, `dal init`, `dal agent serve` |
| **Standard library** | 30 modules: agent, ai, chain, crypto, database, auth, cloud, iot, mobile, desktop, evolve, trust, and more |
| **Testing** | Built-in test framework, mock registry, 400+ tests passing |

---

## CLI

```bash
dal run program.dal          # Run a DAL program
dal agent serve              # Start agent HTTP server
dal agent serve --behavior agent.dal  # Serve with behavior script
dal agent create ai MyAgent  # Create an agent
dal init agent               # Initialize agent project
dal check program.dal        # Syntax check
dal fmt program.dal          # Format code
dal lint program.dal         # Lint code
dal test                     # Run tests
dal repl                     # Interactive REPL
dal new my_project           # Create new project
dal mold list                # List available molds
```

---

## Documentation

**Canonical documentation** is the Markdown under [`docs/`](docs/) on the **`main`** branch ([browse on GitHub](https://github.com/okjason-source/dist_agent_lang/tree/main/docs)). That tree is the source of truth for prose docs; tools and models can fetch stable raw URLs (for example [`PUBLIC_DOCUMENTATION_INDEX.md`](https://raw.githubusercontent.com/okjason-source/dist_agent_lang/main/docs/PUBLIC_DOCUMENTATION_INDEX.md)).

- **Browse in the browser:** [Documentation index on GitHub Pages](https://okjason-source.github.io/dist_agent_lang/docs/PUBLIC_DOCUMENTATION_INDEX.html)
- **LLM / agent entry point:** [`llms.txt`](https://okjason-source.github.io/dist_agent_lang/llms.txt) (same file as [`llms.txt`](llms.txt) in the repo root)

### Getting started
- **[Quick Start](docs/guides/QUICK_START.md)** — Up and running in 5 minutes
- **[Agent Setup and Usage](docs/guides/AGENT_SETUP_AND_USAGE.md)** — Project setup, CLI, HTTP server, DAL APIs, molds

### Agents and skills
- **[Skills and Registry](docs/guides/SKILLS_AND_REGISTRY.md)** — Define custom skills, configure the registry, programmatic encouragement
- **[Persistent Agent Memory](docs/guides/PERSISTENT_AGENT_MEMORY.md)** — Runtime persistence, backends, configuration
- **[Agent Capabilities](docs/AGENT_CAPABILITIES.md)** — Capability definition, validation, and usage

### Retrieval and editor integration
- **[Configuration](docs/CONFIG.md)** — Environment variables for **RAG** (`DAL_RAG`, index paths, `DAL_RAG_TOP_K`), **MCP bridge** (`DAL_AGENT_HTTP_BASE`, `dal mcp-bridge`), and web search / HTTP fetch
- **[Agent setup and usage](docs/guides/AGENT_SETUP_AND_USAGE.md#12-rag-retrieval)** — RAG and MCP in practice (when to enable retrieval, wiring an IDE to your agent HTTP server)
- **[RAG MVP spec](docs/development/RAG_MVP_SPEC.md)** — Lexical retrieval into agent prompts (`rag::prompt_block`, `include_rag`, corpus indexing)
- **[IDE and agent integration](docs/IDE_AND_AGENT_INTEGRATION.md)** — Mental model: agent over HTTP first; **MCP** as an optional stdio bridge for editors (Cursor, etc.); LSP for language tooling

### Reference
- **[Complete Documentation Index](docs/PUBLIC_DOCUMENTATION_INDEX.md)** — All docs in one place
- **[API Reference](docs/guides/API_REFERENCE.md)** — Standard library documentation
- **[Mold Format](docs/MOLD_FORMAT.md)** — `.mold.dal` syntax and lifecycle
- **[AI Features Guide](docs/guides/AI_FEATURES_GUIDE.md)** — AI capabilities and integration

**Release:** [docs/RELEASE_DOCS_BUNDLE.md](docs/RELEASE_DOCS_BUNDLE.md) lists which docs to include with the language install so users and LLMs have access to the most important documentation.

---

## Beta notice

**Current version: v1.0.XX (Beta)**

**Ready for:**
- Development and prototyping
- Learning and experimentation
- Non-critical applications
- Beta testing and validation

**Use with caution for:**
- Production financial applications (wait for v1.1.0+)
- High-value smart contracts (third-party audit recommended)
- Critical infrastructure (additional validation needed)

---

## Contributing

We welcome contributions of all kinds.

- **Test the language** — Run examples, report bugs
- **Improve documentation** — Fix typos, clarify instructions
- **Share feedback** — Tell us what works and what doesn't
- **Code** — See [CONTRIBUTING.md](CONTRIBUTING.md) and [GOOD_FIRST_ISSUES.md](GOOD_FIRST_ISSUES.md)

---

## Installation

### From source
```bash
git clone https://github.com/okjason-source/dist_agent_lang.git
cd dist_agent_lang
cargo build --release
./target/release/dal --version
```

### From binary
Download from [GitHub Releases](https://github.com/okjason-source/dist_agent_lang/releases/latest)

### From Cargo
```bash
cargo install --git https://github.com/okjason-source/dist_agent_lang.git dist_agent_lang --bin dal
```

**Requirements:** Rust 1.70+ ([Install Rust](https://rustup.rs/))

---

## Testing

```bash
cargo test                      # Run all tests
cargo test -- --nocapture       # With output
cargo test --test example_tests # Specific suite
```

400+ tests passing across all standard library modules.

---

## Learn more

- **[Documentation](docs/PUBLIC_DOCUMENTATION_INDEX.md)** — Complete documentation index
- **[Examples](examples/)** — 27+ example programs
- **[GitHub](https://github.com/okjason-source/dist_agent_lang)** — Source code and issues


---

## License

see [LICENSE](LICENSE).

**Made by OK Jason**
