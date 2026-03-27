# DAL CLI Design

**Principles:** Intuitive, Off/On-chain, AI-forward.

---

## Branding

- **Tagline:** "Unified language for Web, Blockchain & AI"
- **Banner:** Shown on `help` and `version`; `--no-banner` for scripting
- **Quiet mode:** Single-line output when `--quiet`

---

## Help Structure (Use Case First)

Commands are grouped by **what users want to do**, not implementation:

| Section | Focus |
|---------|-------|
| **Get Started** | new, init, init &lt;stack&gt;, run, test |
| **Build & Develop** | check, fmt, lint, watch, repl, bench, profile, optimize |
| **AI & Code Assistance** | ai code, explain, review, audit, test, fix, optimize-gas |
| **Agents & Automation** | agent create, send, list, mold |
| **Blockchain** | chain list, gas-price, balance, mint, asset |
| **Data & Infrastructure** | crypto, db, cloud |
| **Devices & IoT** | iot register, status, ai-predict |
| **Tools** | web, convert, doc, completions, **serve** (with optional `--venv <name>`) |

---

## AI Integration (Current & Future)

**Current:** `ai` commands are prominent — code generation, explanation, review, audit, test generation, fixes, gas optimization.

**Future-ready:**
- `dal ask "<prompt>"` — natural language entry point
- `dal "<prompt>"` — shorthand when context is clear
- `--json` for structured output (AI tooling, scripts)
- Help structure surfaces AI capabilities early

---

## Project-Type Agnostic

The same help works for:
- **Smart contracts** — chain, ai audit, ai optimize-gas
- **AI agents** — agent, ai code, ai explain
- **Web apps** — web, db, cloud
- **IoT** — iot, agent, ai-predict
- **Libraries** — check, fmt, lint, doc

Quick examples in help cover multiple project types.

---

## Files

- `src/cli_design.rs` — Banner, tagline, help content
- `src/cli.rs` — clap structure, commands
- `src/main.rs` — Dispatch, print_help, print_version

---

## Where We Are (Current State)

### Implemented and solid

| Area | Status | Notes |
|------|--------|--------|
| **clap structure** | Done | Full `Commands` enum, `ChainSubcommand` fully typed, global flags (`--quiet`, `--verbose`, `--no-banner`, `--color`) |
| **Help structure** | Done | Use-case-first sections in `cli_design::help_content()`; custom `print_help` (no Phase labels) |
| **Branding** | Done | Banner (box-drawing), tagline, `--no-banner` / `--quiet` respected in help/version |
| **Dispatch** | Done | All commands wired in `main.rs`; chain uses `chain_subcommand_to_args` |
| **REPL** | Basic | `dal repl`: read-eval-print loop, `help`/`exit`/`quit`, expression/statement eval; no colors, no history, no multiline |
| **Debug** | Stub | `dal debug <file>` prints “planned” features (step, inspect, call stack); runs lex/parse only |
| **Watch** | Done | File watch, re-run on change, simple console messages |

### Init with stack templates (design)

**Goal:** All `dal init` variants operate in the **current directory** and are **additive**. `dal init` sets up DAL there; `dal init js` (or `rs`, `sol`) **adds** that stack’s template files to the current directory so you can mix stacks in one repo.

- **`dal init`** — Create `dal.toml`, `main.dal`, `README.md`, **`.env.example`** (optional logging/API keys), **`.env`** (if missing), and ensure **`.env`** is in `.gitignore`. Fails if `dal.toml` already exists.
- **`dal init agent`** — **Add** an agent project: ensures `dal.toml` (minimal if missing), creates `agent.toml`, `agent.dal`, `README.md`, `evolve.md` (evolve context), **`.env.example`** (documented env vars, safe to commit), **`.env`** (local overrides, only if missing), and ensures **`.env`** is in `.gitignore` so secrets are not committed. Additive; never overwrites existing files. User can set values in `.env` and start using the agent. See [AGENT_SETUP_AND_USAGE.md](AGENT_SETUP_AND_USAGE.md).
- **`dal init js`** — **Add** a JS/Node template to the current directory: e.g. `package.json`, optional `dal.config.js` or scripts that call the `dal` CLI, and a minimal DAL hook so the app can use DAL for orchestration/agents/chain. Does not require the directory to be empty; adds or updates only the JS-related files. Can be run after `dal init` to add JS to an existing DAL project.
- **`dal init rs`** — **Add** a Rust template: e.g. `Cargo.toml`, `src/`, and optional glue to call or embed `dal`. Adds to current directory; safe to run in a repo that already has DAL or JS.
- **`dal init sol`** — **Add** a Solidity template: e.g. `contracts/` scaffold and optional DAL that uses `chain::` / convert. Adds to current directory.
- **`dal init <other>`** — Reserve names (e.g. `py`, `go`) for future stacks; unknown template prints available templates and exits.

So: **init = add this template here**. Users can run `dal init` then `dal init js`, or run `dal init js` in an existing project, to get a mixed stack in one directory.

