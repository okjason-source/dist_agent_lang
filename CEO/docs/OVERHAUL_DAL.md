# Agent Assistant — Overhaul for dist_agent_lang

This doc describes what needs to be **completed or overhauled** so the agent_assistant works as intended, using **dist_agent_lang (DAL)** as the primary language. We will debug and harden the language as we go.

---

## 1. Entry-point and agent lifecycle (critical)

**Current state**

- `./start.sh` runs **`dal serve server.dal --port 4040`**. That loads and executes `server.dal` once at startup, registers `@route` handlers, and serves HTTP. **`agent.dal` is never run** in this flow.
- **`dal agent serve`** is a different mode: it runs a **behavior script** (e.g. `agent.dal`), which calls `agent::spawn(...)` and `agent::set_serve_agent(agent_id)`, then the **Rust** agent server runs (fixed routes: message, task, status, etc.). That path does **not** use `server.dal`’s routes (history, workflow, x, send, context, etc.).

**Gap**

- With `dal serve server.dal`, there is no formal “serve agent” from the DAL runtime: `load_agent_runtime()` in `server.dal` reads `.dal/agent_runtime.json`, which is only written when **`dal agent serve`** has been used. So under `start.sh`, `/api/status` often shows no `serve_agent` and agent count 0, even though the app works via `ai::agent_run` and `evolve::*` in DAL.

**Overhaul options (pick one and implement)**

- **A) Keep “DAL server” as main entry**  
  - Treat `server.dal` as the single source of truth for HTTP.  
  - Either: run `agent.dal` **before** starting the HTTP server (e.g. a new mode like `dal serve server.dal --init-agent agent.dal`) so that `agent::spawn` and `agent::set_serve_agent` run once and `.dal/agent_runtime.json` is populated; or  
  - Document that “serve agent” in `/api/status` is optional and only set when someone has run `dal agent serve` in this project at least once.  
  - Prefer doing the “init agent” run in DAL (e.g. `agent.dal`) so the lifecycle stays in the language.

- **B) Unify on `dal agent serve`**  
  - Change `start.sh` to `dal agent serve --behavior agent.dal --port 4040`.  
  - Then the Rust agent server runs; to get the **same** API surface as `server.dal` (history, workflow, x, send, context), the Rust agent server would need to either load and dispatch to DAL route handlers from `server.dal`, or reimplement those routes in Rust.  
  - This is a larger change; only do it if the product direction is “one agent-serve entry point.”

**Recommendation:** Option A with an explicit “init agent” step in DAL (run `agent.dal` once at startup before serving `server.dal`) so that `evolve::load()`, `agent::spawn`, and `agent::set_serve_agent` are all expressed in DAL and status reflects the intended agent.

---

## 2. agent.dal — align with runtime and startup

**Current `agent.dal`**

- Uses `evolve::load()` (no import — evolve is a built-in; OK).
- Uses `log::info("agent", "Spawning agent-assistant")` — runtime expects two arguments (source, message); OK.
- Uses `agent::spawn({...})` and `agent::set_serve_agent(agent_id)`.

**Issues to fix**

- **Unused `context`:** `let context = evolve::load();` is never used. Either use it (e.g. pass into a prompt or log) or remove it so the script is minimal and the language can be tested without dead code.
- **Evolve at init:** If the design is “evolve context is loaded each turn by the server,” then agent.dal does not need to load it at spawn; the comment in `agent.dal` says “Serve loads: evolve::load_recent(agent_name?, max_lines) into the prompt each turn.” So in the **DAL server** path, that happens inside the handler (e.g. `enrich_prompt` in `server.dal`). For **`dal agent serve`**, the Rust side does `evolve::load_recent`. So in `agent.dal`, either drop `evolve::load()` or use it only for logging/validation (e.g. `log::info("agent", "Context length: " + to_string(len(context)))`) to harden `evolve::load` and `len` in DAL.

**Suggested agent.dal (minimal, DAL-first)**

