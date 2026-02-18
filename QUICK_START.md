# Quick Start: Phase 1 Testing Setup

## One-Command Setup (Recommended)

Since you have Homebrew installed, run this single command to fix everything:

```bash
cd /Users/jason/lang_mark/dist_agent_lang && ./HOMEBREW_NETWORK_FIX.sh
```

This script will:
1. ✅ Update Homebrew
2. ✅ Install/update SSL certificates
3. ✅ Configure Cargo to use certificates
4. ✅ Set environment variables
5. ✅ Install test dependencies (proptest, quickcheck)
6. ✅ Verify everything works

**Time**: ~5 minutes (depending on download speed)

---

## Or: Manual Step-by-Step

If you prefer to run commands manually:

### 1. Update Homebrew & Install Certificates

```bash
brew update
brew install ca-certificates openssl@3
```

### 2. Configure Cargo

```bash
mkdir -p ~/.cargo && cat > ~/.cargo/config.toml << 'EOF'
[net]
git-fetch-with-cli = true

[http]
cainfo = "$(brew --prefix openssl@3)/etc/openssl@3/cert.pem"
EOF
```

### 3. Install Dependencies

```bash
cd /Users/jason/lang_mark/dist_agent_lang
cargo build --tests
```

### 4. Run Tests

```bash
# Run property-based tests (15 tests)
cargo test --test property_tests

# Run load/stress tests (18 tests)
cargo test --test load_stress_tests --nocapture

# Run all tests (128 total: 95 existing + 33 new)
cargo test --workspace
```

---

## What Gets Installed

### Homebrew Packages:
- `ca-certificates` - Root SSL certificates
- `openssl@3` - OpenSSL 3.x with updated certificates

### Rust Dependencies (via Cargo):
- `proptest = "1.4"` - Property-based testing framework
- `quickcheck = "1.0"` - Alternative property testing
- `quickcheck_macros = "1.0"` - Macros for quickcheck

---

## Expected Results

### After Setup:
```
✅ Homebrew packages updated
✅ SSL certificates configured  
✅ Cargo configured
✅ Test dependencies installed
```

### After Running Tests:

**Property Tests** (15 tests):
```bash
$ cargo test --test property_tests
running 15 tests
test lexer_never_panics_on_arbitrary_input ... ok
test safe_math_properties ... ok
test reentrancy_guard_prevents_reentry ... ok
...
test result: ok. 15 passed; 0 failed; 0 ignored
```

**Load/Stress Tests** (18 tests):
```bash
$ cargo test --test load_stress_tests --nocapture
running 18 tests
Lexer processed 10,000 lines in 234ms
Parser processed 100 nested levels in 89ms
...
test result: ok. 18 passed; 0 failed; 0 ignored
```

**All Tests** (128 total):
```bash
$ cargo test --workspace
...
test result: ok. 128 passed; 0 failed; 0 ignored
```

---

## Troubleshooting

### Script fails with "permission denied"
```bash
chmod +x HOMEBREW_NETWORK_FIX.sh
./HOMEBREW_NETWORK_FIX.sh
```

### Homebrew not in PATH
```bash
# Add Homebrew to PATH
eval "$(/usr/local/bin/brew shellenv)"
```

### Still getting SSL errors
```bash
# Use system certificates as fallback
export SSL_CERT_FILE=/etc/ssl/cert.pem
cargo build --tests
```

### Need to start fresh
```bash
# Clean everything and retry
cargo clean
rm -rf ~/.cargo/registry
./HOMEBREW_NETWORK_FIX.sh
```

---

## After Setup Success

Continue with Phase 1:

1. ✅ **Property-based tests** - Run and verify all pass
2. ✅ **Load/stress tests** - Run and check performance metrics
3. ⏳ **Fuzzing setup** - `cargo install cargo-fuzz`
4. ⏳ **Testnet integration** - Set up testnet deployments

See `PHASE1_PROGRESS.md` for detailed tracking.

---

## Files Created for Phase 1

- ✅ `tests/property_tests.rs` - 15 property-based tests
- ✅ `tests/load_stress_tests.rs` - 18 load/stress tests
- ✅ `PHASE1_PROGRESS.md` - Progress tracking
- ✅ `PHASE1_SETUP_INSTRUCTIONS.md` - Detailed instructions
- ✅ `NETWORK_SETUP_GUIDE.md` - Network troubleshooting
- ✅ `HOMEBREW_NETWORK_FIX.sh` - Automated setup script
- ✅ `QUICK_START.md` - This file

---

**Ready to go? Run:**
```bash
cd /Users/jason/lang_mark/dist_agent_lang && ./HOMEBREW_NETWORK_FIX.sh
```

🚀 **Let's get Phase 1 testing running!**

