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
#   Default: tests (so lexer_mutation_tests, lexer_boundary_tests, lexer_tokens_tests,
#                   http_security_mutation_tests, and serve_security_parity_tests run)
#
# Scoped runs (faster feedback on survivors — same flags as full run, narrower file set):
#   MUTATION_FILE=src/cli.rs ./scripts/mutation_testing/run_with_tmux.sh
#   MUTATION_FILES=$'src/cli.rs\nsrc/reporting.rs' ./scripts/mutation_testing/run_with_tmux.sh
#   MUTATION_MINIMAL_EXCLUDES=1 MUTATION_FILE=src/registry.rs ./scripts/...   # include registry/skills/lsp in run
# Separate output tree (optional):
#   CARGO_MUTANTS_OUTPUT=mutants.out.scoped MUTATION_FILE=src/cli.rs ./scripts/...
#
# Full-tree run (do not leave MUTATION_FILE set in your shell from a prior session):
#   unset MUTATION_FILE MUTATION_FILES
#
# If iteration says "0 mutants" but you changed tests, either remove mutants.out or:
#   MUTATION_NO_ITERATE=1 ./scripts/mutation_testing/run_with_tmux.sh
#
# Optional env (before running):
#   MUTATION_TEST_SUITE=lib   ./scripts/mutation_testing/run_with_tmux.sh
#   MUTATION_TEST_SUITE=tests ./scripts/mutation_testing/run_with_tmux.sh   # same as default

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SESSION_NAME="mutation-testing"
LOG_DIR="$PROJECT_ROOT/logs/mutation_testing"
LOG_FILE="$LOG_DIR/mutation_testing.log"
INNER_SH="$SCRIPT_DIR/run_cargo_mutants.sh"

# lib = only unit tests in src/; tests = unit + integration tests in tests/
TEST_SUITE="${MUTATION_TEST_SUITE:-tests}"

cd "$PROJECT_ROOT"
mkdir -p "$LOG_DIR"

export CARGO_HTTP_CAINFO="${CARGO_HTTP_CAINFO:-/usr/local/etc/ca-certificates/cert.pem}"
export CARGO_NET_GIT_FETCH_WITH_CLI="${CARGO_NET_GIT_FETCH_WITH_CLI:-true}"
export SSL_CERT_FILE="${SSL_CERT_FILE:-/usr/local/etc/ca-certificates/cert.pem}"

if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
  echo "Session '$SESSION_NAME' already exists. Attach with: tmux attach -t $SESSION_NAME"
  exit 0
fi

if [[ -n "${MUTATION_FILE:-}" || -n "${MUTATION_FILES:-}" ]]; then
  echo "Scoped run: MUTATION_FILE=${MUTATION_FILE:-} (MUTATION_FILES set: $([[ -n "${MUTATION_FILES:-}" ]] && echo yes || echo no))"
  echo "Tip: unset MUTATION_FILE MUTATION_FILES to mutate the whole crate (not just this file)."
fi
echo "Using: cargo test $( [[ "$TEST_SUITE" == "lib" ]] && echo --lib || echo --tests ) (set MUTATION_TEST_SUITE=lib for --lib only)"

# Long-run defaults inside the tmux session
export MUTATION_TIMEOUT="${MUTATION_TIMEOUT:-1000}"
export MUTATION_BUILD_TIMEOUT="${MUTATION_BUILD_TIMEOUT:-1000}"

TMUX_INNER=""
TMUX_INNER+="export CARGO_HTTP_CAINFO=$(printf '%q' "$CARGO_HTTP_CAINFO"); "
TMUX_INNER+="export CARGO_NET_GIT_FETCH_WITH_CLI=$(printf '%q' "$CARGO_NET_GIT_FETCH_WITH_CLI"); "
TMUX_INNER+="export SSL_CERT_FILE=$(printf '%q' "$SSL_CERT_FILE"); "
TMUX_INNER+="export MUTATION_TEST_SUITE=$(printf '%q' "$TEST_SUITE"); "
TMUX_INNER+="export MUTATION_FILE=$(printf '%q' "${MUTATION_FILE:-}"); "
TMUX_INNER+="export MUTATION_FILES=$(printf '%q' "${MUTATION_FILES:-}"); "
TMUX_INNER+="export MUTATION_MINIMAL_EXCLUDES=$(printf '%q' "${MUTATION_MINIMAL_EXCLUDES:-}"); "
TMUX_INNER+="export MUTATION_TIMEOUT=$(printf '%q' "$MUTATION_TIMEOUT"); "
TMUX_INNER+="export MUTATION_BUILD_TIMEOUT=$(printf '%q' "$MUTATION_BUILD_TIMEOUT"); "
if [[ -n "${CARGO_MUTANTS_OUTPUT:-}" ]]; then
  TMUX_INNER+="export CARGO_MUTANTS_OUTPUT=$(printf '%q' "$CARGO_MUTANTS_OUTPUT"); "
fi
TMUX_INNER+="export MUTATION_NO_ITERATE=$(printf '%q' "${MUTATION_NO_ITERATE:-}"); "
TMUX_INNER+="bash $(printf '%q' "$INNER_SH") 2>&1 | tee -a $(printf '%q' "$LOG_FILE"); "
TMUX_INNER+="echo; echo Done. Press Enter to close.; read"

tmux new-session -d -s "$SESSION_NAME" -c "$PROJECT_ROOT" "$TMUX_INNER"
