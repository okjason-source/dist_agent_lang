# Phase 0 Developer Tools - Quick Reference

## ✅ All Commands Implemented & Working

### Code Quality
```bash
dal check my_app.dal          # Type check without running
dal fmt my_app.dal             # Format code
dal fmt my_app.dal --check     # Check if formatted
dal lint my_app.dal            # Lint for issues
dal parse my_app.dal           # Parse and validate syntax
```

### Project Management
```bash
# Create new projects
dal new my-project                    # Default project
dal new my-token --mold contract      # Smart contract
dal new my-api --mold web             # Web application
dal new my-tool --mold cli            # CLI tool
dal new my-lib --mold lib             # Library

# Initialize existing directory
dal init

# Package management
dal add @dal/crypto
dal add @dal/testing  
dal install
```

### Development
```bash
dal repl                      # Interactive REPL
dal watch my_app.dal          # Auto-reload on changes
```

### Execution
```bash
dal run my_app.dal            # Run program
dal web server.dal            # Run web server
dal test                      # Run tests
```

### Solidity Integration
```bash
dal convert MyContract.sol -o MyContract.dal
dal analyze MyContract.sol
```

### Info
```bash
dal help                      # Show all commands
dal version                   # Show version
```

## Project Molds (Templates)

| Mold | Use Case | Generated Files |
|------|----------|-----------------|
| **default** | General purpose | `main.dal`, `README.md` |
| **contract** | Smart contracts | `contract.dal`, `README.md` |
| **web** | Web applications | `web.dal`, `README.md` |
| **cli** | CLI tools | `cli.dal`, `README.md` |
| **lib** | Libraries | `lib.dal`, `README.md` |

## Typical Workflows

### Starting a New Project
```bash
dal new my-project --mold contract
cd my-project
dal check contract.dal
dal fmt contract.dal
dal lint contract.dal
dal run contract.dal
```

### Development Loop
```bash
# Terminal 1: Watch mode
dal watch my_app.dal

# Terminal 2: Make changes to my_app.dal
# Changes auto-reload in Terminal 1
```

### Before Committing
```bash
dal fmt src/                  # Format all files
dal lint src/                 # Check for issues  
dal check src/                # Validate syntax
dal test                      # Run tests
```

## REPL Examples

```bash
$ dal repl

dal[1]> let x = 42
dal[2]> let y = 10
dal[3]> x + y
=> 52

dal[4]> fn greet(name: string) { print("Hello " + name); }
dal[5]> greet("World")
Hello World

dal[6]> exit
Goodbye!
```

## Comparison to Other Languages

| Task | Rust | Go | Node.js | **DAL** |
|------|------|----|---------|---------| 
| New project | `cargo new` | `go mod init` | `npm init` | **`dal new`** |
| Format | `cargo fmt` | `gofmt` | `prettier` | **`dal fmt`** |
| Lint | `cargo clippy` | `golint` | `eslint` | **`dal lint`** |
| Check | `cargo check` | ❌ | `tsc --noEmit` | **`dal check`** |
| REPL | ❌ | ❌ | `node` | **`dal repl`** |
| Watch | `cargo-watch` | ❌ | `nodemon` | **`dal watch`** |

**Result:** DAL has best-in-class developer experience!

## Next: Phase 1 Performance Commands

```bash
# Coming soon
dal bench                     # Run benchmarks
dal profile my_app.dal        # Profile execution
dal optimize my_app.dal       # Apply optimizations
```

---

**Status:** Phase 0 Complete ✅  
**Commands:** 16 total (8 new)  
**Build:** Compiling successfully  
**Ready for:** Phase 1 implementation
