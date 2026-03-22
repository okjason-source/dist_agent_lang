# Comprehensive Review: Smart Contracts with DAL (Current State)

**Version:** 2.0  
**Date:** 2026-03-20  
**Purpose:** Single, up-to-date reference for DAL smart-contract workflows: trust split model, EVM path maturity, ABI/decode reliability, and remaining gaps.

---

## 1. Executive Summary

DAL now supports a stronger smart-contract path than the previous review:

- You can write contract-oriented DAL services with explicit trust boundaries.
- `@compile_target("blockchain")` emits Solidity/ABI/bin artifacts when toolchain is present.
- Hybrid services can auto-split orchestration-only methods to HTTP artifacts while preserving bytecode-safe methods for EVM artifacts.
- `chain::call_typed` and `chain::deploy_typed` expose typed evidence fields.
- A typed ABI decode surface (`stdlib::abi_codec`) is extracted and used at integration boundaries.
- ABI registry-based decode is active, with overload-safe disambiguation via optional `function_signature`.

What is still not complete:

- Generated Solidity method bodies are still conservative/stub-oriented for many constructs.
- Full decentralized codegen subset and end-to-end “DAL source -> production-grade on-chain logic” is still in-progress.

---

## 2. DAL Contract Model

### 2.1 Services as Contract Units

DAL “contracts” are services with trust and chain attributes. There is no separate `contract` keyword.

Key attributes:

| Attribute | Role |
|---|---|
| `@trust("decentralized" \| "hybrid" \| "centralized")` | Declares trust model and policy envelope |
| `@chain("...")` | Declares chain context and allowed chain namespace usage |
| `@secure` / `@public` | Access semantics; mutually exclusive |
| `@compile_target("blockchain")` | Routes service into blockchain compile backend |

### 2.2 Trust Split (Current Enforcement)

- **`decentralized`**: deterministic-only path; disallowed orchestration namespaces are blocked by policy/checks.
- **`hybrid`**: orchestration + chain interaction; now supports dual-artifact split for mixed logic.
- **`centralized`**: runtime-centric orchestration path.

Important product direction:

- Eventual naming migration from `decentralized` to `zero` is planned with compatibility aliasing; not yet cut over.

---

## 3. EVM Compile Path: What Works Today

### 3.1 Blockchain Backend

`@compile_target("blockchain")` currently supports:

- DAL service selection for blockchain target
- Solidity source emission
- solc invocation for `.abi`/`.bin` outputs (when available)
- compile manifest generation

### 3.2 Hybrid Auto-Split Artifacts

For `@trust("hybrid")` + `@compile_target("blockchain")`:

- Methods using orchestration/admin namespaces (`auth::`, `cloudadmin::`, `cloud` alias) are auto-routed to HTTP/orchestration artifact output.
- Bytecode-safe methods continue in EVM artifact flow.
- HTTP split artifact emitted as `<Service>.http.json`.

This enables one hybrid service definition to produce dual artifacts without silent cross-boundary leakage.

---

## 4. Chain Runtime and Typed Evidence

### 4.1 Typed Chain Surfaces

- `chain::deploy_typed(...)`
- `chain::call_typed(...)`

Both carry evidence contracts including:

- payload fields (`contract_address` / `result_hex`)
- `tx_hash`
- `receipt_status`
- `revert_data`
- `error_code`
- `message`

`chain::deploy`/`chain::call` remain as compatibility wrappers.

### 4.2 Strict Mode

Strict policy controls synthetic fallback behavior:

- enabled via env policy path
- rejects missing signed deploy tx / missing calldata in strict contexts
- enforces explicit, auditable failure over implicit simulation in protected paths

---

## 5. ABI/Decode Maturity (Major Progress)

### 5.1 Vector-Hardened Coverage

ABI golden vectors now cover:

