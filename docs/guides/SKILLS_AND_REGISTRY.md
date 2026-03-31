# Skills and registry guide

This guide describes how to use built-in skills, define custom skills, and configure the skill registry in dist_agent_lang. Skills are user-owned — you build them, you own them.

---

## Table of contents

1. [Overview](#1-overview)
2. [Built-in skills](#2-built-in-skills)
3. [Defining custom skills](#3-defining-custom-skills)
4. [Configuration](#4-configuration)
5. [How skills work in the prompt](#5-how-skills-work-in-the-prompt)
6. [Using skills with molds](#6-using-skills-with-molds)
7. [Programmatic encouragement](#7-programmatic-encouragement)
8. [Runtime skill registration](#8-runtime-skill-registration)
9. [Examples](#9-examples)
10. [References](#10-references)

---

## 1. Overview

Skills are named bundles of **description** and **tools** that define what an agent can do. When an agent has a skill, its prompt includes the skill's description and the model is told it can use that skill's tools.

**Ownership principle**: Skills follow the same pattern as molds. They are **built by and belong to users**. The language does not maintain or market a central registry. Built-in skills ship with the runtime; everything else is yours.

There are three ways to give an agent skills:

1. **Built-in skills** — Always available: development, creative, office, home, project_init
2. **User-defined skills** — Defined in `.skill.dal` files you create
3. **Runtime-registered skills** — Added programmatically from DAL code (persisted across restarts)

---

## 2. Built-in skills

| Skill | Category | Description | Tools granted |
|-------|----------|-------------|---------------|
| `project_init` | development | Initialize a DAL project | `dal_init` |
| `development` | development | Coding, scripts, DAL projects | `read_file`, `write_file`, `list_dir`, `dal_check`, `dal_run` |
| `creative` | creative | Writing, design, ideation | (base only) |
| `office` | office | Tasks, scheduling, docs, meetings | (base only) |
| `home` | home | Personal tasks, routines, health | (base only) |

**Base tools** (always available to every agent): `reply`, `run` (shell), `search`, `ask_user`, `dal_init`.

`project_init` is a **hard skill** — it's always included regardless of the agent's skill list.

If an agent has no explicit skills (and no mold), the default learning path is used: development, creative, office, home.

---

## 3. Defining custom skills

### The `.skill.dal` format

Skills are defined in `.skill.dal` files using a declarative format:

```
skill "<name>" {
  category "<category>"        # optional: development, creative, office, home
  description "<description>"  # required: what this skill enables
  tools "<tool1>" "<tool2>"    # optional: tool IDs this skill grants
}
```

### Example: office skills

```
# office_skills.skill.dal

skill "ms_office" {
  category "office"
  description "Use MS Office tools (Word, Excel, Outlook) via run or scripts."
  tools "run" "search"
}

skill "social_media_manager" {
  category "office"
  description "Manage social media accounts, schedule posts, analyze engagement."
  tools "run" "search"
}
```

### Example: home skills

```
# home_skills.skill.dal

skill "samsung_smart_home" {
  category "home"
  description "Control and query Samsung SmartThings devices."
  tools "run" "search"
}
```

### Example: custom platform integration

```
# my_calendar.skill.dal

skill "calendar_scheduler" {
  category "office"
  description "Manage a custom calendar platform for scheduling meetings and events."
  tools "run" "search"
}
```

### Format details

- Multiple `skill` blocks per file
- Comments with `#` or `//`
- Escape sequences in strings: `\"`, `\\`, `\n`, `\t`
- `category` is optional — skills without a category still work
- `tools` is optional — skills without tools are "description-only" (guide the model's understanding without granting extra tools)
- Files must be named `*.skill.dal` when loaded from a directory

---

## 4. Configuration

### Where to put skill files

Skills are loaded from a configurable path. Resolution order:

1. **`DAL_SKILLS_PATH`** env var — file or directory path
2. **`[agent] skills_path`** in `agent.toml` or `dal.toml`
3. **Default**: `.dal/` directory under cwd (if it contains `*.skill.dal` files)

### Example project layout

```
my-agent/
  agent.dal
  agent.toml
  evolve.md
  .dal/
    office_skills.skill.dal
    home_skills.skill.dal
```

### agent.toml config

```toml
[agent]
skills_path = ".dal"    # directory of .skill.dal files
```

### Environment variable

```bash
export DAL_SKILLS_PATH=/path/to/skills
```

---

## 5. How skills work in the prompt

When the agent handles a request:

1. The global skill registry is consulted (built-in + user-defined + runtime-registered)
2. The agent's skill names (from `AgentConfig.skills`) are resolved against the registry
3. Each found skill contributes its **description** to the prompt's Skills section
4. Each found skill's **tools** are added to the available tools list
5. Unknown skill names are skipped (logged at debug level, not an error)

The result is a prompt like:

```
You are operating as the DAL AGENT surface, serving your principal. Skills:
- project_init: Initialize and set up a DAL project...
- office: Tasks, scheduling, docs, meetings...
- ms_office: Use MS Office tools (Word, Excel, Outlook) via run or scripts.
Available tools: reply, run (shell), search, ask_user, dal_init; run
```

---

## 6. Using skills with molds

Molds list skill names in their agent block:

```
mold "office_assistant" {
  agent {
    type "ai"
    name "Office Helper"
    role "office assistant"
    skills "office" "ms_office" "social_media_manager"
  }
}
```

When an agent is created from this mold, it gets `office`, `ms_office`, and `social_media_manager` as its skills. At prompt-build time, the runtime resolves these names against the global registry — so `ms_office` and `social_media_manager` work as long as they're defined in a `.skill.dal` file at the configured path.

No changes to mold format are needed. Molds continue to list skill names as strings.

---

## 7. Programmatic encouragement

Every agent prompt includes a guidance block that encourages the agent to use `search` and `run` when it doesn't have a specific tool:

> When you don't have a specific tool for a task, use **search** or **run** to find documentation, APIs, or commands you need. Use your past executions in context as examples of how you've solved similar problems.

This is always included — no configuration needed.

### Meta-from-memory

When the agent has evolve or persistent memory context that contains evidence of past `search` or `run` use, an additional sentence is appended:

> In past executions you have used search and run when you didn't have a dedicated tool. This approach has worked well — apply it again when needed.

This makes the encouragement concrete with the agent's own history. It activates automatically when relevant context is available.

### Reinforcement

When the agent successfully discovers something via `search` or `run`, a note can be written to memory:

```
Used search to discover: MS Office API endpoint. Apply this pattern in future when no direct tool is available.
```

This builds up the agent's history of successful tool use, which feeds back into meta-from-memory on future turns.

---

## 8. Runtime skill registration

Skills can also be registered programmatically from DAL code or Rust. These are **persisted** in the agent runtime snapshot so they survive restarts.

From Rust:

```rust
use dist_agent_lang::skills::{SkillDefinition, SkillCategory};
use dist_agent_lang::stdlib::agent::register_runtime_skills;

register_runtime_skills(vec![
    SkillDefinition {
        name: "my_custom_api".to_string(),
        category: Some(SkillCategory::Development),
        description: "Interact with my custom API.".to_string(),
        tools: vec!["run".to_string(), "search".to_string()],
        builtin: false,
    },
]);
```

Runtime-registered skills:

- Are immediately available in the global registry
- Persist in the agent runtime snapshot (alongside memory, tasks, etc.)
- Are restored automatically on restart
- Cannot override built-in skills
- Are stored separately from `.skill.dal` files (they live in the snapshot, not on the filesystem)

---

## 9. Examples

### Minimal setup: add one skill

1. Create `.dal/office.skill.dal`:

```
skill "ms_office" {
  category "office"
  description "Use MS Office tools (Word, Excel, Outlook) via run or scripts."
  tools "run" "search"
}
```

2. In your mold or agent config, add `"ms_office"` to the skills list.

3. Start the agent — `ms_office` resolves automatically from `.dal/`.

### Multi-skill project

```
my-agent/
  agent.dal
  agent.toml
  .dal/
    office.skill.dal     # ms_office, social_media_manager
    home.skill.dal       # samsung_smart_home
    dev.skill.dal        # custom_ci, docker_manager
```

All `.skill.dal` files in `.dal/` are loaded automatically. The agent can reference any of these skill names.

### Description-only skill (no tools)

```
skill "advisor" {
  category "creative"
  description "Provide strategic business advice, market analysis, and brainstorming."
}
```

This guides the model's understanding without granting extra tools. The agent uses base tools (reply, run, search) but with the context that it should behave as a strategic advisor.

---

## 10. References

- [SKILLS_DESIGN_PLAN.md](../SKILLS_DESIGN_PLAN.md) — Implementation plan and architecture details
- [PERSISTENT_AGENT_MEMORY.md](PERSISTENT_AGENT_MEMORY.md) — Persistent runtime (skills + memory work together)
- [AGENT_SETUP_AND_USAGE.md](AGENT_SETUP_AND_USAGE.md) — Agent setup, CLI, serve, molds
- `src/skills.rs` — Skills implementation (registry, parser, encouragement)
- `src/agent_serve.rs` — Agent HTTP server (prompt building with registry)

**Ownership and licensing**: Skills follow the same pattern as molds — user-built, user-owned. Future licensing via smart registry is planned alongside the mold NFT system. See [PROJECT_PRIORITIES_TRACKER.md](PROJECT_PRIORITIES_TRACKER.md) for status.
