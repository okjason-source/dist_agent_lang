# Comprehensive Review: Writing Smart Contracts with DAL

**Version:** 1.0  
**Date:** 2026-02-17  
**Purpose:** Single reference for what DAL offers when writing “smart contract” style logic: model, security, chain interaction, limitations, and how it fits with Solidity/EVM.

---

## 1. Executive Summary

DAL (dist_agent_lang) lets you write **contract-like services** that combine on-chain interaction, built-in security, and hybrid trust in one language. You do **not** write Solidity or EVM bytecode directly; you write DAL **services** that can call chains via `chain::`, enforce `@secure` and reentrancy protection, and use `@advanced_security` for MEV-aware DeFi. Deployment to real chains today is via **pre-signed raw transactions** or orchestration around existing contracts; **DAL does not compile to EVM bytecode**. This review covers the contract-writing model, security attributes, chain usage, testing, and practical limits.

---

## 2. The Contract Model in DAL

### 2.1 Services, Not `contract` Keyword

DAL uses **services** as the unit of stateful, callable logic. There is no `contract` keyword; a “smart contract” in DAL is a **service** with chain-related attributes and optional `chain::` calls.

```dal
@trust("hybrid")
@chain("ethereum")
@secure
service DefiToken {
    name: string = "DeFi Token";
    symbol: string = "DFT";
    total_supply: int = 0;
    balances: map<string, int> = {};

    fn balance_of(account: string) -> int {
        if (self.balances.contains_key(account)) {
            return self.balances[account];
        }
        return 0;
    }

    fn transfer(to: string, amount: int) -> bool {
        let from = auth::session("user", ["holder"]).user_id;
        if (self.balances[from] < amount) {
            return false;
        }
        self.balances[from] = self.balances[from] - amount;
        self.balances[to] = self.balances[to] + amount;
        return true;
    }
}
```

- **Fields:** Typed state (e.g. `map<string, int>` for balances).  
- **Methods:** `fn name(params) [ -> type ] { body }`.  
- **Attributes:** Applied at service or function level (see below).

### 2.2 Core Attributes for “Contract” Behavior

| Attribute | Level | Effect |
|-----------|--------|--------|
| **`@trust("hybrid" \| "decentralized" \| "centralized")`** | Service | Trust model for the service; required with `@chain` in examples. |
| **`@chain("ethereum" \| "polygon" \| ...)`** | Service | Declares which chain(s) the service targets (chain_id resolved via stdlib registry). |
| **`@secure`** | Service or function | Auth + reentrancy guard + audit logging. Enforced at runtime. |
| **`@public`** | Service or function | No auth required; mutually exclusive with `@secure`. |
| **`@txn`** | Function | Wraps execution in a transaction (atomicity/rollback in DAL runtime). |
| **`@limit(n)`** | Service or function | Resource cap (e.g. max operations). |
| **`@advanced_security`** / **`@advanced_security("monitor" \| "advisory" \| "strict")`** | Service | MEV-aware checks: monitor (warn), advisory (suggest), strict (block unprotected execution). |
| **`@compile_target("blockchain")`** | Service | Declares intent to compile to chain; **not yet implemented** (no EVM bytecode emission). |

Function-level `@secure` / `@public` override service-level; if a function has neither, it inherits from the service.

### 2.3 Events

Services can declare **events** and “emit” them in logic (for audit and downstream tooling; not on-chain event logs unless you push them via chain/oracle).

```dal
event Transfer(from: string, to: string, amount: int);
event AdminAction(admin: string, action: string);

// In a method:
event Transfer { from: from, to: to, amount: amount };
```

---

## 3. Security Model

### 3.1 @secure: Auth, Reentrancy, Audit

When a service or function is `@secure`:

- **Authentication:** Runtime requires a valid `current_caller` (not null/default address). Otherwise `AccessDenied`.  
- **Reentrancy:** A reentrancy guard prevents re-entry into the **same** `instance_id::method_name`. Same instance + different method is allowed.  
- **Audit:** Access attempts (allowed/denied) and reentrancy attempts are logged. With file logging enabled (`LOG_SINK`, `LOG_DIR`), logs go to disk for compliance.

So for “contract” style code, `@secure` gives you a single attribute for “only authenticated callers” and “no reentrancy into this method.”

### 3.2 @public vs @secure

