# Contributing to dist_agent_lang

First off, thank you for considering contributing to dist_agent_lang! 🎉

This document provides guidelines and instructions for contributing to the project. Whether you're fixing bugs, adding features, improving documentation, or testing the language, your contributions are valuable.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Workflow](#development-workflow)
- [Testing Guidelines](#testing-guidelines)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)
- [Areas Needing Help](#areas-needing-help)

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow. Please be respectful, inclusive, and constructive in all interactions.

## Getting Started

### Prerequisites

- **Rust 1.70+** ([Install Rust](https://rustup.rs/))
- **Git** for version control
- **Basic familiarity** with programming languages, compilers, or blockchain development (helpful but not required)

### Setting Up Your Development Environment

1. **Fork and Clone the Repository**
   ```bash
   git clone https://github.com/your-username/dist_agent_lang.git
   cd dist_agent_lang
   ```

2. **Build the Project**
   ```bash
   cargo build
   ```

3. **Run Tests**
   ```bash
   # Run all tests
   cargo test
   
   # Run specific test suite
   cargo test --test property_tests
   cargo test --test lexer_tokens_tests
   ```

4. **Verify Everything Works**
   ```bash
   # Run a simple example
   cargo run -- run examples/hello_world_demo.dal
   ```

## How Can I Contribute?

### 🐛 Reporting Bugs

Found a bug? Great! Here's how to report it:

1. **Check Existing Issues**: Search [GitHub Issues](https://github.com/dist_agent_lang/dist_agent_lang/issues) to see if it's already reported.

2. **Create a New Issue** with:
   - **Clear title**: Brief description of the issue
   - **Steps to reproduce**: What you did to encounter the bug
   - **Expected behavior**: What should have happened
   - **Actual behavior**: What actually happened
   - **Environment**: OS, Rust version, dist_agent_lang version
   - **Code example**: Minimal code that reproduces the issue (if applicable)

3. **Label appropriately**: Use labels like `bug`, `help-wanted`, `good-first-issue`

### 💡 Suggesting Features

Have an idea? We'd love to hear it!

1. **Check Existing Discussions**: See if it's been discussed in [GitHub Discussions](https://github.com/dist_agent_lang/dist_agent_lang/discussions)

2. **Create a Feature Request** with:
   - **Use case**: What problem does this solve?
   - **Proposed solution**: How should it work?
   - **Alternatives considered**: Other approaches you've thought about
   - **Examples**: Code examples showing how it would be used

### 🧪 Testing

**Testing is one of the most valuable contributions!** The language is in beta, and thorough testing helps us reach production readiness.

#### Types of Testing We Need:

1. **Manual Testing**
   - Try the examples in `examples/` directory
   - Write your own programs
   - Test edge cases
   - Report any issues you find

2. **Property-Based Testing**
   - Add tests to `tests/property_tests.rs`
   - Test invariants that should always hold
   - Use `proptest` or `quickcheck` frameworks

3. **Integration Testing**
   - Test real-world scenarios
   - Test multi-chain operations
   - Test AI agent workflows
   - Add tests to `tests/integration/`

4. **Fuzzing**
   - Run fuzzing tests to find edge cases
   - See `docs/testing/fuzzing/` for guides

5. **Mutation Testing**
   - Help improve test quality
   - See `docs/testing/mutation_testing/` for guides

#### Running Tests

```bash
# All tests
cargo test

# Specific test suites
cargo test --test property_tests
cargo test --test lexer_tokens_tests
cargo test --test load_stress_tests

# With output
cargo test -- --nocapture

# Single test
cargo test test_name
```

### 📝 Improving Documentation

Good documentation is crucial! Help us improve:

- **Code comments**: Add or improve inline documentation
- **README files**: Make setup instructions clearer
- **API documentation**: Document functions and modules
- **Tutorials**: Create or improve tutorials
- **Examples**: Add more example programs

To build documentation:
```bash
cargo doc --open
```

### 💻 Writing Code

Ready to write code? Here are some areas that need help:

#### 🔴 High Priority (Security & Core)

1. **Cryptographic Signatures** (`src/stdlib/crypto_signatures.rs`)
   - Implement real ECDSA verification (currently mocked)
   - Implement real ECDSA signing
   - Implement real EdDSA verification
   - **Effort**: 4-6 hours total
   - **Impact**: High - Security critical

2. **JWT Authentication** (`src/http_server_security.rs`)
   - Implement proper JWT validation
   - Replace placeholder implementation
   - **Effort**: 2-3 hours
   - **Impact**: High - Security critical

#### 🟡 Medium Priority (Features)

3. **HTTP Server Route Registration** (`src/http_server.rs`)
   - Dynamic route registration from DAL code
   - Bridge DAL parser/runtime to Axum router
   - **Effort**: 4-6 hours

4. **Parser Improvements** (`src/parser/`)
   - Support `if` statements without parentheses
   - Better error messages
   - **Effort**: 2-3 hours

5. **Security Test Coverage** (`tests/integration/security_tests.rs`)
   - Reentrancy protection tests
   - Safe math operation tests
   - Cross-chain security tests
   - **Effort**: 3-4 hours

#### 🟢 Good First Issues

- Add more example programs
- Improve error messages
- Add unit tests for lexer/parser
- Write documentation
- Fix typos and improve clarity

See [CODEBASE_TODOS.md](../CODEBASE_TODOS.md) for detailed TODO list.

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

**Branch naming conventions:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `test/` - Test additions
- `refactor/` - Code refactoring

### 2. Make Your Changes

- Write clean, readable code
- Follow the code style guidelines
- Add tests for new features
- Update documentation as needed

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific tests related to your changes
cargo test --test your_test_file

# Check for linting issues
cargo clippy

# Format code
cargo fmt
```

### 4. Commit Your Changes

Write clear, descriptive commit messages:

```bash
git commit -m "Add ECDSA signature verification

- Replace mock implementation with real k256-based verification
- Add test vectors for ECDSA
- Update documentation

Fixes #123"
```

**Commit message format:**
- First line: Brief summary (50 chars or less)
- Blank line
- Detailed explanation (if needed)
- Reference issues: "Fixes #123" or "Closes #456"

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub with:
- **Clear title**: What does this PR do?
- **Description**: Why is this change needed? How does it work?
- **Related issues**: Link to any related issues
- **Testing**: How was this tested?
- **Checklist**: Mark items as complete

## Testing Guidelines

### Writing Tests

1. **Unit Tests**: Test individual functions/modules
   - Place in `#[cfg(test)]` modules within source files
   - Or in `tests/` directory

2. **Integration Tests**: Test complete workflows
   - Place in `tests/integration/`
   - Test real-world scenarios

3. **Property Tests**: Test invariants
   - Use `proptest` or `quickcheck`
   - Place in `tests/property_tests.rs`

4. **Performance Tests**: Benchmark critical paths
   - Use `criterion` for benchmarks
   - Place in `benches/` directory

### Test Naming

- Use descriptive names: `test_ecdsa_verification_with_valid_signature`
- Group related tests: `mod ecdsa_tests { ... }`
- Use `#[should_panic]` for tests that should fail

### Example Test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecdsa_verification_with_valid_signature() {
        let message = b"test message";
        let signature = create_signature(message);
        let public_key = get_public_key();
        
        assert!(verify_signature(message, &signature, &public_key).is_ok());
    }
}
```

## Code Style

### Rust Style

We follow standard Rust conventions:

1. **Format with rustfmt**:
   ```bash
   cargo fmt
   ```

2. **Check with clippy**:
   ```bash
   cargo clippy -- -D warnings
   ```

3. **General guidelines**:
   - Use `snake_case` for functions and variables
   - Use `PascalCase` for types
   - Use `SCREAMING_SNAKE_CASE` for constants
   - Prefer `let` bindings over mutability
   - Use meaningful variable names
   - Add comments for complex logic

### dist_agent_lang Style

For `.dal` files (dist_agent_lang source):

- Use 4 spaces for indentation
- Use descriptive service and function names
- Add `@trust` attributes appropriately
- Include comments for complex logic

## Submitting Changes

### Pull Request Checklist

Before submitting a PR, ensure:

- [ ] Code compiles without errors
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated (if needed)
- [ ] Tests are added for new features
- [ ] Commit messages are clear
- [ ] PR description explains the change

### Review Process

1. **Automated Checks**: CI will run tests and linting
2. **Code Review**: Maintainers will review your code
3. **Feedback**: Address any requested changes
4. **Merge**: Once approved, your PR will be merged!

## Areas Needing Help

### 🎯 Most Needed Contributions

1. **Testing** ⭐⭐⭐⭐⭐
   - Test the language with real-world scenarios
   - Write property-based tests
   - Find and report bugs
   - **No coding required** - just use the language!

2. **Cryptographic Implementations** ⭐⭐⭐⭐
   - Replace mock crypto with real implementations
   - See `CODEBASE_TODOS.md` for details
   - **Good for**: Rust developers familiar with crypto

3. **Documentation** ⭐⭐⭐⭐
   - Improve tutorials
   - Add more examples
   - Write API documentation
   - **Good for**: Anyone who can write clearly

4. **Parser Improvements** ⭐⭐⭐
   - Better error messages
   - Syntax improvements
   - **Good for**: Compiler/interpreter enthusiasts

5. **Example Programs** ⭐⭐⭐
   - Create more example programs
   - Show real-world use cases
   - **Good for**: Anyone learning the language

### 🆘 Beginner-Friendly Tasks

- Fix typos in documentation
- Improve error messages
- Add comments to code
- Write example programs
- Test examples and report issues
- Improve README clarity

### 🚀 Advanced Tasks

- Implement cryptographic functions
- Optimize performance
- Add new language features
- Improve compiler optimizations
- Add new standard library modules

## Getting Help

- **GitHub Discussions**: Ask questions, share ideas
- **GitHub Issues**: Report bugs, request features
- **Email**: jason.dinh.developer@gmail.com
- **Documentation**: Check `docs/` directory

## Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md (coming soon)
- Credited in release notes
- Appreciated by the community! 🙏

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

---

**Thank you for contributing to dist_agent_lang!** 🚀

Every contribution, no matter how small, helps make this language better. Whether you're fixing a typo, writing tests, or implementing features, your help is valuable and appreciated.
