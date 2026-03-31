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
#
# Scoped runs (see run_with_tmux.sh for the same variables):
#   MUTATION_FILE=src/cli.rs ./scripts/mutation_testing/run_from_terminal.sh

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LOG_DIR="$PROJECT_ROOT/logs/mutation_testing"
LOG_FILE="$LOG_DIR/mutation_testing.log"
INNER_SH="$SCRIPT_DIR/run_cargo_mutants.sh"

cd "$PROJECT_ROOT"
mkdir -p "$LOG_DIR"

export CARGO_HTTP_CAINFO="${CARGO_HTTP_CAINFO:-/usr/local/etc/ca-certificates/cert.pem}"
export CARGO_NET_GIT_FETCH_WITH_CLI="${CARGO_NET_GIT_FETCH_WITH_CLI:-true}"
export SSL_CERT_FILE="${SSL_CERT_FILE:-/usr/local/etc/ca-certificates/cert.pem}"

export MUTATION_TIMEOUT="${MUTATION_TIMEOUT:-300}"
export MUTATION_BUILD_TIMEOUT="${MUTATION_BUILD_TIMEOUT:-300}"
export MUTATION_EXCLUDE_AST="${MUTATION_EXCLUDE_AST:-1}"

TEST_SUITE="${MUTATION_TEST_SUITE:-tests}"
echo "Using: cargo test $( [[ "$TEST_SUITE" == "lib" ]] && echo --lib || echo --tests )"

if [[ -n "${MUTATION_FILE:-}" || -n "${MUTATION_FILES:-}" ]]; then
  echo "Scoped mutation run (MUTATION_FILE / MUTATION_FILES)."
fi

bash "$INNER_SH" 2>&1 | tee -a "$LOG_FILE"
