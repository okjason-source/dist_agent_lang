# dist_agent_lang — Documentation index

**Version:** 1.0.8 (Beta)
**Last updated:** 2026-03-03
**License:** Apache 2.0

---

## Quick navigation

| Section | Description | Start here if... |
|---------|-------------|-----------------|
| [Getting started](#getting-started) | Install, first program, first agent | You're new to DAL |
| [HTTP vs MCP vs LSP](IDE_AND_AGENT_INTEGRATION.md#first-time-mental-model) | One-page mental model: agent = HTTP; MCP = optional IDE bridge; LSP = editor | You're new to agents or confused about MCP |
| [Agent development](#agent-development) | Agent setup, skills, persistent memory, molds | You're building agents |
| [Language reference](#language-reference) | Syntax, types, attributes | You need syntax details |
| [Standard library](#standard-library) | All 30 stdlib modules | You need API docs |
| [CLI reference](#cli-reference) | All commands | You need CLI help |
| [Blockchain and hybrid](#blockchain-and-hybrid-trust) | Multi-chain, trust models, security | You're integrating blockchain |
| [IDE and agent integration](#ide-and-agent-integration) | DAL IDE vs MCP vs your HTTP agent (§1), LSP, editor tools | You're setting up an editor or wiring agents |
| [Project state](#project-state-and-alignment-maintainers) | Deploy/production readiness, shipped vs planned, gap checklist | You maintain releases, CI, or audit docs |
| [Configuration](#configuration) | Environment variables index | You deploy or set keys / policies |
| [Examples](#examples) | Code samples and tutorials | You learn by example |

---

## Getting started

### Installation

```bash
# From Cargo (recommended)
cargo install --git https://github.com/okjason-source/dist_agent_lang.git dist_agent_lang --bin dal

# From source
git clone https://github.com/okjason-source/dist_agent_lang.git
cd dist_agent_lang && cargo build --release

# From binary
# Download from GitHub Releases: https://github.com/okjason-source/dist_agent_lang/releases/latest
```

**Requirements:** Rust 1.70+ ([Install Rust](https://rustup.rs/))

### First program

```dal
@trust("hybrid")
service HelloWorld {
    fn main() {
        print("Hello, dist_agent_lang!");
    }
}
```

```bash
dal run hello.dal
```

### First agent

```bash
mkdir my-agent && cd my-agent
dal init agent
dal agent serve
# Agent running at http://localhost:4040
```

**Guides:**
- [Quick Start](guides/QUICK_START.md) — Up and running in 5 minutes
- [Agent Setup and Usage](guides/AGENT_SETUP_AND_USAGE.md) — Complete agent development guide

---

## Agent development

DAL is agent-first. Agents are native language constructs with lifecycle operations (spawn, coordinate, communicate, evolve), persistent memory, composable skills, and HTTP serving.

| Guide | What it covers |
|-------|---------------|
| [Agent Setup and Usage](guides/AGENT_SETUP_AND_USAGE.md) | Project setup, CLI, HTTP server, DAL APIs, molds, evolve context |
| [Skills and Registry](guides/SKILLS_AND_REGISTRY.md) | Built-in and custom skills, `.skill.dal` format, programmatic encouragement, runtime registration |
| [Persistent Agent Memory](guides/PERSISTENT_AGENT_MEMORY.md) | Runtime persistence, backends (file/SQLite), configuration, schema versioning |
| [Agent Capabilities](AGENT_CAPABILITIES.md) | Capability definition, validation, and per-type/per-agent configuration |
| [Mold Format](MOLD_FORMAT.md) | `.mold.dal` syntax, lifecycle hooks, principal vs mold trust |
| [Fleet deployment](FLEET_DEPLOYMENT.md) | CLI walkthrough (`dal agent fleet …`), `.dal/fleets.json`, deploy/run/health/export |
| [Comprehensive Agent Plans](COMPREHENSIVE_AGENT_AND_MOLD_PLANS.md) | Agent reasoning, fleet, mold config, off-chain implementation |

### Key concepts

- **Agent types**: `ai`, `system`, `worker`, `custom:<name>`
- **Skills**: Named bundles of capabilities (tools + description). Built-in categories: development, creative, office, home. Users define custom skills in `.skill.dal` files.
- **Molds**: Reusable agent configurations (`.mold.dal`). User-built, user-owned. Future licensing via on-chain smart registry.
- **Persistent memory**: Agent state (memory, tasks, messages, evolution, skills) persists across restarts. On by default.
- **Evolve**: File-based conversation and action history. Loaded into the agent prompt each turn.

---

## Language reference

### Core concepts

DAL is a multi-paradigm, dynamically-typed language hosted in a Rust interpreter. Programs are executed with `dal run`.

- **Services**: Stateful constructs with attributes (`@trust`, `@chain`, `@secure`, `@txn`, `@limit`)
- **Functions**: Named, closures, async
- **Types**: int, float, string, bool, null, list, map, set, struct, result, option, closure
- **Error handling**: try-catch-finally, Result types
- **Modules**: import/export, stdlib namespaces

### Files

- [Syntax Reference](syntax.md) — Complete grammar and syntax
- [Attributes Reference](attributes.md) — Service and function attributes
- [Documentation.md](Documentation.md) — Architecture, type system, and internals

---

## Standard library

30 modules covering agents, AI, blockchain, data, web, IoT, security, and administration.

| Module | Purpose | Key functions |
|--------|---------|---------------|
| **agent** | Agent lifecycle | `spawn()`, `coordinate()`, `communicate()`, `evolve()` |
| **ai** | AI/ML operations | `generate_text()`, `classify()`, `embed()`, `analyze_text()` |
| **chain** | Blockchain | `deploy()`, `call()`, `get_balance()`, `transfer()` |
| **crypto** | Cryptography | `hash()`, `sign()`, `verify()`, `encrypt()` |
| **auth** | Authentication | `create_user()`, `login()`, `validate_token()` |
| **database** | Database ops | `query()`, `connect()`, `migrate()` |
| **evolve** | Context management | `load()`, `append_conversation()`, `append_log()` |
| **mold** | Agent molds | `load()`, `spawn_from()`, `list()` |
| **trust** | Trust models | `validate_hybrid_trust()`, `authorize()` |
| **oracle** | Data feeds | `fetch()`, `stream()`, `verify()` |
| **web** | HTTP | `get_request()`, `post_request()` |
| **log** | Logging | `info()`, `warn()`, `error()` |
| **cloudadmin** | Cloud admin | `authorize()`, `grant()`, `audit_log()` |
| **iot** | IoT devices | `connect_device()`, `read_sensor()` |
| **mobile** | Mobile integration | Platform-specific APIs |
| **desktop** | Desktop integration | System-level APIs |
| **config** | Configuration | `get_env()`, `get_database_config()` |
| **aml** | Anti-money laundering | `perform_check()`, `get_status()` |
| **kyc** | Know Your Customer | `verify()`, `get_verification()` |
| **sh** | Shell execution | `run()` with trust controls |

**Full reference:** [STDLIB_REFERENCE.md](STDLIB_REFERENCE.md) — Machine-readable with complete signatures, parameters, return types, and examples.

**Module guides:**
- [API Reference](guides/API_REFERENCE.md) — Grouped by module
- [AI Features Guide](guides/AI_FEATURES_GUIDE.md) — AI module details
- [CloudAdmin Guide](guides/CLOUDADMIN_GUIDE.md) — Hybrid trust and admin control

---

## CLI reference

| Command | Description |
|---------|-------------|
| `dal run <file>` | Execute a DAL file |
| `dal agent serve` | Start agent HTTP server |
| `dal agent create <type> <name>` | Create an agent |
| `dal agent chat [name]` | Interactive agent chat |
| `dal agent send <from> <to> "<msg>"` | Send message between agents |
| `dal agent task assign <id> "<desc>"` | Assign task to agent |
| `dal agent mold list` | List local molds |
| `dal init agent` | Initialize agent project |
| `dal check <file>` | Syntax check |
| `dal fmt <file>` | Format code |
| `dal lint <file>` | Lint code |
| `dal test` | Run tests |
| `dal parse <file>` | Show AST |
| `dal repl` | Interactive REPL |
| `dal watch <file>` | Watch for changes |
| `dal new <name>` | Create new project |
| `dal convert <file>` | Solidity to DAL |

---

## Blockchain and hybrid trust

DAL provides multi-chain blockchain integration through its trust model and chain module.

| Guide | What it covers |
|-------|---------------|
| [Hybrid Integration Guide](guides/HYBRID_INTEGRATION_GUIDE.md) | On-chain/off-chain patterns, trust model, sync, security |
| [Pre-signed Deployment Guide](guides/PRESIGNED_DEPLOYMENT_GUIDE.md) | Building and submitting pre-signed deploy tx (Foundry/Hardhat → DAL), use cases |
| [Solidity Integration Guide](SOLIDITY_INTEGRATION_GUIDE.md) | Orchestrating Solidity contracts from DAL |
| [AI Best Practices](guides/AI_BEST_PRACTICES.md) | Security and optimization for AI + blockchain |
| [Architecture Separation](guides/ARCHITECTURE_SEPARATION.md) | Separating DAL contracts from frontend |

**Supported chains:** Ethereum, Polygon, Solana, Arbitrum (and others via chain module).

**Trust levels:**
- `@trust("decentralized")` — On-chain only
- `@trust("hybrid")` — On-chain + off-chain
- `@trust("centralized")` — Off-chain only

---

## IDE and agent integration

- [IDE and Agent Integration](IDE_AND_AGENT_INTEGRATION.md) — **§1:** DAL IDE (`dal ide serve`) vs external editors vs your HTTP agent; LSP setup; editor agents; tool schema
- [LSP and Agent Integration Plan (detailed)](development/LSP_AND_AGENT_INTEGRATION_PLAN.md) — Full LSP feature plan, phases, stdio vs IDE HTTP routes
- [DAL CEO / agent app plan](AGENT_ASSISTANT_PLAN.md) — Executive sub-agents, skills, molds, code-editor tools (legacy filename; app lives under `CEO/`)

### Project state and alignment (maintainers)

- **[Deploy and production readiness](development/DEPLOY_AND_PRODUCTION_READINESS.md)** — Fast iteration vs production discipline: CI, release artifacts, Docker, CEO run paths, linked checklists.
- **[RAG MVP spec](development/RAG_MVP_SPEC.md)** — Retrieval into `context_blocks` (`source: rag`), corpus, indexer, lexical MVP-A, `agent_serve` wiring.
- **[Project state and alignment](PROJECT_STATE_AND_ALIGNMENT.md)** — Single matrix: what is shipped vs stub vs planned; links to trust-split EVM, LSP, checklists; **gap backlog** to keep docs and code honest across releases.
- **[Production grade checklist](PRODUCTION_GRADE_CHECKLIST.md)** — Unwrap/panic, stubs, JWT, logging
- **[CODEBASE_TODOS.md](../CODEBASE_TODOS.md)** — Short index into the above (repo root)

### Configuration

- **[CONFIG.md](CONFIG.md)** — Index of environment variables (`DAL_*`, LLM keys, JWT, etc.) with links to full guides.

**File extensions:**
- `.dal` — Source files
- `.test.dal` — Test files
- `.mold.dal` — Mold definitions
- `.skill.dal` — Skill definitions

**Syntax highlighting:** Use Rust syntax as a base. DAL-specific keywords: `service`, `@chain`, `@trust`, `@ai`, `@secure`, `@txn`.

---

## Examples

### In the repo

27+ example programs in the [examples/](../examples/) directory:

- `01_hello_world.dal` — Basic language features
- `02_nft_marketplace.dal` — Blockchain operations and attributes
- `03_trading_bot.dal` — Async and agent system
- `04_error_handling.dal` — Error management patterns

### Tutorials

- [DeFi Token](tutorials/01_defi_token.md)
- [AI Trading Agent](tutorials/02_ai_trading_agent.md)
- [Hybrid Marketplace](tutorials/03_hybrid_marketplace_cloudadmin.md)

### Quick snippets

**Agent with persistent memory and skills:**
```dal
let agent_id = agent::spawn({
    "name": "office-bot",
    "type": "ai",
    "role": "Office assistant with calendar and email skills"
});
agent::set_serve_agent(agent_id);
```

**Blockchain operation:**
```dal
@trust("hybrid")
@chain("ethereum")
service Token {
    fn deploy() {
        let result = chain::deploy("MyToken", "{}");
        log::info("Deployed: " + result.address);
    }
}
```

**Agent coordination:**
```dal
let task = agent::create_task("Process batch 42", "high");
agent::coordinate(agent_id, task, "task_distribution");
```

---

## Package management

- [Packaging and Distribution](PACKAGING.md) — Install methods, versioning, configuration
- [DAL Venv](VENV.md) — Named execution environments with security profiles

**Release maintainers:** [RELEASE_DOCS_BUNDLE.md](RELEASE_DOCS_BUNDLE.md) lists which docs and guides to include with the language release so installs (and LLMs) have access to the most important documentation.

---

## Project status

**Current:** Beta (v1.0.8). Actively maintained with consistent updates.

| What works | Status |
|-----------|--------|
| Interpreter runtime (`dal run`) | Stable |
| Agent framework (spawn, serve, persist, skills) | Stable |
| 30-module stdlib | Stable |
| CLI toolchain | Stable |
| Mold system | Stable |
| HTTP agent server | Stable |
| Solidity converter | Stable |
| Testing framework | Stable |
| Blockchain transpilation | Experimental |
| WASM/native compile targets | Experimental |

For the roadmap, see [Language Vision](guides/GENERAL_PURPOSE_LANGUAGE_ANALYSIS.md) and [Beta Release Summary](guides/BETA_RELEASE_SUMMARY.md).

**Design docs:**
- [Runtime value methods and stdlib wiring](design/RUNTIME_VALUE_METHODS_AND_STDLIB_WIRING.md) — Production-grade plan for value-method support (list/map methods, receiver-first dispatch) and stdlib wiring (IoT reference). Phase 1 and chain fixes implemented.
- [Chain namespace gaps and fixes](design/CHAIN_NAMESPACE_GAPS_AND_FIXES.md) — deploy 3rd arg, get as Map, get_chain_config / get_supported_chains; remaining optional improvements.
- [Mobile, edge, and IoT targets](design/MOBILE_EDGE_IOT_TARGETS.md) — Edge = IoT; IoT served by runtime (not compiled to); transpile deferred, same pattern; mobile compile deferred unless transpile path exists.

---

## Version history

| Version | Date | Highlights |
|---------|------|-----------|
| 1.0.8 | 2026-03 | Persistent agent memory, extensible skills registry, documentation overhaul |
| 1.0.5 | 2026-02 | Mutation testing hardening, HTTP middleware tests |
| 1.0.3 | 2026-02 | mold:: stdlib, agent molds, CLI mold commands |
| 1.0.2 | 2026-01 | CloudAdmin module, improved stdlib |
| 1.0.0 | 2025-11 | Initial release |

---

## License

**Apache License 2.0** — see [LICENSE](../LICENSE).

---

## Contact and community

- **GitHub:** [okjason-source/dist_agent_lang](https://github.com/okjason-source/dist_agent_lang)
- **Discord:** [discord.gg/tu7tg9eN](https://discord.gg/tu7tg9eN)
- **Email:** jason.dinh.developer@gmail.com
