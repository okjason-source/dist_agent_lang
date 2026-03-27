#!/bin/bash
set -euo pipefail

BASE_URL="${1:-http://localhost:4040}"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Missing required command: $1"
    exit 1
  }
}

need_cmd curl
need_cmd python3

echo "Smoke check against: $BASE_URL"

echo
echo "1) Health check"
curl -fsS "$BASE_URL/health" >/dev/null
echo "OK /health"

echo
echo "2) /api/message (tool loop route expectation not strict)"
MESSAGE_JSON="$(curl -fsS -X POST "$BASE_URL/api/message" \
  -H 'Content-Type: application/json' \
  -d '{"content":"Give a one sentence hello.","sender_id":"smoke","policy":"auto"}')"

echo "$MESSAGE_JSON" | python3 - <<'PY'
import json, sys
payload = json.loads(sys.stdin.read())
required = ["reply", "route", "tool_steps_count", "max_steps_reached"]
missing = [k for k in required if k not in payload]
if missing:
    raise SystemExit(f"/api/message missing keys: {missing}")
print("OK /api/message keys present")
print(f"route={payload.get('route')} tool_steps_count={payload.get('tool_steps_count')} max_steps_reached={payload.get('max_steps_reached')}")
PY

echo
echo "3) /api/task in reply_only posture"
TASK_JSON="$(curl -fsS -X POST "$BASE_URL/api/task" \
  -H 'Content-Type: application/json' \
  -d '{"description":"Say exactly: smoke ok","policy":"reply_only"}')"

echo "$TASK_JSON" | python3 - <<'PY'
import json, sys
payload = json.loads(sys.stdin.read())
required = ["result", "route", "tool_steps_count", "max_steps_reached"]
missing = [k for k in required if k not in payload]
if missing:
    raise SystemExit(f"/api/task missing keys: {missing}")
route = payload.get("route")
if route != "reply_only":
    raise SystemExit(f"/api/task expected route=reply_only, got {route!r}")
print("OK /api/task keys present and route=reply_only")
PY

echo
echo "4) /api/status check"
STATUS_JSON="$(curl -fsS "$BASE_URL/api/status")"
echo "$STATUS_JSON" | python3 - <<'PY'
import json, sys
payload = json.loads(sys.stdin.read())
if not isinstance(payload, dict):
    raise SystemExit("/api/status did not return a JSON object")
print("OK /api/status returned object")
PY

echo
echo "Smoke check passed."
