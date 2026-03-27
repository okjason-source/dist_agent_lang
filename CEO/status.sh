#!/bin/bash
cd "$(dirname "$0")"

PORT="${PORT:-${DAL_CEO_PORT:-4040}}"
PID_FILE=".dal-ceo.pid"

if [ -f "$PID_FILE" ]; then
  PID="$(cat "$PID_FILE" 2>/dev/null || true)"
  if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
    echo "DAL CEO running (pid=$PID, port=$PORT)."
    if curl -fsS --max-time 1 "http://localhost:$PORT/health" >/dev/null 2>&1; then
      echo "Health: OK"
    else
      echo "Health: Unreachable"
    fi
    exit 0
  fi
  echo "PID file exists but process is not running; removing stale pid file."
  rm -f "$PID_FILE"
fi

PIDS="$(lsof -ti "tcp:$PORT" -sTCP:LISTEN 2>/dev/null || true)"
if [ -n "$PIDS" ]; then
  echo "Port $PORT has listener(s): $PIDS (not managed by this pid file)."
  exit 0
fi

echo "DAL CEO is not running."
exit 1
