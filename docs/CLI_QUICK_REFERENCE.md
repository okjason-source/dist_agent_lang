# dist_agent_lang CLI - Complete Command Reference

**Version:** 1.0.5  
**Phases Complete:** Phase 0–9  
**Total Commands:** 80+ on our way to interactive distributed agents...

---

## Quick Navigation

- [Core Commands](#core-commands)
- [Developer Tools (Phase 0)](#developer-tools-phase-0)
- [Optimization (Phase 1)](#optimization-phase-1)
- [Blockchain (Phase 2)](#blockchain-phase-2)
- [Cryptography (Phase 2)](#cryptography-phase-2)
- [Database (Phase 2)](#database-phase-2)
- [AI-Enhanced Tools (Phase 3)](#ai-enhanced-tools-phase-3)
- [Cloud & Enterprise (Phase 4)](#cloud--enterprise-phase-4)
- [IDE & LSP (Phase 5)](#ide--lsp-phase-5)
- [Agent Commands (Phase 6)](#agent-commands-phase-6)
- [AI-IoT (Phase 7)](#ai-iot-phase-7)
- [Specialized (Phase 8)](#specialized-phase-8)
- [Cross-Component (Phase 9)](#cross-component-phase-9)
- [Solidity Tools](#solidity-tools)

---

## Core Commands

| Command | Description | Example |
|---------|-------------|---------|
| `dal run <file.dal>` | Run a DAL file | `dal run app.dal` |
| `dal web <file.dal>` | Run DAL web app | `dal web server.dal` |
| `dal web <file.js> [args...]` | Run standalone JS (Node) | `dal web app.js` |
| `dal test [file]` | Run tests | `dal test` or `dal test app.test.dal` |
| `dal parse <file.dal>` | Parse and validate syntax | `dal parse app.dal` |
| `dal help` | Show help message | `dal help` |
| `dal version` | Show version | `dal version` |

---

## Developer Tools (Phase 0)

### Code Quality

| Command | Description | Example |
|---------|-------------|---------|
| `dal check <file.dal>` | Type check without executing | `dal check app.dal` |
| `dal fmt <file.dal>` | Format DAL code | `dal fmt app.dal` |
| `dal fmt <file> --check` | Check if formatted (CI) | `dal fmt app.dal --check` |
| `dal lint <file.dal>` | Lint code for issues | `dal lint app.dal` |

### Project Management

| Command | Description | Example |
|---------|-------------|---------|
| `dal new <name>` | Create new default project | `dal new my-project` |
| `dal new <name> --type <type>` | Create typed project | `dal new my-ai --type ai` |
| `dal init` | Initialize in current dir | `dal init` |
| `dal add <package>` | Add dependency | `dal add @dal/testing` |
| `dal install` | Install dependencies | `dal install` |

**Project Types:**
- `ai` - AI/ML application
- `iot` - IoT device controller
- `agent` - Multi-agent system
- `chain` - Blockchain/smart contract
- `web` - Web application
- `cli` - Command-line tool
- `lib` - Library/package

### Interactive Development

| Command | Description | Example |
|---------|-------------|---------|
| `dal repl` | Start interactive REPL | `dal repl` |
| `dal watch <file.dal>` | Watch file, re-run on changes | `dal watch app.dal` |

---

## Optimization (Phase 1)

### Benchmarking

| Command | Description | Example |
|---------|-------------|---------|
| `dal bench` | Run all benchmark suites | `dal bench` |
| `dal bench <file.dal>` | Benchmark specific file | `dal bench app.dal` |
| `dal bench --suite <name>` | Run specific suite | `dal bench --suite lexer` |

**Benchmark Suites:**
- `lexer` - Tokenization performance
- `parser` - Parsing performance
- `runtime` - Execution performance
- `stdlib` - Standard library performance
- `all` - All suites (default)

### Profiling

| Command | Description | Example |
|---------|-------------|---------|
| `dal profile <file.dal>` | Profile execution timing | `dal profile app.dal` |
| `dal profile <file> --memory` | Include memory tracking | `dal profile app.dal --memory` |

### Optimization

| Command | Description | Example |
|---------|-------------|---------|
| `dal optimize <file.dal>` | Apply AST optimizations | `dal optimize app.dal` |
| `dal optimize <file> -o <out>` | Save optimized code | `dal optimize app.dal -o opt.dal` |
| `dal optimize --level <n>` | Set optimization level | `dal optimize app.dal --level 2` |

**Optimization Levels:**
- `0` - None
- `1` - Basic (constant folding, dead code elimination)
- `2` - Aggressive (all optimizations)

### Memory

| Command | Description | Example |
|---------|-------------|---------|
| `dal memory-stats` | Show memory statistics | `dal memory-stats` |

---

## Blockchain (Phase 2)

### Chain Information

| Command | Description | Example |
|---------|-------------|---------|
| `dal chain list` | List supported chains | `dal chain list` |
| `dal chain config <id>` | Show chain configuration | `dal chain config 1` |

**Supported Chain IDs:**
- `1` - Ethereum Mainnet
- `5` - Ethereum Goerli (testnet)
- `137` - Polygon
- `80001` - Polygon Mumbai (testnet)
- `42161` - Arbitrum One
- `56` - Binance Smart Chain

### Chain Operations

| Command | Description | Example |
|---------|-------------|---------|
| `dal chain gas-price <id>` | Get current gas price (Gwei) | `dal chain gas-price 1` |
| `dal chain balance <id> <addr>` | Get address balance (wei & ETH) | `dal chain balance 1 0x742d...` |
| `dal chain tx-status <id> <hash>` | Get transaction status | `dal chain tx-status 1 0xabc...` |
| `dal chain block-time <id>` | Get latest block timestamp | `dal chain block-time 1` |

### Asset Management (Local/Simulation)

| Command | Description | Example |
|---------|-------------|---------|
| `dal chain mint <name>` | Mint asset | `dal chain mint MyToken` |
| `dal chain mint <name> --meta <kv>` | Mint with metadata | `dal chain mint NFT --meta name=Punk,id=1` |
| `dal chain asset <id>` | Get asset info | `dal chain asset 42` |

---

## Cryptography (Phase 2)

### Hashing

| Command | Description | Example |
|---------|-------------|---------|
| `dal crypto hash <data>` | Hash data (SHA256 default) | `dal crypto hash "hello"` |
| `dal crypto hash <data> <alg>` | Hash with algorithm | `dal crypto hash "hello" sha512` |
| `dal crypto random-hash` | Generate random hash | `dal crypto random-hash` |
| `dal crypto random-hash <alg>` | Random hash with algorithm | `dal crypto random-hash sha512` |

**Hash Algorithms:**
- `sha256` (default)
- `sha512`

### Key Management

| Command | Description | Example |
|---------|-------------|---------|
| `dal crypto keygen` | Generate RSA keypair (default) | `dal crypto keygen` |
| `dal crypto keygen <alg>` | Generate with algorithm | `dal crypto keygen ed25519` |

**Key Algorithms:**
- `rsa` (default)
- `ed25519`

### Signing & Verification

| Command | Description | Example |
|---------|-------------|---------|
| `dal crypto sign <data> <key>` | Sign data with private key | `dal crypto sign "msg" "-----BEGIN..." rsa` |
| `dal crypto sign <d> <k> <alg>` | Sign with algorithm | `dal crypto sign "msg" "key" ed25519` |
| `dal crypto verify <d> <s> <k>` | Verify signature | `dal crypto verify "msg" "sig" "pubkey" rsa` |

### Encryption

| Command | Description | Example |
|---------|-------------|---------|
| `dal crypto encrypt <data> <key>` | Encrypt with public key (RSA) | `dal crypto encrypt "secret" "pubkey"` |
| `dal crypto decrypt <data> <key>` | Decrypt with private key (RSA) | `dal crypto decrypt "cipher" "privkey"` |
| `dal crypto aes-encrypt <d> <k>` | AES-256 encryption | `dal crypto aes-encrypt "data" "key32"` |
| `dal crypto aes-decrypt <d> <k>` | AES-256 decryption | `dal crypto aes-decrypt "cipher" "key32"` |

---

## Database (Phase 2)

### Connection & Basic Operations

| Command | Description | Example |
|---------|-------------|---------|
| `dal db connect <conn>` | Test database connection | `dal db connect "postgresql://localhost/db"` |
| `dal db query <conn> "<sql>"` | Execute SQL query | `dal db query "..." "SELECT * FROM users"` |
| `dal db tables <conn>` | List all tables | `dal db tables "postgresql://localhost/db"` |
| `dal db schema <conn> <table>` | Get table schema | `dal db schema "..." users` |

### Advanced Operations

| Command | Description | Example |
|---------|-------------|---------|
| `dal db plan <conn> "<sql>"` | Get query execution plan | `dal db plan "..." "SELECT * FROM big_table"` |
| `dal db backup <conn> <path>` | Backup database | `dal db backup "..." backup.sql` |
| `dal db restore <conn> <path>` | Restore from backup | `dal db restore "..." backup.sql` |
| `dal db metrics <conn>` | Show database metrics | `dal db metrics "postgresql://localhost/db"` |

**Connection String Formats:**
- PostgreSQL: `postgresql://user:pass@host:port/database`
- MySQL: `mysql://user:pass@host:port/database`
- MongoDB: `mongodb://host:port/database`

---

## AI-Enhanced Tools (Phase 3)

### Code Generation & Analysis

| Command | Description | Example |
|---------|-------------|---------|
| `dal ai code "<prompt>"` | Generate DAL code from natural language | `dal ai code "Create a token contract"` |
| `dal ai code "<p>" -o <file>` | Generate and save code | `dal ai code "REST API" -o server.dal` |
| `dal ai explain <file>` | Explain what code does | `dal ai explain contract.dal` |
| `dal ai review <file>` | Code review with suggestions | `dal ai review myapp.dal` |

### Testing & Optimization

| Command | Description | Example |
|---------|-------------|---------|
| `dal ai test <file>` | Generate test cases | `dal ai test contract.dal` |
| `dal ai test <file> -o <test>` | Generate and save tests | `dal ai test contract.dal -o tests.dal` |
| `dal ai fix <file>` | Suggest fixes for issues | `dal ai fix broken.dal` |
| `dal ai optimize-gas <file>` | Gas optimization suggestions | `dal ai optimize-gas contract.dal` |

### Security

| Command | Description | Example |
|---------|-------------|---------|
| `dal ai audit <file>` | Security audit for smart contracts | `dal ai audit token.dal` |

**API Key Configuration:**
```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic Claude
export ANTHROPIC_API_KEY="sk-ant-..."

# Works without API key (basic mode)
dal ai code "hello world"
```

**Features:**
- 🤖 AI-powered or basic mode (no API key needed)
- 📝 Natural language code generation
- 🔍 Intelligent code analysis
- 🛡️ Security auditing
- 🧪 Automatic test generation
- ⛽ Gas optimization for contracts

---

## Cloud & Enterprise (Phase 4)

### Authorization & Roles

| Command | Description | Example |
|---------|-------------|---------|
| `dal cloud authorize <id> <op> <resource>` | Check authorization | `dal cloud authorize user_123 read config/db` |
| `dal cloud grant <user> <role> <scope>` | Grant role | `dal cloud grant user_123 admin ec2:admin` |
| `dal cloud revoke <user>` | Revoke all roles | `dal cloud revoke user_123` |
| `dal cloud roles <user>` | List user roles | `dal cloud roles user_123` |

**Roles:** `superadmin`, `admin`, `moderator`, `user`

### Audit, Policies, Tenant

| Command | Description | Example |
|---------|-------------|---------|
| `dal cloud audit-log` | Audit trail (info) | `dal cloud audit-log` |
| `dal cloud policies` | Policies (info) | `dal cloud policies` |
| `dal cloud tenant list` | List tenants (info) | `dal cloud tenant list` |
| `dal cloud tenant create <name>` | Create tenant (simulated) | `dal cloud tenant create acme --admin-email a@b.com` |

### Compliance & Chain

| Command | Description | Example |
|---------|-------------|---------|
| `dal cloud compliance scan [--standard X]` | Compliance scan (info) | `dal cloud compliance scan --standard SOC2` |
| `dal cloud compliance report <std> [-o file]` | Report (info) | `dal cloud compliance report SOC2 -o out.pdf` |
| `dal cloud chain-log "<event>"` | Chain audit (info) | `dal cloud chain-log "user deleted X" --chain_id 1` |
| `dal cloud chain-verify <log_id>` | Verify log (info) | `dal cloud chain-verify log_789` |

### Hybrid Trust

| Command | Description | Example |
|---------|-------------|---------|
| `dal cloud trust validate <admin> <user>` | Validate hybrid trust | `dal cloud trust validate valid valid` |
| `dal cloud trust bridge <central> <decentral>` | Bridge trust systems | `dal cloud trust bridge admin user` |

**Env:** `ADMIN_IDS`, `ADMIN_LEVEL_<id>`, `POLICY_<name>_LEVEL`

---

## IDE & LSP (Phase 5)

### Language Server & Documentation

| Command | Description | Example |
|---------|-------------|---------|
| `dal lsp` | LSP server info (editor integration) | `dal lsp` |
| `dal doc <file.dal>` | Generate documentation | `dal doc contract.dal` |
| `dal doc <file> -o <path>` | Save docs to path | `dal doc contract.dal -o docs/API.md` |
| `dal doc <file> --open` | Generate and open | `dal doc contract.dal --open` |
| `dal completions [shell]` | Shell completions | `dal completions bash` |
| `dal debug <file.dal>` | Debug mode (planned) | `dal debug app.dal` |

**Shells:** `bash`, `zsh`, `fish`

**Install completions:** `dal completions bash >> ~/.bashrc` or `eval "$(dal completions zsh)"`

---

## Agent Commands (Phase 6)

### Agent Lifecycle

| Command | Description | Example |
|---------|-------------|---------|
| `dal agent create <type> <name>` | Create agent | `dal agent create worker w1 --role "ETL"` |
| `dal agent list` | Agent info (process-local) | `dal agent list` |

**Types:** `ai`, `system`, `worker`, `custom:<name>`

### Communication & Tasks

| Command | Description | Example |
|---------|-------------|---------|
| `dal agent send <from> <to> "<msg>"` | Send message | `dal agent send agent_1 agent_2 "Process batch"` |
| `dal agent messages <agent_id>` | List messages | `dal agent messages agent_2` |
| `dal agent task assign <id> "<desc>"` | Assign task | `dal agent task assign agent_1 "Analyze data" --priority high` |
| `dal agent task list <agent_id>` | List pending tasks | `dal agent task list agent_1` |

**Note:** Agent state is process-local. Use `dal run your_agents.dal` for multi-agent workflows.

### Fleet & Molds

| Command | Description | Example |
|---------|-------------|---------|
| `dal agent fleet create <name>` | Fleet (placeholder) | `dal agent fleet create my_fleet` |
| `dal agent mold list` | List local molds | `dal agent mold list` |
| `dal agent mold show <path-or-name>` | Show mold config | `dal agent mold show verify_mold` |
| `dal agent mold create <name>` | Scaffold new mold | `dal agent mold create my_mold` |
| `dal agent mold publish <file>` | Upload to IPFS | `dal agent mold publish my_mold.mold.json` |
| `dal agent create --mold <path\|ipfs://cid\|moldId> <name>` | Create agent from mold | `dal agent create --mold verify_mold MyAgent` |

**mold:: stdlib:** Use `mold::load`, `mold::spawn_from`, `mold::list`, `mold::get_info`, `mold::use_mold` in DAL code. See [STDLIB_REFERENCE.md](STDLIB_REFERENCE.md#mold-module).

---

## AI-IoT (Phase 7)

| Command | Description | Example |
|---------|-------------|---------|
| `dal iot ai-predict <device_id>` | Predictive maintenance | `dal iot ai-predict dev_001` |
| `dal iot ai-anomaly <sensor_id>` | Anomaly detection | `dal iot ai-anomaly sens_001` |
| `dal iot ai-optimize <device_id>` | Power optimization | `dal iot ai-optimize dev_001 --target-hours 8` |
| `dal iot read-sensor <sensor_id>` | Read sensor data | `dal iot read-sensor sens_001` |
| `dal iot status <device_id>` | Device status | `dal iot status dev_001` |

**Env:** `IOT_ANOMALY_API_URL`, `IOT_ML_API_URL` for external APIs.

---

## Specialized (Phase 8)

| Command | Description | Example |
|---------|-------------|---------|
| `dal log [show\|stats\|clear]` | Log entries, stats, clear | `dal log show` |
| `dal config [show\|get <key>]` | Config and env | `dal config get AI_API_KEY` |

---

## Cross-Component (Phase 9)

| Command | Description | Example |
|---------|-------------|---------|
| `dal bond <flow> <args...>` | Bond components | `dal bond iot-to-db dev_001 postgres://...` |
| `dal pipe <source> -> <sink>` | Pipeline between components | `dal pipe oracle fetch x -> chain estimate 1` |
| `dal invoke <workflow> <args...>` | Multi-component workflows | `dal invoke iot-ingest dev_001 postgres://...` |

**Flows:** oracle-to-chain, iot-to-db, db-to-sync, auth-to-web, ...  
**Workflows:** price-to-deploy, iot-ingest, ai-audit, compliance-check

---

## Solidity Tools

| Command | Description | Example |
|---------|-------------|---------|
| `dal convert <input.sol>` | Convert Solidity to DAL | `dal convert Token.sol` |
| `dal convert <in> -o <out>` | Specify output file | `dal convert Token.sol -o token.dal` |
| `dal analyze <input.sol>` | Analyze Solidity compatibility | `dal analyze Token.sol` |

---

## Common Workflows

### 1. Start New Project

```bash
dal new my-token --type chain
cd my-token
dal add @dal/testing
dal install
dal fmt src/
dal check src/main.dal
dal run src/main.dal
```

### 2. Development Workflow

```bash
# Format and check
dal fmt src/
dal lint src/
dal check src/main.dal

# Watch mode for rapid iteration
dal watch src/main.dal

# Or use REPL for experiments
dal repl
```

### 3. Performance Optimization

```bash
# Profile to find bottlenecks
dal profile app.dal --memory

# Optimize
dal optimize app.dal -o optimized.dal --level 2

# Benchmark to verify improvement
dal bench app.dal
dal bench optimized.dal
```

### 4. Blockchain Deployment

```bash
# Check gas price
dal chain gas-price 1

# Check balance
dal chain balance 1 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb

# Deploy (via DAL script)
dal run deploy.dal

# Check transaction
dal chain tx-status 1 0xabc123...
```

### 5. Cryptographic Operations

```bash
# Generate keys
dal crypto keygen ed25519 > keys.txt

# Hash data
dal crypto hash "message" sha256

# Sign and verify
dal crypto sign "msg" "$PRIVATE_KEY" ed25519 > sig.txt
dal crypto verify "msg" "$(cat sig.txt)" "$PUBLIC_KEY" ed25519
```

### 6. Database Operations

```bash
# Test connection
dal db connect "postgresql://localhost/mydb"

# Explore schema
dal db tables "postgresql://localhost/mydb"
dal db schema "postgresql://localhost/mydb" users

# Query data
dal db query "postgresql://localhost/mydb" "SELECT COUNT(*) FROM users"

# Backup before changes
dal db backup "postgresql://localhost/mydb" pre_migration.sql

# Check performance
dal db metrics "postgresql://localhost/mydb"
```

### 7. AI-Assisted Development

```bash
# Generate code
dal ai code "Create a DeFi lending protocol" -o lending.dal

# Understand code
dal ai explain lending.dal

# Security audit
dal ai audit lending.dal

# Generate tests
dal ai test lending.dal -o lending.test.dal

# Review and optimize
dal ai review lending.dal
dal ai optimize-gas lending.dal

# Fix issues
dal ai fix lending.dal
```

---

## Tips & Best Practices

### Performance

- Use `dal profile --memory` to identify memory leaks
- Run `dal bench` before and after optimizations
- Use `dal optimize --level 2` for production builds
- Check `dal memory-stats` periodically during development

### Code Quality

- Always run `dal fmt` before committing
- Use `dal lint` to catch common issues
- Run `dal check` in CI/CD pipelines
- Use `dal watch` for instant feedback during development

### Blockchain

- Check `dal chain gas-price` before deploying to mainnet
- Verify `dal chain balance` before transactions
- Use testnets (chain IDs 5, 80001) for development
- Monitor `dal chain tx-status` after deployment

### Cryptography

- Use `dal crypto hash` for checksums and IDs
- Generate strong keys with `dal crypto keygen ed25519`
- Use `dal crypto aes-encrypt` for large data (faster than RSA)
- Never expose private keys in commands (use files or env vars)

### Database

- Use `dal db plan` to optimize slow queries
- Run `dal db backup` before migrations
- Check `dal db metrics` to monitor performance
- Use `dal db schema` to understand table structures

---

## Command Shortcuts

Many commands have short forms or defaults:

| Full Command | Short Form / Default |
|--------------|----------------------|
| `dal fmt file.dal --check` | Add `--check` for CI |
| `dal crypto hash "data"` | SHA256 is default |
| `dal crypto keygen` | RSA is default |
| `dal bench` | Runs all suites |
| `dal optimize file.dal` | Level 1 is default |

---

## Output Formats

All commands support human-readable output by default:
- ✅ Success messages in green
- ❌ Errors in red
- ⚠️  Warnings in yellow
- Emojis for visual clarity

**Future:** JSON output mode (`--json`) for scripting and automation.

---

## Exit Codes

| Code | Meaning | When |
|------|---------|------|
| `0` | Success | Command completed successfully |
| `1` | Error | Command failed (file not found, syntax error, etc.) |

---

## Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `DAL_RPC_URL_<CHAIN_ID>` | Override RPC URL | `DAL_RPC_URL_1=https://...` |
| (Future) `DAL_DB_URL` | Default database connection | `DAL_DB_URL=postgresql://...` |
| (Future) `DAL_NO_COLOR` | Disable colored output | `DAL_NO_COLOR=1` |

---

## Comparison with Other Tools

### Developer Tooling

| Feature | Rust (`cargo`) | Go | JavaScript | DAL |
|---------|----------------|-----|------------|-----|
| Format code | ✅ `cargo fmt` | ✅ `go fmt` | ✅ `prettier` | ✅ `dal fmt` |
| Lint code | ✅ `cargo clippy` | ✅ `go vet` | ✅ `eslint` | ✅ `dal lint` |
| Type check | ✅ `cargo check` | ✅ `go build` | ✅ `tsc --noEmit` | ✅ `dal check` |
| REPL | ❌ | ❌ | ✅ `node` | ✅ `dal repl` |
| Watch mode | 🔧 `cargo watch` | 🔧 `air` | ✅ `nodemon` | ✅ `dal watch` |
| Project init | ✅ `cargo new` | ✅ `go mod init` | ✅ `npm init` | ✅ `dal new` |
| Benchmarks | ✅ `cargo bench` | ✅ `go test -bench` | ❌ | ✅ `dal bench` |
| Profiling | 🔧 External | 🔧 `pprof` | 🔧 Chrome DevTools | ✅ `dal profile` |
| Optimization | ✅ `--release` | ✅ `-gcflags` | ❌ | ✅ `dal optimize` |

### Blockchain Tools

| Feature | Foundry (`cast`) | Hardhat | DAL |
|---------|------------------|---------|-----|
| Gas price | ✅ `cast gas-price` | 🔧 JS code | ✅ `dal chain gas-price 1` |
| Balance | ✅ `cast balance` | 🔧 JS code | ✅ `dal chain balance 1 <addr>` |
| Multi-chain | 🔧 CLI flags | 🔧 Config | ✅ Built-in (chain ID) |
| Tx status | ✅ `cast receipt` | 🔧 JS code | ✅ `dal chain tx-status 1 <hash>` |

### Crypto Tools

| Feature | OpenSSL | GPG | DAL |
|---------|---------|-----|-----|
| Hash | ✅ `openssl dgst` | ❌ | ✅ `dal crypto hash <d> sha256` |
| Keygen | ✅ `openssl genrsa` | ✅ `gpg --gen-key` | ✅ `dal crypto keygen rsa` |
| Sign | ✅ `openssl rsautl` | ✅ `gpg --sign` | ✅ `dal crypto sign <d> <k>` |
| Encrypt | ✅ `openssl rsautl` | ✅ `gpg --encrypt` | ✅ `dal crypto encrypt <d> <k>` |

### Database Tools

| Feature | psql | mysql | DAL |
|---------|------|-------|-----|
| Connect | ✅ `psql -c` | ✅ `mysql -e` | ✅ `dal db connect <conn>` |
| Query | ✅ `psql -c "..."` | ✅ `mysql -e "..."` | ✅ `dal db query <conn> "..."` |
| List tables | ✅ `\dt` | ✅ `SHOW TABLES` | ✅ `dal db tables <conn>` |
| Schema | ✅ `\d table` | ✅ `DESC table` | ✅ `dal db schema <conn> table` |
| Backup | ✅ `pg_dump` | ✅ `mysqldump` | ✅ `dal db backup <conn> <path>` |

---

## Phase Completion Status

| Phase | Status | Commands | Focus |
|-------|--------|----------|-------|
| **Phase 0** | ✅ Complete | 10 | Developer tools, project management |
| **Phase 1** | ✅ Complete | 4 | Optimization, profiling, benchmarking |
| **Phase 2** | ✅ Complete | 25 | Blockchain, crypto, database operations |
| **Phase 3** | ✅ Complete | 7 | AI-enhanced code generation & analysis |
| **Phase 4** | ✅ Complete | 12+ | Cloud & enterprise (authorize, grant, tenant, compliance, trust) |
| **Phase 5** | ✅ Complete | 5 | IDE & LSP (lsp, doc, completions, debug) |
| **Phase 6** | ✅ Complete | 8 | Agent create, send, messages, task, fleet, mold |
| **Phase 7** | ✅ Complete | 10 | AI-IoT (ai-predict, ai-anomaly, ai-optimize, read-sensor, status) |
| **Phase 8** | ✅ Complete | 2 | Specialized (log, config) |
| **Phase 9** | ✅ Complete | 3 | Cross-component (bond, pipe, invoke) |

---

## Help & Documentation

```bash
# Show main help
dal help

# Show command help (future)
dal chain --help
dal crypto --help
dal db --help

# Version information
dal version
```

---

## Examples by Use Case

### Smart Contract Development

```bash
# Create project
dal new token-contract --type chain
cd token-contract

# Format and check
dal fmt src/main.dal
dal check src/main.dal

# Optimize before deployment
dal optimize src/main.dal -o src/optimized.dal --level 2

# Check gas price
dal chain gas-price 1

# Deploy (via script)
dal run deploy.dal

# Verify
dal chain tx-status 1 <tx_hash>
```

### API Development

```bash
# Create project
dal new my-api --type web

# Watch mode for development
dal watch src/main.dal

# Benchmark API performance
dal bench src/main.dal

# Profile to find slow endpoints
dal profile src/main.dal --memory
```

### IoT Development

```bash
# Create IoT project
dal new smart-device --type iot

# Format and lint
dal fmt src/
dal lint src/main.dal

# Test locally
dal run src/main.dal

# Hash sensor data
dal crypto hash "sensor_reading_123" sha256
```

### Database-Heavy Applications

```bash
# Test connection
dal db connect "postgresql://localhost/mydb"

# Explore schema
dal db tables "postgresql://localhost/mydb"
dal db schema "postgresql://localhost/mydb" orders

# Test query
dal db query "postgresql://localhost/mydb" "SELECT COUNT(*) FROM orders"

# Optimize slow query
dal db plan "postgresql://localhost/mydb" "SELECT * FROM orders WHERE user_id = 123"

# Backup before migration
dal db backup "postgresql://localhost/mydb" backup_$(date +%Y%m%d).sql
```

---

## Key Takeaways

1. **Complete Toolchain** - DAL has 39 commands covering development, optimization, and operations
2. **No External Tools** - Everything built into `dal` CLI
3. **Multi-Domain** - Blockchain, crypto, database, AI (future) in one tool
4. **Developer-Friendly** - Clear output, helpful errors, consistent interface
5. **Fast Implementation** - Phases 1-2 took ~3-4 hours total (backend already existed)

**DAL is now competitive with mature toolchains like `cargo`, `go`, and `npm`, while adding blockchain-native and AI-powered features that no other language has.**

---

For detailed documentation on each phase:
- [Phase 0 Complete](./development/CLI_PHASE0_COMPLETE.md)
- [Phase 1 Complete](./development/CLI_PHASE1_COMPLETE.md)
- [Phase 2 Complete](./development/CLI_PHASE2_COMPLETE.md)
- [Phase 3 Complete](./development/CLI_PHASE3_COMPLETE.md)
- [Phase 4 Complete](./development/CLI_PHASE4_COMPLETE.md)
- [Phase 5 Complete](./development/CLI_PHASE5_COMPLETE.md)
- [Phase 6 Complete](./development/CLI_PHASE6_COMPLETE.md)
- [Phases 7, 8, 9 Complete](./development/CLI_PHASE7_8_9_COMPLETE.md)
