#!/bin/bash
cd "$(dirname "$0")"

PORT="${PORT:-${DAL_CEO_PORT:-4040}}"
PID_FILE=".dal-ceo.pid"

stopped=0

if [ -f "$PID_FILE" ]; then
  PID="$(cat "$PID_FILE" 2>/dev/null || true)"
  if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
    echo "Stopping DAL CEO pid=$PID"
    kill "$PID" 2>/dev/null || true
    sleep 1
    if kill -0 "$PID" 2>/dev/null; then
      echo "Process still running; sending SIGKILL to pid=$PID"
      kill -9 "$PID" 2>/dev/null || true
    fi
    stopped=1
  fi
  rm -f "$PID_FILE"
fi

PIDS="$(lsof -ti "tcp:$PORT" -sTCP:LISTEN 2>/dev/null || true)"
if [ -n "$PIDS" ]; then
  echo "Stopping listener(s) on port $PORT: $PIDS"
  for pid in $PIDS; do
    kill "$pid" 2>/dev/null || true
  done
  stopped=1
fi

if [ "$stopped" -eq 1 ]; then
  echo "Stopped."
else
  echo "No running DAL CEO found."
fi
