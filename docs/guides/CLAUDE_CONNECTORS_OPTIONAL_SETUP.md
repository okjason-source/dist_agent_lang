# Claude Connectors (Optional) for DAL Developers

This guide shows how to make DAL agents easy to access from Claude via MCP/connector tooling.

## Goals

- Keep DAL's default model/tooling behavior unchanged (no forced switch).
- Give developers a fast, copy-paste path to expose DAL as an optional connector.
- Reuse DAL's existing HTTP-first contract and MCP bridge.

## Mental model

- Your DAL agent remains an HTTP app (`dal agent serve` or `dal serve`).
- MCP/connector is an optional adapter that forwards tool calls to the same DAL HTTP endpoints.
- Claude integration is therefore additive, not a replacement for COO or existing DAL flows.

See also:
- `docs/IDE_AND_AGENT_INTEGRATION.md`
- `docs/guides/AGENT_SETUP_AND_USAGE.md#13-mcp-ide-bridge`

## 1) Start a DAL agent HTTP server

Run from your DAL project:

```bash
dal agent serve --port 4040
```

Or run your custom DAL app:

```bash
dal serve app.dal --port 4040
```

## 2) Install MCP bridge dependencies (one-time)

From the repository checkout:

```bash
cd COO/mcp
# or, from dist_agent_lang/: cd ../COO/mcp
npm install
```

## 3) Run the DAL MCP bridge

From a second terminal:

```bash
export DAL_AGENT_HTTP_BASE="http://127.0.0.1:4040"
dal mcp-bridge
```

Notes:
- Default transport is `stdio`.
- `DAL_MCP_TRANSPORT=http-stream` is available for environments that support streamable HTTP MCP transport.

## 4) Register DAL bridge in your Claude connector/MCP surface

In any Claude environment that supports MCP connectors/servers, add a server entry that runs:

- Command: `node`
- Args: `<path-to>/COO/mcp/src/server.js` (or `<path-to>/../COO/mcp/src/server.js` from `dist_agent_lang`)
- Working directory: `<path-to>/COO/mcp` (or `<path-to>/../COO/mcp`)
- Environment: `DAL_AGENT_HTTP_BASE=http://127.0.0.1:4040`

The MCP server then exposes DAL tool endpoints backed by your running DAL agent HTTP server.

## 5) Team-friendly developer experience (recommended)

For language users, provide this as an optional profile:

- Keep existing default setup as-is.
- Add a short "Enable Claude connector" section in onboarding docs.
- Include a local script to run both processes for developers.

Example helper script:

```bash
#!/usr/bin/env bash
set -euo pipefail

export DAL_AGENT_HTTP_BASE="${DAL_AGENT_HTTP_BASE:-http://127.0.0.1:4040}"

echo "[1/2] Starting DAL agent server on :4040"
dal agent serve --port 4040
```

Run MCP bridge in another terminal:

```bash
export DAL_AGENT_HTTP_BASE="${DAL_AGENT_HTTP_BASE:-http://127.0.0.1:4040}"
dal mcp-bridge
```

## Optional: use Claude as model provider without MCP

If a developer only wants Claude for generation (not connector tools), DAL already supports Anthropic directly:

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
export ANTHROPIC_MODEL="claude-3-5-sonnet-20241022"
```

This is independent from MCP connector setup.

## Troubleshooting quick checks

- `dal mcp-bridge` says missing `node_modules`:
  - Run `cd COO/mcp && npm install` (or `cd ../COO/mcp && npm install` from `dist_agent_lang`).
- MCP server starts but tool calls fail:
  - Verify `DAL_AGENT_HTTP_BASE` points to your active DAL HTTP server.
- Wrong script path:
  - Set `DAL_MCP_BRIDGE_SCRIPT` (or `DAL_COO_MCP_SCRIPT`) explicitly.

## Why this approach works for COO and language users

- COO keeps its current defaults and roadmap.
- Developers who prefer Claude connectors get a clear optional path.
- The same DAL HTTP API remains the source of truth, so behavior is consistent across curl, IDE, and connector clients.
