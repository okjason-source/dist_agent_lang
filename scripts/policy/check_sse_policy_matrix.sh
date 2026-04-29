#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "Running SSE policy stability matrix..."

cargo test --test ide_sse_phase0_tests cors_policy_legacy_allows_any_origin -- --nocapture
cargo test --test ide_sse_phase0_tests cors_policy_strict_respects_explicit_origin -- --nocapture
cargo test --test ide_sse_phase0_tests stream_auth_does_not_block_health_or_metrics -- --nocapture
cargo test --test ide_sse_phase0_tests events_stream_requires_auth_token_when_configured -- --nocapture
cargo test --test ide_sse_phase0_tests events_stream_enforces_per_client_concurrency_limit -- --nocapture
cargo test --test ide_sse_phase0_tests events_stream_enforces_establish_rate_limit -- --nocapture

echo "SSE policy stability matrix: PASS"
