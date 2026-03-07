# 📘 Best Practices Guide

> **dist_agent_lang (DAL)** — This guide uses **DAL syntax and stdlib**. Services use `@trust`, `@chain`, `@secure`; functions use `fn`; chain and oracle calls use the actual `chain::` and `oracle::` APIs. See [syntax.md](../syntax.md), [attributes.md](../attributes.md), and [STDLIB_REFERENCE.md](../STDLIB_REFERENCE.md).

Comprehensive guide to writing secure, efficient, and maintainable **dist_agent_lang** code.

---

## 📋 Table of Contents

1. [Security Best Practices](#security-best-practices)
2. [Performance and Code Organization](#performance-and-code-organization)
3. [Error Handling](#error-handling)
4. [Testing Strategies](#testing-strategies)
5. [Chain and Gas Usage](#chain-and-gas-usage)
6. [Oracle Integration](#oracle-integration)
7. [Multi-Chain Development](#multi-chain-development)
8. [Common Patterns](#common-patterns)
9. [Anti-Patterns (What to Avoid)](#anti-patterns-what-to-avoid)

---

## 🔒 Security Best Practices

### 1. Use service attributes for chain and trust

**✅ DO:**
```dal
@trust("hybrid")
@chain("ethereum")
@secure
service SecureService {
    balances: map<string, int> = {};
    fn transfer(to: string, amount: int) -> bool {
        let caller = chain::caller();
        if (amount <= 0 || !self.balances.contains_key(caller)) { throw "Invalid transfer"; }
        if (self.balances[caller] < amount) { throw "Insufficient balance"; }
        self.balances[caller] = self.balances[caller] - amount;
        if (!self.balances.contains_key(to)) { self.balances[to] = 0; }
        self.balances[to] = self.balances[to] + amount;
        return true;
    }
}
```

**❌ DON'T:**
```dal
service Vulnerable {
    // No @trust, @chain, @secure — chain access and auth not enforced
}
```

`@secure` enables authentication (caller must be set), reentrancy guard, and audit logging. Use `@public` only for read-only or unauthenticated endpoints; do not combine `@secure` and `@public` on the same service or function.

### 2. Validate inputs before state changes

**✅ DO:**
```dal
@trust("hybrid")
@chain("ethereum")
@secure
service Token {
    balances: map<string, int> = {};
    fn transfer(to: string, amount: int) -> bool {
        if (to == "" || to == "0x0000000000000000000000000000000000000000") {
            throw "Invalid address";
        }
        if (amount <= 0) { throw "Amount must be positive"; }
        let caller = chain::caller();
        if (!self.balances.contains_key(caller) || self.balances[caller] < amount) {
            throw "Insufficient balance";
        }
        self.balances[caller] = self.balances[caller] - amount;
        if (!self.balances.contains_key(to)) { self.balances[to] = 0; }
        self.balances[to] = self.balances[to] + amount;
        return true;
    }
}
```

**❌ DON'T:**
```dal
fn transfer(to: string, amount: int) {
    self.balances[caller] = self.balances[caller] - amount;
    self.balances[to] = self.balances[to] + amount;
}
```

### 3. Update state before external chain calls (checks–effects–interactions)

**✅ DO:**
```dal
@txn
@secure
fn withdraw(amount: int) {
    let caller = chain::caller();
    if (!self.pending.contains_key(caller) || self.pending[caller] < amount) {
        throw "Insufficient pending";
    }
    self.pending[caller] = self.pending[caller] - amount;
    let result = chain::call(chain_id, self.vault_address, "withdraw", { "amount": amount.to_string() });
    if (!result.contains("success")) { throw "Withdraw failed"; }
}
```

**❌ DON'T:** Call `chain::call` (or any external interaction) before updating service state; that can create reentrancy risk.

### 4. Use events and logging for important actions

**✅ DO:**
```dal
event Transfer { from: string, to: string, amount: int };

fn transfer(to: string, amount: int) {
    // ... transfer logic ...
    event Transfer { from: chain::caller(), to: to, amount: amount };
    log::info("transfer", { "to": to, "amount": amount });
    return true;
}
```

### 5. Secure oracle usage

**✅ DO:**
```dal
let query = oracle::create_query("BTC/USD");
let result = oracle::fetch_with_consensus(
    ["source1", "source2", "source3"],
    query,
    0.66
);
if (result == null) { throw "Oracle consensus failed"; }
let price = result.data;
let ts = chain::get_block_timestamp(1);
if (result.timestamp != null && ts - result.timestamp > 300) {
    throw "Price too old";
}
```

**❌ DON'T:** Rely on a single oracle source without consensus or freshness checks when handling value or critical decisions.

---

## ⚡ Performance and Code Organization

### 1. Batch work and limit chain calls

Prefer computing totals or payloads in memory, then one or few `chain::call` / `chain::deploy` invocations. Use `@limit(n)` on services or functions to cap resource use.

### 2. Cache expensive or remote data

Use service fields to cache oracle or chain data when validity windows allow (e.g. block timestamp or TTL). Use `chain::get_block_timestamp(chain_id)` for time; avoid calling oracle or RPC in a tight loop.

### 3. Project structure

```
my-project/
├── main.dal or lib.dal    # Entry; services and top-level fns
├── core/                  # Core services (e.g. token.dal, marketplace.dal)
├── tests/
│   └── *.test.dal        # describe/it, test::expect_*, expect_throws
├── scripts/
│   └── deploy.sh         # Build + sign tx; see PRESIGNED_DEPLOYMENT_GUIDE.md
├── docs/
├── .env.example
└── README.md
```

### 4. Service layout

Use `@trust`, `@chain`, `@secure` (or `@public`) consistently. Put fields first, then methods; use events and `log::` for important state changes. Caller identity: `chain::caller()`.

```dal
@trust("hybrid")
@chain("ethereum", "polygon")
@secure
service WellOrganized {
    total_supply: int = 0;
    balances: map<string, int> = {};
    event Transfer { from: string, to: string, amount: int };

    fn transfer(to: string, amount: int) -> bool {
        if (amount <= 0) { throw "Invalid amount"; }
        let from = chain::caller();
        if (!self.balances.contains_key(from) || self.balances[from] < amount) {
            throw "Insufficient balance";
        }
        self.balances[from] = self.balances[from] - amount;
        if (!self.balances.contains_key(to)) { self.balances[to] = 0; }
        self.balances[to] = self.balances[to] + amount;
        event Transfer { from: from, to: to, amount: amount };
        return true;
    }

    fn balance_of(account: string) -> int {
        if (self.balances.contains_key(account)) { return self.balances[account]; }
        return 0;
    }
}
```

---

## 🚨 Error Handling

### 1. Use descriptive messages with throw

**✅ DO:**
```dal
fn withdraw(amount: int) {
    if (amount <= 0) { throw "Withdrawal amount must be greater than zero"; }
    let caller = chain::caller();
    if (!self.balances.contains_key(caller) || self.balances[caller] < amount) {
        throw "Insufficient balance";
    }
    if (self.paused) { throw "Contract is paused"; }
    self.balances[caller] = self.balances[caller] - amount;
    // ... complete withdrawal ...
}
```

**❌ DON'T:** Use empty or vague messages: `throw "Error";` — be specific so callers and logs are actionable.

### 2. Use Result for recoverable failures

**✅ DO:** Return `Result<T, string>` and use `Ok`/`Err` when the caller should handle failure without aborting.
```dal
fn try_transfer(to: string, amount: int) -> Result<bool, string> {
    if (to == "") { return Err("Invalid address"); }
    if (amount <= 0) { return Err("Invalid amount"); }
    let from = chain::caller();
    if (!self.balances.contains_key(from) || self.balances[from] < amount) {
        return Err("Insufficient balance");
    }
    self.balances[from] = self.balances[from] - amount;
    if (!self.balances.contains_key(to)) { self.balances[to] = 0; }
    self.balances[to] = self.balances[to] + amount;
    return Ok(true);
}
```

### 3. Handle chain and external call results

**✅ DO:** Check return values from `chain::call` and other stdlib calls; throw or return Err on failure.
```dal
let result = chain::call(chain_id, contract_address, "withdraw", { "amount": amount.to_string() });
if (result == null || !result.contains("success")) {
    throw "External call failed: " + (result ?? "null");
}
```

Use **try/catch** when you want to handle thrown errors and continue:
```dal
try {
    self.do_risky_operation();
} catch (e) {
    log::error("operation", { "error": e });
    return false;
}
```

---

## 🧪 Testing Strategies

Use the **three-layer** approach: (1) Rust unit tests for parsing/syntax, (2) semantic validators in `test::`, (3) DAL test files (`.test.dal`) for behavior. See [TESTING_QUICK_REFERENCE.md](../TESTING_QUICK_REFERENCE.md).

### 1. DAL test files: describe / it / expect

**✅ DO:**
```dal
// tests/token.test.dal
describe("Token", fn() {
    let token;
    beforeEach(fn() {
        token = MyToken::new();
        token.deposit(chain::caller(), 1000000);
    });
    it("transfers correctly", fn() {
        token.transfer("0xRecipient", 1000);
        expect(token.balance_of("0xRecipient")).to_equal(1000);
    });
    it("fails on insufficient balance", fn() {
        expect_throws(fn() { token.transfer("0xRecipient", 2000000); }, "Insufficient balance");
    });
});
```

### 2. Semantic validation (test::)

**✅ DO:** Use `test::expect_*` for types, ranges, and structure in DAL tests or validators.
```dal
test::expect_type(&result, "number");
test::expect_in_range(price, 0.0, 1000000.0);
test::expect_has_key(config, "chain_id");
test::expect_valid_trust_model("hybrid");
test::expect_valid_chain("ethereum");
```

### 3. Integration and Rust tests

Run full flows with `dal run file.test.dal`. For parser/AST and example-file validation, use `cargo test` (e.g. `cargo test --test example_tests`).

---

## ⛽ Chain and Gas Usage

### 1. Use chain stdlib for gas and cost

**✅ DO:** Use `chain::estimate_gas(chain_id, operation)` and `chain::get_gas_price(chain_id)` (or `chain::get_current_gas_price(chain_id)`) to compare chains or warn users.
```dal
let eth_gas = chain::estimate_gas(1, "transfer");
let poly_gas = chain::estimate_gas(137, "transfer");
let eth_price = chain::get_gas_price(1);
let poly_price = chain::get_gas_price(137);
if (poly_gas * poly_price < eth_gas * eth_price) {
    log::info("gas", "Polygon is cheaper for this operation");
}
```

### 2. Batch chain operations

Prefer one `chain::call` with batched args (or a contract that batches on-chain) over many small calls. Use `chain::get_chain_config(chain_id)` and `chain::get_supported_chains()` when building multi-chain flows.

### 3. Pre-signed deployment

For real contract deployment, use pre-signed transactions (see [PRESIGNED_DEPLOYMENT_GUIDE.md](PRESIGNED_DEPLOYMENT_GUIDE.md)); avoid putting private keys in DAL.

---

## 🔮 Oracle Integration

### 1. Multi-source consensus

**✅ DO:** Use `oracle::create_query` and `oracle::fetch_with_consensus` with multiple sources and a threshold.
```dal
let query = oracle::create_query("ETH/USD");
let result = oracle::fetch_with_consensus(
    ["source1", "source2", "source3"],
    query,
    0.66
);
if (result == null) { throw "Oracle consensus failed"; }
self.eth_price = result.data;
self.last_update = chain::get_block_timestamp(1);
```

### 2. Freshness and verification

**✅ DO:** Check age with `chain::get_block_timestamp(chain_id)`; use `oracle::verify(data, signature)` when signatures are available. Reject or refresh when data is too old (e.g. > 300 seconds for prices).

---

## 🌍 Multi-Chain Development

### 1. Declare supported chains

**✅ DO:** Use `@chain("ethereum", "polygon", ...)` and pass `chain_id` explicitly in `chain::` calls so the same service can run on multiple chains.
```dal
@trust("hybrid")
@chain("ethereum", "polygon", "arbitrum")
@secure
service MultiChain {
    fn transfer(chain_id: int, to: string, amount: int) -> bool {
        let caller = chain::caller();
        if (self.balances[caller] < amount) { throw "Insufficient balance"; }
        self.balances[caller] = self.balances[caller] - amount;
        if (!self.balances.contains_key(to)) { self.balances[to] = 0; }
        self.balances[to] = self.balances[to] + amount;
        return true;
    }
}
```

### 2. Chain-specific logic

**✅ DO:** Use `chain::get_chain_config(chain_id)` and `chain::get_gas_price(chain_id)` (or `chain::get_current_gas_price(chain_id)`) when you need chain-dependent behavior (e.g. gas pricing, RPC, explorer).
```dal
fn get_gas_cost(chain_id: int) -> float {
    let gas = chain::estimate_gas(chain_id, "transfer");
    let price = chain::get_gas_price(chain_id);
    return gas * price;
}
```

---

## 🎯 Common Patterns

### 1. Pull over push for withdrawals

**✅ DO:** Update pending state first, then perform the external transfer or chain call so reentrancy cannot drain more than the cleared amount.
```dal
pending: map<string, int> = {};
fn withdraw() {
    let caller = chain::caller();
    let amount = 0;
    if (self.pending.contains_key(caller)) { amount = self.pending[caller]; }
    if (amount <= 0) { throw "No funds to withdraw"; }
    self.pending[caller] = 0;
    let result = chain::call(chain_id, vault, "withdraw", { "amount": amount.to_string() });
    if (!result.contains("success")) { throw "Withdraw failed"; }
}
```

### 2. Pause / admin guard

**✅ DO:** Use a `paused` field and an owner/caller check; throw at the start of sensitive methods when paused or when caller is not allowed.
```dal
paused: bool = false;
owner: string = "0x...";
fn pause() {
    if (chain::caller() != self.owner) { throw "Not owner"; }
    self.paused = true;
}
fn critical_operation() {
    if (self.paused) { throw "Contract is paused"; }
    // ...
}
```

---

## ❌ Anti-Patterns (What to Avoid)

### 1. Don't use origin/caller for auth without validation

Use `chain::caller()` for the current caller and validate against a stored owner or capability; don't rely on unvalidated identity for privileged actions.

### 2. Don't ignore chain or oracle return values

**❌ DON'T:** Call `chain::call` or `oracle::fetch` and ignore the result.
**✅ DO:** Check the return value and throw or return Err when the call failed or returned invalid data.

### 3. Don't use block or time for randomness

**❌ DON'T:** Use `chain::get_block_timestamp` or block-derived values as the sole source of randomness for value-bearing logic (manipulable).
**✅ DO:** Use an oracle or verifiable randomness source (e.g. oracle::fetch with a VRF source) when you need randomness.

### 4. Don't put secrets in source

Never put private keys or long-lived secrets in DAL source or in constructor args. Use env, config, or a secure deploy pipeline; see [PRESIGNED_DEPLOYMENT_GUIDE.md](PRESIGNED_DEPLOYMENT_GUIDE.md).

---

## 📚 Additional Resources

- [Syntax Reference](../syntax.md) — Grammar and structure
- [Attributes Reference](../attributes.md) — `@trust`, `@chain`, `@secure`, `@txn`, `@limit`, `@compile_target`, etc.
- [STDLIB_REFERENCE.md](../STDLIB_REFERENCE.md) — All stdlib modules and functions
- [API Reference](API_REFERENCE.md) — Grouped API overview
- [PRESIGNED_DEPLOYMENT_GUIDE.md](PRESIGNED_DEPLOYMENT_GUIDE.md) — Real contract deployment with DAL
- [TESTING_QUICK_REFERENCE.md](../TESTING_QUICK_REFERENCE.md) — Three-layer testing, describe/it, test::expect_*
- [ADVANCED_SECURITY_BEST_PRACTICES.md](../ADVANCED_SECURITY_BEST_PRACTICES.md) — Security deep dive
- [Tutorials](../tutorials.md) — Learn by building

---

**Next:** [API Reference →](API_REFERENCE.md)

