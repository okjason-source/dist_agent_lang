## Wiring into the command bar

The command bar has three modes: **Run** (execute a shell command), **Chat** (single-turn LLM chat, same as `dal agent chat`), and **Agent** (run a DAL agent with tools on the prompt). You can **bypass** or **plug in** from your own script (e.g. a custom HTML page that embeds the IDE or a userscript).

### Bypass (skip default behavior)

Set a global **before** handler; if it returns `true` (or a Promise that resolves to `true`), the built-in handling is skipped and the input is cleared for **any** mode (Run, Chat, Agent):

```javascript
window.DAL_IDE_ON_BEFORE_COMMAND = function(mode, text) {
  // mode is 'run' | 'chat' | 'agent'
  console.log('Command:', mode, text);
  // Return true to bypass (skip default). Return false/undefined to use default.
  return false;
};
```

### Plug-in for Chat (custom reply)

For **Chat** only, you can supply the reply yourself so the IDE never calls the backend. Set `DAL_IDE_CHAT_HANDLER(text)`. If it returns a string (or Promise resolving to a string), that is shown as the agent reply. If it returns `null`/`undefined`, the default runs (`POST /api/agent/chat`):

```javascript
window.DAL_IDE_CHAT_HANDLER = async function(text) {
  // Return a string to use as the reply (no backend call). Return null to use default.
  return null;
};
```

### Backend APIs used by default

- **Run:** `POST /api/agent/run_command` with `{ cmd, args?, cwd? }`
- **Chat:** `POST /api/agent/chat` with `{ text }` → `{ reply }` (DAL LLM, same as `dal agent chat`)
- **Agent:** `POST /api/agent/prompt` with `{ text, context?, workspace? }`

You can call those endpoints from your handlers to keep the same backend behavior but change the UI, or use your own.
