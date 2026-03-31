#!/usr/bin/env bash
# Shared cargo-mutants invocation for run_from_terminal.sh and run_with_tmux.sh.
#
# Environment:
#   MUTATION_TEST_SUITE   lib | tests (default: tests)
#   MUTATION_FILE         Optional single glob/path passed as --file (quote paths with spaces)
#   MUTATION_FILES        Optional newline-separated list; each non-empty line becomes --file ...
#   MUTATION_MINIMAL_EXCLUDES  Set to 1 to stop excluding src/registry.rs, src/skills.rs, src/lsp.rs
#                            (use when scoping to those modules so --file is not ignored)
#   MUTATION_EXCLUDE_AST     Set to 1 to add --exclude src/parser/ast.rs (fewer lexer timeouts; optional)
#   CARGO_MUTANTS_OUTPUT  Set to a directory name to use mutants.out there (scoped vs full baseline)
#   MUTATION_NO_ITERATE     Set to 1 to omit --iterate (re-test everything; use after clearing mutants.out
#                            or when iteration reports 0 mutants but you changed tests/source)
#
# Timeouts: full runs use 500s (tmux); terminal script overrides by calling with env if needed.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

export CARGO_HTTP_CAINFO="${CARGO_HTTP_CAINFO:-/usr/local/etc/ca-certificates/cert.pem}"
export CARGO_NET_GIT_FETCH_WITH_CLI="${CARGO_NET_GIT_FETCH_WITH_CLI:-true}"
export SSL_CERT_FILE="${SSL_CERT_FILE:-/usr/local/etc/ca-certificates/cert.pem}"

TEST_SUITE="${MUTATION_TEST_SUITE:-tests}"
CARGO_TEST_ARGS="--tests"
if [[ "$TEST_SUITE" == "lib" ]]; then
  CARGO_TEST_ARGS="--lib"
fi

MUT_TIMEOUT="${MUTATION_TIMEOUT:-500}"
MUT_BUILD_TIMEOUT="${MUTATION_BUILD_TIMEOUT:-500}"

ITER_FLAG=(--iterate)
if [[ "${MUTATION_NO_ITERATE:-}" == "1" ]]; then
  ITER_FLAG=()
fi

MUT_ARGS=(
  cargo mutants "${ITER_FLAG[@]}" --gitignore true --jobs 1
  --exclude src/main.rs
  --exclude src/stdlib/iot.rs
  --exclude src/stdlib/desktop.rs
  --exclude tests/ffi_performance_tests.rs
  --exclude tests/example_tests.rs
  --exclude src/ffi/security.rs
  --timeout "$MUT_TIMEOUT"
  --build-timeout "$MUT_BUILD_TIMEOUT"
)

if [[ "${MUTATION_EXCLUDE_AST:-}" == "1" ]]; then
  MUT_ARGS+=(--exclude src/parser/ast.rs)
fi

if [[ "${MUTATION_MINIMAL_EXCLUDES:-}" != "1" ]]; then
  MUT_ARGS+=(
    --exclude src/registry.rs
    --exclude src/skills.rs
    --exclude src/lsp.rs
  )
fi

if [[ -n "${MUTATION_FILE:-}" ]]; then
  MUT_ARGS+=(--file "$MUTATION_FILE")
fi

if [[ -n "${MUTATION_FILES:-}" ]]; then
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ -z "$line" ]] && continue
    MUT_ARGS+=(--file "$line")
  done <<< "$MUTATION_FILES"
fi

exec "${MUT_ARGS[@]}" -- $CARGO_TEST_ARGS
