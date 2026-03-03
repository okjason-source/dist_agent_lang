# DAL Venv — User Guide

**Venvs** are named, reusable execution environments for DAL. Each venv has a **project root**, a **dependency set** (from that project’s `dal.toml` / `dal.lock`), and a **security profile**. Running a script “in a venv” uses that environment every time, so you get consistent deps and clear security boundaries.

---

## Quick start

```bash
# Create a venv for the current directory (default profile: relaxed)
dal venv create myvenv

# Run a script inside that venv (script path is relative to your current directory)
dal venv run myvenv ./scripts/task.dal

# List venvs
dal venv list

# Show one venv
dal venv show myvenv

# Remove a venv (does not delete files on disk)
dal venv delete myvenv
```

---

## What is a venv?

A **venv** is a named environment that defines:

| What | Meaning |
|------|--------|
| **Root** | A project directory. Imports and dependency resolution use this as the project root. |
| **Dependencies** | Resolved from that root’s `dal.toml` and `dal.lock` when you run a script. |
| **Profile** | Security rules: which built-in namespaces are allowed (see [Profiles](#profiles)). |

When you run `dal venv run myapp script.dal`, DAL uses **myapp**’s root and dependencies to resolve imports, and applies **myapp**’s profile. The script file itself is located relative to your **current working directory**; only imports inside the script use the venv’s root and deps.

---

## Commands

All venv commands live under `dal venv`:

| Command | Description |
|---------|-------------|
| `dal venv create <name> [--dir <path>] [--profile strict\|relaxed]` | Create a venv. `--dir` defaults to the current directory. `--profile` defaults to **relaxed**. |
| `dal venv list` | List venvs (project-local, then global if configured). |
| `dal venv show <name>` | Show a venv’s root and profile. |
| `dal venv run <name> <script.dal>` | Run a script inside the venv. |
| `dal venv delete <name>` | Remove the venv from the registry (no files are deleted). |

### Create

- **&lt;name&gt;** is a unique label (e.g. `myapp`, `ci`).
- **--dir** is the project root. If omitted, the current directory is used. Can be relative (e.g. `./other-project`) or absolute.
- **--profile** can be `strict` or `relaxed` (default: `relaxed`).

If the root has no `dal.toml`, you’ll get a warning; dependency resolution may fail when you run scripts until you add one (e.g. `dal init`).

### Run

- **&lt;script.dal&gt;** is the script to run. It is resolved relative to your **current directory** when you invoke `dal`, not relative to the venv root.
- Imports inside that script are resolved using the **venv’s root** and the venv’s **resolved dependencies**.

Example: from `/home/me`, running `dal venv run myapp ./work/task.dal` runs `/home/me/work/task.dal`; any `import foo` in `task.dal` is resolved from the **myapp** venv’s project root and its lockfile.

---

## Profiles

| Profile | Use case | What’s allowed |
|--------|----------|----------------|
| **relaxed** | Normal development, trusted scripts | All stdlib namespaces (current default behavior). Shell and FFI follow your normal config. |
| **strict** | CI, untrusted code, least privilege | Only a fixed set of stdlib namespaces (e.g. `chain`, `crypto`, `log`, `config`, `key`, `auth`, `evolve`, `sync`, `json`, `test`). No `sh`, no stdlib `service`. User-defined services (your own types) are still allowed. |

- **relaxed** is the default when you create a venv without `--profile`.
- Use **strict** when you want a small, auditable surface (e.g. a script that only needs chain/crypto/log). If the script calls a disallowed namespace, you’ll get a clear error: `namespace 'X' not allowed in this venv profile (strict)`.

---

## Where venvs are stored

- **Project-local (default):** `.dal/venvs.json` in the directory where you run `dal venv create`. That file maps venv names to `root` and `profile`.
- **Global (optional):** Set `DAL_VENV_REGISTRY` to a file path, or use the default `~/.config/dal/venvs.json` (or your platform’s config dir). When you run `dal venv run <name>`, DAL looks up the name in project-local first, then in the global registry.

So “create once, run many” means: create a venv (stored in `.dal/venvs.json` or the global registry), then run any script in that venv with `dal venv run <name> <script.dal>`.

---

## Examples

**Create a venv for the current project (relaxed):**
```bash
dal venv create myvenv
```

**Create a strict venv for CI:**
```bash
dal venv create ci-env --dir . --profile strict
dal venv run ci-env ./tests/suite.dal
```

**Create a venv for another project:**
```bash
dal venv create other --dir ../other-project --profile relaxed
dal venv run other ./scripts/run.dal
```

**List and inspect:**
```bash
dal venv list
dal venv show myvenv
```

**Delete a venv:**
```bash
dal venv delete myvenv
```

---

## Security and boundaries

- **Blast radius:** A script run in a venv only uses that venv’s root and dependencies; it doesn’t see other projects’ code or arbitrary paths outside that setup.
- **Auditability:** You can document “this job always runs in venv X with profile Y” and review what that allows.
- **Strict profile:** Use it to lock down which stdlib a script can call (no shell, no service namespace, etc.) while still allowing your own service types and the allow-listed namespaces.

---

## See also

- **[VENV_FIRST_CLASS_DESIGN.md](VENV_FIRST_CLASS_DESIGN.md)** — Design and implementation details.
- **[VENV_CONCEPT.md](VENV_CONCEPT.md)** — Rationale and security benefits.
