# Memory and agents (agent project)

This doc clarifies how memory and sub-agents work in the agent_assistant, using only existing DAL and runtime behavior.

---

## Evolve vs history

Two stores are used; they have different roles.

| Store | Purpose | DAL / usage |
|-------|---------|-------------|
| **evolve** | Agent memory. Long-lived context the LLM sees each turn: conversation transcript, action log, summaries. | `evolve::load()`, `evolve::append_conversation(user, agent)`, `evolve::append_log(action, detail, result)`, `evolve::append_summary(text)`, `evolve::trim_retention(n)`, `evolve::get_path()`. File: `evolve.md` (path from `agent.toml` `context_path` or `DAL_AGENT_CONTEXT_PATH`). |
| **history** | UI history. List of recent messages/tasks for the web UI (History panel). | `history_append(sender_id, content, reply, entry_type)`, `history_read(max_entries)`, `history_count()`, `history_trim()`. File: `history.jsonl` (one JSON object per line). |

- **evolve** = what the agent “remembers” and what is injected into the prompt.
- **history** = what the UI shows as past conversations; it is not necessarily the same as evolve content and can be cleared independently (`POST /api/history/clear`).

Both are updated from `server.dal` (e.g. `api_message` calls `log_conversation` → evolve and `history_append` → history).

---

## agents.dal: role as prompt prefix

In this project, **researcher**, **coder**, and **reviewer** are implemented as **role labels and prompt prefixes**, not as separate processes.

- `agents::run_with_agent(role, task)` does:
  1. `get_or_spawn(role)` → ensures an agent id exists for that role (and calls `agent::coordinate` for bookkeeping).
  2. Builds a role-specific prompt (e.g. “You are a research agent…”) and calls **`ai::agent_run({"message": role_prompt + "Task: " + task})`** in-process.

So the real work is a single LLM call with a role prefix plus non-critical diagnostics (e.g. route and tool-step counts). **Coordination** (`agent::coordinate`) is used for state/bookkeeping; execution is not driven by a separate worker or task queue. For future multi-process or multi-agent execution, the semantics could be extended; for now, “role” is the prompt prefix and the run is in-process.

---

## Optional: verify evolve path

To confirm that DAL and the runtime use the same evolve file, from the agent project directory run:

```bash
dal run scripts/verify_evolve_path.dal
```

The script prints the path returned by `evolve::get_path()`. It should match the path used by the Rust evolve stdlib (same `agent.toml` / `DAL_AGENT_CONTEXT_PATH`). No language changes required; the script only calls existing `evolve::get_path()` and `log::info`.

---

## See also

- **[CONTEXT_PRIORITY_AND_EPSILON_PLAN.md](./CONTEXT_PRIORITY_AND_EPSILON_PLAN.md)** — Plan for prioritizing what the LLM reads (tiered context), reinforcement from feedback, and epsilon-style explore/exploit over context to reduce bloat while keeping task-relevant information.
