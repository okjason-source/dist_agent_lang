#!/usr/bin/env bash
# Run mutation testing in a tmux session (survives disconnect).
#
# INSTRUCTIONS
# ------------
# From repo root (lang_mark):
#   dist_agent_lang/scripts/mutation_testing/run_with_tmux.sh
# From dist_agent_lang:
#   ./scripts/mutation_testing/run_with_tmux.sh
#
# Tmux:
#   Detach:  Ctrl+B then D
#   Reattach: tmux attach -t mutation-testing
#   Kill:    tmux kill-session -t mutation-testing
#
# Test suite (what gets run for each mutant):
#   MUTATION_TEST_SUITE=lib   -> cargo test --lib (faster; integration tests in tests/*.rs do NOT run)
#   MUTATION_TEST_SUITE=tests -> cargo test --tests (slower; runs tests in tests/*.rs, catches more lexer/etc.)
#   Default: tests (so lexer_mutation_tests, lexer_boundary_tests, lexer_tokens_tests run)
#
# Optional env (before running):
#   MUTATION_TEST_SUITE=lib   ./scripts/mutation_testing/run_with_tmux.sh   # lib-only
#   MUTATION_TEST_SUITE=tests ./scripts/mutation_testing/run_with_tmux.sh   # same as default

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SESSION_NAME="mutation-testing"
LOG_DIR="$PROJECT_ROOT/logs/mutation_testing"
LOG_FILE="$LOG_DIR/mutation_testing.log"

# lib = only unit tests in src/; tests = unit + integration tests in tests/
TEST_SUITE="${MUTATION_TEST_SUITE:-tests}"
CARGO_TEST_ARGS="--tests"
if [[ "$TEST_SUITE" == "lib" ]]; then
  CARGO_TEST_ARGS="--lib"
fi

cd "$PROJECT_ROOT"
mkdir -p "$LOG_DIR"

export CARGO_HTTP_CAINFO="${CARGO_HTTP_CAINFO:-/usr/local/etc/ca-certificates/cert.pem}"
export CARGO_NET_GIT_FETCH_WITH_CLI="${CARGO_NET_GIT_FETCH_WITH_CLI:-true}"
export SSL_CERT_FILE="${SSL_CERT_FILE:-/usr/local/etc/ca-certificates/cert.pem}"

if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
  echo "Session '$SESSION_NAME' already exists. Attach with: tmux attach -t $SESSION_NAME"
  exit 0
fi

echo "Using: cargo test $CARGO_TEST_ARGS (set MUTATION_TEST_SUITE=lib for --lib only)"

tmux new-session -d -s "$SESSION_NAME" -c "$PROJECT_ROOT" " \
  export CARGO_HTTP_CAINFO=\"$CARGO_HTTP_CAINFO\"; \
  export CARGO_NET_GIT_FETCH_WITH_CLI=\"$CARGO_NET_GIT_FETCH_WITH_CLI\"; \
  export SSL_CERT_FILE=\"$SSL_CERT_FILE\"; \
  cargo mutants --iterate --gitignore true --jobs 1 \
    --exclude src/main.rs \
    --exclude src/stdlib/iot.rs \
    --exclude src/stdlib/desktop.rs \
    --exclude tests/ffi_performance_tests.rs \
    --exclude tests/example_tests.rs \
    --exclude src/ffi/security.rs \
    --exclude src/parser/ast.rs \
    --timeout 300 \
    --build-timeout 300 \
    -- $CARGO_TEST_ARGS \
  2>&1 | tee -a '$LOG_FILE'; \
  echo; echo Done. Press Enter to close.; read
"
echo "Started tmux session: $SESSION_NAME"
echo "  Attach: tmux attach -t $SESSION_NAME"
echo "  Detach: Ctrl+B then D"
echo "  Log:    $LOG_FILE"
