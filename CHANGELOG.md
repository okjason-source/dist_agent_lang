# Changelog

All notable changes to the `dist_agent_lang` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Chain namespace (example-only APIs):** Implemented in `chain.rs` + engine so examples work without changing example files:
  - `chain::get(chain_id, address)` — 2-arg overload returns map with `balance`, `chain_id`, `address`
  - `chain::get_info(chain_id)` — alias for `get_chain_config`
  - `chain::deploy_contract(...)` — alias for `deploy(chain_id, contract_name, constructor_args)`
  - `chain::get_current_gas_price(chain_id)` — alias for `get_gas_price`
  - `chain::get_token_balance(chain_id, token_symbol_or_contract, address)` — ERC20 `balanceOf` via RPC when `http-interface` enabled; known symbols (USDC, USDT, WETH, DAI, WBTC) on chain_id 1
  - `chain::get_block_hash()` / `get_block_hash(chain_id)` — latest block hash via RPC or placeholder
- **Pre-signed deployment:** New guide and example:
  - [docs/guides/PRESIGNED_DEPLOYMENT_GUIDE.md](docs/guides/PRESIGNED_DEPLOYMENT_GUIDE.md) — how to build a pre-signed deploy tx (Foundry/Hardhat/Node), pass hex to DAL, use cases (CI/CD, multi-chain, governance, agent-driven)
  - [examples/deploy_presigned_example.dal](examples/deploy_presigned_example.dal) — `deploy_with_presigned`, `deploy_demo_only`, multi-chain pattern
  - Guide clarifies DAL → EVM pipeline (`dal build --target blockchain` → Solidity skeleton → solc → bytecode), hybrid use case, and practical stub uses (oracle/NFT placeholders, interface validation)
- **Documentation:** PUBLIC_DOCUMENTATION_INDEX links to Pre-signed Deployment Guide and Solidity Integration; CHAIN_NAMESPACE_GAPS_AND_FIXES §5 and pre-signed pointer
- **IDE + SSE (phase 0):** Structured event envelopes, bounded replay buffers, request body limits, correct `Content-Type` for static images, and per-client stream state; new observability counters (SSE connections, resume, gaps, replay, lag, etc.); new `ide_sse_phase0_tests` and an SSE schema drift guard; CI also runs `scripts/policy/check_sse_policy_matrix.sh` via a **Streaming Reliability Gates** job (Node 20).
- **MCP:** New `mcp` stdlib for MCP bridge lifecycle (stdio vs `http-stream`), list/status, and tool invocation against DAL agent HTTP; CI runs MCP transport parity tests and the in-repo Node MCP bridge test suite (see `.github/workflows/ci.yml` for paths).
- **`graph` stdlib:** Bearer-based graph-style HTTP client with retries/backoff for Microsoft Graph–class REST APIs.
- **Database (SQLite):** With the `sqlite-storage` feature, `database::connect("sqlite://…")` uses a real SQLite backend (WAL, bound parameters, backup, introspection helpers) via the new `database_sqlite` module; `rusqlite` now enables the `backup` feature.
- **`dal serve` / HTTP hardening (optional Basic Auth):** `DAL_HTTP_USER` with `DAL_HTTP_PASSWORD_HASH` (bcrypt, preferred) or `DAL_HTTP_PASSWORD` (dev); `DAL_HTTP_AUTH_EXEMPT` and `DAL_HTTP_AUTH_*` for lockout tuning. Documented in [docs/CONFIG.md](docs/CONFIG.md). Middleware and helpers renamed from `coo_basic_auth_*` to `dal_serve_basic_auth_*`; `DalServeBasicAuthBruteForce` replaces `CooBasicAuthBruteForce`.
- **CLI `crypto`:** `dal crypto forge` (aka `bcrypt-hash` / `http-password-hash`) outputs bcrypt for **`DAL_HTTP_PASSWORD_HASH`**.
- **AI & agent tool loop:** `DAL_LLM_PRIMARY` plus env-first provider selection (Kimi `DAL_AI_*`, DeepSeek, Ollama, OpenAI, Anthropic); normalizes OpenAI-style `/v1` bases to `…/chat/completions`; parses legacy JSON tool lines embedded in assistant `content`, caps tool-result size, raises `DAL_AGENT_MAX_TOOL_STEPS` ceiling, surfaces `last_tool_success` on turn traces, and extends tool-loop / citation guidance; `prompt_variant_contract_tests` for prompt invariants.
- **RAG:** `DAL_RAG_TOP_K` cap honored and covered by tests in `rag_retrieval`.
- **`evolve`:** `load_recent_for_prompt` and section/summary/conversation extraction so prompts avoid broken markdown tables; note on when `load_recent` is a poor fit.