- Use **`@public`** for read-only or unauthenticated endpoints (e.g. `balance_of`, public config).  
- Use **`@secure`** for state-changing or privileged operations (e.g. `transfer`, `mint`, admin).  
- Do not combine both on the same service/function; they are mutually exclusive.

### 3.3 @advanced_security (MEV / DeFi)

For DeFi-style logic, `@advanced_security` adds:

- **Monitor (default):** Scans for MEV-related patterns, logs warnings, does **not** block.  
- **Advisory:** Suggests protection patterns (e.g. slippage, commit-reveal).  
- **Strict:** Can **block** execution when unprotected high-risk patterns are detected.

The runtime distinguishes “monitoring” code (e.g. `find_*`, `detect_*`) from “execution” code (e.g. `execute_swap`, `transfer`). Monitoring is allowed; execution may be blocked in strict mode if protections are missing. Manual patterns (commit-reveal, slippage checks) are documented in `MEV_PROTECTION_MANUAL.md`.

### 3.4 No Separate @reentrancy_guard / @safe_math in Docs

Reentrancy is covered by **`@secure`**. Overflow/safe math is not a separate attribute in the current attributes reference; use careful arithmetic or patterns as needed. The Solidity converter suggests `@reentrancy_guard` and `@safe_math` when converting from OpenZeppelin; in DAL, the runtime reentrancy guard is part of `@secure`.

---

## 4. Chain Interaction

### 4.1 What chain:: Provides

- **chain::deploy(chain_id, contract_name, constructor_args)**  
  - If `constructor_args` contains **`raw_transaction`** or **`signed_tx`** (hex): sends via `eth_sendRawTransaction` and returns the contract address.  
  - Otherwise returns a **mock** address (no on-chain deploy).  

- **chain::call(chain_id, contract_address, function_name, args)**  
  - If `args` contains **`data`** or **`calldata`** (ABI-encoded hex): performs **eth_call** and returns result.  
  - Otherwise returns a string message only (no real call).  

- **chain::get_balance**, **chain::get_gas_price**, **chain::estimate_gas**, **chain::get_transaction_status**, **chain::get_block_timestamp**  
  - Use JSON-RPC when the `http-interface` feature is enabled and RPC is configured; otherwise fallbacks/mocks.  

- **chain::mint(name, metadata)**  
  - Generates an in-memory asset id and logs; no chain mint unless you wire raw tx elsewhere.  

- **chain::get(asset_id)** / **chain::exists(asset_id)**  
  - When `CHAIN_ASSET_CHAIN_ID` and `CHAIN_ASSET_CONTRACT` are set, can call ERC721 `tokenURI` / `ownerOf` via hardcoded selectors.  

So: **real on-chain reads and real deploy/call are possible when you supply signed tx or ABI-encoded `data`.** DAL does not today compile your service to EVM bytecode or produce that encoding for you.

### 4.2 Multi-Chain

You can declare multiple chains and pass `chain_id` into `chain::` calls. The chain registry (e.g. Ethereum, Polygon, BSC, Arbitrum, Optimism) provides RPC URLs and metadata. Examples use `@chain("ethereum")` or multiple `@chain(...)`; configuration can be overridden via env (e.g. RPC URL).

### 4.3 Orchestration vs On-Chain Bytecode

- **Orchestration:** DAL services run in the DAL runtime. They keep state in the runtime (e.g. `self.balances`), call `chain::*` to read/write chains, and can coordinate agents, oracles, and HTTP. This is the **current** model.  
- **On-chain bytecode:** Some docs (e.g. SMART_CONTRACT_INTERFACE_SEPARATION, PHASE2_COMPILATION_TARGETS) describe `@compile_target("blockchain")` and compiling to EVM bytecode. **That pipeline is not implemented.** Today, “deploying a DAL contract” means either (1) supplying a pre-signed raw transaction from an external build (e.g. Solidity compiled elsewhere), or (2) running the DAL service as an off-chain orchestrator that talks to existing contracts.

---

## 5. Writing Patterns

### 5.1 Token / Balance Logic

Keep balances in a `map<string, int>` (or similar); use `@secure` on transfer/mint, check balances and use `@txn` if you want atomicity in the DAL runtime. Emit events for audit. Example: `tutorials/01_defi_token.md`, `examples/smart_contract.dal`.

### 5.2 Calling Existing Solidity Contracts

To call an already-deployed contract you need ABI-encoded **`data`**. Today you must:

