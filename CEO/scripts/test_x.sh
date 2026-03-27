#!/bin/bash
# Test X (Twitter) connection: 1) check /api/x/status  2) optionally post a test tweet.
# Run from agent_assistant: ./scripts/test_x.sh [--post]
# Server must be running (./start.sh) and .env must have X_* vars.

BASE="${BASE_URL:-http://localhost:4040}"

echo "=== X status ($BASE/api/x/status) ==="
curl -s "$BASE/api/x/status" | python3 -m json.tool 2>/dev/null || curl -s "$BASE/api/x/status"
echo ""

if [ "$1" = "--post" ]; then
  echo "=== Posting test tweet ==="
  curl -s -X POST "$BASE/api/x/post" -H "Content-Type: application/json" -d '{"text":"Test from Agent Assistant (scripts/test_x.sh)"}' | python3 -m json.tool 2>/dev/null || curl -s -X POST "$BASE/api/x/post" -H "Content-Type: application/json" -d '{"text":"Test from Agent Assistant"}'
  echo ""
fi
