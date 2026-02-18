# DAL CLI Design

**Principles:** Intuitive, on-brand, project-type agnostic, AI-forward.

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
| **Get Started** | new, init, run, test |
| **Build & Develop** | check, fmt, lint, watch, repl, bench, profile, optimize |
| **AI & Code Assistance** | ai code, explain, review, audit, test, fix, optimize-gas |
| **Agents & Automation** | agent create, send, list, mold |
| **Blockchain** | chain list, gas-price, balance, mint, asset |
| **Data & Infrastructure** | crypto, db, cloud |
| **Devices & IoT** | iot register, status, ai-predict |
| **Tools** | web, convert, doc, completions |

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
