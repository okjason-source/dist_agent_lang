#!/bin/bash
# Create a Python venv and install deps for scripts (e.g. x_post.py).
# This is separate from "dal venv" (which is for DAL project root/deps only).
# Run from agent_assistant: ./scripts/setup_venv.sh
# Then start the server with ./start.sh as usual.

set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"
# If script lives in scripts/, project root is parent
[ "$(basename "$ROOT")" = "scripts" ] && ROOT="$(dirname "$ROOT")"
cd "$ROOT"
if [ ! -d "venv" ]; then
  echo "Creating Python venv (for x_post.py; not the same as dal venv)..."
  python3 -m venv venv
fi
echo "Installing requirements..."
./venv/bin/pip install -r scripts/requirements.txt
echo "Done. Use ./start.sh; x_post.py will use venv/bin/python3."
echo "Optional: dal venv create agent_assistant && dal serve server.dal --port 4040 --venv agent_assistant (for DAL env; Python venv still used for x_post.py)."
