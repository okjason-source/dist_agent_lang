# DAL Assistant — Skill for Editor Agents (e.g. Cursor)

**Purpose:** Use DAL (Distributed Agent Language) agents and the DAL CLI from an editor agent so the user gets a consistent “DAL assistant” experience. Copy or adapt this into a Cursor skill (e.g. `.cursor/skills/dal-assistant.md`) if you want the editor to follow these patterns.

---

## When to use

- The user is working in a DAL project (has `dal.toml` and `.dal` files) or wants to create one.
- The user wants to run, check, or edit DAL code; or to talk to a DAL agent (e.g. `dal agent serve`).
- You (the editor agent) should use the same toolchain as DAL agents: `dal check`, `dal run`, `dal init`, and optionally the agent HTTP API.

---

## DAL CLI (toolchain)

- **`dal init [template]`** — Initialize a project. Templates: `dal`/`general` (main.dal), `chain` (chain.dal), `iot` (iot.dal), `agent` (agent.dal + evolve). Example: `dal init chain`.
- **`dal check <file>`** — Validate a DAL file (syntax/semantics). Use after editing.
- **`dal run <file>`** — Run a DAL script. Example: `dal run main.dal`.
- **`dal agent serve`** — Start the agent HTTP server (messages, tasks). Requires an agent project (agent.dal, agent.toml, evolve.md).

Assume the `dal` binary is on PATH or the user has set it up (e.g. via `scripts/install.sh` or package manager).

---

## Working context

- File paths in DAL projects are usually relative to the project root (where `dal.toml` lives).
- When calling the agent API, send **`working_root`** (absolute or relative path to project root) so file tools (read_file, write_file, list_dir, dal_check, dal_run) and dal_init run in the right directory.

---

## Agent HTTP API (when `dal agent serve` is running)

- **POST /message** — Send a user message. Body can include:
  - `content`, `sender_id`, optional `objective`, `sub_tasks`, `include_dal_summary`, `dal_file`, **`working_root`** (path for file tools).
- **POST /task** — Submit a task. Body can include `description`, optional `sub_tasks`, `include_dal_summary`, `dal_file`, **`working_root`**.

The agent uses skills (development, creative, office, home) and the **project_init** hard skill (dal_init). If the agent has the **development** skill, it can use read_file, write_file, list_dir, dal_check, dal_run.

---

## Editor agent workflow

1. **Detect DAL project** — Presence of `dal.toml` (and optionally agent.dal / agent.toml).
2. **Validate after edit** — Run `dal check <path>` for the file you edited.
3. **Run a script** — Run `dal run <path>` when the user wants to execute.
4. **Create a project** — Run `dal init [template]` in the chosen directory.
5. **Use the agent** — If the user runs `dal agent serve`, you can POST to `/message` or `/task` with `working_root` set to the project root so the agent can read/edit files and run dal check/dal run there.

---

## References

- [AGENT_ASSISTANT_PLAN.md](AGENT_ASSISTANT_PLAN.md) — Skills, tools, working context, phased plan.
- [AGENT_SETUP_AND_USAGE.md](guides/AGENT_SETUP_AND_USAGE.md) — Agent project setup, evolve, shell trust.
- [PUBLIC_DOCUMENTATION_INDEX.md](PUBLIC_DOCUMENTATION_INDEX.md) — Full docs index.