### Changed
- **BREAKING:** Renamed `cap` module to `key` — capability-based access control
  - `stdlib::cap` → `stdlib::key`
  - CLI: `dal cap` → `dal key create|check|principal`
  - DAL code: `cap::create()` → `key::create()`, etc.
  - Capability IDs now use `key_` prefix instead of `cap_`
- **Best Practices guide:** [docs/guides/BEST_PRACTICES.md](docs/guides/BEST_PRACTICES.md) rewritten to use real DAL syntax and stdlib throughout (services with `@trust`/`@chain`/`@secure`, `fn`, `chain::`/`oracle::` APIs, `throw`/Result/try-catch, `test::expect_*`, describe/it); removed Solidity-style examples
- **Documentation:** Add links for optional Claude connector setup, Cursor gating, IDE SSE / MCP / SSE release matrix runbooks, and `STDLIB_REFERENCE` expansion; installation, config, and README/Makefile text aligned.
- **Runtime / integration:** Parser, `runtime::engine`, `http_server_*`, `agent_serve`, IDE `agent_runner`, `rag_index`, and related tests updated in support of the features above; fuzz `corpus_seed` DAL examples refreshed.

## [1.0.5] - 2026-02-08

### Added
- Mutation-testing hardening: lexer escape and tokenize tests, mold memory-calculation tests, HTTP security middleware invocation tests
- Version set to 1.0.5 across package and documentation

### Changed
- Documentation version references updated from 1.0.3 to 1.0.5 for conformity

## [1.0.3] - 2026-01-27

### Added
- Optional `source` parameter to all log functions (info, warning, error, debug, audit)
  - Defaults to "system" (or "audit"/"debug" for respective functions) if None is provided
  - Enables custom source identifiers for better log filtering and debugging
- Comprehensive security integration tests using actual DAL language code
  - End-to-end security workflow tests
  - Tests cover authentication, authorization, cryptographic signatures, cross-chain security
  - Tests cover KYC/AML compliance, security logging, input validation
  - Aligned with PRODUCTION_ROADMAP.md goals

### Changed
- HTTP client timeout standardized to 30000ms (30 seconds) for consistency with HTTP libraries
  - Added clear documentation comment explaining timeout unit
  - Aligns with reqwest and other standard HTTP client conventions
- Updated all log function calls throughout codebase to include source parameter
  - Appropriate source identifiers added (e.g., "web", "ai", "runtime", "chain")
  - Backward compatible with None parameter using default sources

### Fixed
- Log source hardcoding issue (now supports custom sources)
- HTTP timeout unit confusion (now standardized to milliseconds)
- All recommendations from STDLIB_TESTS_BUG_REPORT.md implemented

### Test Coverage
- 140 stdlib tests passing (100%)
- Comprehensive security integration tests added
- Zero compilation errors
- All log function calls updated and tested

## [1.0.1] - 2025-12-15

### Fixed
- Fixed integer overflow bug in `chain::get_balance` function
  - Used `checked_mul` for safe arithmetic operations
  - Reduced multiplier to prevent integer overflow
  - All tests now passing (95/95)

### Changed
- Updated documentation to reflect complete test coverage
- Updated security disclaimer with current test status
- Re-enabled previously ignored tests after bug fix

### Added
- Comprehensive testing infrastructure
  - 95 tests across 6 test suites
  - 17 performance benchmarks
  - Integration workflow tests
  - Security feature tests

## [Unreleased] (Previous)

### Added
- **Solidity Contract Integration**
  - `add_sol` module for seamless Solidity contract orchestration
  - ABI parsing and validation for type-safe contract calls
  - Event listening capabilities for Solidity contract events
  - Auto-generated wrapper code from Solidity ABIs
  - Testing utilities for Solidity contract integration
  - Comprehensive integration guide and examples

## [1.0.0] - 2024-12-19

### Added
- **Core Language Features**
  - Hybrid compiled programming language with AI agent integration
  - Abstract Syntax Tree (AST) with service declarations
  - Lexer and parser for `dist_agent_lang` syntax
  - Runtime environment with scope management
  - Type system supporting `map<string, any>`, `vector<any>`, `string`, `int`, `float`, `bool`

- **AI Agent Framework**
  - `Agent` struct with memory, capabilities, and message queues
  - `AgentCoordinator` for multi-agent orchestration
  - `Workflow` system with step dependencies and execution
  - `Task` management with status tracking and results
  - AI-powered text and image analysis capabilities
  - Agent communication protocols and message passing

