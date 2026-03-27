#!/bin/bash
# Load .env and start the DAL CEO server (so OPENAI_API_KEY is always set).
cd "$(dirname "$0")"

PROFILE="${DAL_CEO_PROFILE:-auto}"
PORT="${PORT:-${DAL_CEO_PORT:-4040}}"
PID_FILE=".dal-ceo.pid"
LOCK_DIR=".dal-ceo.start.lock"
LOCK_PID_FILE="$LOCK_DIR/pid"
REPLACE_RUNNING="${DAL_CEO_REPLACE_RUNNING:-0}"
LOCK_STALE_SECONDS="${DAL_CEO_LOCK_STALE_SECONDS:-120}"
# Init behavior on start:
# - if_missing (default): run agent.dal only when no serve_agent_id is present in runtime snapshot
# - always: always run agent.dal before serve
# - never: never run agent.dal in start.sh
INIT_MODE="${DAL_CEO_INIT_MODE:-if_missing}"
if [ $# -ge 1 ]; then
  case "$1" in
    --profile)
      PROFILE="${2:-auto}"
      ;;
    safe|fallback|ramp|auto)
      PROFILE="$1"
      ;;
  esac
fi

release_start_lock() {
  [ -d "$LOCK_DIR" ] && rmdir "$LOCK_DIR" 2>/dev/null || true
}

acquire_start_lock() {
  if mkdir "$LOCK_DIR" 2>/dev/null; then
    echo "$$" > "$LOCK_PID_FILE"
    trap 'release_start_lock; rm -f "$PID_FILE"' EXIT INT TERM
    return 0
  fi

  # Lock already exists. Try to determine whether it is stale and self-heal.
  local lock_pid=""
  if [ -f "$LOCK_PID_FILE" ]; then
    lock_pid="$(cat "$LOCK_PID_FILE" 2>/dev/null || true)"
  fi

  if is_pid_running "$lock_pid"; then
    echo "Another start operation is already in progress (pid=$lock_pid, lock: $LOCK_DIR)."
    exit 1
  fi

  # No live lock owner PID; if lock is old enough, clear and retry once.
  local now mtime age
  now="$(date +%s)"
  mtime="$(stat -f %m "$LOCK_DIR" 2>/dev/null || echo "$now")"
  age=$((now - mtime))
  if [ "$age" -lt 0 ]; then age=0; fi

  if [ "$age" -ge "$LOCK_STALE_SECONDS" ]; then
    echo "Detected stale start lock (age=${age}s). Recovering..."
    rm -rf "$LOCK_DIR"
    if mkdir "$LOCK_DIR" 2>/dev/null; then
      echo "$$" > "$LOCK_PID_FILE"
      trap 'release_start_lock; rm -f "$PID_FILE"' EXIT INT TERM
      return 0
    fi
  fi

  echo "Another start operation is already in progress (lock: $LOCK_DIR, age=${age}s)."
  echo "If this is truly stale, wait ${LOCK_STALE_SECONDS}s or remove it manually: rm -rf $LOCK_DIR"
  exit 1
}

port_pids() {
  lsof -ti "tcp:$PORT" -sTCP:LISTEN 2>/dev/null || true
}

is_pid_running() {
  local pid="$1"
  [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null
}

cleanup_stale_pidfile() {
  if [ -f "$PID_FILE" ]; then
    local pid
    pid="$(cat "$PID_FILE" 2>/dev/null || true)"
    if ! is_pid_running "$pid"; then
      rm -f "$PID_FILE"
    fi
  fi
}

preflight_single_instance() {
  cleanup_stale_pidfile
  if [ -f "$PID_FILE" ]; then
    local pid
    pid="$(cat "$PID_FILE" 2>/dev/null || true)"
    if is_pid_running "$pid"; then
      echo "DAL CEO already running (pid=$pid)."
      echo "Stop it first or run: DAL_CEO_REPLACE_RUNNING=1 ./start.sh"
      exit 1
    fi
    rm -f "$PID_FILE"
  fi
}

preflight_port() {
  local pids
  pids="$(port_pids)"
  [ -z "$pids" ] && return 0

  if [ "$REPLACE_RUNNING" = "1" ]; then
    echo "Port $PORT is in use. Replacing listener(s): $pids"
    for pid in $pids; do
      kill "$pid" 2>/dev/null || true
    done
    sleep 1
    return 0
  fi

  if curl -fsS --max-time 1 "http://localhost:$PORT/health" >/dev/null 2>&1; then
    echo "DAL CEO is already running on http://localhost:$PORT."
  else
    echo "Port $PORT is in use by PID(s): $pids."
  fi
  echo "Refusing to start a second server."
  echo "Use one of:"
  echo "  - Stop existing process and rerun"
  echo "  - Start on another port: DAL_CEO_PORT=4041 ./start.sh"
  echo "  - Replace listener automatically: DAL_CEO_REPLACE_RUNNING=1 ./start.sh"
  exit 1
}

write_pidfile() {
  echo "$$" > "$PID_FILE"
}

should_init_agent() {
  case "$INIT_MODE" in
    always) return 0 ;;
    never) return 1 ;;
    if_missing|*)
      if [ ! -f ".dal/agent_runtime.json" ]; then
        return 0
      fi
      # Lightweight check without jq: consider initialized when serve_agent_id is present and not null/empty.
      if grep -Eq '"serve_agent_id"[[:space:]]*:[[:space:]]*"[^"]+"' ".dal/agent_runtime.json"; then
        return 1
      fi
      return 0
      ;;
  esac
}

