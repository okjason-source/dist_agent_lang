# dist_agent_lang: Language vision and roadmap

> **Status: Vision document.** This describes the long-term direction for dist_agent_lang. For current capabilities, see [AGENT_SETUP_AND_USAGE.md](AGENT_SETUP_AND_USAGE.md) and the [root README](../../README.md).

---

## What dist_agent_lang is today

dist_agent_lang (DAL) is a domain-specific language executed by a **tree-walking interpreter written in Rust**. It is designed around two primary differentiators: **autonomous agent orchestration** and **blockchain-native programming**.

### Current capabilities

- **Interpreter**: `dal run <file.dal>` executes programs via the Rust-hosted tree-walking interpreter.
- **Standard library**: ~30 modules covering math, string, collections, I/O, crypto, HTTP, JSON, and more.
- **Agent framework**: First-class primitives to spawn, coordinate, communicate with, and evolve autonomous agents, including persistent memory and a skills registry.
- **Blockchain integration**: Solidity converter, trust model annotations (`@trust`), and chain interaction primitives.
- **HTTP server**: Built-in `dal serve` for hosting agent endpoints and web services.
- **Molds**: Reusable agent configuration modules (`.mold.dal`).
- **CLI toolchain**: `dal run`, `dal serve`, `dal test`, `dal fmt`, and related commands.

### Current execution model

DAL source is parsed into an AST and evaluated directly by the interpreter. There is no compilation to machine code, bytecode, or intermediate representation in the current release. The Rust host provides memory safety and performance for the interpreter itself, but DAL programs run at interpreted speed.

Compile targets (`@compile_target("solidity")`, etc.) exist as **transpilation / code-generation backends** -- they emit source code for other platforms rather than producing native binaries.

---

## Primary differentiators

### Agents as a language primitive

Most languages treat agents as an application-layer concern. DAL provides agent lifecycle management -- spawn, message-passing, coordination, evolution, and persistent memory -- as built-in language constructs. This makes multi-agent systems expressible in tens of lines rather than hundreds.

```
agent Coordinator {
    fn on_message(msg) {
        let workers = spawn_agents("Worker", 3);
        broadcast(workers, msg);
        let results = collect_responses(workers);
        return aggregate(results);
    }
}
```

### Blockchain-native semantics

DAL includes trust-model annotations, on-chain/off-chain separation, and a Solidity transpiler. Smart contract logic can be written in DAL syntax and converted to deployable Solidity, reducing context-switching for teams that work across application and contract layers.

```
@trust("hybrid")
service TokenTransfer {
    fn transfer(from, to, amount) {
        let tx = chain::send_transaction({
            "from": from, "to": to, "amount": amount
        });
        log::info("transfer", {"tx_hash": tx.hash});
        return tx;
    }
}
```

---

## Vision: multi-domain, cross-platform language

The following sections describe **planned capabilities** that do not yet exist. They represent the long-term direction for DAL.

### Vision: native compilation

A future compiler pipeline would lower DAL programs to machine code (likely via Cranelift or LLVM), enabling:

- Ahead-of-time compilation to native binaries
- Competitive runtime performance with systems languages
- Deployment without requiring the Rust interpreter host

**Status**: Not started. The interpreter remains the sole execution path.

### Vision: WebAssembly target

A WASM compilation target would allow DAL programs to run in browsers and edge runtimes. Combined with the agent framework, this could enable client-side autonomous agents with near-native performance.

**Status**: Not started. The Solidity transpiler demonstrates the code-generation architecture that a WASM backend would follow.

### Vision: mobile and edge targets

Cross-compilation to iOS, Android, and resource-constrained edge devices would extend DAL's agent and blockchain primitives to mobile and IoT contexts.

**Status**: Not started. Depends on the native compilation pipeline.

### Vision: zero-cost abstractions and memory safety guarantees

Future compiler work could introduce ownership analysis or region-based memory management, providing compile-time safety guarantees beyond what the interpreter currently offers. Today, memory safety is inherited from the Rust interpreter host -- DAL programs themselves do not have independent memory safety properties.

**Status**: Research phase. No concrete design exists yet.

### Vision: cross-platform single-codebase development

The end-state goal is a single DAL codebase that compiles to server (native), browser (WASM), mobile, and edge targets, with the agent and blockchain primitives available on all platforms.

```
@compile_target("wasm")      // Browser
@compile_target("native")    // Server
@compile_target("mobile")    // iOS / Android
@compile_target("edge")      // IoT / edge devices
```

**Status**: Aspirational. Each target depends on the compilation pipeline described above.

---

## Roadmap summary

| Capability | Status | Dependencies |
|---|---|---|
| Tree-walking interpreter | **Exists** | -- |
| 30-module stdlib | **Exists** | -- |
| Agent framework (spawn/coordinate/evolve) | **Exists** | -- |
| Persistent agent memory | **Exists** | -- |
| Skills registry | **Exists** | -- |
| HTTP server (`dal serve`) | **Exists** | -- |
| Solidity transpiler | **Exists** | -- |
| Molds (module system) | **Exists** | -- |
| CLI toolchain | **Exists** | -- |
| Compile-time memory safety | Research | Ownership model design |

---

## Who DAL is for today

- **Agent system builders** who want first-class agent primitives.
- **Blockchain developers** who need integrated on-chain/off-chain logic with Solidity output.
- **Rapid prototypers** who benefit from a concise syntax with a large stdlib and built-in HTTP serving.

For general-purpose compiled application development, DAL is not yet a substitute for Rust, Go, or TypeScript. The roadmap above describes the path toward that goal.
