# Fleet deployment

How fleets work and how to get the most out of the fleet API for deployment.

---

## Start here (first time)

Work from a **project directory** you care about. Fleet data is written to **`.dal/fleets.json`** under the current working directory (see [File layout](#file-layout)). That path is typically **gitignored** (e.g. `agent_assistant` ignores `.dal/`).

### Example: `agent_assistant` (in-tree sample project)

From the **dist_agent_lang** repo root:

```bash
cd agent_assistant
dal agent fleet create assistant-workers --from-mold mold/assistant.mold.dal --count 2
dal agent fleet list -v
dal agent fleet deploy assistant-workers "Smoke test task"
dal agent fleet run assistant-workers
dal agent fleet health assistant-workers
```

- **Mold:** [`agent_assistant/mold/assistant.mold.dal`](../agent_assistant/mold/assistant.mold.dal) — minimal Worker mold for CLI experiments (same tree as the [Vibes Job / Agent Assistant](../agent_assistant/README.md) app).
- Use **`dal agent fleet delete assistant-workers`** when you are done (fleet metadata only; see [Reality check](#reality-check) below).

### 1. Create a fleet (any project)

**Option A — empty fleet** (name only; add members later with `add-from-mold` or `add-member`):

```bash
dal agent fleet create my-fleet
```

**Option B — fleet from a mold** (spawns N agents and records their IDs):

```bash
dal agent fleet create my-fleet --from-mold mold/worker.mold.dal --count 3
```

Molds are `.mold.dal` files; see [MOLD_FORMAT.md](MOLD_FORMAT.md).

### 2. Inspect

```bash
dal agent fleet list
dal agent fleet list -v
dal agent fleet show my-fleet
```

### 3. Record a task, then run it

`deploy` stores the **task string** on the fleet (what you intend to run). It does not by itself execute long-lived workers across machines.

```bash
dal agent fleet deploy my-fleet "Process daily logs"
dal agent fleet run my-fleet
```

`run` uses the built-in dispatcher: it loads the fleet, ensures members (may add from mold if the roster is empty), and sends `last_deployed_task` to each member via agent coordination. Use **`dal agent fleet health my-fleet`** for a quick status.

### 4. Scale (optional)

```bash
dal agent fleet scale my-fleet 5
```

### 5. Export for Docker / Kubernetes (optional)

```bash
dal agent fleet export my-fleet --format docker-compose
dal agent fleet export my-fleet --format k8s
```

### Reality check

- **Fleet** = roster + last-deploy metadata in `.dal/fleets.json`. **Agents** = runtime objects (memory, tasks, messaging) in the **process that created them**.
- If you only `create` / `deploy` and exit, the **next** `dal agent fleet run` may **re-create** members from the mold when it needs agents again—behavior is described in [Where execution happens](#3-where-execution-happens) below.

When you are comfortable with this flow, read [Fleet vs agent](#fleet-vs-agent) and the rest of this page for scaling, blue/green, and automation.

---

## Fleet vs agent

- **A fleet is not one agent.** It is a **named collection of agent IDs** (`member_ids`). So one fleet = many agents (or zero).
- **A fleet has no agent-like state.** It has no memory, message queue, or task queue. It only stores: name, optional mold source, list of member IDs, and optional last-deployed task/timestamp. That’s it.
- **Each member is its own agent.** When you create a fleet from a mold (e.g. `create_from_mold` with count 10), you get 10 separate agents, each with its own `agent_id`, and each with its own agent state (memory, messages, tasks) in the runtime that spawned them. The fleet is just a roster of those IDs.
- **Different process/abstraction.** A fleet is a grouping and deployment descriptor (stored in `.dal/fleets.json`). Agents are runtime entities with lifecycle, memory, and messaging. So: fleet = metadata/roster; agents = the entities that do work and hold state.

---

## Current model

- **Fleet** = named set of agent IDs, optionally created from a **mold** (one mold + N instances).
- **Storage:** in-memory; when a base path is provided, persisted to **`.dal/fleets.json`**.
- **Off-chain only:** fleets are a grouping and deployment descriptor, not on-chain.

**API (Rust):** `fleet::create`, `fleet::create_from_mold`, `fleet::list`, `fleet::show`, `fleet::scale`, `fleet::delete`, `fleet::deploy`, `fleet::add_from_mold`, `fleet::add_member`, `fleet::run`, `fleet::health`, `fleet::export` (with `ExportFormat::K8s` or `DockerCompose`). Fleet has optional `last_create_params` (used when scaling up).

**CLI:** `dal agent fleet create|list|show|scale|delete|deploy|add-from-mold|add-member|run|health|export ...`

---

## Deployment flow (use of the API)

### 1. Define once (mold), scale as needed

- Create a **mold** (`.mold.dal`) that describes agent type, role, skills, and lifecycle.
- Create a fleet from that mold with an initial size:
  ```bash
  dal agent fleet create workers --from-mold worker.mold.dal --count 10
  ```
- Scale up or down as needed (scale-up spawns more from the same mold; scale-down truncates the member list):
  ```bash
  dal agent fleet scale workers 20
  dal agent fleet scale workers 5
  ```

### 2. Deploy a task to the fleet

- **Deploy** records the task as the fleet’s “last deployed” intent. It does not by itself run code in a long-lived process; it makes the fleet the single source of truth for *what* was deployed.
  ```bash
  dal agent fleet deploy workers "Process daily logs"
  ```
- The fleet file (`.dal/fleets.json`) is updated with `last_deployed_task` and `last_deployed_at`. Any runner or automation can read this and distribute the task to fleet members.

### 3. Where execution happens

- **In-process:** If you create a fleet and keep the process alive (e.g. a server that ran `create_from_mold`), the same process holds the agents and can distribute the deployed task to them (e.g. via your own code that reads the fleet and calls your task API).
- **Out-of-process:** If the CLI process exits after `create` or `deploy`, agents created in that process are gone. The fleet file still has `member_ids` and `last_deployed_task`. You can use the built-in runner **`dal agent fleet run [name]`** (dispatches `last_deployed_task` to each member via agent coordination), or your own runner that:
  1. Reads `.dal/fleets.json`.
  2. For each fleet that has a `last_deployed_task`, (re)start or connect to N workers (one per member or one pool per fleet).
  3. Executes the task (e.g. run a DAL script, call an HTTP endpoint, or run a mold lifecycle hook).

So: **deploy** = “record this task for this fleet”; **execution** = `dal agent fleet run` or your own long-lived process that reads the fleet and runs the task.

### 4. Getting the most out of the API

| Goal | Use |
|------|-----|
| Homogeneous worker pool | `create_from_mold` with one mold and desired count; `scale` to adjust. |
| Record what to run | `deploy <name> <task>`; then use `show` or read `.dal/fleets.json` for `last_deployed_task` / `last_deployed_at`. |
| Blue/green or canary | Use two fleets (e.g. `workers-blue`, `workers-green`); deploy to one, then switch traffic or scale the other to 0. |
| Rollback | Redeploy a previous task string, or scale down and scale up from a previous mold version. |
| Export for orchestrators | Use `dal agent fleet export [name] --format k8s|docker-compose` or read `.dal/fleets.json` to generate custom job definitions. |

---

## File layout

- **`.dal/fleets.json`** – Fleet names → `{ name, mold_source?, member_ids, last_deployed_task?, last_deployed_at?, last_create_params? }`. One file for definition + last deployment.
- **Molds** – `*.mold.dal` under project or `mold/` (see mold docs).

---

## Implemented (production-grade)

| Area | Implementation |
|------|----------------|
| **Task execution** | `dal agent fleet run [name]` loads fleets, ensures members (adds from mold if empty), and dispatches `last_deployed_task` to each member via `agent::coordinate(..., "task_distribution")`. |
| **Scale-up params** | Fleet has optional `last_create_params`; set by `create_from_mold` and used by `scale()` when spawning new members. |
| **Add members** | `add_from_mold(name, mold_source, count, base, params?)` adds N agents from a mold (sets `mold_source` if fleet was empty). `add_member(name, agent_id)` registers an existing agent. |
| **Health** | `dal agent fleet health <name>` reports member count, has_mold, last_deployed_task/at, status (ok/empty). |
| **DAL/stdlib API** | `fleet` namespace in runtime: `fleet::create`, `fleet::create_from_mold`, `fleet::list`, `fleet::show`, `fleet::scale`, `fleet::delete`, `fleet::deploy`, `fleet::add_from_mold`, `fleet::add_member`, `fleet::health`. |
| **Export** | `dal agent fleet export [name] [--format k8s\|docker-compose]` emits YAML (JobList or docker-compose services). |
| **list output** | `dal agent fleet list [--verbose\|-v]` shows last_deployed_task and last_deployed_at. |

---

## Optional / future

| Area | Notes |
|------|--------|
| **Scale-down teardown** | Scale-down still only truncates `member_ids`; no agent terminate/teardown. Can be added when agent lifecycle API supports it. |
| **Concurrency** | Single-writer recommended for `.dal/fleets.json`. Multiple processes writing may overwrite; use one runner/CLI at a time or add advisory file locking (e.g. `fs2`) if needed. |

---

## Summary

- Use **create_from_mold** + **scale** to size a fleet from a single mold (scale-up reuses **last_create_params**).
- Use **deploy** to record the task for a fleet; use **`dal agent fleet run [name]`** (built-in runner) or your own process to dispatch the task to fleet members.
- Use **add_from_mold** / **add_member** to grow a fleet; **health** and **list -v** for status; **export** for Kubernetes or Docker Compose YAML.
- Use **show** / **list** and the fleet file to drive automation, blue/green, and export to external orchestrators.