```dal
import stdlib::agent;

fn main() {
    log::info("agent", "Spawning agent-assistant");
    let agent_id = agent::spawn({
        "name": "agent-assistant",
        "type": "ai",
        "role": "Personal assistant — chat, shell commands, web search, code, messaging",
        "capabilities": ["task_execution", "communication", "automation", "problem_solving", "learning"],
        "skills": ["development", "creative", "office", "communications"]
    });
    log::info("agent", "Spawned: " + agent_id);
    agent::set_serve_agent(agent_id);
    log::info("agent", "Serve agent set — ready");
}
main();
```

Then add an optional “with evolve” variant that calls `evolve::load()` and logs `len(context)` to validate the language.

---

## 3. server.dal — language surface and robustness

**Built-ins used (all must be supported by the runtime)**

- `config::get_ai_config()`, `config::get_env(...)` — used; engine has `call_config_function` with `get_ai_config` and `get_env`. OK.
- `evolve::load()`, `evolve::append_conversation`, `evolve::append_log`, `evolve::append_summary`, `evolve::trim_retention`, `evolve::get_path` — all wired in `call_evolve_function`. OK.
- `log::info(source, message)`, `log::audit(source, message)` — two arguments; engine expects exactly two. OK.
- `sh::run(cmd)` — must respect `[agent.sh] trust` from `agent.toml` when running under agent serve; for `dal serve server.dal`, trust is typically applied per execution. Ensure one canonical place that reads agent.toml/dal.toml and passes trust into sh so we can harden the language.
- `ai::generate_text(prompt)`, `ai::agent_run({"message": prompt, ...})` (and legacy `respond_with_tools` wrappers) — engine dispatches to stdlib/runtime. OK.
- `json::parse`, `json::stringify` — standard. OK.
- `to_string`, `to_int`, `len`, `type` — used widely; ensure consistent behavior (e.g. `len(null)` or `to_string(map)`). Harden with tests.

**Overhaul items**

- **Error handling:** Many handlers use `try { ... } catch { ... }` and fall back to empty string or a default. Add a small set of **canonical** patterns: e.g. `result_or(err_val)` or a standard error map `{ "ok": false, "error": msg }` so the language’s try/catch and return values are consistent and testable.
- **Route handler return shape:** Handlers return a map with `status`, `headers`, `body`. The runtime and HTTP layer must agree on this contract; add a single place in the docs (and ideally one helper in DAL) that defines the route response shape so we don’t drift (e.g. optional `body` as string vs bytes).
- **History and evolve:** `history_append` builds JSON and shells out to `echo ... >> history.jsonl`. Prefer evolving toward a single store (e.g. evolve.md + optional JSONL) or a clear split (“evolve = agent memory, history.jsonl = UI history”) and document it in DAL terms (`evolve::append_conversation` vs `history_append`).
- **api_message / api_task:** They call `ai::agent_run({"message": enriched, "policy": ...})`. Enriched prompt is built in DAL with `load_context()` (evolve::load), skills, personality, etc. Ensure `evolve::load()` in DAL runs in the same process and cwd so `evolve::get_path()` and `agent.toml` `context_path` resolve to the same file the Rust evolve module uses. Already the case when cwd is the project root; document and add a single test that evolve path from DAL matches Rust.

---

## 4. agents.dal and workflows.dal — agent:: and ai:: usage

**agents.dal**

- Uses `agent::spawn`, `agent::coordinate(agent_id, task_description, "task_distribution")`, `agent::communicate`, `agent::get_status`. Runtime implements these. OK.
- `run_with_agent(role, task)` does: get_or_spawn(role) → assign (agent::coordinate) → then **ignores** the coordination result and calls `ai::agent_run({"message": role_prompt + "Task: " + task})` in-process. So the “agent” is mostly a role label; the real work is one LLM call with a role prefix. Either document this (“role is a prompt prefix; coordination is for future multi-process agents”) or change semantics so that coordination actually drives execution (e.g. task queue consumed by a worker). For language hardening, the current behavior is fine; just document it in DAL.

**workflows.dal**

