# Tutorial 2: Building an AI-Powered Trading Agent

> **📢 Beta:** Uses DAL service and agent APIs. Test before production.

Learn to build an AI trading agent in DAL using **services**, **agents**, and the **`ai::`** namespace.

**Time**: 45 minutes  
**Difficulty**: Intermediate  
**Prerequisites**: Basic DAL (services, `fn`, literals)

> **📝 Note:** DAL uses **`ai::create_agent(config)`** and **`ai::create_agent_coordinator()`** for agents. Config is a **literal** `{ role: "...", capabilities: [...] }`. See [Syntax Reference](../syntax.md) and [API Reference](../api_reference.md).

---

## 🎯 What You'll Build

An AI-powered trading agent that:
- Uses a **service** for state and methods
- Creates agents via **`ai::create_agent(config)`**
- Uses **literals** for config and data
- Uses **`if (condition)`** and **`log::info`** (parser-accurate)

---

## 📦 Setup

```bash
mkdir ai-trading-agent && cd ai-trading-agent
touch TradingAgent.dal
```

---

## Step 1: Basic Service Structure

Use **`@trust`** and **`@chain`** with a **service**. Store agent id and config in fields (maps/literals).

```dal
// TradingAgent.dal
@trust("hybrid")
@chain("ethereum")
service AITradingAgent {
    agent_id: string = "";
    total_trades: int = 0;
    successful_trades: int = 0;

    fn create_agent(name: string, strategy: string, budget: int, risk_level: int) {
        let config = {
            role: "trader",
            capabilities: ["analysis", "reasoning"]
        };
        let agent_instance = ai::create_agent(config);
        self.agent_id = agent_instance;
        log::info("agent", "AI Trading Agent created");
    }

    fn get_agent_id() -> string {
        return self.agent_id;
    }
}
```

---

## Step 2: Market Analysis (Simplified)

Use **`ai::`** functions as in the [API Reference](../api_reference.md). Pass **literals** for options.

```dal
    fn analyze_sentiment(asset: string) -> string {
        let market_data = { asset: asset };
        let sentiment = ai::analyze_text("sentiment for " + asset);
        return sentiment;
    }

    fn predict_price(asset: string, timeframe: int) -> int {
        let data = {
            asset: asset,
            timeframe_hours: timeframe
        };
        let prediction = ai::generate_text("price prediction");
        return 1000;
    }
```

---

## Step 3: Trading Decision Flow

Use **`if (condition)`** with parentheses. Use **`log::info("tag", message)`** (two arguments).

```dal
    fn make_trade_decision() -> string {
        if (self.agent_id == "") {
            log::info("agent", "Create agent first");
            return "error";
        }
        let sentiment = self.analyze_sentiment("ETH");
        if (sentiment == "bullish") {
            log::info("agent", "Decision: BUY");
            return "buy";
        } else if (sentiment == "bearish") {
            log::info("agent", "Decision: SELL");
            return "sell";
        } else {
            log::info("agent", "Decision: HOLD");
            return "hold";
        }
    }
```

---

## Step 4: Coordinator and Workflow (Optional)

Use **`ai::create_agent_coordinator()`** and **`ai::create_workflow`** as in the API.

```dal
    fn setup_coordinator() -> string {
        let coordinator_id = ai::create_agent_coordinator();
        log::info("agent", "Coordinator created");
        return coordinator_id;
    }
```

---

## Step 5: Control Flow and Literals

- **If:** Always `if (condition) { }` with parentheses.
- **Literals:** `{ key: value }` with colon; keys can be identifier or string.
- **Agents:** Config is a literal: `ai::create_agent({ role: "...", capabilities: [...] })`.
- **Logging:** `log::info("tag", message)` or `log::audit("tag", message)`.

---

## Step 6: Testing

Create a test file that parses: use **service**, **fn**, **let**, **if (condition)**.

```dal
// Create instance and call methods
let token = AITradingAgent::new();
token.create_agent("Trader1", "moderate", 1000000, 5);
let decision = token.make_trade_decision();
```

---

## Step 7: Compile & Run

```bash
cargo run -- TradingAgent.dal
cargo test
```

---

## 🎉 Summary

You used:
- **Service** with **@trust** and **@chain**
- **`ai::create_agent(config)`** with a **literal** config
- **`ai::create_agent_coordinator()`**
- **`if (condition)`** with parentheses
- **`log::info("tag", message)`**
- **Literals** `{ key: value }` for config and data

---

## 📚 Next Steps

- [Tutorial 3: Hybrid Marketplace](03_hybrid_marketplace_cloudadmin.md)
- [Syntax Reference](../syntax.md)
- [Syntax Reference](../syntax.md)
- [API Reference](../api_reference.md)
