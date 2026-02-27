# Agent capabilities: how they’re built and used

This doc describes how agent capabilities are defined, set, validated, and used so you can extend or add new ones.

---

## 1. Where capabilities live

| Place | Purpose |
|-------|--------|
| **`AgentConfig.capabilities`** | Per-agent list of capability strings (stored on the agent context). |
| **`builtin_capabilities(AgentType)`** (Rust) | Default list per type used when **no** registry override exists. |
| **`CAPABILITY_REGISTRY`** (Rust) | Optional override: per–agent-type list used by **validation** instead of built-ins. |
| **Mold `agent.capabilities`** | Capabilities for agents spawned from that mold. |
| **DAL `agent` block / `create_agent_config`** | Capabilities supplied from DAL (agent declaration or config map). |

---

## 2. How capabilities get set (the “build” process)

### A. At spawn time (Rust: `agent::spawn`)

1. **`AgentConfig::new(name, agent_type)`** creates config with **`capabilities: Vec::new()`**.
2. **`initialize_<type>_agent(agent_context)`** is called and **overwrites** `agent_context.config.capabilities` with the type-specific list (see table below).

So by default, every spawned agent gets the list from the initializer for its type. No separate “registration” step is required for these defaults.

### B. Overriding from config (before or instead of init)

- **Rust:** Use **`AgentConfig::with_capabilities(vec!["cap1", "cap2"])`** when building the config. If you build the config yourself and then spawn, the initializer will still overwrite `capabilities` unless you change the flow (e.g. only run initializer when capabilities are still empty, or set capabilities after init).
- **Mold:** In the mold’s `agent` block, set **`capabilities`**. When creating `AgentConfig` from the mold, the runtime uses **`config.with_capabilities(mold.agent.capabilities.clone())`** (`mold/mod.rs`), so the mold’s list is applied when spawning from that mold.
- **DAL (stdlib agent):** When the runtime creates config from a map (e.g. `create_agent_config`-style flow in `engine.rs`), it reads **`fields["capabilities"]`** and, if present and a list of strings, calls **`config.with_capabilities(capability_strings)`**. So DAL can supply capabilities via the config map.
- **DAL (AI agent declaration):** For **`agent`** statements, **`agent_stmt.capabilities`** is copied onto **`ai::AgentConfig`** when executing the declaration. So the list you write in the agent block in DAL is what that AI agent gets.

So “building out” capabilities for an agent means: either rely on the **built-in list for that type**, or **set/override** via config (Rust builder, mold, or DAL config/agent block).

---

## 3. Built-in lists (current defaults)

Defined in **`src/stdlib/agent.rs`**:

- **`builtin_capabilities(AgentType)`** (used for **validation** when registry is not set):
  - **AI:** `analysis`, `learning`, `communication`, `task_execution`
  - **System:** `monitoring`, `coordination`, `resource_management`
  - **Worker:** `task_execution`, `data_processing`, `automation`
  - **Custom:** `custom_processing`

- **`initialize_<type>_agent`** (sets **actual** config at spawn):
  - **AI:** same four + `problem_solving`
  - **System:** same three + `system_optimization`
  - **Worker:** same three + `workflow_management`
  - **Custom:** `custom_processing`, `adaptation`, `flexibility`

Validation uses either the **registry** (if set) or **`builtin_capabilities`**; it does **not** read the per-agent `AgentContext.config.capabilities` — it only checks that the **agent type** has the required capabilities in the registry/built-in list.

---

## 4. Validation vs runtime checks

- **`agent::validate_capabilities(agent_type, required_capabilities)`**  
  Checks that the **type**’s allowed set (registry or built-in) contains every string in `required_capabilities`. Used to decide “can this type do this?” (e.g. before assigning a task). It does **not** look at a specific agent instance’s `config.capabilities`.

- **`AgentContext::is_capable(capability)`**  
  Checks whether **this agent’s** `config.capabilities` contains the given string. So per-agent overrides (mold, DAL, or manual config) are reflected here.

So:
- To **add a capability** that a type is **allowed** to have: extend **`builtin_capabilities`** and/or the corresponding **`initialize_<type>_agent`** list (and optionally **`register_capabilities`** if you use the registry).
- To **give a specific agent** a capability: set it via **`with_capabilities`** (Rust), mold **`agent.capabilities`**, or DAL config / agent block; then **`is_capable`** will reflect it.

---

## 5. How to add or extend capabilities (concrete steps)

### Option A: Add a new capability to an existing type (built-in)

1. In **`src/stdlib/agent.rs`**:
   - In **`builtin_capabilities(AgentType)`**, add the new string to the appropriate type’s `vec![]`.
   - In **`initialize_<type>_agent`** for that type, add the same string to **`agent_context.config.capabilities`**.
2. No DAL or mold change required; new agents of that type will get the capability by default, and **`validate_capabilities`** will accept it for that type.

### Option B: Custom list for a type (registry)

1. From Rust or from DAL (if you expose it), call **`agent::register_capabilities(agent_type_string, vec!["cap1", "cap2", ...])`**.
2. Validation will use this list instead of **`builtin_capabilities`** for that `agent_type`. Spawn initializers still set **`config.capabilities`** per agent; the registry only affects **validation**.

### Option C: Per-agent list (no new built-ins)

1. **Rust:** Build config with **`.with_capabilities(vec!["my_cap", ...])`** and spawn. The initializer will overwrite unless you change init to respect existing capabilities (e.g. only set when empty).
2. **Mold:** In the mold’s **`agent`** block set **`capabilities: ["my_cap", ...]`**.
3. **DAL:** In the agent’s config or agent block, set **`capabilities`** to a list of strings.

Then **`is_capable("my_cap")`** will be true for that agent; **`validate_capabilities`** will only accept it if the type’s built-in/registry list also includes `"my_cap"` (if you validate that type for that capability).

### Option D: New agent type with its own list

1. Add a new variant to **`AgentType`** (e.g. **`Custom("my_type")`** or a dedicated variant).
2. In **`builtin_capabilities`**, add a branch returning the new type’s list.
3. In **`initialize_<type>_agent`** (or a new initializer), set **`agent_context.config.capabilities`** for the new type.
4. Wire the new type through spawn/parser/mold as needed so it’s usable from DAL and molds.

---

## 6. Summary

- **Defined:** In code as **`builtin_capabilities`** and in **`initialize_*_agent`**; optionally in **`CAPABILITY_REGISTRY`**; and in molds/DAL as **`capabilities`** on config or agent block.
- **Set on agents:** By spawn initializers (defaults) and/or by **`with_capabilities`** (Rust), mold **`agent.capabilities`**, or DAL **config / agent block**.
- **Validated:** By **`validate_capabilities(agent_type, required)`** using registry or **`builtin_capabilities`**.
- **Checked at runtime:** By **`AgentContext::is_capable(capability)`** using the agent’s **`config.capabilities`**.

To build out agent capabilities you either extend the built-in/init lists and optionally the registry, or supply custom lists per agent via config/mold/DAL; validation and **`is_capable`** then reflect those choices.