- Uses `agents::run_with_agent`, `json::stringify`, `sh::run` with Python for custom workflow steps. The custom workflow runner uses string concatenation and `sed` for placeholders; that is fragile. Overhaul: introduce a small **DAL-native** loop (e.g. `for` or `while` over steps, or a `workflow::run_steps(steps, input)` built-in) so we don’t depend on Python and shell for control flow. That will stress-test the language (arrays, iteration, maps) and make workflows debuggable in pure DAL.

---

## 5. communications.dal and x.dal

- **communications.dal:** `send`, `reply_to_sender`, `parse_sender_channel`; uses `config::get_env`, `sh::run`, `log::audit`. No missing built-ins; ensure `reply_to_sender` is covered by a quick test (e.g. returns `{"ok": true, "channel": "web", ...}` for `sender_id == "web"`).
- **x.dal:** Uses `sh::run` for Python script and temp file. `write_temp_text` uses heredoc; ensure DAL string escaping and `sh::run` safety are documented (e.g. no user-controlled delimiter). Prefer a small **DAL-oriented** helper or convention (e.g. “write to file via stdlib” if/when the language gains one) to reduce injection risk and harden the language.

---

## 6. Skills and context_path

- **agent.toml** has `context_path = "./evolve.md"` and `skills_path = "./skills"`. The Rust evolve stdlib uses `get_context_path()` from env or agent.toml; the **DAL** server uses `evolve::load()` which goes through the same path. So evolve is consistent.
- **Skills:** `load_skills_context()` in server.dal runs `sh::run("cat skills/*.skill.dal ...")`. That’s fine for “list and describe skills.” To harden the language, consider exposing a minimal **stdlib or built-in** that returns “skill name + description” from a directory (e.g. `skills::list(skills_path)`) so parsing and file layout are standardized and testable in DAL.

---

## 7. Logging and observability

- **log::info** in DAL is two-argument (source, message). The Rust stdlib `log::info` used in ai.rs etc. has a different signature (message, data map, source). The **DAL** `log::info` is the one used by server.dal and agent.dal; keep it as the single contract for “info” from DAL scripts. Ensure any new DAL examples use `log::info("source", "message")` so the language stays consistent.
- **Throttling:** “Generating text response” is already throttled in the Rust ai stdlib (once per 2 seconds). No change needed in DAL for that.

---

## 8. What to implement next (priority)

1. **Entry and agent lifecycle (Section 1):** Done. start.sh runs `dal run agent.dal` before `dal serve server.dal` so .dal/agent_runtime.json is populated and /api/status shows serve_agent (`|| true` so server still starts if init fails). Previously: decide A or B; if A, add “run agent.dal once before serve” (or document that serve_agent is optional).
2. **agent.dal (Section 2):** Done. agent.dal is minimal (no unused evolve::load); role/capabilities/skills updated for okjason, tools, batch tweets.
3. **server.dal (Section 3):** Done. Comment block in server.dal defines route return shape and standard error body. Evolve path verification: see last item below.
4. **workflows.dal (Section 4):** Done. Added `workflow::run_steps(steps, input)` built-in in the runtime; `run_custom` in workflows.dal now calls it (no Python/sed). Steps are an array of `{role, prompt}`; placeholders `{input}` and `{prev}` are substituted in Rust.
5. **Skills (Section 6):** Done. Added `skills::list(path?)` in the runtime (returns array of `{name, description}`); `load_skills_context()` in server.dal uses it when available, with fallback to shell grep/sed.

**Also done:** Run `scripts/requirements.txt` (e.g. `venv/bin/pip install -r scripts/requirements.txt`) so x_post.py batch has requests-oauthlib.

All listed overhaul items (1–5) are done. The agent_assistant now uses DAL-native workflow and skills where possible.

**Next (agent project only, no language changes):**
- **Memory and agents:** See `docs/MEMORY_AND_AGENTS.md` for evolve vs history and role-as-prompt-prefix.
- **Verify evolve path (last):** From the agent project directory run:
  ```bash
  dal run scripts/verify_evolve_path.dal
  ```
  This prints the path from `evolve::get_path()` so you can confirm it matches `agent.toml` `context_path` (or `DAL_AGENT_CONTEXT_PATH`). Script: `scripts/verify_evolve_path.dal`.
