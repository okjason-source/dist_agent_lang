# Vibes Job. DAL CEO

Personal agent you interact with via **web UI**, **iMessage**, **Telegram**, **Discord**, or the **CLI**. Only **your** messages reach the agent; everyone else is ignored. Built with [DAL](https://github.com/okjason-source/dist_agent_lang) (dist_agent_lang).

The agent **remembers conversations** using DAL's evolve system — every exchange and action is logged to `evolve.md` and fed back as context on the next turn. (Evolve = agent memory; `history.jsonl` = UI history. See `docs/MEMORY_AND_AGENTS.md`.) It can **chat**, **run shell commands**, **search the web**, **make recommendations**, and **generate or run code**.

## What's new in v0.2.0

- **Memory** — conversations and actions persist in `evolve.md` via `evolve::append_conversation` and `evolve::append_log`. The LLM sees recent context on every turn.
- **Context trimming** — `evolve::trim_retention` keeps `evolve.md` bounded (default 200 lines). Old entries are pruned automatically.
- **Context summarization** — `POST /api/context/summary` uses the LLM to compress context into a summary.
- **Communications skill** — `communications.dal` + `skills/communications.skill.dal` make messaging a first-class DAL skill. Outbound sends via `/api/send` and `/api/reply`. Inbound via the bridge.
- **Separate venv** — DAL CEO has its own DAL venv (`relaxed` profile, full stdlib access).
- **Fixed config** — `agent.toml` no longer has duplicate sections; shell is `trusted` as intended.
- **Utility scripts** — `check-env.sh` and `test-openai-key.sh` are included.
- **Cleaner agent.dal** — removed invalid dotenv import; loads evolve context on spawn.

## Quick start

1. **Configure AI**

   Copy `.env.example` to `.env` and add your API key:

   ```bash
   cp .env.example .env
   # Edit .env — set OPENAI_API_KEY=sk-...
   ```

2. **Start the server**

   ```bash
   ./start.sh
   ```

   Profile-based startup for real-agent rollout testing:

   ```bash
   ./start.sh --profile safe
   ./start.sh --profile fallback
   ./start.sh --profile ramp
   ```

   Real-world process controls:

   ```bash
   ./status.sh
   ./stop.sh
   DAL_CEO_PORT=4041 ./start.sh
   DAL_CEO_REPLACE_RUNNING=1 ./start.sh
   ```

   Initialization mode controls whether `agent.dal` is run on startup:

   ```bash
   DAL_CEO_INIT_MODE=if_missing ./start.sh   # default
   DAL_CEO_INIT_MODE=always ./start.sh
   DAL_CEO_INIT_MODE=never ./start.sh
   ```

   The profile applies sensible defaults only when a var is unset in your environment or `.env`, so explicit overrides still win.

   Or manually:

   ```bash
   export $(grep -v '^#' .env | xargs)
   dal serve server.dal --port 4040
   ```

3. **Open the web UI** — http://localhost:4040  
   The UI includes **Chat** (main area), plus side panels: **History**, **Task**, and **Workflow**. The UI is **built in DAL** (`ui.dal`): HTML, CSS, and JS are assembled there; optional `static/` files override the built-in content when present.

4. **CLI quick test** (no server needed):

   ```bash
   export $(grep -v '^#' .env | xargs)
   dal run main.dal
   ```

5. **Fleets (optional)** — group agents and record deploy/run intent from this directory (writes `.dal/fleets.json`, gitignored):

   ```bash
   dal agent fleet create assistant-workers --from-mold mold/assistant.mold.dal --count 2
   dal agent fleet deploy assistant-workers "Your task description"
   dal agent fleet run assistant-workers
   ```

   Full walkthrough: [FLEET_DEPLOYMENT.md](../docs/FLEET_DEPLOYMENT.md) (uses `mold/assistant.mold.dal` in this folder).

6. **Docker — clean GUI** (web UI only, no extra stack):

   From the **repo root** (dist_agent_lang):

   ```bash
   docker build -f Vibes_Job/Dockerfile -t agent-assistant-gui .
   docker run -p 4040:4040 -e OPENAI_API_KEY=yourkey agent-assistant-gui
   ```

   Open http://localhost:4040 for the assistant UI.

## API endpoints

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/` | Chat web UI |
| POST | `/api/message` | Send message, get AI reply with context. Body: `{"content":"...", "sender_id":"web", "policy":"auto|reply_only|tool_loop"}` (`policy` optional; if omitted, uses `DAL_AGENT_POLICY_DEFAULT`, runtime default `auto`). Response includes `reply` plus diagnostics: `route`, `policy`, `tool_steps_count`, `max_steps_reached`, `tool_trace`, `last_tool_names`, `termination_reason`, `guard_stopped`, and nested `observability` (same contract as `ai::agent_run`). |
| POST | `/api/task` | Execute a task. Body: `{"description":"...", "policy":"auto|reply_only|tool_loop"}` (`policy` optional; if omitted, uses `DAL_AGENT_POLICY_DEFAULT`). Response includes `ok`, `result`, and the same diagnostics as `/api/message` (`route`, `policy`, `termination_reason`, `guard_stopped`, `tool_trace`, `observability`, etc.). |
| POST | `/api/send` | Send outbound message. Body: `{"channel":"telegram", "recipient":"chat_id", "message":"..."}` |
| POST | `/api/reply` | Reply through originating channel. Body: `{"sender_id":"telegram_123", "message":"..."}` |
| GET | `/api/history` | Retrieve conversation history (last 50 entries) |
| POST | `/api/history/clear` | Clear all conversation history |
| GET | `/api/agents` | List active sub-agents (researcher, coder, reviewer) |
| POST | `/api/agents/run` | Run a task with a specific agent. Body: `{"role":"researcher", "task":"..."}`. Result payload matches `agents::run_with_agent`: `result` plus `route`, `policy`, `termination_reason`, `guard_stopped`, `tool_trace`, `last_tool_names`, `observability`. |
| GET | `/api/workflow/list` | List available workflows (built-in + custom) |
| POST | `/api/workflow` | Execute a workflow. Body: `{"workflow":"research_and_summarize", "input":"..."}` |
| POST | `/api/workflow/define` | Save a custom workflow definition |
| POST | `/api/workflow/delete` | Delete a custom workflow. Body: `{"name":"..."}` |
| GET | `/api/context` | View current agent memory (evolve context) |
| POST | `/api/context/summary` | Summarize and compress context using LLM |
| POST | `/api/preference` | Append a learned preference. Body: `{"preference":"User prefers bullet points."}` |
| POST | `/api/feedback` | Record feedback for reinforcement. Body: `{"rating":1 or "up" or -1 or "down", "comment":"optional"}` |
| GET | `/api/status` | Agent status: server metrics, agent config, runtime info |
| GET | `/api/ai-status` | LLM connection check (`connected: true/false`) |
| GET | `/api/diag` | Diagnostics: AI status, placeholder detection |
| GET | `/api/debug-ai` | Raw `ai::generate_text` test (500 on failure) |
| GET | `/health` | Liveness check |

### Real-agent smoke checks

Use this after startup to validate the host-protocol wiring and diagnostic fields without switching to a fake test harness:

```bash
./scripts/smoke-host-protocol.sh
```

Optional custom base URL:

```bash
./scripts/smoke-host-protocol.sh http://localhost:4040
```

### Production hardening toggles

Set these env vars for safer rollout/rollback in production:

- `DAL_AGENT_POLICY_DEFAULT=auto|reply_only|tool_loop`
- `DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=1|0`
- `DAL_AGENT_GUARDS_STRICT_MODE=1|0`

Optional hard caps:

- `DAL_AGENT_MAX_WALL_CLOCK_MS`
- `DAL_AGENT_MAX_TOOL_CALLS_PER_TYPE`
- `DAL_AGENT_MAX_REPEATED_IDENTICAL_INVOCATIONS`
- `DAL_AGENT_MAX_CONSECUTIVE_NO_PROGRESS`
- `DAL_AGENT_MAX_TOTAL_TOKENS`
- `DAL_AGENT_MAX_COST_MICROUSD`

## How memory works

Every message and task goes through this flow:

1. **Load** — `evolve::load()` reads `evolve.md` (recent conversations and action log).
2. **Enrich** — Context is prepended to the user's message before calling `ai::respond_with_tools_result`.
3. **Respond** — The runtime executes host tools through the provider API path and returns a natural-language final reply plus optional diagnostics.
4. **Log** — `evolve::append_conversation(user_msg, reply)` and `evolve::append_log(action, detail, result)` persist the exchange.
5. **Trim** — `evolve::trim_retention(200)` prunes old lines to keep context bounded.

To manually summarize context (compresses old entries into a paragraph):

```bash
curl -X POST http://localhost:4040/api/context/summary
```

## Personality and learning

You can personalize the assistant and have it learn from feedback (see [docs/PERSONALITY_AND_LEARNING_PLAN.md](docs/PERSONALITY_AND_LEARNING_PLAN.md)):

- **Personality** — Create `personality.md` in the project root (or copy from `personality.md.example`). The agent sees this on every turn. Edit it to set tone and style (e.g. “You are a calm, concise technical assistant…”).
- **Learned preferences** — Create `learned_preferences.md` and add lines (e.g. “User prefers bullet points.”), or call `POST /api/preference` with `{"preference": "..."}`. The agent sees these as “Learned preferences (follow these).”
- **Reinforcement** — Call `POST /api/feedback` with `{"rating": 1}` or `{"rating": "up"}` (positive) or `-1`/`"down"` (negative), and optional `"comment"`. Entries are stored in `feedback.jsonl` and a short reinforcement line is added to the prompt so the agent favors what you rate positively. Avoid unescaped single quotes in preference and comment text.

All of this is implemented in DAL (`server.dal`, `evolve::`, `sh::`, `json::`).

## Explore and exploit (skills and environment)

The assistant is prompted to **explore** (use `run` and `search` to discover commands and files, e.g. `run "ls"`, `run "cat file"`) and **exploit** (reuse patterns that worked before, using recent memory). This uses dist_agent_lang patterns: encouragement block + meta-from-memory style guidance. Skills in `skills/*.skill.dal` describe what the agent can do and are injected into the prompt. For full tool set (read_file, list_dir, dal_check, dal_run) and automatic tool→evolve logging, use `dal agent serve` with a mold that has the development skill. See [docs/EXPLORE_EXPLOIT_AND_DAL.md](docs/EXPLORE_EXPLOIT_AND_DAL.md).

## Conversation history

Every message and task is also persisted to `history.jsonl` — a JSONL file (one JSON object per line) that survives server restarts. Unlike evolve context (which is trimmed and summarized), history is the full record.

```bash
# View recent history
curl http://localhost:4040/api/history | python3 -m json.tool

# Clear all history
curl -X POST http://localhost:4040/api/history/clear
```

Each entry contains:

```json
{"ts":"2026-03-05T04:30:00Z","type":"message","sender":"web","content":"list files","reply":"..."}
```

History is capped at 500 entries (oldest pruned automatically). The file is gitignored.

## Multi-agent coordination

The agent can spawn specialized sub-agents for complex tasks:

| Agent | Role | Strengths |
|-------|------|-----------|
| **researcher** | Search, gather information, compile findings | Web search, data collection |
| **coder** | Write code, run scripts, create files | Code generation, testing |
| **reviewer** | Check quality, verify results, summarize | Analysis, feedback, reports |

Agents are spawned on demand and reused. Use `agent::coordinate` for task assignment and `agent::communicate` for inter-agent messaging.

```bash
# Run a task with a specific agent
curl -X POST http://localhost:4040/api/agents/run \
  -H 'Content-Type: application/json' \
  -d '{"role":"researcher","task":"find the latest DAL release notes"}'

# List active agents
curl http://localhost:4040/api/agents
```

## Workflows

Workflows chain agents together for multi-step tasks. Each step runs on a specific agent and passes its output to the next.

### Built-in workflows

| Workflow | Steps | Description |
|----------|-------|-------------|
| `research_and_summarize` | researcher -> reviewer | Research a topic and produce a summary report |
| `code_and_review` | coder -> reviewer | Write code and review it for quality |
| `deep_research` | researcher -> coder -> reviewer | Full pipeline: search, gather data, compile report |

```bash
# Run a workflow
curl -X POST http://localhost:4040/api/workflow \
  -H 'Content-Type: application/json' \
  -d '{"workflow":"research_and_summarize","input":"current state of AI agent frameworks"}'

# List all workflows (built-in + custom)
curl http://localhost:4040/api/workflow/list
```

### Custom workflows

Define your own step chains and save them by name. Each step specifies a `role` (researcher, coder, or reviewer) and a `prompt` template. Two placeholders are available:

- `{input}` — the original user input passed when running the workflow
- `{prev}` — the output from the previous step (empty for the first step)

**Define a custom workflow:**

```bash
curl -X POST http://localhost:4040/api/workflow/define \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "write_and_test",
    "description": "Write code, then write tests for it",
    "steps": [
      {"role": "coder", "prompt": "Write clean, production-ready code for: {input}"},
      {"role": "coder", "prompt": "Write comprehensive unit tests for the following code:\n\n{prev}"},
      {"role": "reviewer", "prompt": "Review the code and tests below. Check for coverage gaps, edge cases, and correctness.\n\n{prev}"}
    ]
  }'
```

**Run a custom workflow** (same endpoint as built-ins):

```bash
curl -X POST http://localhost:4040/api/workflow \
  -H 'Content-Type: application/json' \
  -d '{"workflow":"write_and_test","input":"a rate limiter in Python using token bucket"}'
```

**Delete a custom workflow:**

```bash
curl -X POST http://localhost:4040/api/workflow/delete \
  -H 'Content-Type: application/json' \
  -d '{"name":"write_and_test"}'
```

Custom workflows are persisted to `custom_workflows.json` and survive server restarts. The `GET /api/workflow/list` endpoint returns both built-in and custom workflows, with a `custom: true` flag on user-defined ones.

### Workflow response format

The response includes each step's output and the `final_result`:

```json
{
  "ok": true,
  "workflow": "research_and_summarize",
  "steps": [
    {"role": "researcher", "result": "...findings..."},
    {"role": "reviewer", "result": "...summary..."}
  ],
  "final_result": "...summary..."
}
```

Custom workflow responses also include `"custom": true` and `"step_count"` fields.

## Project layout

| File / folder | Purpose |
|---------------|---------|
| `dal.toml` | DAL package manifest (`agent-assistant` v0.2.0) |
| `agent.toml` | Agent config: trusted shell, evolve context path, AI provider, skills path |
| `evolve.md` | Agent memory — recent context for LLM (auto-managed, trimmed) |
| `history.jsonl` | Full conversation history (JSONL, persists across restarts, gitignored) |
| `server.dal` | HTTP server: routes, memory helpers, chat UI |
| `communications.dal` | Outbound messaging module (Telegram, Discord, iMessage via `sh::run`) |
| `agents.dal` | Multi-agent module — spawn and coordinate researcher, coder, reviewer |
| `workflows.dal` | Workflow engine — chain agents for multi-step tasks |
| `skills/` | DAL skill definitions (auto-discovered by runtime) |
| `skills/communications.skill.dal` | Communications skill — tells the agent it can send/receive across channels |
| `skills/multi_agent.skill.dal` | Multi-agent skill — coordinate sub-agents and run workflows |
| `agent.dal` | Agent behavior for `dal agent serve` |
| `main.dal` | CLI entry point for quick testing |
| `start.sh` | Load `.env` and start the server |
| `check-env.sh` | Verify environment variables are set |
| `test-openai-key.sh` | Smoke-test your OpenAI API key |
| `bridge/` | Node.js inbound listener for iMessage, Telegram, Discord |
| `.dal/` | DAL runtime state (generated, gitignored) |

## Communications skill

Communications is a registered DAL skill (`skills/communications.skill.dal`). The architecture:

- **Inbound** — `bridge/index.js` listens for messages on Telegram/Discord/iMessage, filters by your IDs, and POSTs to `/api/message` with a `sender_id` like `telegram_123456`, `discord_789`, or `imessage_+15551234567`.
- **Outbound** — `communications.dal` sends messages via `sh::run` + curl (Telegram/Discord APIs) or osascript (iMessage). The server exposes `/api/send` and `/api/reply`.
- **Skill definition** — Registered in the agent's prompt so the LLM knows it can send messages across channels.

Send a message programmatically:

```bash
# Direct send to a channel
curl -X POST http://localhost:4040/api/send \
  -H 'Content-Type: application/json' \
  -d '{"channel":"telegram","recipient":"YOUR_CHAT_ID","message":"Hello from the agent"}'

# Reply through the originating channel
curl -X POST http://localhost:4040/api/reply \
  -H 'Content-Type: application/json' \
  -d '{"sender_id":"telegram_123456","message":"Task complete."}'
```

Required env vars for outbound (add to `.env`):

| Variable | Purpose |
|----------|---------|
| `TELEGRAM_BOT_TOKEN` | Telegram Bot API token (from @BotFather) |
| `DISCORD_WEBHOOK_URL` | Discord webhook URL for outbound messages |

iMessage outbound uses osascript (macOS only, no extra config needed).

## DAL features used

| Module | Functions | Purpose |
|--------|-----------|---------|
| `ai` | `generate_text`, `respond_with_tools` | LLM text generation and tool-calling |
| `evolve` | `load`, `append_conversation`, `append_log`, `append_summary`, `trim_retention` | Persistent memory |
| `agent` | `spawn`, `set_serve_agent`, `get_status`, `coordinate`, `communicate` | Agent lifecycle, coordination, inter-agent messaging |
| `sh` | `run` | Shell command execution (trusted) + outbound messaging |
| `json` | `parse`, `stringify` | JSON handling |
| `config` | `get_env`, `get_ai_config` | Environment and AI config validation |
| `log` | `info`, `audit` | Structured logging — info for events, audit for sensitive actions |
| skills | `communications.skill.dal`, `multi_agent.skill.dal` | Registered skills for messaging and multi-agent coordination |

## What the agent can do

- **Chat** — Ask questions or give directions; the LLM replies with context from past interactions.
- **Run shell commands** — "run ls -la" or "list files" triggers `sh::run(cmd)` (gated by `[agent.sh] trust` in `agent.toml`).
- **Search the web** — "search for X" triggers DuckDuckGo search.
- **Remember** — The agent recalls previous conversations and actions via evolve context.
- **Send messages** — Send outbound messages to Telegram, Discord, or iMessage via the communications skill.
- **Code** — Ask it to write scripts or run code.

## Messaging (iMessage, Telegram, Discord)

**Inbound** — the bridge listens and forwards your messages to the agent.

Set **your** identity per channel so only your messages reach the agent:

```bash
cd bridge
npm install

# Set your IDs (required per channel):
export ALLOWED_TELEGRAM_IDS=123456789
export ALLOWED_DISCORD_IDS=123456789012345678
export ALLOWED_IMESSAGE_HANDLES=+15551234567

# Set bot tokens:
export TELEGRAM_BOT_TOKEN=...
export DISCORD_BOT_TOKEN=...

npm start
```

Run one channel only: `npm run imessage`, `npm run telegram`, or `npm run discord`.

iMessage requires macOS and Full Disk Access for Terminal. Install the optional dependency: `npm install osa-imessage`.

## Venv

`agent_assistant` runs in its own DAL venv (`relaxed` profile):

```bash
dal venv list          # see registered venvs
dal venv show agent_assistant
```

## Trust and shell

- `agent.toml` sets `[agent.sh] trust = "trusted"` — the agent can run any shell command.
- The server runs **locally**. The bridge only forwards **your** messages.

## AI provider

Set one in `.env`:

- **OpenAI:** `OPENAI_API_KEY=sk-...` and optionally `OPENAI_MODEL=gpt-4o`
- **Anthropic:** `ANTHROPIC_API_KEY=sk-ant-...`

`agent.toml` defaults to `provider = "openai"`, `model = "gpt-4o"`.

On startup the server validates config via `config::get_ai_config()` and `config::get_env("OPENAI_API_KEY")`, logging the result:

```
[INFO] agent-assistant: Starting v0.2.0
[INFO] config: AI model: gpt-4o
[INFO] config: OPENAI_API_KEY is set
```

## Logging

The server emits structured log lines to stdout:

- **`log::info`** — normal events: startup, messages received/replied, tasks started/completed
- **`log::audit`** — sensitive actions: outbound sends, channel replies

Examples:

```
[INFO] message: from:web len:42
[INFO] message: replied to:web len:256
[INFO] task: started: summarize my emails
[INFO] task: completed: summarize my emails
[AUDIT] send: channel:telegram recipient:123456 len:85
[AUDIT] comms:telegram: sent to:123456
[AUDIT] reply: to:discord_789 len:120
```

Pipe to a file for persistence: `./start.sh 2>&1 | tee agent.log`

## Agent status

`GET /api/status` returns a combined view of server metrics and agent runtime state:

```bash
curl http://localhost:4040/api/status | python3 -m json.tool
```

```json
{
  "ok": true,
  "version": "0.2.0",
  "server": {
    "messages_handled": 12,
    "tasks_completed": 3,
    "outbound_sends": 1,
    "errors": 0
  },
  "agent_count": 1,
  "serve_agent": {
    "agent_id": "agent_177...",
    "name": "agent-assistant",
    "type": "ai",
    "role": "Personal assistant — chat, shell commands, web search, code, messaging",
    "status": "idle",
    "skills": ["development", "creative", "office", "communications"],
    "trust_score": 1.0,
    "metrics": {
      "tasks_completed": 0,
      "messages_sent": 0,
      "uptime_percentage": 100.0
    }
  },
  "context_length": 1842
}
```

Server metrics (messages, tasks, sends, errors) are in-memory counters that reset on restart. Agent runtime metrics persist in `.dal/agent_runtime.json`.

## Troubleshooting

1. **Check env:** `./check-env.sh`
2. **Test key:** `./test-openai-key.sh`
3. **Status stays red:** Stop server, run `./start.sh` (loads `.env` into the process).
4. **Placeholder responses:** Rebuild DAL with `--release` (default features), restart with `./start.sh`.
5. **"AI access denied":** Visit http://localhost:4040/api/debug-ai for the exact error.
6. **Browser cache:** Hard refresh (Cmd+Shift+R).

## Requirements

- **DAL** v1.0.8+ (`cargo install --git https://github.com/okjason-source/dist_agent_lang.git dist_agent_lang --bin dal`)
- **Node.js** 18+ (for the bridge only)

## Future

- Improved web UI (extract inline HTML, history display, task panel, workflow runner)
- Agent templates (`mold::load`, `mold::spawn_from`)
- Scheduled / recurring tasks
- Agent memory sharing (agents reading each other's evolve context)
