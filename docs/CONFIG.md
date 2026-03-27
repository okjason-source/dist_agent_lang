# Configuration and environment variables

**Purpose:** Single index of commonly used environment variables. For agent behavior and trust in depth, see [guides/AGENT_SETUP_AND_USAGE.md](guides/AGENT_SETUP_AND_USAGE.md). For IDE backend guardrails when hosted, see `dal-ide/README.md` (agent guardrails section).

| Area | Variable(s) | Notes |
|------|----------------|------|
| **Process logging (Rust `tracing`)** | `RUST_LOG`, `DAL_LOG_FORMAT`, `DAL_METRICS` | `RUST_LOG` — standard filter (e.g. `info`, `dal_http=debug`). `DAL_LOG_FORMAT=json` — JSON lines to stderr. `DAL_METRICS=1` — expose `GET /metrics` (Prometheus text counters) on HTTP servers that ship observability (`dal web`, `dal ide serve`, `dal agent serve`, `dal-registry`, `start_http_server`). |
| **LLM / AI** | `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `DAL_AI_PROVIDER`, `DAL_AI_ENDPOINT`, `DAL_AI_MODEL`, `DAL_AI_TEMPERATURE`, … | Provider-specific; see [guides/AI_FEATURES_GUIDE.md](guides/AI_FEATURES_GUIDE.md), [CUSTOM_AI_PROVIDERS.md](CUSTOM_AI_PROVIDERS.md). |
| **Agent serve / IDE agent loop** | `DAL_AGENT_*` (e.g. `DAL_AGENT_MAX_TOOL_STEPS`, `DAL_AGENT_SHELL_TRUST`, `DAL_AGENT_CONTEXT_PATH`, `DAL_AGENT_GUARDS_STRICT_MODE`, runtime persistence) | [AGENT_SETUP_AND_USAGE.md](guides/AGENT_SETUP_AND_USAGE.md). |
| **Hosted IDE agent safety** | `DAL_AGENT_MAX_TOOL_STEPS`, `DAL_AGENT_MAX_WALL_CLOCK_MS`, `DAL_AGENT_MAX_TOTAL_TOKENS`, `DAL_AGENT_MAX_COST_MICROUSD`, `DAL_AGENT_GUARDS_STRICT_MODE`, … | Listed in [PRODUCTION_ROADMAP.md](PRODUCTION_ROADMAP.md) and `dal-ide/README.md`. |
| **HTTP server auth** | `JWT_SECRET` | Required when JWT auth is used; enforced in `http_server_security`. See [SECURITY.md](../SECURITY.md). |
| **Trust-split / compile** | `DAL_COMPILE_TRUST_MODE`, chain RPC / strict policy vars | See [TRUST_SPLIT_EVM_HARDENING_REFACTOR_PLAN.md](development/implementation/TRUST_SPLIT_EVM_HARDENING_REFACTOR_PLAN.md), [SMART_CONTRACTS_WITH_DAL_REVIEW.md](SMART_CONTRACTS_WITH_DAL_REVIEW.md). |