apply_profile_defaults() {
  case "$PROFILE" in
    safe)
      : "${DAL_AGENT_POLICY_DEFAULT:=reply_only}"
      : "${DAL_AGENT_GUARDS_STRICT_MODE:=1}"
      : "${DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED:=1}"
      : "${DAL_AGENT_ENABLE_LEGACY_TEXT_JSON:=0}"
      ;;
    fallback)
      : "${DAL_AGENT_POLICY_DEFAULT:=reply_only}"
      : "${DAL_AGENT_GUARDS_STRICT_MODE:=1}"
      : "${DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED:=0}"
      : "${DAL_AGENT_ENABLE_LEGACY_TEXT_JSON:=1}"
      ;;
    ramp)
      : "${DAL_AGENT_POLICY_DEFAULT:=auto}"
      : "${DAL_AGENT_GUARDS_STRICT_MODE:=0}"
      : "${DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED:=1}"
      : "${DAL_AGENT_ENABLE_LEGACY_TEXT_JSON:=0}"
      ;;
    auto|*)
      # Preserve explicit env values; otherwise rely on runtime defaults.
      ;;
  esac
}

if [ -f .env ]; then
  while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    [[ -z "${line// }" ]] && continue
    export "$line"
  done < .env
fi
apply_profile_defaults
# DAL CEO server relies on internal `sh::run` reads (public UI files, runtime snapshots).
# Default to trusted unless explicitly overridden by env/.env.
: "${DAL_AGENT_SHELL_TRUST:=trusted}"
# Sync OpenAI env: DAL stdlib accepts OPENAI_* or DAL_OPENAI_*; server.dal and some tools expect OPENAI_*
[ -n "$OPENAI_API_KEY" ] && [ -z "$DAL_OPENAI_API_KEY" ] && export DAL_OPENAI_API_KEY="$OPENAI_API_KEY"
[ -n "$DAL_OPENAI_API_KEY" ] && [ -z "$OPENAI_API_KEY" ] && export OPENAI_API_KEY="$DAL_OPENAI_API_KEY"
[ -n "$DAL_OPENAI_MODEL" ] && [ -z "$OPENAI_MODEL" ] && export OPENAI_MODEL="$DAL_OPENAI_MODEL"
[ -n "$OPENAI_MODEL" ] && [ -z "$DAL_OPENAI_MODEL" ] && export DAL_OPENAI_MODEL="$OPENAI_MODEL"
# Project root (directory containing static/ and server.dal) for static file serving
export DAL_CEO_ROOT="$(pwd)"
# Prefer repo-built dal so all routes (history, workflow, x, etc.) register. Build with: cd .. && cargo build
DAL_BIN=""
if [ -x "../target/release/dal" ]; then DAL_BIN="../target/release/dal"; fi
if [ -x "../target/debug/dal" ] && [ -z "$DAL_BIN" ]; then DAL_BIN="../target/debug/dal"; fi
if [ -n "$DAL_BIN" ]; then
  echo "Using repo-built dal: $DAL_BIN"
else
  DAL_BIN="dal"
fi
preflight_single_instance
preflight_port
acquire_start_lock
write_pidfile

# Initialize only when needed (or when explicitly forced).
if should_init_agent; then
  echo "Initializing agent (agent.dal, mode=$INIT_MODE)..."
  "$DAL_BIN" run agent.dal || true
else
  echo "Skipping agent initialization (mode=$INIT_MODE, serve_agent appears present)."
fi

echo "Starting server on port $PORT (profile=$PROFILE, OPENAI_API_KEY is ${OPENAI_API_KEY:+set}, X is ${X_API_KEY:+set})..."
echo "Policy=${DAL_AGENT_POLICY_DEFAULT:-auto} strict=${DAL_AGENT_GUARDS_STRICT_MODE:-0} native_tools=${DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED:-1} legacy_text_json=${DAL_AGENT_ENABLE_LEGACY_TEXT_JSON:-0}"
echo "To verify X: curl -s http://localhost:$PORT/api/x/status"
exec "$DAL_BIN" serve server.dal --port "$PORT"