**Always add DAL if not there; protect dal.toml.** When adding a stack template (`dal init js` | `rs` | `sol`), if `dal.toml` does not exist, create a minimal one so the directory is DAL-aware. If `dal.toml` already exists, **do not overwrite it** — leave the user's config and content intact. Same for plain `dal init`: we already fail when `dal.toml` exists (refuse to overwrite). Rule: **ensure dal.toml exists when adding a stack, and never overwrite an existing dal.toml.**

**Minimum template prints for all.** Every init variant (`dal init`, `dal init dal`, `dal init agent`, `js`, `rs`, `sol`) uses minimal console output: one short summary line (e.g. "Initialized DAL project." or "Added js template."). No per-file "Created …" lines. When `--quiet` is set, suppress the summary as well. Same rule for all templates.

**Implementation sketch:** `Init { template }`; for `"dal"` use current behavior (create dal.toml etc., fail if exists). For `"js"` | `"rs"` | `"sol"`, add only that template’s files to the current directory (create dirs like `contracts/` or `src/` if needed, write or merge package.json / Cargo.toml; avoid overwriting existing files without a flag, or document “adds only missing files”).

### Gaps (design & interactivity)

| Gap | Impact |
|-----|--------|
| **`--color` unused** | `ColorChoice` is parsed but never passed to output; no colored errors, success, or syntax highlighting |
| **No structured output** | No `--json` for scripts/AI; all output is human-oriented prose |
| **REPL is minimal** | No readline (history, editing), no colors, no multiline input, no `ai repl` / AI-assisted mode |
| **No TUI / rich prompts** | No interactive wizards (e.g. `dal new` with prompts), no progress bars, no tables for list output |
| **Error formatting** | Errors use `eprintln!` and reporter formatting; not yet unified with a “pretty” or color-aware pipeline |
| **Subcommand help** | `dal <cmd> --help` is clap’s default; not yet aligned with use-case-first wording or examples |
| **No “dal ask” / NL entry** | Design doc and expansion plan mention it; not implemented |

### Reference

- **Implementation:** `src/cli.rs`, `src/cli_design.rs`, `src/main.rs` (dispatch, `print_help`, `print_version`, `run_repl`, `handle_debug_command`).
- **Plans:** [09_CLI_EXPANSION_PLAN.md](../development/stdlib_implementation_plans/09_CLI_EXPANSION_PLAN.md) (Phases 0–10 done; Phase 11 clap + §21 design implemented; `dal --help` works).
- **§21 design:** See 09_CLI_EXPANSION_PLAN §21 (Appearance, Options, Help Layout). Banner, option naming, help layout done; `--color` parsed but not yet wired to output (Phase A).
- **Quick reference:** [CLI_QUICK_REFERENCE.md](../CLI_QUICK_REFERENCE.md).

---

## Next Phases: Comprehensive, User-Friendly, Beautiful Design

Goals: **respectful of terminals** (color when useful, scriptable when needed), **discoverable** (help and errors that guide), **interactive where it helps** (REPL, wizards, progress), **consistent** (one voice, one style).

---

### Phase A: Output pipeline and color (foundation)

**Objective:** One place to decide “is this a TTY? do we want color?” and route all CLI output through it.

1. **Use `--color` everywhere**
   - Pass `cli.color` (or an `OutputStyle` built from it) into helpers that print success/error/info.
   - Respect `NO_COLOR` and TTY detection for `auto`; `always` / `never` override.

2. **Central output helpers**
   - e.g. `message::success("Done")`, `message::error("File not found")`, `message::info("...")`, `message::warning("...")`.
   - Optional: dim hints, consistent prefixes (e.g. `✔` / `✘` or `Success:` / `Error:`).

3. **Errors**
   - Keep existing reporter formatting; add color for file/line, error kind, and suggestions (e.g. red for error, cyan for location, dim for hint).
   - Ensure `--quiet` suppresses everything except fatal error message (and exit code).

**Deliverables:** Color-aware output module; `--color` wired; errors and key commands (e.g. `run`, `check`, `test`) using it.

---

### Phase B: REPL upgrade (discoverable, pleasant)

**Objective:** Make `dal repl` a place users want to stay: history, clarity, and a path to AI.

1. **Input UX**
   - **History:** Line editing and history (e.g. `rustyline` or `nu-ansi-term` + readline-style).
   - **Multiline:** Accept pasted blocks and continue on incomplete input (e.g. open `{` / `(`) instead of erroring.
   - **Clear prompt:** Keep `dal[n]>`; optional short hint (e.g. `help`, `exit`) on first line or when empty.

2. **Output UX**
   - **Color:** Syntax highlight input (optional, can be Phase B.2); colorize result vs error (reuse Phase A).
   - **Values:** Pretty-print `Value` (indent structs/lists, truncate long strings with `…`).
   - **REPL help:** Keep `help`; add one-line summaries for `run`, `check`, `ai code`, etc., so REPL feels like a hub.

