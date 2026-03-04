# dist_agent_lang: Current Status and Readiness

## Status: Beta (v1.0.8)

dist_agent_lang (DAL) is a domain-specific language for building distributed AI agent systems.
The runtime is a tree-walking interpreter written in Rust. You run DAL programs with `dal run <file>.dal`.

---

## What Works Today

### Core Language

- **Lexer and parser** -- full coverage of DAL syntax including `@` attributes, service declarations, spawn expressions, and message passing
- **Tree-walking interpreter** -- executes DAL files directly via `dal run`
- **CLI** -- `dal run`, `dal test`, `dal repl`, `dal serve`, `dal convert`
- **Testing framework** -- built-in test runner with assertions

### Standard Library (30+ modules)

Includes: `ai`, `agent`, `chain`, `oracle`, `web`, `bridge`, `math`, `string`, `array`, `map`, `io`, `json`, `crypto`, `http`, `time`, `env`, `fs`, `log`, `regex`, `uuid`, `hash`, `base64`, `url`, `net`, `async`, `stream`, `buffer`, `event`, `error`, `config`, and others.

### Agent Framework

- **Agent spawning** -- `spawn name:ai { ... }` for creating autonomous agents
- **Message passing** -- `msg agent method(args)` for inter-agent communication
- **Persistent memory** -- agents store and recall state across sessions
- **Skills registry** -- agents register, discover, and invoke shared skills

### Infrastructure

- **HTTP server** -- `dal serve` runs an agent-serving HTTP endpoint
- **Molds** -- reusable service templates for common patterns
- **Solidity converter** -- `dal convert` transpiles DAL services to Solidity contracts

---

## Experimental / Transpilation Backends

These compile targets exist as experimental code-generation backends, not the primary execution path.

| Target | Status | Notes |
|--------|--------|-------|
| `blockchain` | Experimental | Generates Solidity via `dal convert` |
| `wasm` | Experimental | WebAssembly code generation, limited coverage |
| `native` | Experimental | Stub for native compilation |
| `mobile` | Experimental | Stub for mobile targets |
| `edge` | Experimental | Stub for edge deployment |

The supported way to run DAL programs is through the interpreter: `dal run`.

---

## How to Use

### Install and Build

```bash
git clone <repository>
cd dist_agent_lang
cargo build --release
```

### Write a DAL Program

```
service Greeter {
    field greeting: string = "Hello";

    fn greet(name: string) -> string {
        return greeting + ", " + name + "!";
    }
}
```

### Run, Test, Serve

```bash
dal run hello.dal
dal test tests/
dal serve --port 8080
```

---

## Known Limitations

- No JIT or AOT compilation -- all execution is interpreted
- Single-threaded runtime
- Transpilation backends are experimental, not production-ready
- Error messages are improving but not yet on par with mature toolchains
- Small community; limited third-party libraries

---

## Development Workflow

1. Write `.dal` files using `@` attributes and service declarations
2. Run and iterate with `dal run` and `dal test`
3. Use `dal serve` to expose agents over HTTP
4. Use `dal convert` to generate Solidity when targeting blockchain deployment

---

## Summary

DAL is a working beta-stage language with a functional interpreter, a broad standard library,
and a focused toolset for AI agent development. It is suitable for prototyping, experimentation,
and early-stage agent applications. Production use should account for the limitations above.
