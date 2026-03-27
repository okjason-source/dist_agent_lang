# X (Twitter) integration — quick checklist

Use this to verify X is connected and .env is loaded.

## 1. .env has all four X vars

```bash
cd agent_assistant
grep -E '^X_API_KEY=|^X_API_SECRET=|^X_ACCESS_TOKEN=|^X_ACCESS_TOKEN_SECRET=' .env | wc -l
```

Should print **4**. If not, add the missing vars from your X Developer app (Keys and tokens).

## 2. Start the server so .env is loaded

**Always start with:**

```bash
cd agent_assistant
./start.sh
```

You should see: `X is set` in the startup line if `X_API_KEY` is in `.env`. If you start with `dal serve server.dal --port 4040` instead, .env is **not** loaded and X will not work.

## 3. Check X status (server must be running)

```bash
curl -s http://localhost:4040/api/x/status
```

- **Expected when configured:**  
  `{"configured":true,"message":"X configured (all 4 env vars set)..."}`
- **When not configured:**  
  `{"configured":false,"message":"X not configured. Add X_API_KEY..."}`

If you get no response or 404, **restart the server** with `./start.sh` so it loads the latest routes.

## 4. Python dependency for posting (use a Python venv on macOS)

The post flow uses `scripts/x_post.py`, which needs the **Python** package `requests_oauthlib`. On macOS (Homebrew Python) use a **Python** venv so you don’t hit “externally-managed-environment”. (This is separate from **dal venv**, which is for DAL project root/deps only and doesn’t install Python packages.)

```bash
cd agent_assistant
./scripts/setup_venv.sh
```

This creates `venv/` and installs `scripts/requirements.txt`. The server will use `venv/bin/python3` when present. Then start with `./start.sh` as usual. Optionally you can also use a DAL venv for the server: `dal venv create agent_assistant && dal serve server.dal --port 4040 --venv agent_assistant` (still need the Python venv for x_post.py).

## 5. Test post (optional)

```bash
cd agent_assistant
chmod +x scripts/test_x.sh   # once, if needed
./scripts/test_x.sh          # status only
./scripts/test_x.sh --post   # status + one test tweet
```

Check your X timeline for the test tweet.

## 6. If the agent says it posted but you don’t see tweets

1. Run `curl -s http://localhost:4040/api/x/status`. If `configured` is **false**, restart with `./start.sh`.
2. If `configured` is **true**, run `./scripts/test_x.sh --post`. If that returns `"ok": true` and you see the tweet, the backend is fine — the agent may be calling the wrong URL or not actually calling the API.
3. If the test script returns an error (e.g. invalid credentials), fix the four values in `.env` and restart.
