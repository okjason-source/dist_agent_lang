# Good First Issues

Welcome! 👋 This document lists tasks that are perfect for new contributors. These issues are well-defined, have clear acceptance criteria, and don't require deep knowledge of the codebase.

## 🎯 How to Get Started

1. **Pick an issue** that interests you
2. **Comment on the GitHub issue** to claim it (or create one if it doesn't exist)
3. **Fork and clone** the repository
4. **Create a branch** for your work
5. **Make your changes** and test them
6. **Submit a Pull Request**

## 📋 Issue Categories

### 🟢 Beginner-Friendly (No Prior Experience Needed)

#### 1. Documentation Improvements

**Tasks:**
- Fix typos in README, docs, or code comments
- Improve clarity of existing documentation
- Add missing documentation for functions/modules
- Translate documentation to other languages

**Example:**
- Issue: "README.md has a typo in line 45: 'langauge' should be 'language'"
- Fix: Simple text edit
- Impact: Makes project more professional

**Files to check:**
- `README.md`
- `docs/*.md`
- `src/**/*.rs` (code comments)

---

#### 2. Example Programs

**Tasks:**
- Create new example programs demonstrating language features
- Improve existing examples with better comments
- Add examples for undocumented features
- Create tutorials with step-by-step examples

**Example:**
- Create `examples/my_first_agent.dal` showing basic AI agent creation
- Add comments explaining each step
- Include expected output

**Where:**
- `examples/` directory
- `docs/tutorials/` directory

---

#### 3. Testing - Manual Testing

**Tasks:**
- Run existing examples and report any issues
- Try writing your own programs and report bugs
- Test edge cases (very large numbers, empty strings, etc.)
- Test on different operating systems

**Example:**
- Try running all examples: `cargo run -- run examples/*.dal`
- Document any that fail or behave unexpectedly
- Create GitHub issues for bugs found

**No coding required!** Just use the language and report what you find.

---

#### 4. Error Message Improvements

**Tasks:**
- Make error messages more helpful
- Add suggestions for fixing common errors
- Improve error message formatting

**Example:**
- Current: "Error: Invalid syntax"
- Improved: "Error: Invalid syntax at line 5, column 12. Did you mean to use `@trust("hybrid")` instead of `@trust(hybrid)`?"

**Files:**
- `src/lexer/error.rs`
- `src/parser/error.rs`
- `src/runtime/error.rs`

---

### 🟡 Intermediate (Some Rust/Programming Experience)

#### 5. Unit Tests

**Tasks:**
- Add unit tests for lexer functions
- Add unit tests for parser functions
- Add unit tests for runtime functions
- Improve test coverage

**Example:**
```rust
// Add to src/lexer/lexer.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lexer_tokenizes_keyword() {
        let tokens = tokenize("@trust");
        assert_eq!(tokens[0].kind, TokenKind::At);
        assert_eq!(tokens[1].kind, TokenKind::Trust);
    }
}
```

**Where:**
- Add `#[cfg(test)]` modules to source files
- Or add to `tests/` directory

---

#### 6. Property-Based Tests

**Tasks:**
- Add property tests using `proptest` or `quickcheck`
- Test invariants that should always hold
- Test edge cases automatically

**Example:**
```rust
// Add to tests/property_tests.rs
proptest! {
    #[test]
    fn lexer_never_panics_on_arbitrary_string(s in ".*") {
        // This should never panic, no matter what string we give it
        let _ = tokenize(&s);
    }
}
```

**Where:**
- `tests/property_tests.rs`

---

#### 7. Integration Tests

**Tasks:**
- Add integration tests for complete workflows
- Test multi-chain operations
- Test AI agent coordination
- Test error handling in real scenarios

**Example:**
```rust
// Add to tests/integration/workflow_tests.rs
#[test]
fn test_complete_defi_workflow() {
    // Test deploying a token, transferring, and querying balance
    // This tests the entire flow end-to-end
}
```

**Where:**
- `tests/integration/` directory

---

### 🟠 Advanced (Rust Experience Required)

#### 8. Cryptographic Implementations

**Tasks:**
- Implement real ECDSA verification (replace mock)
- Implement real ECDSA signing
- Implement real EdDSA verification
- Add test vectors

**Current State:**
- Mock implementations exist in `src/stdlib/crypto_signatures.rs`
- Crypto crates (`k256`, `ed25519-dalek`) are already in dependencies
- Need to replace mocks with real implementations

**Example:**
```rust
// Replace this mock:
pub fn verify_ecdsa_signature(_message: &[u8], _signature: &[u8], _public_key: &[u8]) -> Result<bool, String> {
    // TODO: Implement actual verification
    Ok(true) // Mock
}

// With real implementation:
use k256::ecdsa::{Signature, VerifyingKey, signature::Verifier};

pub fn verify_ecdsa_signature(message: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool, String> {
    // Real implementation here
}
```

**Files:**
- `src/stdlib/crypto_signatures.rs`
- See `CODEBASE_TODOS.md` for details

**Effort:** 4-6 hours total
**Impact:** High - Security critical

---

#### 9. JWT Authentication

**Tasks:**
- Implement proper JWT validation
- Replace placeholder implementation
- Add tests

**Current State:**
- Placeholder in `src/http_server_security.rs`
- `jsonwebtoken` crate already in dependencies
- Need to implement real validation

**Files:**
- `src/http_server_security.rs`
- See `CODEBASE_TODOS.md` for details

**Effort:** 2-3 hours
**Impact:** High - Security critical

---

#### 10. Parser Improvements

**Tasks:**
- Support `if` statements without parentheses
- Improve error messages
- Better error recovery

**Example:**
```dal
// Currently may require:
if (x > 5) { }

// Should also support:
if x > 5 { }
```

**Files:**
- `src/parser/parser.rs`
- `src/parser/error.rs`

**Effort:** 2-3 hours

---

## 🎓 Learning Resources

### For Beginners

- **Rust Book**: https://doc.rust-lang.org/book/
- **Rust by Example**: https://doc.rust-lang.org/rust-by-example/
- **dist_agent_lang Docs**: `docs/` directory
- **Examples**: `examples/` directory

### For Testing

- **Property Testing Guide**: `docs/testing/mutation_testing/`
- **Fuzzing Guide**: `docs/testing/fuzzing/`
- **Proptest Docs**: https://docs.rs/proptest/

### For Cryptography

- **k256 Docs**: https://docs.rs/k256/
- **ed25519-dalek Docs**: https://docs.rs/ed25519-dalek/
- **jsonwebtoken Docs**: https://docs.rs/jsonwebtoken/

## 📝 Creating Issues

If you find a task that's not listed here:

1. **Check existing issues** to avoid duplicates
2. **Create a new issue** with:
   - Clear title
   - Description of the task
   - Expected outcome
   - Label as `good-first-issue` if appropriate
   - Add any relevant code examples

## 🤝 Getting Help

- **GitHub Discussions**: Ask questions
- **GitHub Issues**: Comment on issues
- **Email**: jason.dinh.developer@gmail.com

## ✅ Completion Checklist

When you finish a task:

- [ ] Code compiles (`cargo build`)
- [ ] Tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated (if needed)
- [ ] PR created with clear description

---

**Ready to contribute?** Pick an issue, comment to claim it, and start coding! 🚀

Every contribution helps, no matter how small. Thank you for helping improve dist_agent_lang!
