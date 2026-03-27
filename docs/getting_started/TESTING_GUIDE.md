# Testing Guide for Contributors

**Testing is one of the most valuable contributions you can make!** This guide helps you test dist_agent_lang, even if you've never written code before.

## 🎯 Why Testing Matters

Testing helps us:
- **Find bugs** before they affect users
- **Improve quality** by catching edge cases
- **Build confidence** in the language
- **Reach production readiness** faster

**You don't need to be a programmer to help test!** This guide is designed for everyone.

## 🚀 Getting Started

### Step 1: Install dist_agent_lang

See the [Installation Guide](README.md#-installation) in the README.

Quick version:
```bash
# Clone the repository
git clone https://github.com/dist_agent_lang/dist_agent_lang.git
cd dist_agent_lang

# Build the project
cargo build --release
```

### Step 2: Verify Installation

```bash
# Check version
cargo run -- --version

# Should output something like:
# dist_agent_lang 1.0.1
```

### Step 3: Run Your First Test

```bash
# Run a simple example
cargo run -- run examples/hello_world_demo.dal
```

If this works, you're ready to start testing! 🎉

## 📋 Types of Testing You Can Do

### 1. Example Testing (Easiest)

**What to do:**
- Run all the examples in the `examples/` directory
- See if they work as expected
- Report any that fail or behave unexpectedly

**How to do it:**
```bash
# Run a specific example
cargo run -- run examples/hello_world_demo.dal

# Try different examples
cargo run -- run examples/agent_system_demo.dal
cargo run -- run examples/smart_contract.dal
cargo run -- run examples/simple_web_api_example.dal
```

**What to report:**
- Which example failed
- What error message you got (copy it exactly)
- What you expected to happen
- Your operating system (Windows, macOS, Linux)
- Your Rust version (`rustc --version`)

**Example Report:**
```
Example: hello_world_demo.dal
Error: "Syntax error at line 5, column 12"
Expected: Should print "Hello, World!"
OS: macOS 14.0
Rust: 1.75.0
```

### 2. Manual Testing (Try Things Out)

**What to do:**
- Write your own simple programs
- Try different language features
- See what works and what doesn't

**How to do it:**

1. Create a test file:
```bash
# Create a new file
nano test_my_program.dal
```

2. Write a simple program:
```rust
@trust("hybrid")
service MyTest {
    fn main() {
        print("Testing dist_agent_lang!");
        
        let x = 10;
        let y = 20;
        let sum = x + y;
        
        print("Sum: " + sum);
    }
}
```

3. Run it:
```bash
cargo run -- run test_my_program.dal
```

4. Try variations:
- Change numbers
- Try different operations
- Test error cases (divide by zero, etc.)

**What to report:**
- Programs that don't work as expected
- Confusing error messages
- Features that are hard to use
- Things that work well (positive feedback!)

### 3. Edge Case Testing (Find Limits)

**What to do:**
- Try extreme values
- Test boundary conditions
- Find where things break

**Examples to try:**

```rust
// Very large numbers
let huge = 999999999999999999;

// Very small numbers
let tiny = 0.0000001;

// Empty strings
let empty = "";

// Special characters
let special = "!@#$%^&*()";

// Very long strings
let long = "a".repeat(10000);
```

**What to report:**
- Values that cause crashes
- Values that produce wrong results
- Performance issues (very slow operations)

### 4. Documentation Testing (Follow Instructions)

**What to do:**
- Read the documentation
- Follow the instructions exactly
- See if they work

**How to do it:**
1. Pick a tutorial or guide from `docs/`
2. Follow it step-by-step
3. Note any:
   - Instructions that don't work
   - Steps that are unclear
   - Missing information
   - Typos or errors

**What to report:**
- Which document you tested
- Which step had problems
- What was confusing
- Suggestions for improvement

### 5. Cross-Platform Testing (Different Operating Systems)

**What to do:**
- Test on different operating systems
- See if behavior differs
- Report platform-specific issues

**If you have access to:**
- Windows, macOS, and/or Linux
- Test the same examples on each
- Note any differences

**What to report:**
- Which OS you tested on
- Examples that work on one OS but not another
- Different error messages on different OSes

## 🐛 How to Report Bugs

### When to Report

Report a bug if:
- ✅ A program crashes unexpectedly
- ✅ A program produces wrong results
- ✅ An error message is confusing or unhelpful
- ✅ Documentation doesn't match actual behavior
- ✅ Something works differently than expected

### How to Report

1. **Check if it's already reported**
   - Search [GitHub Issues](https://github.com/dist_agent_lang/dist_agent_lang/issues)
   - See if someone else found the same bug

2. **Create a new issue** with:
   - **Title**: Brief description (e.g., "hello_world_demo.dal crashes on macOS")
   - **Description**: What happened
   - **Steps to reproduce**: What you did
   - **Expected behavior**: What should happen
   - **Actual behavior**: What actually happened
   - **Environment**: OS, Rust version, dist_agent_lang version
   - **Code**: The code that caused the problem (if applicable)

### Bug Report Template

```markdown
**Description:**
[What happened]

**Steps to Reproduce:**
1. Run: `cargo run -- run examples/hello_world_demo.dal`
2. See error: [error message]

**Expected Behavior:**
[What should happen]

**Actual Behavior:**
[What actually happened]

**Environment:**
- OS: [Your operating system]
- Rust Version: [rustc --version]
- dist_agent_lang Version: [cargo run -- --version]

**Code:**
```
[Paste the code that caused the issue]
```

**Additional Notes:**
[Any other relevant information]
```

## ✅ Testing Checklist

Use this checklist to guide your testing:

### Basic Functionality
- [ ] Can install dist_agent_lang
- [ ] Can run `--version` command
- [ ] Can run `--help` command
- [ ] Can run a simple example program

### Example Programs
- [ ] `hello_world_demo.dal` works
- [ ] `agent_system_demo.dal` works
- [ ] `smart_contract.dal` works
- [ ] At least 5 other examples work

### Language Features
- [ ] Variables work (int, string, bool)
- [ ] Arithmetic operations work (+, -, *, /)
- [ ] Functions can be called
- [ ] Print statements work
- [ ] Error messages are helpful

### Edge Cases
- [ ] Very large numbers handled correctly
- [ ] Division by zero gives error (not crash)
- [ ] Empty strings work
- [ ] Long programs work

### Documentation
- [ ] README is clear
- [ ] Installation instructions work
- [ ] Examples in docs match actual behavior
- [ ] Tutorials are easy to follow

## 🎓 Learning While Testing

### For Beginners

**Start with:**
1. Run examples and see what happens
2. Try changing small things (numbers, text)
3. Read error messages carefully
4. Ask questions if confused

**Learn:**
- How programming languages work
- What different features do
- How to read error messages
- Basic programming concepts

### For Experienced Programmers

**Try:**
1. Write complex programs
2. Test advanced features
3. Find edge cases
4. Test performance
5. Compare with other languages

**Learn:**
- Language design decisions
- Compiler/interpreter internals
- Blockchain integration
- AI agent systems

## 💡 Tips for Effective Testing

### Be Systematic
- Test one thing at a time
- Keep notes of what you tested
- Document what works and what doesn't

### Be Detailed
- Copy error messages exactly
- Include all relevant information
- Provide code examples

### Be Patient
- Some bugs are hard to reproduce
- Some issues take time to understand
- Your testing is valuable even if you don't find bugs!

### Be Positive
- Report what works well too!
- Suggest improvements, not just problems
- Help others learn from your testing

## 🎯 Testing Goals

### Short Term (This Week)
- [ ] Run 10 different examples
- [ ] Report at least 1 bug (or confirm everything works!)
- [ ] Test on your operating system

### Medium Term (This Month)
- [ ] Test all examples in `examples/` directory
- [ ] Write 3 test programs of your own
- [ ] Report 3-5 bugs or improvements

### Long Term (Ongoing)
- [ ] Regularly test new features
- [ ] Help test bug fixes
- [ ] Share testing tips with others

## 🆘 Getting Help

**Stuck?** Don't worry! Here's where to get help:

- **GitHub Discussions**: Ask questions
- **GitHub Issues**: Report bugs (even if you're not sure it's a bug)
- **Email**: jason.dinh.developer@gmail.com
- **Documentation**: Check `docs/` directory

## 🙏 Thank You!

**Your testing contributions are incredibly valuable!** Even if you don't find bugs, confirming that things work is helpful. Every test makes the language better.

**Remember:**
- No contribution is too small
- Questions are welcome
- Your feedback matters
- You're helping build something great!

---

**Ready to start testing?** Pick an example and run it! 🚀

```bash
cargo run -- run examples/hello_world_demo.dal
```

Good luck, and thank you for helping improve dist_agent_lang!
