# Agent setup and usage guide

This guide describes how to set up and use agents in dist_agent_lang: project setup, CLI commands, the agent HTTP server, DAL APIs, molds, and evolve context. It reflects what users and developers can do today.

---

## Table of contents

1. [Overview](#1-overview)
2. [Quick start: agent project](#2-quick-start-agent-project)
3. [Project layout and config](#3-project-layout-and-config)
4. [CLI: agent commands](#4-cli-agent-commands)
5. [Agent HTTP server](#5-agent-http-server)
6. [DAL: agent and evolve APIs](#6-dal-agent-and-evolve-apis)
7. [Molds](#7-molds)
8. [Shell trust and evolve context](#8-shell-trust-and-evolve-context)
9. [Persistent memory](#9-persistent-memory)
10. [Skills and registry](#10-skills-and-registry)
11. [Capabilities and types](#11-capabilities-and-types)
12. [References](#12-references)

---

## 1. Overview

Agents in dist_agent_lang are in-process entities with:

- **Types**: `ai`, `system`, `worker`, or `custom:<name>`
- **Lifecycle**: spawn, coordinate (tasks), communicate (messages), evolve (learning)
- **Serving**: One agent per process can be exposed via HTTP with `dal agent serve`

You can:

- **Initialize** an agent project with `dal init agent` (adds `agent.dal`, `agent.toml`, `evolve.md`, etc.).
- **Run** agent logic with `dal run agent.dal` or run the **HTTP server** with `dal agent serve`.
- **Create** agents from the CLI (`dal agent create <type> <name>`) or from DAL (`agent::spawn(config)`).
- **Send messages and assign tasks** via CLI or HTTP, and use **molds** to spawn agents from reusable configs (local or IPFS/on-chain).
- **Persist conversation and action history** in an **evolve** markdown file and control **shell execution** with `[agent.sh]` in `agent.toml`.

---

## 2. Quick start: agent project

### Create an agent project

```bash
mkdir my-agent && cd my-agent
dal init agent
```

This creates (only if missing):

- `dal.toml` — minimal package config
- `agent.toml` — agent config: `[agent.sh]` trust, `[agent]` context_path
- `agent.dal` — entry script that spawns an agent and sets it as the serve agent
- `evolve.md` — evolve context (conversation + action log)
- `playground.dal` — small language sandbox (`dal run playground.dal`)
- `README.md` — short project summary
- `.env.example` and `.env` — env vars (`.env` in `.gitignore`)

### Run the agent script

```bash
dal run agent.dal
```

This runs `agent.dal`: it spawns one agent and calls `agent::set_serve_agent(agent_id)`. The process then exits; the “serve” designation is used when you start the HTTP server with a behavior script.

### Run the agent HTTP server

```bash
dal agent serve
```

If `agent.dal` exists in the current directory, it is run first; the script must spawn an agent and call `agent::set_serve_agent(agent_id)`. That agent is then served at `http://localhost:4040` (default port). See [Agent HTTP server](#5-agent-http-server).

### Optional: environment

Copy `.env.example` to `.env`, set any keys (e.g. `OPENAI_API_KEY`, `ANTHROPIC_API_KEY` for AI), and load them (e.g. `export $(cat .env | xargs)` or use a .env loader). See [Shell trust and evolve context](#8-shell-trust-and-evolve-context).

### Host protocol API note

For DAL apps, prefer the typed request shape:

```dal
let ai_result = ai::agent_run({"message": "Run pwd once and summarize", "policy": "tool_loop"});
```

`ai::respond_with_tools_result(...)` remains available for compatibility, but new first-party examples and templates use `ai::agent_run(...)`.

For `dal init agent`, the generated `.env` includes a minimal host-protocol profile intended for task completion with safety guardrails:

```bash
DAL_AGENT_SHELL_TRUST=sandboxed
DAL_AGENT_CONTEXT_PATH=./evolve.md
DAL_AGENT_POLICY_DEFAULT=auto
DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=1
DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=0
DAL_AGENT_GUARDS_STRICT_MODE=1
```

- `DAL_AGENT_POLICY_DEFAULT=auto` favors execution for actionable requests and direct replies for purely conversational prompts.
- `DAL_AGENT_GUARDS_STRICT_MODE=1` keeps conservative loop safeguards enabled for fresh projects.
- If you want chat-only behavior, set `DAL_AGENT_POLICY_DEFAULT=reply_only`.

---

## 3. Project layout and config

### Files created by `dal init agent`

| File | Purpose |
|------|--------|
| `agent.dal` | Agent behavior: spawn agent, call `agent::set_serve_agent(agent_id)`. Used by `dal run agent.dal` and by `dal agent serve` when no `--behavior` is given. |
| `agent.toml` | Agent config: `[agent.sh]` (shell trust), `[agent]` (e.g. `context_path` for evolve). |
| `evolve.md` | Evolve context file: conversation history and action log. Path set by `[agent] context_path` or `DAL_AGENT_CONTEXT_PATH`. |
| `playground.dal` | Minimal DAL language sandbox for quick experimentation (`dal run playground.dal`). |
| `dal.toml` | Minimal DAL package (created only if missing). |
| `.env.example` | Documented env vars (safe to commit). |
| `.env` | Local overrides (do not commit; in `.gitignore`). |

### agent.toml

```toml
# Agent project config

[agent.sh]
trust = "sandboxed"
# forbidden_patterns = ["rm -rf", "sudo"]
# allowed_prefixes = ["npm", "cargo", "git"]

[agent]
context_path = "./evolve.md"
```

- **`[agent.sh]`** — Used by `sh::run(cmd)` in DAL: `trust` = `off` \| `sandboxed` \| `confirmed` \| `trusted`; optional `forbidden_patterns` / `allowed_prefixes`.
- **`[agent]`** — `context_path`: path to the evolve context file (default `./evolve.md`).

Context path can be overridden with `DAL_AGENT_CONTEXT_PATH`.

**Evolve wired at init:** When you run `dal init agent`, evolve is wired by default: `agent.toml` gets `context_path = "./evolve.md"` and `evolve.md` is created. The serve path loads evolve into the prompt each turn; lifecycle hooks (e.g. in molds) can use `evolve::append_log`, `evolve::append_summary`, `evolve::append_conversation`. To **disable** evolve, comment out or remove the `context_path` line in `agent.toml` (and optionally remove `evolve.md`). To **opt in** again, uncomment `context_path` and ensure the evolve file exists.

---

## 4. CLI: agent commands

All agent subcommands: `dal agent <subcommand> [args...]`.

### dal agent serve

Run the agent HTTP server (one agent per process).

```bash
dal agent serve [name] [--port PORT] [--mold path] [--behavior path] [--prompt-only]
```

- **name** — Agent name used when not using a behavior script (default: `serve_agent`).
- **--port** — Port (default: 4040).
- **--mold** — Mold path or `ipfs://<cid>` to spawn from mold instead of default config.
- **--behavior** — DAL file to run; script must spawn an agent and call `agent::set_serve_agent(agent_id)`. If omitted and `agent.dal` exists in cwd, `agent.dal` is used.
- **--prompt-only** — No behavior script; spawn a default worker agent and respond to each message via AI (LLM) and post the reply. Set `DAL_AGENT_PROMPT_ONLY=1` for the same effect.

Examples:

```bash
dal agent serve
dal agent serve my-bot --port 5000
dal agent serve --behavior ./scripts/agent.dal
dal agent serve --prompt-only
```

### dal agent create

Create an agent in the current process (in-memory; process exit clears it).

**By type and name:**

```bash
dal agent create <type> <name> [--role "role"]
```

Types: `ai`, `system`, `worker`, `custom:<name>`.

**From a mold:**

```bash
dal agent create --mold <path|ipfs://cid> <name>
```

With web3: `--mold <numeric_mold_id>` uses on-chain registry (pay fee, then load from IPFS).

### dal agent send

Send a message between two agent IDs (same process).

```bash
dal agent send <sender_id> <receiver_id> "<message>"
```

### dal agent messages

Print messages received by an agent (consumes the queue).

```bash
dal agent messages <agent_id>
```

### dal agent task assign / list

Assign a task or list pending tasks:

```bash
dal agent task assign <agent_id> "<description>" [--priority low|medium|high|critical]
dal agent task list <agent_id>
```

### dal agent chat

Interactive chat with an AI agent (same process): messages go to the agent; replies come from the LLM (OpenAI/Anthropic/local). Requires API keys or `DAL_AI_ENDPOINT`.

```bash
dal agent chat [name]
```

### dal agent list

Prints how to run chat, serve, and multi-agent DAL patterns (agent state is process-local).

### dal agent fleet

Off-chain fleet: named set of agents, optionally created from a mold. See [FLEET_DEPLOYMENT.md](../FLEET_DEPLOYMENT.md) for full deployment flow.

- **create** &lt;name&gt; — Create an empty fleet. Use `--from-mold <path> [--count N] [--param k=v ...]` to create a fleet of N agents from a mold (default count 1).
- **list** [**-v** \| **--verbose**] — List fleet names and agent counts; with `-v` also shows last_deployed_task and last_deployed_at.
- **show** &lt;name&gt; — Show fleet details: mold path (if any), member count, agent IDs, last deployed task (if set).
- **scale** &lt;name&gt; &lt;N&gt; — Resize fleet to N members. Scale down truncates the member list; scale up spawns more from the same mold (uses stored `last_create_params` if set).
- **delete** &lt;name&gt; — Remove the fleet (metadata only; agent contexts remain).
- **deploy** &lt;name&gt; &lt;task&gt; — Record the task as the fleet’s last deployment. Use **run** to dispatch it to members.
- **add-from-mold** &lt;name&gt; &lt;mold_source&gt; &lt;count&gt; [--param k=v ...] — Add N agents from a mold to an existing fleet (sets mold_source if fleet was empty).
- **add-member** &lt;name&gt; &lt;agent_id&gt; — Register an existing agent as a fleet member.
- **run** [&lt;name&gt;] — For each fleet with a deployed task (optionally filter by name), ensure members (add from mold if empty), then dispatch last_deployed_task to each member via agent coordination.
- **health** &lt;name&gt; — Report member count, has_mold, last_deployed_task/at, status (ok/empty).
- **export** [&lt;name&gt;] [**--format** k8s\|docker-compose] — Emit YAML (Kubernetes JobList or docker-compose services) for the fleet(s).

Fleet state is stored in `base/.dal/fleets.json` when using the CLI (current working directory as base).

### dal agent mold

- **list** — List local mold paths (`.`, `mold/`, `mold/samples`; `*.mold.dal`, `*.mold.json`).
- **show &lt;path-or-name&gt;** — Print mold name, version, agent type, role, capabilities, trust, memory, flags.
- **create &lt;name&gt;** — Create a new mold file (scaffold).
- **publish &lt;file&gt;** — Publish mold (implementation-specific).

---

## 5. Agent HTTP server

When you run `dal agent serve`, the server listens (default port 4040) and exposes one agent.

### Endpoints

| Method | Path | Description |
|--------|------|--------------|
| GET | `/status` | Agent id, name, type, status |
| POST | `/message` | Send message (body: `sender_id`, `content`; optional `message_type`) |
| GET | `/messages` | Receive (and consume) messages for this agent |
| POST | `/task` | Assign task (body: `description`; optional `task_id`, `priority`, `requester_id`) |
| GET | `/tasks` | Receive (and consume) pending tasks |
| GET | `/health` | Liveness |

### Example: send message and get messages

```bash
curl -X POST http://localhost:4040/message \
  -H "Content-Type: application/json" \
  -d '{"sender_id": "user1", "content": "Hello"}'

curl http://localhost:4040/messages
curl http://localhost:4040/status
```

### Prompt-only mode

With `--prompt-only` (or `DAL_AGENT_PROMPT_ONLY=1`), no behavior script is run. The server spawns a default worker agent and, for each incoming message, calls the LLM and posts the reply back to the sender. Use when you want a simple chat-style API without custom DAL logic.

---

## 6. DAL: agent and evolve APIs

### agent:: — lifecycle and coordination

From DAL you call the `agent` module as follows.

**Spawn an agent**

- `agent::spawn(config)` — `config` is a map with:
  - `name` (string)
  - `type` (string): `ai`, `system`, `worker`, `custom:<name>`
  - optional: `role`, `capabilities` (list of strings), `trust_level`, `metadata`
- Returns: agent ID (string).

Example:

```dal
use agent;

let agent_id = agent::spawn({
    "name": "my-agent",
    "type": "worker",
    "role": "Agent serve"
});
agent::set_serve_agent(agent_id);
```

**Set the serve agent** (required when using `dal agent serve` with a behavior script):

- `agent::set_serve_agent(agent_id)` — Registers this agent as the one to serve over HTTP.

**Coordination**

- `agent::coordinate(agent_id, task_description, coordination_type)` — Assigns a task to the agent. Coordination type e.g. `"task_distribution"`. Task is created with default priority.

**Communication**

- `agent::communicate(sender_id, receiver_id, message)` — Sends a message. You build the message with `agent::create_message(sender_id, receiver_id, message_type, content)` (or the runtime creates one with a generated id and type when you use the three-arg `communicate`).

**Other**

- `agent::create_config(name, type_str, role)` — Build a config (returns a value used internally).
- `agent::create_task(description, priority?)` — Create a task (task_id is generated).
- `agent::create_message(sender_id, receiver_id, message_type, content)` — Create a message (message_id generated).
- `agent::evolve(agent_id, evolution_data)` — Record evolution data; molds can hook `on_evolve` (e.g. call `evolve::append_summary`).
- `agent::validate_capabilities(agent_type, required_capabilities)` — Validate that an agent type has the required capabilities.
- `agent::terminate(agent_id)` — Mark agent terminated (in-memory).
- `agent::get_status(agent_id)` — Get status string.

Note: Messages and tasks are consumed when received (e.g. `receive_messages` / `receive_pending_tasks`); the HTTP server uses the same in-memory bus and queues.

### evolve:: — context file

Evolve uses a single markdown file (path from `[agent] context_path` or `DAL_AGENT_CONTEXT_PATH`; default `./evolve.md`).

- `evolve::load(agent_name?)` — Load full context. Creates file with header if missing.
- `evolve::load_recent(agent_name?, max_lines)` — Last N lines (or full if `max_lines <= 0`).
- `evolve::append_conversation(user_message, agent_response, agent_name?)` — Append a conversation turn.
- `evolve::append_log(action, detail, result)` — Append an action log row (e.g. after `sh::run`).
- `evolve::append_summary(summary_text, title?)` — Append a summary section.
- `evolve::get_path()` — Resolved context file path.
- `evolve::trim_retention(keep_tail_lines)` — Keep only the last N lines of content after the header.

Use these from DAL to keep the context file in sync with agent behavior (e.g. after answering a user or running a command).

### sh::run — shell execution

- `sh::run(cmd)` — Run a shell command. Trust is determined by `DAL_AGENT_SHELL_TRUST` or `[agent.sh]` in `agent.toml` / `dal.toml`. Returns a map: `stdout`, `stderr`, `exit_code`.

Use `evolve::append_log` after `sh::run` to record the action in the evolve file.

---

## 7. Molds

Molds are reusable agent configs (type, role, capabilities, trust, memory, lifecycle hooks). They can be local files or IPFS/on-chain.

### Mold format (canonical)

- **Format:** .mold.dal only (canonical). File naming: `*.mold.dal`. DAL-native block syntax; not plain JSON. See docs/MOLD_FORMAT.md.
- **Discovery:** Current directory, `mold/`, `mold/samples`. Legacy `*.mold.json` still loads but prefer `.mold.dal`.

### Mold structure (.mold.dal)

- **name**, **version**
- **agent**: type, role, capabilities, trust_level, learning, communication, coordination, memory_limit
- **lifecycle** (optional): onCreate, onEvolve run at spawn/evolve; onMessage, onDestroy are reserved (see MOLD_FORMAT.md)
- **parameters**, **dependencies**, **metadata** (optional)

### CLI

- `dal agent mold list` — List local mold paths.
- `dal agent mold show <path-or-name>` — Show mold details.
- `dal agent create --mold <path|ipfs://cid> <name> [--param k=v ...]` — Create agent from mold; optional params merged into metadata and substituted in role/capabilities (`{{key}}`). If the CLI reports "unexpected argument '--mold'", use: `dal agent create -- --mold <source> <name> [--param k=v ...]`.
- `dal agent serve --mold <path> [name]` — Serve an agent spawned from that mold.

### Principal vs mold: trust and evolve path

When you create or serve an agent from a mold, **trust** (shell execution) and **evolve path** (context file) always come from the **process** (agent.toml, dal.toml, or env), not from the mold. The mold can set role, capabilities, lifecycle, etc., but the operator controls trust and where evolution is stored. See COMPREHENSIVE_AGENT_AND_MOLD_PLANS.md §3–4.

### From DAL

- `mold::spawn_from(source, name_override?, params?)` — Load mold from path/name or `ipfs://<cid>`, spawn agent, return agent_id. Optional name override; optional params map (string keys/values) merged into metadata and substituted in role/capabilities (`{{key}}`).

---

## 8. Shell trust and evolve context

### Shell trust

`sh::run(cmd)` respects:

1. `DAL_AGENT_SHELL_TRUST` — `off` | `sandboxed` | `confirmed` | `trusted`
2. `[agent.sh]` in `agent.toml` or `dal.toml`: `trust`, optional `forbidden_patterns`, `allowed_prefixes`

If key-based gating is used and the check denies, config falls back to `[agent.sh]`.

### Evolve context path

1. `DAL_AGENT_CONTEXT_PATH` — explicit path
2. `[agent] context_path` in `agent.toml` or `dal.toml`
3. Default: `./evolve.md`

### Multi-step tool loop (agent serve)

- **`DAL_AGENT_MAX_TOOL_STEPS`** — max tool steps (run/search) per message or task before the agent is asked to summarize. Default **20**, clamped 1–50. Used when `dal agent serve` runs in prompt_only mode with the multi-step loop.

---

## 9. Persistent memory

Agent runtime state persists across restarts by default. No configuration is required — when you run `dal agent serve`, the runtime automatically saves and restores agent memory, tasks, messages, evolution data, and registered skills.

### What persists

- **Agent contexts**: Key-value memory, config, tasks, messages, metrics, lifecycle hooks
- **Task queue**: Pending tasks across all agents
- **Message bus**: Undelivered messages
- **Evolution store**: Per-agent evolution data
- **Registered skills**: Skills registered at runtime (not built-ins, which are always loaded)
- **Serve agent ID**: Which agent is served over HTTP

### Backends

| Backend | Default? | Config | Notes |
|---------|----------|--------|-------|
| **File** (JSON) | Yes | `DAL_AGENT_RUNTIME_PATH=./agent_runtime.json` | Atomic writes, human-readable |
| **SQLite** | No | `DAL_AGENT_RUNTIME_BACKEND=sqlite` | WAL mode, higher throughput, requires `sqlite-storage` feature |
| **Disabled** | No | `DAL_AGENT_RUNTIME_PERSIST=0` | In-memory only, state lost on exit |

### Configuration

Set via environment variables or `agent.toml` / `dal.toml`:

```bash
# Environment variables
DAL_AGENT_RUNTIME_PERSIST=1          # 1 (default) or 0
DAL_AGENT_RUNTIME_BACKEND=file       # file (default) or sqlite
DAL_AGENT_RUNTIME_PATH=./my_state.json  # Custom path
```

```toml
# agent.toml
[agent.persistence]
enabled = true
backend = "file"
path = "./agent_runtime.json"
```

### Behavior

- On first access (e.g. `dal agent serve`), the runtime checks for a saved snapshot and restores state.
- After every state-mutating operation (spawn, coordinate, communicate, evolve, register skills), the runtime writes the snapshot.
- Queues are capped at 10,000 entries per type to prevent unbounded growth.
- Schema versioning (`version` field) enables forward-compatible migrations.

For full details, see [Persistent Agent Memory](PERSISTENT_AGENT_MEMORY.md).

---

## 10. Skills and registry

Skills define what an agent can do. Each skill is a named bundle of tools and a description that gets included in the agent prompt at serve time.

### Built-in skills

Four categories ship with every DAL install:

| Category | Skill name | Tools |
|----------|-----------|-------|
| Development | `development` | read, write, search, run, lint, test, debug |
| Creative | `creative` | read, write, search, generate, transform |
| Office | `office` | read, write, search, run, schedule, email |
| Home | `home` | read, search, run, control, monitor |

### User-defined skills

Create `.skill.dal` files in your project's `.dal/` directory (or set `DAL_SKILLS_PATH`):

```
// .dal/calendar.skill.dal
skill "my_calendar" {
  category "office"
  description "Manage my custom calendar app via CLI and API calls."
  tools "run" "search" "read" "write"
}
```

Skills are loaded at startup and included in the agent prompt when the agent's config references them by name.

### Runtime skill registration

From DAL or Rust, register skills programmatically:

```dal
let skills = [{
    "name": "data_pipeline",
    "category": "development",
    "description": "Run ETL pipelines and data transformations.",
    "tools": ["run", "read", "write", "search"]
}];
agent::register_runtime_skills(skills);
```

Runtime-registered skills persist across restarts (stored in the agent runtime snapshot).

### Programmatic encouragement

The skills system includes built-in guidance that helps agents discover and use tools effectively:

- **Encouragement block**: Always included in the prompt — reminds the agent to search for tools, try commands, and learn from results.
- **Meta-from-memory**: If the agent's evolve history shows past tool usage (e.g. searching, running commands), the prompt includes reinforcement ("You have successfully used search before...").

For full details, see [Skills and Registry](SKILLS_AND_REGISTRY.md).

---

## 11. Capabilities and types

Agent **types**: `ai`, `system`, `worker`, `custom:<name>`.

**Capabilities** are per-type (built-in or registry) and per-agent (config/mold). Use `agent::validate_capabilities(agent_type, required_capabilities)` to check a type; at runtime, an agent context’s capabilities are set at spawn (from type initializer or from config/mold).

For how capabilities are defined, set, and validated, see [AGENT_CAPABILITIES.md](../AGENT_CAPABILITIES.md).

---

## 12. References

- [Persistent Agent Memory](PERSISTENT_AGENT_MEMORY.md) — Runtime persistence, backends, configuration, schema versioning.
- [Skills and Registry](SKILLS_AND_REGISTRY.md) — Custom skills, `.skill.dal` format, programmatic encouragement, runtime registration.
- [AGENT_CAPABILITIES.md](../AGENT_CAPABILITIES.md) — How capabilities are defined, set, and validated.
- [MOLD_FORMAT.md](../MOLD_FORMAT.md) — Canonical `.mold.dal` syntax and lifecycle hooks.
- [API_REFERENCE.md](API_REFERENCE.md) — Full stdlib reference (agent, ai, mold, etc.).
- [CLI_DESIGN.md](CLI_DESIGN.md) — CLI structure and init templates.
- [AI_FEATURES_GUIDE.md](AI_FEATURES_GUIDE.md) — AI module and LLM integration.