3. **Future hook**
   - Design so `ai repl` or an in-REPL `ask "..."` can be added without redoing the loop (e.g. “command” vs “DAL code” dispatch).

**Deliverables:** REPL with history and multiline; colored result/error; improved `help`; doc update.

---

### Phase C: Structured output and scripting

**Objective:** Scripts and AI tooling can consume output without scraping prose.

1. **`--json`**
   - Global or per-command: when set, print only machine-readable JSON (e.g. `{ "ok": true, "result": ... }` or `{ "ok": false, "error": "..." }`).
   - Commands to prioritize: `run` (exit code + result summary), `check`, `test` (results), `chain list`, `chain balance`, `db query`, `ai code` (generated code or error).

2. **Exit codes**
   - Document and hold: `0` success, `1` usage/application failure, `2` (optional) partial/configuration error. Use consistently so scripts can rely on them.

3. **Stability**
   - Mark which commands guarantee stable JSON schema (e.g. in CLI_QUICK_REFERENCE or a small “Scripting” section). Avoid breaking those schemas in minor releases.

**Deliverables:** `--json` on 5–10 high-value commands; exit-code doc; scripting note in docs.

---

### Phase D: Interactive wizards and progress

**Objective:** First-run and “many steps” flows feel guided, not cryptic.

1. **Wizards**
   - **`dal new`:** If no args or `--interactive`, prompt for name, project type, optional template; show summary and confirm before creating.
   - **`dal init`:** Optional prompts for project type and key options (e.g. chain, web, lib).
   - Keep non-interactive behavior when args are provided (script-friendly).

2. **Progress and long operations**
   - For long-running commands (e.g. `test` with many files, `bench`, `ai code` when slow): progress indicator or spinner when stderr is a TTY; plain log lines when not.
   - Optional: progress bar for “N of M” (e.g. tests, files).

3. **Tables**
   - For list output (`chain list`, `agent list`, `db tables`): print aligned tables when TTY; when not TTY or `--json`, use JSON or compact line format.

**Deliverables:** Interactive `dal new` (and optionally `dal init`); progress/spinner for 2–3 heavy commands; table formatting for 2–3 list commands.

---

### Phase E: Help and discovery polish

**Objective:** Every surface (main help, subcommand help, errors) reinforces “what can I do?” and “what’s next?”.

1. **Main help**
   - Consider 1–2 line “Getting started” at top (e.g. “New? Try: dal new myapp --type web && dal run myapp/app.dal”).
   - Keep use-case sections; ensure QUICK EXAMPLES are copy-pastable and cover chain, AI, web.

2. **Subcommand help**
   - Where clap generates `dal <cmd> --help`, add short “Use case” lines and 1 example per subcommand (e.g. in long_about or after_options_help).
   - Align wording with CLI_DESIGN (use-case first, project-type agnostic).

3. **Errors**
   - Where possible, append “Try: dal &lt;suggestion&gt;” or “See: dal help” / “dal &lt;cmd&gt; --help”.
   - Document common errors and fixes in a short “Troubleshooting” in CLI_QUICK_REFERENCE or getting started.

**Deliverables:** Updated main help text; subcommand examples and use-case blurbs; error suggestions; troubleshooting snippet.

---

### Phase F: Natural language and AI entry points

**Objective:** Realize the “AI-forward” principle with concrete entry points.

1. **`dal ask "<prompt>"`**
   - Single entry: “ask” runs through AI (existing service/ai stack); response printed (and optionally `--json` or `-o file`).
   - Help: “Ask anything about DAL, your project, or get code suggestions.”

2. **`dal "<prompt>"` (optional)**
   - If first arg doesn’t match a subcommand and looks like a question/prompt (e.g. no `-`, not a path), treat as `dal ask "..."`.
   - Requires clear rules so `dal run app.dal` never becomes “ask”.

3. **`ai repl`**
   - AI-assisted REPL: same REPL loop as Phase B, but user can type a natural language request and get code or explanation inline (or as a special command like `ask "..."` inside `dal repl`).

**Deliverables:** `dal ask "<prompt>"`; optional shorthand; design and (if time) stub for `ai repl` or in-REPL ask.

---

### Phase order and dependencies

| Phase | Depends on | Suggested order |
|-------|------------|------------------|
| A (color) | — | 1 |
| B (REPL) | A | 2 |
| C (--json) | — | Can parallel with A |
| D (wizards, progress) | A | 3 |
| E (help polish) | — | 4 (can start early) |
| F (dal ask, AI) | C (for --json on ask) | 5 |

---

### Design principles (reminder)

- **Progressive:** Default experience is friendly; power users get `--quiet`, `--no-banner`, `--json`.
- **Consistent:** Same color rules, same message style, same exit codes across commands.
- **Discoverable:** Help and errors answer “what can I do?” and “what went wrong?”.
- **Beautiful:** Clean banner, readable tables, subtle color, optional progress — not noisy or childish.
