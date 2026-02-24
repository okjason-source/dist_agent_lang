# Tutorial 1: Building a DeFi Token with Oracle Integration

> **📢 Beta:** Test thoroughly in development. Feedback appreciated!

Learn to build a DeFi-style token in DAL: services, maps, and chain integration.

**Time**: 30 minutes  
**Difficulty**: Beginner  
**Prerequisites**: Basic programming knowledge

---

## 🎯 What You'll Build

A token service that:
- Holds balances in a map
- Supports transfer and balance lookup
- Uses oracle-style price data (via stdlib)
- Uses DAL service syntax: `@trust`, `@chain`, `fn`, `let`, literals

---

## 📦 Setup

```bash
mkdir defi-token && cd defi-token
touch DefiToken.dal
```

---

## Step 1: Basic Token Structure

DAL uses **services** (not `contract`). Use `@trust` for the service trust model; add `@chain` when the service interacts with a blockchain.

```dal
// DefiToken.dal
@trust("hybrid")
@chain("ethereum")
service DefiToken {
    name: string = "DeFi Token";
    symbol: string = "DFT";
    total_supply: int = 0;
    balances: map<string, int> = {};
    current_price: int = 0;
}
```

---

## Step 2: Add Core Functions

Use **`fn`**, **`if (condition)`** (parentheses required), and **semicolons** after statements.

```dal
@trust("hybrid")
@chain("ethereum")
service DefiToken {
    name: string = "DeFi Token";
    symbol: string = "DFT";
    total_supply: int = 0;
    balances: map<string, int> = {};
    current_price: int = 0;

    fn balance_of(account: string) -> int {
        if (self.balances.contains_key(account)) {
            return self.balances[account];
        }
        return 0;
    }

    fn transfer(to: string, amount: int) -> bool {
        let from = auth::session("user", ["holder"]).user_id;
        if (self.balances[from] < amount) {
            log::info("token", "Insufficient balance");
            return false;
        }
        self.balances[from] = self.balances[from] - amount;
        self.balances[to] = self.balances[to] + amount;
        log::info("token", "Transfer done");
        return true;
    }

    fn initialize(owner: string, initial_supply: int) {
        self.balances[owner] = initial_supply;
        self.total_supply = initial_supply;
        log::info("token", "Token initialized");
    }
}
```

---

## Step 3: Add Oracle-Based Pricing

Use **`oracle::fetch`** (or similar stdlib) and **literals** for data. Literals use **`key: value`** with a colon.

```dal
    fn update_price() {
        let price_response = oracle::fetch("DFT/USD", { source: "chainlink" });
        self.current_price = price_response;
        log::info("token", "Price updated");
    }

    fn get_price() -> int {
        return self.current_price;
    }
```

---

## Step 4: Burn and Mint (Optional)

Use **`if (condition)`** and **`return`** with semicolons.

```dal
    fn mint(to: string, amount: int) {
        self.total_supply = self.total_supply + amount;
        self.balances[to] = self.balances[to] + amount;
        log::info("token", "Minted");
    }

    fn burn(amount: int) -> bool {
        let from = auth::session("user", ["holder"]).user_id;
        if (self.balances[from] < amount) {
            log::info("token", "Insufficient balance to burn");
            return false;
        }
        self.total_supply = self.total_supply - amount;
        self.balances[from] = self.balances[from] - amount;
        log::info("token", "Burned");
        return true;
    }
```

---

## Step 5: Control Flow Reminders

- **If:** Always use parentheses: `if (condition) { }`.
- **Literals:** Use colons: `{ key: value }`.
- **Semicolons:** Required after `let`, `return`, and expression statements.
- **Logging:** Use `log::info("tag", message)` or `log::audit("tag", message)` (no `log::error` in current engine).

---

## Step 6: Compile & Test

```bash
# Parse/compile (project-specific)
cargo run -- DefiToken.dal

# Run tests if you have a test runner
cargo test
```

---

## Step 7: Deploy

Deployment is environment-specific. Use your chain tooling; the service can be instantiated with **`DefiToken::new()`** or **`service::new("DefiToken")`** in DAL.

---

## 🎉 Summary

You used:
- **`@trust("hybrid")`** and **`@chain("ethereum")`** on a service
- **Fields** with types and defaults: `balances: map<string, int> = {}`
- **`fn`** with parameters and **`-> return_type`**
- **`if (condition)`** with parentheses
- **`log::info("tag", message)`**
- **Literals** `{ key: value }` for data

---

## 📚 Next Steps

- [Tutorial 2: AI Trading Agent](02_ai_trading_agent.md)
- [Tutorial 3: Hybrid Marketplace](03_hybrid_marketplace_cloudadmin.md)
- [Syntax Reference](../syntax.md) – full DAL syntax
- [Syntax Reference](../syntax.md) – common pitfalls and required syntax
