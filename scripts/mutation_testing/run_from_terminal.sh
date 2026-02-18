#!/usr/bin/env bash
# Run mutation testing in current terminal (use inside tmux/screen).
#
# INSTRUCTIONS
# ------------
# From dist_agent_lang:
#   ./scripts/mutation_testing/run_from_terminal.sh
# From repo root:
#   cd dist_agent_lang && ./scripts/mutation_testing/run_from_terminal.sh
#
# Test suite:
#   MUTATION_TEST_SUITE=lib   -> --lib only (faster)
#   MUTATION_TEST_SUITE=tests -> --tests (default; runs tests/*.rs, catches more)

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LOG_DIR="$PROJECT_ROOT/logs/mutation_testing"
LOG_FILE="$LOG_DIR/mutation_testing.log"

TEST_SUITE="${MUTATION_TEST_SUITE:-tests}"
CARGO_TEST_ARGS="--tests"
[[ "$TEST_SUITE" == "lib" ]] && CARGO_TEST_ARGS="--lib"

cd "$PROJECT_ROOT"
mkdir -p "$LOG_DIR"

export CARGO_HTTP_CAINFO="${CARGO_HTTP_CAINFO:-/usr/local/etc/ca-certificates/cert.pem}"
export CARGO_NET_GIT_FETCH_WITH_CLI="${CARGO_NET_GIT_FETCH_WITH_CLI:-true}"
export SSL_CERT_FILE="${SSL_CERT_FILE:-/usr/local/etc/ca-certificates/cert.pem}"

echo "Using: cargo test $CARGO_TEST_ARGS"

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
  2>&1 | tee -a "$LOG_FILE"