- selector determinism and catalog parity
- static words (`uint`, `bool`, `address`, `bytes32`)
- revert payloads (`Error(string)`, `Panic(uint256)`)
- custom error family parity
- dynamic returns (`string`, `bytes`)
- nested dynamic tuple decode (`tuple(string,bytes)`)
- malformed payload rejection
- selector mismatch rejection

### 5.2 Extracted Typed Codec Surface

`src/stdlib/abi_codec.rs` now contains decode-focused helpers:

- word decoders
- dynamic payload decoders
- nested tuple decoder
- custom error payload decoder
- revert payload decoders

This extraction is additive and compatibility-safe: behavior remains pinned by vectors.

---

## 6. ABI Registry Integration (Current State)

### 6.1 Registry Model

ABI metadata is registered per contract identity:

- key: `(chain_id, contract_address)`
- value: parsed function metadata

Registration occurs during `add_sol::register_contract(...)`.

### 6.2 Runtime Decode Resolution

`chain::call_typed` now attempts decode through registry identity:

- resolves by `chain_id + contract_address + function_name`
- populates additive fields:
  - `decoded`
  - `decode_error`

### 6.3 Overload Safety

For overloaded function names:

- decode does **not** guess
- optional `function_signature` hint is used for deterministic disambiguation
- if omitted under overload ambiguity, decode is skipped with explicit `decode_error`

This is aligned with audit-ready deterministic behavior.

---

## 7. add_sol Boundary Status

`add_sol` is now runtime-integrated (no longer “stdlib-only/not wired”):

- `add_sol::call_with_abi(...)` (legacy string projection)
- `add_sol::call_with_abi_typed(...)` (typed envelope with decoded/evidence fields)

Typed boundary fields include:

- `result_hex`
- `decoded`
- `decode_error`
- `tx_hash`
- `receipt_status`
- `revert_data`
- `error_code`
- `message`

---

## 8. Security and Policy Posture

- `@secure` remains the primary runtime guard for authenticated, non-reentrant execution paths.
- Trust-mode boundary enforcement is active and increasingly compile/test-gated.
- Hybrid split and overload-safe decode reduce ambiguity and policy leakage risks.
- CI trust-gate has expanded forensic logs for each hardening increment.

---

## 9. Remaining Gaps (Honest Status)

| Area | Current gap |
|---|---|
| Full decentralized codegen | Broad subset still pending; many generated method bodies remain conservative |
| End-to-end production contract path | Still not equivalent to “drop-in Solidity replacement” for all patterns |
| Typed decode conventions in DAL call-sites | Signature-hint conventions and guidance are still being normalized |
| Audit package completeness | In progress; not yet final-form |

---

## 10. Practical Guidance: Use DAL for Smart Contract Work

**Good fit now**

- Trust-aware orchestration over on-chain systems
- Typed chain interaction with evidence-first outputs
- ABI-driven integration with existing Solidity contracts
- Hybrid services requiring clear split between chain-safe and orchestration-only blocks

**Use caution / expect work-in-progress**

- Fully replacing Solidity-only contract development for complex on-chain-only programs
- Assuming all DAL methods under blockchain target generate production-grade non-stub contract logic today

---

## 11. Reference Pointers (Live Source of Truth)

- `docs/development/implementation/TRUST_SPLIT_EVM_PRODUCTION_PLAN.md`
- `docs/development/testing/TRUST_SPLIT_EVM_TEST_AND_RELEASE_PLAN.md`
- `docs/development/implementation/TRUST_SPLIT_EVM_HARDENING_REFACTOR_PLAN.md`
- `src/stdlib/chain.rs`
- `src/stdlib/abi_codec.rs`
- `src/stdlib/add_sol.rs`
- `.github/workflows/ci.yml` (`trust-split-fast`)

---

**Bottom line:** DAL’s smart-contract path is now materially stronger than earlier states: typed evidence, hardened ABI decode, hybrid dual-artifact split, and overload-safe registry decode are in place. The strategic goal remains unchanged: continue converting this reliable integration/runtime surface into a full audit-ready contract-native path by closing the remaining decentralized codegen and final operational audit gaps.
