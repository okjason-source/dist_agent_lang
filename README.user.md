# DAL IDE

**Unified language for Web, Blockchain & AI**

DAL IDE is the browser-based control station for the Distributed Agent Language (DAL). You use it to open projects, edit files, run scripts, and work with the DAL runtime.

## Running it locally

You need two things: the **backend** (the `dal` program that serves your project) and the **IDE** (this web app). Both run on your machine. The only traffic that may leave your computer is **LLM API calls** when you use AI features in DAL code.

### 1. One-time setup (only if you have the full project)

**If you cloned or downloaded the full project** (the folder that has `install.sh`, `dist_agent_lang/`, and `dal-ide/`), run from that folder:

```bash
./install.sh
```

That installs Rust/Node if needed, builds the `dal` backend, and puts it on your PATH. If your terminal can’t find `dal` later, use the full path: `~/.local/bin/dal` or `/usr/local/bin/dal`.

**If you didn’t clone the repo** — for example you have `dal` from `cargo install` or a release binary — you don’t need `install.sh`. Just use `dal ide serve` (see below).

### 2. Start the backend

In a terminal, go to the folder you want to work in (your project directory), then run:

```bash
dal ide serve -w . -p 3847
```

The **`-w .`** means “use the **current directory** as the workspace.”

**Run the backend with a Python venv:** If you want the backend (and any Python the backend uses, e.g. for FFI or agent tools) to use a specific virtual environment, activate the venv in the same terminal before starting the backend:

```bash
cd /path/to/your/project
source .venv/bin/activate   # or: .venv\Scripts\activate on Windows
dal ide serve -w . -p 3847
```

The backend process inherits your shell’s environment, so `VIRTUAL_ENV` and `PATH` point at the venv’s Python. To use a venv that is **not** in the current directory, activate it first (e.g. `source /path/to/other/.venv/bin/activate`), then run `dal ide serve`. The **current directory** is whatever folder your terminal is in when you run the command — for example your home directory (`~`) if you just opened a terminal, or a project folder if you ran `cd my-project` first. Names like **my-project** are just example folder names (your own project); there is no special “DAL project” folder unless you create one. So: `cd my-project` then `dal ide serve -w .` uses `my-project` as the workspace; or use an absolute path, e.g. `-w /Users/you/my-project`.

Leave the command running. The backend will serve that folder on port **3847**.

### 3. Open the IDE

- If you’re developing: run `npm run dev` in **`../dal-ide/`** (sibling of `dist_agent_lang` under `lang_mark/`), then open **http://localhost:5173** in your browser.
- If you’re using a pre-built copy (e.g. from a download or AWS): open the URL you were given for the IDE.

In the IDE, click **Open Folder** and enter the same path you used with `-w` (e.g. `.` for the current directory). Then open a file, choose a run config from the **Run** dropdown, and click **Run**.

## Stopping the servers

- **Backend** (`dal ide serve`): In the terminal where it’s running, press **Ctrl+C**.
- **Frontend** (`npm run dev`): Same — in the terminal where the dev server is running, press **Ctrl+C**.
- **A script you started from the IDE** (e.g. you clicked Run): Use the **Stop** button in the IDE’s command bar next to Run; it’s enabled while a run is in progress. That stops the script, not the backend.

**If a server is running in the background** (you closed the terminal or it’s detached), kill it by port. On macOS/Linux:

- Backend (port 3847): `lsof -ti :3847 | xargs kill`  
- Frontend (port 5173): `lsof -ti :5173 | xargs kill`

Or find the process: `lsof -i :3847` (or `:5173`), note the PID, then `kill <PID>`. On Windows you can use Task Manager or `netstat -ano` to find the PID, then `taskkill /PID <pid> /F`.

## Install as an app

You can install the IDE as a Progressive Web App from your browser (“Install” in Chrome/Edge or “Add to Home Screen” on mobile) for a standalone window. To wire into the command bar (Run/Chat/Agent), see [Wiring into the command bar](README.wiring.md).

## If something doesn’t work

- **“dal: command not found”** — Run `./install.sh` from the project root, or use the full path: `~/.local/bin/dal ide serve -w . -p 3847`. Make sure `~/.local/bin` is in your PATH (install.sh usually adds it to your shell config).
- **“Run failed” or 404 in the IDE** — Ensure the backend is running (`dal ide serve -w . -p 3847`) and the IDE is pointing at it (default is http://localhost:3847). If you built the IDE yourself and use a different port, set `VITE_IDE_BACKEND_URL` when building.  
  **If POST /api/lsp/document_symbols returns 404:** the running `dal` binary is from an **older build** that doesn’t include that route. **Fix:** run the backend from a fresh build so you don’t rely on whatever `dal` is on your PATH:
  ```bash
  # From repo root: build then run this binary (not the global dal)
  cargo build --release --manifest-path dist_agent_lang/Cargo.toml
  ./dist_agent_lang/target/release/dal ide serve -w . -p 3847
  ```
  Or run `./install.sh` to rebuild and reinstall `dal` to your PATH, then start the backend again.  
  **What breaks when document_symbols 404s:** Go to definition (F12) and the document outline stay empty for DAL files; the app does not crash.
- **Theme errors (e.g. “Unable to load … package.nls.json” or “dark_modern.json”)** — The Vite config serves the theme-defaults extension resources. If you still see 404s, clear the Vite cache and restart: `rm -rf node_modules/.vite` then `npm run dev`.

---

DAL IDE — part of the Distributed Agent Language toolchain.
