# Fuzzing Guide for dist_agent_lang

This guide explains how to run fuzzing campaigns to discover edge cases and potential bugs in the `dist_agent_lang` compiler and runtime.

## Prerequisites

1. **Nightly Rust Toolchain**: Fuzzing requires the nightly Rust toolchain
   ```bash
   rustup install nightly
   ```

2. **cargo-fuzz**: Already installed (v0.13.1)
   ```bash
   cargo install cargo-fuzz
   ```

3. **SSL Certificates**: Configured (see `HOMEBREW_NETWORK_FIX.sh` if needed)

## Fuzz Targets

We have 4 fuzz targets covering critical components:

1. **`fuzz_lexer`**: Fuzzes the lexer with arbitrary string input
2. **`fuzz_parser`**: Fuzzes the parser with tokenized input
3. **`fuzz_runtime`**: Fuzzes the runtime execution engine
4. **`fuzz_stdlib`**: Fuzzes standard library functions

## Running Fuzzing Campaigns

### List Available Targets

```bash
cd /Users/jason/lang_mark/dist_agent_lang
cargo +nightly fuzz list
```

### Build a Fuzz Target

```bash
# Build a specific target
cargo +nightly fuzz build fuzz_lexer

# Build all targets
cargo +nightly fuzz build
```

### Seed Corpus

The `fuzz/corpus_seed/` directory contains committed example files that seed the corpus with real DAL code:

- `dynamic_nft_examples.dal` - Dynamic NFT patterns with oracle, chain, and AI
- `defi_nft_rwa_contract.dal` - DeFi contract with KYC/AML, asset tokenization
- `simple_web_api_example.dal` - Web API with routes, WebSocket, templates

Use the seed corpus when running fuzzing (combines with runtime corpus if present):

```bash
# Run with seed corpus (recommended for initial runs)
cargo +nightly fuzz run fuzz_lexer fuzz/corpus_seed/fuzz_lexer fuzz/corpus/fuzz_lexer
```

### Run a Fuzzing Campaign

```bash
# Run fuzzing for a specific target (runs indefinitely until stopped)
cargo +nightly fuzz run fuzz_lexer

# Run with seed corpus + existing corpus
cargo +nightly fuzz run fuzz_lexer fuzz/corpus_seed/fuzz_lexer fuzz/corpus/fuzz_lexer

# Run with a time limit (e.g., 1 hour = 3600 seconds)
timeout 3600 cargo +nightly fuzz run fuzz_lexer fuzz/corpus_seed/fuzz_lexer fuzz/corpus/fuzz_lexer

# Run with corpus from previous runs
cargo +nightly fuzz run fuzz_lexer -- -max_total_time=3600
```

### Run All Targets

```bash
# Run all fuzz targets sequentially
for target in fuzz_lexer fuzz_parser fuzz_runtime fuzz_stdlib; do
    echo "Running $target..."
    cargo +nightly fuzz run $target -- -max_total_time=900  # 15 minutes each
done
```

## Long-Running Campaigns (100+ Hours)

For production readiness, run extended fuzzing campaigns:

```bash
# Run each target for 25 hours (100 hours total)
for target in fuzz_lexer fuzz_parser fuzz_runtime fuzz_stdlib; do
    echo "Running $target for 25 hours..."
    cargo +nightly fuzz run $target -- -max_total_time=90000  # 25 hours
done
```

Or run in the background:

```bash
# Run all targets in parallel (one per CPU core)
cargo +nightly fuzz run fuzz_lexer -- -max_total_time=90000 &
cargo +nightly fuzz run fuzz_parser -- -max_total_time=90000 &
cargo +nightly fuzz run fuzz_runtime -- -max_total_time=90000 &
cargo +nightly fuzz run fuzz_stdlib -- -max_total_time=90000 &
wait
```

## Analyzing Results

### View Corpus

The fuzzer builds a corpus of interesting inputs:

```bash
# View corpus directory
ls -la fuzz/artifacts/fuzz_lexer/

# View crash inputs (if any)
ls -la fuzz/artifacts/fuzz_lexer/crash-*
```

### Reproduce Crashes

If the fuzzer finds a crash:

```bash
# Reproduce a specific crash
cargo +nightly fuzz run fuzz_lexer fuzz/artifacts/fuzz_lexer/crash-<hash>
```

### Minimize Crash Inputs

```bash
# Minimize a crash input to the smallest possible
cargo +nightly fuzz tmin fuzz_lexer fuzz/artifacts/fuzz_lexer/crash-<hash>
```

## Continuous Fuzzing

For continuous fuzzing in CI/CD:

```bash
# Run fuzzing for a fixed duration and exit
cargo +nightly fuzz run fuzz_lexer -- -max_total_time=3600 -timeout=10
```

## Expected Results

- **No crashes**: The fuzzer should run without finding crashes
- **Coverage growth**: The corpus should grow over time as new code paths are discovered
- **Edge cases**: The fuzzer will discover edge cases that manual testing might miss

## Troubleshooting

### "the option `Z` is only accepted on the nightly compiler"

**Solution**: Use `cargo +nightly fuzz` instead of `cargo fuzz`

### SSL Certificate Errors

**Solution**: Run `./HOMEBREW_NETWORK_FIX.sh` to configure SSL certificates

### Out of Memory

**Solution**: Reduce the number of parallel fuzzing processes or increase system memory

## Next Steps

1. Run initial fuzzing campaigns (1-2 hours per target)
2. Fix any discovered crashes
3. Run extended campaigns (100+ hours total)
4. Document discovered edge cases
5. Add regression tests for fixed issues

---

**Status**: Infrastructure complete, ready for fuzzing campaigns.