- Provide that encoding externally, or  
- Use a future add_sol that is wired in the engine and does ABI encoding (see SOLIDITY_EVM_INTEGRATION_SCOPE.md).

Then pass `data` (or `calldata`) in the args to **chain::call**.

### 5.3 Oracle and External Data

Use **oracle::** for price/data feeds and **chain::** for on-chain state. DeFi examples combine oracle data with transfer/pool logic; MEV patterns (commit-reveal, slippage) are manual in DAL as in `MEV_PROTECTION_MANUAL.md`.

### 5.4 Admin and Access

Use **auth::** (e.g. `auth::session()`, `auth::has_role()`) for caller identity and roles. **cloudadmin::** provides authorize/grant/revoke and policy for hybrid admin control. Apply **@secure** on admin-only methods so only authenticated callers with the right context can execute.

---

## 6. Testing

- **Unit / integration:** Use the built-in test framework (e.g. `*.test.dal`), `dal test`, and assertions. You can test service methods and chain calls (with mocks or real RPC as needed).  
- **Security:** Reentrancy and safe math are covered by runtime tests. `@advanced_security` behavior can be tested with monitor/advisory/strict examples (see READINESS_CHECKLIST.md).  
- **Testnet:** Deploy via raw tx to a testnet and run DAL against that RPC to validate chain:: behavior end-to-end.

---

## 7. Limitations and Caveats

| Area | Limitation |
|------|------------|
| **EVM bytecode** | DAL does not compile services to EVM bytecode. No “dal build → deploy bytecode” today. |
| **ABI encoding** | No built-in encoding of (function, args) to `data`. Callers must supply encoded `data` or use a future add_sol. |
| **Deploy** | Real deploy only via pre-signed raw transaction. No “compile Solidity/DAL and sign inside DAL” in one step. |
| **add_sol** | ABI parse/register/call_with_abi exist in stdlib but are **not** wired in the runtime; DAL code cannot call `add_sol::*` yet. |
| **Function-level @secure** | Documented; implementation may only enforce at service level in some paths (see SECURE_SCOPE.md). |
| **Events** | Event declarations and emission are in-DAL; they do not automatically become chain logs unless you explicitly push them (e.g. via chain or logging). |

---

## 8. When to Use DAL for “Smart Contract” Work

- **Good fit:**  
  - Off-chain orchestration that coordinates multiple contracts, oracles, and agents.  
  - Services that need built-in auth, reentrancy protection, and audit logging in one place.  
  - DeFi-style logic where you want MEV awareness and manual protection patterns in the same language.  
  - Multi-chain reads and writes when you can supply raw tx or pre-encoded call data.  

- **Less fit today:**  
  - Deploying new contract bytecode from DAL only (need external compile + sign).  
  - Calling arbitrary Solidity functions by name + args without bringing your own ABI encoding.  
  - Replacing Solidity entirely for contracts that must live only on-chain; DAL is a hybrid runtime.

---

## 9. References

| Document | Content |
|----------|---------|
| `docs/syntax.md` | Service declaration, attributes, events |
| `docs/attributes.md` | Full attribute list |
| `docs/REENTRANCY_CLARITY.md` | What @secure does (reentrancy + auth) |
| `docs/SECURE_SCOPE.md` | Scope of @secure, service vs function |
| `docs/guides/SECURE_ATTRIBUTE_USAGE.md` | @secure vs @public, patterns |
| `docs/MEV_PROTECTION_MANUAL.md` | Manual MEV patterns in DAL |
| `docs/ADVANCED_SECURITY_DESIGN.md` | @advanced_security tiers |
| `docs/tutorials/01_defi_token.md` | DeFi token tutorial |
| `docs/guides/DEPLOYMENT_GUIDE.md` | Deployment checklist and env |
| `docs/project/SOLIDITY_EVM_INTEGRATION_SCOPE.md` | What’s implemented vs missing for EVM |
| `docs/guides/SMART_CONTRACT_INTERFACE_SEPARATION.md` | Separation of contract vs UI |
| `examples/smart_contract.dal`, `examples/defi_nft_rwa_contract.dal` | Example “contract” services |

---

**Bottom line:** DAL gives you a **single language** for contract-like services with **@trust**, **@chain**, **@secure**, and **@advanced_security**, plus **chain::** for real RPC and raw-tx deploy/call. Use it for orchestration and hybrid apps; treat “compile to EVM” as a future or external step and supply signed tx or ABI-encoded `data` for real on-chain deployment and calls today.
