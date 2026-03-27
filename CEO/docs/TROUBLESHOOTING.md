# Troubleshooting: see what processing is running

## App-side (agent and server)

- **GET /api/processing** — Pending scheduled tasks, completed task ids, server metrics (messages_handled, tasks_completed, etc.). Use this to see what the app considers "in progress" or queued.
- **GET /api/tasks** — Same task list in task-monitor shape: `tasks` (scheduled) and `completed_ids`.
- **GET /api/status** — Server metrics, agent count, serve agent info, context length, frontend source.
- **GET /api/ai-status** — Whether the LLM is reachable (OPENAI_API_KEY etc.).

Example:

```bash
curl -sS http://localhost:4040/api/processing | jq
```

## OS processes (dal, node, python)

From a terminal on the host:

```bash
# Processes matching dal, node, or python (exclude grep itself)
ps aux | grep -E 'dal|node|python' | grep -v grep

# Or by name
pgrep -fl dal
pgrep -fl node
```

Typical agent_assistant processes:

- `dal serve server.dal --port 4040` — HTTP server
- `dal run agent.dal` — agent runtime (if started by start.sh)
- `node` — bridge (e.g. Telegram/Discord) if running
- `python3` / `venv/bin/python3` — scripts (e.g. x_post.py) when posting tweets

## "Max tool steps reached" on a single post

Each **tool use** (run, search, read_file, write_file, etc.) counts as **one step**. The loop does: LLM → tool call → execute → feed result back to LLM → repeat until the LLM replies in plain text or the step limit is hit.

For **one tweet** the agent typically uses **1 tool step** (e.g. `run: curl ... POST /api/x/post`) then needs **one more turn** to return a final reply. If the limit is **1**, the runtime stops right after that first tool and returns "Max tool steps reached" before the LLM can say "Posted."

**Fix:** Set `DAL_AGENT_MAX_TOOL_STEPS` in `.env` to at least **2**; for normal use (multiple tools or batch posts) use **10–20** or leave unset (default **40**). Restart the server after changing `.env`.

```bash
# In .env (or ensure it's not set to 1)
DAL_AGENT_MAX_TOOL_STEPS=20
```

## Quick checklist

1. Server up? `curl -sS http://localhost:4040/health`
2. LLM configured? `curl -sS http://localhost:4040/api/ai-status`
3. What’s queued/done? `curl -sS http://localhost:4040/api/processing`
4. What processes are running? `pgrep -fl dal` and `ps aux | grep -E 'dal|node|python'`
5. "Max tool steps reached" on one post? Check `DAL_AGENT_MAX_TOOL_STEPS` in `.env` (use ≥ 2, e.g. 20).
