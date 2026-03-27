# Scheduling and task monitor

**Decision: agent only.** Scheduling and task monitor are implemented in the agent_assistant (server.dal, in-memory or file store). Do **not** add to the dist_agent_lang language/runtime unless we explicitly approve it later (e.g. a stdlib `schedule::` or `task::` namespace for reuse across projects).

---

## In the agent

- **Scheduling:** `POST /api/schedule` — add a task with optional `run_at` (ISO8601). Stored in memory (or later `scheduled_tasks.json`).
- **Task monitor:** `GET /api/tasks` — list scheduled tasks and their status (pending, completed, failed). The agent (or a human) can call this to see what’s queued and what’s done.
- **Running due tasks:** The agent can poll `GET /api/tasks`, then for each due task call `POST /api/task` with that description and then `POST /api/tasks/complete` with the task id. Alternatively an external cron can call an endpoint that runs due tasks.

Persistence (file-based) and a “run due” endpoint can be added later without changing the language.