- **Blockchain Integration**
  - Multi-chain support (Ethereum, Polygon, Binance, Solana, Avalanche, Arbitrum, Optimism)
  - `@chain` attributes for blockchain network specification
  - Cross-chain operations and bridge configurations
  - Smart contract deployment and interaction
  - Chain-specific gas estimation and transaction management

- **Standard Library Modules**
  - `chain::` - Blockchain operations and smart contract management
  - `auth::` - Authentication, authorization, and session management
  - `kyc::` - Know Your Customer verification
  - `aml::` - Anti-Money Laundering compliance checks
  - `oracle::` - External data feeds and real-world data integration
  - `ai::` - AI agent creation, management, and coordination
  - `database::` - Database connectivity and query management
  - `web::` - Web API and HTTP operations
  - `crypto::` - Cryptographic operations and security
  - `log::` - Logging and debugging utilities
  - `service::` - Service management and lifecycle
  - `admin::` - Administrative operations and system management
  - `sync::` - Synchronization and concurrency primitives
  - `cap::` - Capability-based security
  - `config::` - Configuration management
  - `cloudadmin::` - Cloud infrastructure management
  - `trust::` - Trust model and security profiles
  - `desktop::` - Desktop application development
  - `mobile::` - Mobile application development
  - `iot::` - Internet of Things device management

- **Compilation Target System**
  - Multi-target compilation (Blockchain, WebAssembly, Native, Mobile, Edge)
  - `@compile_target` attributes for target specification
  - Target constraints and validation
  - Trust model integration with security profiles
  - Permission system with audit trails

- **Interface Generation**
  - `@interface` attributes for client interface generation
  - Multi-language interface support (TypeScript, JavaScript, Python, Rust, Java, Go)
  - Cross-chain bridge operation interfaces
  - Deployment configuration generation
  - Client library generation for different platforms

- **Security Features**
  - `@trust` attributes for trust level specification
  - `@secure` attributes for security requirements
  - `@limit` attributes for resource constraints
  - Comprehensive audit trail system
  - KYC/AML compliance integration
  - Capability-based security model

- **Development Tools**
  - Comprehensive test suite with 100% pass rate
  - Performance benchmarking and optimization
  - Error detection and resolution system
  - Documentation generation
  - Example codebase with 11 comprehensive examples

### Examples Included
- `agent_system_demo.rs` - AI agent system demonstration
- `backend_connectivity_patterns.rs` - Database and API connectivity patterns
- `chain_selection_example.rs` - Multi-chain selection and interaction
- `cross_chain_patterns.rs` - Cross-chain asset management and operations
- `defi_nft_rwa_contract.rs` - DeFi smart contract with NFT RWAs (Arbitrum)
- `dynamic_nft_examples.rs` - Dynamic NFT examples with real-world data (Ethereum)
- `dynamic_rwa_examples.rs` - Real World Asset tokenization examples (Ethereum)
- `enhanced_language_features.rs` - Advanced language features demonstration
- `general_purpose_demo.rs` - General-purpose programming examples
- `integrated_spawn_ai_examples.rs` - Integrated spawn and AI agent examples
- `keys_token_implementation.rs` - Complete token implementation example (Ethereum)

### Technical Specifications
- **Language**: Rust-based implementation
- **Target Platforms**: Linux, macOS, Windows
- **Blockchain Support**: EVM-compatible chains, Solana, custom chains
- **AI Integration**: GPT, BERT, custom AI models
- **Database Support**: PostgreSQL, MySQL, SQLite, NoSQL
- **Web Support**: HTTP/HTTPS, WebSocket, REST APIs
- **Security**: AES-256, SHA-256, ECDSA, multi-signature

### Performance
- **Compilation Speed**: < 5 seconds for typical projects
- **Runtime Performance**: Near-native execution speed
- **Memory Usage**: Optimized for embedded and edge devices
- **Concurrency**: Async/await support with worker pools
- **Scalability**: Horizontal scaling with agent coordination

### Documentation
- Complete API reference
- 12-part tutorial series (Beginner to Advanced)
- Best practices guide
- Security guidelines
- Deployment instructions
- Example codebase with comprehensive demonstrations

### Production Readiness
- ✅ Zero critical compilation errors
- ✅ Comprehensive test coverage
- ✅ Performance optimization complete
- ✅ Security audit passed
- ✅ Documentation complete
- ✅ Examples validated
- ✅ Cross-platform compatibility verified

## [Unreleased]

### Planned Features
- WebAssembly compilation target
- Mobile SDK for iOS and Android
- Cloud deployment automation
- Advanced AI model integration
- Additional blockchain networks
- Enterprise security features
- Performance monitoring and analytics
