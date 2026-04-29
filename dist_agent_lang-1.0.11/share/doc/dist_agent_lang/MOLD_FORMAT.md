# Mold format (canonical: .mold.dal)

**Canonical format: .mold.dal only.** File naming: `*.mold.dal`. This format is DAL-native and not plain JSON, so molds are more resistant to copying without the DAL tooling. Legacy `.mold.json` is still accepted when loading; new molds should use `.mold.dal` and `dal agent mold create` scaffolds `.mold.dal`.

## File location and naming

- **Discovery:** Current directory, `mold/`, `mold/samples` (see `dal agent mold list`). `.mold.dal` files are listed before `.mold.json`.
- **Filename:** `*.mold.dal` (e.g. `my_agent.mold.dal`). Use `dal agent mold create <name>` to scaffold.

## .mold.dal syntax

Line-oriented block syntax. Comments: `//` to end of line.

### Header

```
mold "name" "version"
```

Name and version are required; version defaults to `"1.0"` if omitted. Both can be quoted strings or unquoted identifiers.

### Agent block

```
agent
  type AI
  role "Role description with {{param}}"
  capabilities "read" "write"
  memory_limit "256MB"
  learning true
  communication true
  coordination true
  trust_level standard
```

- **type** — `AI`, `System`, `Worker`, or `custom:<name>`. Default `AI` if omitted.
- **role** — Quoted or unquoted; may contain `{{param}}` for create-time substitution.
- **capabilities** — Space-separated quoted strings (or unquoted).
- **memory_limit** — e.g. `"256MB"`, `"2GB"`.
- **learning**, **communication**, **coordination** — `true` or `false`.
- **trust_level** — Display only; runtime trust is always from the principal (agent.toml / env).

### Lifecycle block (optional)

```
lifecycle
  on_create "evolve::append_log(agent_id, \"created\")"
  on_evolve "evolve::append_summary(agent_id, evolution_data)"
```

- **on_create**, **on_evolve** — Supported; DAL code run at spawn / on evolve.
- **on_message**, **on_destroy** — Reserved for future use.

### Example

```
// Fraud detector mold
mold "fraud_detector" "1.0"
agent
  type AI
  role "Detect fraud with {{env}}"
  capabilities "read" "analyze" "report"
  memory_limit "512MB"
  learning true
lifecycle
  on_create "evolve::append_log(agent_id, \"created\")"
  on_evolve "evolve::append_summary(agent_id, evolution_data)"
```

## Legacy JSON

Content that starts with `{` is still parsed as JSON (legacy `.mold.json`). Prefer creating and editing `.mold.dal` for new molds.

## Principal vs mold

Trust (shell execution) and evolve path (context file) are **always** from the process (agent.toml, env), never from the mold. See COMPREHENSIVE_AGENT_AND_MOLD_PLANS.md §3–4.

## Create-time parameters

Use `--param k=v` (CLI) or the optional third argument to `mold::spawn_from` (DAL). Params are merged into agent metadata and substituted for `{{key}}` in `role` and `capabilities`.
