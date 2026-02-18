#!/usr/bin/env bash
# Run all fuzz targets in a tmux session (survives disconnect).
#
# INSTRUCTIONS
# ------------
# From repo root (lang_mark):
#   dist_agent_lang/scripts/fuzzing/run_with_tmux.sh
# From dist_agent_lang:
#   ./scripts/fuzzing/run_with_tmux.sh
#
# Tmux:
#   Detach:  Ctrl+B then D
#   Reattach: tmux attach -t dal-fuzzing
#   Kill:    tmux kill-session -t dal-fuzzing
#
# Logs:
#   logs/fuzzing/fuzz_lexer.log
#   logs/fuzzing/fuzz_parser.log
#   logs/fuzzing/fuzz_runtime.log
#   logs/fuzzing/fuzz_stdlib.log

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SESSION_NAME="dal-fuzzing"
LOG_DIR="$PROJECT_ROOT/logs/fuzzing"

cd "$PROJECT_ROOT"
mkdir -p "$LOG_DIR"

if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
  echo "Session '$SESSION_NAME' already exists. Attach with: tmux attach -t $SESSION_NAME"
  exit 0
fi

# Create session with first pane running fuzz_lexer
tmux new-session -d -s "$SESSION_NAME" -c "$PROJECT_ROOT" -n fuzz \
  "cargo +nightly fuzz run fuzz_lexer 2>&1 | tee '$LOG_DIR/fuzz_lexer.log'"

# Split and run fuzz_parser
tmux split-window -h -t "$SESSION_NAME" -c "$PROJECT_ROOT" \
  "cargo +nightly fuzz run fuzz_parser 2>&1 | tee '$LOG_DIR/fuzz_parser.log'"

# Split bottom-left for fuzz_runtime
tmux split-window -v -t "$SESSION_NAME:0.0" -c "$PROJECT_ROOT" \
  "cargo +nightly fuzz run fuzz_runtime 2>&1 | tee '$LOG_DIR/fuzz_runtime.log'"

# Split bottom-right for fuzz_stdlib
tmux select-pane -t "$SESSION_NAME:0.1"
tmux split-window -v -t "$SESSION_NAME:0.1" -c "$PROJECT_ROOT" \
  "cargo +nightly fuzz run fuzz_stdlib 2>&1 | tee '$LOG_DIR/fuzz_stdlib.log'"

# Balance panes
tmux select-layout -t "$SESSION_NAME" tiled

echo "Started tmux session: $SESSION_NAME"
echo "  Attach:  tmux attach -t $SESSION_NAME"
echo "  Detach:  Ctrl+B then D"
echo "  Logs:    $LOG_DIR/fuzz_*.log"
