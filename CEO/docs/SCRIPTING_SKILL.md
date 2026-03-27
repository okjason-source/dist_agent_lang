# Scripting skill

The agent can write and run DAL or shell scripts.

## Phase 1 (always)

- Use the **run** tool with heredoc/printf to create scripts, then `dal run` or `bash`.
- The scripting skill describes patterns for this in `skills/scripting.skill.dal`.

## Phase 2 (when enabled)

When `AGENT_ASSISTANT_SCRIPTING=1` or `AGENT_ASSISTANT_ROOT` is set, the agent gets:

- **write_file** — create `.dal` or `.sh` files
- **read_file** — read file contents
- **list_dir** — list directory contents
- **dal_run** — run a DAL file
- **dal_check** — check a DAL file for syntax errors

Paths are relative to `AGENT_ASSISTANT_ROOT/scripts` (created if missing). If only `AGENT_ASSISTANT_SCRIPTING=1` is set (no root), file tools use the current working directory.

## Enabling scripting

```bash
# Option A: Enable scripting tools (uses cwd for file ops)
export AGENT_ASSISTANT_SCRIPTING=1

# Option B: Set project root (creates scripts/ subdir, uses it as working root)
export AGENT_ASSISTANT_ROOT=/path/to/agent_assistant
```

Start the server with `./start.sh` or `dal serve server.dal` so `.env` is loaded.
