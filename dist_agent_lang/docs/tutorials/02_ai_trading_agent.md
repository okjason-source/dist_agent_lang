# Tutorial 2: Building an AI-Powered Trading Agent (v1.0.1)

> **📢 Beta Release v1.0.1:** Uses new AI Simplified Wrapper API. Test thoroughly before production. **Beta testing feedback welcome!** 🙏

Learn to build an intelligent trading agent that uses AI for market analysis and automated trading decisions.

**Time**: 45 minutes  
**Difficulty**: Intermediate  
**Prerequisites**: Basic understanding of smart contracts and trading concepts

> **📝 Note**: This tutorial uses simplified function names for clarity (e.g., `ai::classify()`, `ai::predict()`). The actual implementation uses the agent framework with `ai::spawn_agent()`, `ai::create_task()`, etc. See the [AI Features Guide](../AI_FEATURES_GUIDE.md#-important-notes) for the real API.

---

## 🎯 What You'll Build

An AI-powered trading agent that:
- Analyzes market sentiment using AI
- Makes trading decisions based on ML models
- Manages risk automatically
- Learns from performance
- Operates autonomously 24/7
- Supports multiple trading strategies

---

## 📦 Setup

```bash
mkdir ai-trading-agent && cd ai-trading-agent
touch TradingAgent.dal
```

---

## Step 1: Basic Agent Structure

```dal
// TradingAgent.dal
@contract
@blockchain("ethereum")
@blockchain("polygon")
@version("1.0.0")
@ai  // Enable AI features
@reentrancy_guard
@safe_math
contract AITradingAgent {
    // Agent configuration
    struct AgentConfig {
        string name;
        string strategy;         // "conservative", "moderate", "aggressive"
        uint256 budget;          // Trading budget in wei
        uint256 riskLevel;       // 1-10 scale
        uint256 minTradeAmount;  // Minimum trade size
        uint256 maxTradeAmount;  // Maximum trade size
        bool active;
        uint256 createdAt;
    }
    
    // Agent state
    struct AgentState {
        uint256 totalTrades;
        uint256 successfulTrades;
        uint256 failedTrades;
        uint256 totalProfit;      // Can be negative
        uint256 lastTradeTime;
        uint256 consecutiveLosses;
    }
    
    // Owner to agent mapping
    mapping(address => AgentConfig) public agentConfigs;
    mapping(address => AgentState) public agentStates;
    
    // Events
    event AgentCreated(address indexed owner, string name, string strategy);
    event TradeExecuted(address indexed owner, string action, uint256 amount, uint256 price);
    event StrategyAdjusted(address indexed owner, string oldStrategy, string newStrategy);
    event AgentPaused(address indexed owner);
    event AgentResumed(address indexed owner);
    
    // Constructor
    constructor() {
        // Initialize any global state
    }
}
```

---

## Step 2: Create and Configure Agent

```dal
    @public
    function createAgent(
        string memory name,
        string memory strategy,
        uint256 budget,
        uint256 riskLevel
    ) payable {
        require(msg.value >= budget, "Insufficient funds sent");
        require(riskLevel >= 1 && riskLevel <= 10, "Risk level must be 1-10");
        require(
            string::equals(strategy, "conservative") ||
            string::equals(strategy, "moderate") ||
            string::equals(strategy, "aggressive"),
            "Invalid strategy"
        );
        require(!agentConfigs[msg.sender].active, "Agent already exists");
        
        // Set trade limits based on strategy
        uint256 minTrade = budget / 100;  // 1% of budget
        uint256 maxTrade;
        
        if (string::equals(strategy, "conservative")) {
            maxTrade = budget / 20;  // 5% max per trade
        } else if (string::equals(strategy, "moderate")) {
            maxTrade = budget / 10;  // 10% max per trade
        } else {
            maxTrade = budget / 5;   // 20% max per trade
        }
        
        // Create agent config
        agentConfigs[msg.sender] = AgentConfig({
            name: name,
            strategy: strategy,
            budget: budget,
            riskLevel: riskLevel,
            minTradeAmount: minTrade,
            maxTradeAmount: maxTrade,
            active: true,
            createdAt: block.timestamp
        });
        
        // Initialize agent state
        agentStates[msg.sender] = AgentState({
            totalTrades: 0,
            successfulTrades: 0,
            failedTrades: 0,
            totalProfit: 0,
            lastTradeTime: 0,
            consecutiveLosses: 0
        });
        
        emit AgentCreated(msg.sender, name, strategy);
        
        log::info("AI Trading Agent created: " + name);
    }
    
    @public
    @view
    function getAgentInfo(address owner) -> (AgentConfig, AgentState) {
        return (agentConfigs[owner], agentStates[owner]);
    }
```

---

## Step 3: AI Market Analysis

```dal
    // AI-powered market sentiment analysis
    @public
    @view
    function analyzeMarketSentiment(string memory asset) -> string {
        // Fetch recent news and social media data (simulated)
        let marketData = fetchMarketData(asset);
        
        // Use AI to analyze sentiment
        let sentiment = ai::classify("sentiment_model", marketData);
        // Returns: "bullish", "bearish", or "neutral"
        
        return sentiment;
    }
    
    // AI-powered price prediction
    @public
    @view
    function predictPrice(string memory asset, uint256 timeframe) -> uint256 {
        // Get historical price data
        let historicalPrices = getHistoricalPrices(asset, 30);  // 30 days
        
        // Use AI to predict future price
        let prediction = ai::predict("price_model", 
            json::stringify({
                "asset": asset,
                "prices": historicalPrices,
                "timeframe_hours": timeframe
            })
        );
        
        return prediction;
    }
    
    // AI-powered risk assessment
    @public
    @view
    function assessRisk(
        string memory asset,
        uint256 amount,
        string memory action  // "buy" or "sell"
    ) -> uint256 {
        // Gather market data
        let volatility = calculateVolatility(asset);
        let liquidity = getLiquidity(asset);
        let currentPrice = getCurrentPrice(asset);
        
        // Prepare data for AI model
        let riskData = json::stringify({
            "asset": asset,
            "amount": amount,
            "action": action,
            "volatility": volatility,
            "liquidity": liquidity,
            "current_price": currentPrice,
            "portfolio_exposure": calculateExposure(msg.sender, asset)
        });
        
        // AI risk score (0-100, higher = riskier)
        let riskScore = ai::predict("risk_model", riskData);
        
        return riskScore;
    }
    
    @private
    function fetchMarketData(string memory asset) -> string {
        // Simulated - would fetch from oracle in production
        return "Recent market data for " + asset;
    }
    
    @private
    function getHistoricalPrices(string memory asset, uint256 days) -> array<uint256> {
        // Simulated - would fetch from oracle in production
        array<uint256> prices;
        for (uint256 i = 0; i < days; i++) {
            prices.push(1000 + (i * 10));  // Mock data
        }
        return prices;
    }
```

---

## Step 4: Automated Trading Decision

```dal
    // Main AI decision-making function
    @public
    @async
    async function makeTradeDecision() {
        require(agentConfigs[msg.sender].active, "Agent not active");
        require(
            block.timestamp - agentStates[msg.sender].lastTradeTime >= 300,
            "Trade cooldown (5 minutes)"
        );
        
        AgentConfig memory config = agentConfigs[msg.sender];
        AgentState memory state = agentStates[msg.sender];
        
        // Step 1: Analyze market sentiment
        let sentiment = analyzeMarketSentiment("ETH");
        
        // Step 2: Get price prediction
        let predictedPrice = predictPrice("ETH", 24);  // 24 hour prediction
        let currentPrice = getCurrentPrice("ETH");
        
        // Step 3: Calculate potential profit
        let priceChangePercent = ((predictedPrice - currentPrice) * 100) / currentPrice;
        
        // Step 4: Assess risk
        let tradeAmount = calculateTradeAmount(config, state);
        let riskScore = assessRisk("ETH", tradeAmount, 
            priceChangePercent > 0 ? "buy" : "sell"
        );
        
        // Step 5: Make AI-powered decision
        let decisionData = json::stringify({
            "sentiment": sentiment,
            "current_price": currentPrice,
            "predicted_price": predictedPrice,
            "price_change": priceChangePercent,
            "risk_score": riskScore,
            "strategy": config.strategy,
            "risk_level": config.riskLevel,
            "consecutive_losses": state.consecutiveLosses,
            "success_rate": state.totalTrades > 0 ? 
                (state.successfulTrades * 100) / state.totalTrades : 0
        });
        
        let decision = ai::generate("trading_decision_model",
            "Should the agent trade based on this data? " + decisionData
        );
        
        // Step 6: Execute trade if AI recommends
        await executeAIDecision(decision, tradeAmount, currentPrice);
    }
    
    @private
    async function executeAIDecision(
        string memory decision,
        uint256 amount,
        uint256 price
    ) {
        // Parse AI decision
        let action = parseTradeAction(decision);
        
        if (string::equals(action, "buy")) {
            await executeBuy(amount, price);
        } else if (string::equals(action, "sell")) {
            await executeSell(amount, price);
        } else {
            // Hold - no action
            log::info("AI Decision: HOLD");
            return;
        }
        
        // Update agent state
        agentStates[msg.sender].lastTradeTime = block.timestamp;
        agentStates[msg.sender].totalTrades++;
        
        emit TradeExecuted(msg.sender, action, amount, price);
    }
    
    @private
    function calculateTradeAmount(
        AgentConfig memory config,
        AgentState memory state
    ) -> uint256 {
        // Start with base amount
        uint256 amount = (config.budget * config.riskLevel) / 20;  // Scale by risk
        
        // Adjust based on recent performance
        if (state.consecutiveLosses >= 3) {
            // Reduce trade size after losses
            amount = amount / 2;
        } else if (state.successfulTrades > state.failedTrades && 
                   state.totalTrades > 10) {
            // Increase slightly after consistent wins
            amount = amount * 3 / 2;
        }
        
        // Ensure within limits
        if (amount < config.minTradeAmount) {
            amount = config.minTradeAmount;
        }
        if (amount > config.maxTradeAmount) {
            amount = config.maxTradeAmount;
        }
        
        return amount;
    }
```

---

## Step 5: Trade Execution & Integration

```dal
    @private
    async function executeBuy(uint256 amount, uint256 price) {
        // In production, integrate with DEX
        // For now, simulate trade
        
        log::info("Executing BUY: " + string::from_int(amount) + " at " + 
                  string::from_int(price));
        
        // Simulate trade execution
        bool success = simulateTrade("buy", amount, price);
        
        if (success) {
            agentStates[msg.sender].successfulTrades++;
            agentStates[msg.sender].consecutiveLosses = 0;
        } else {
            agentStates[msg.sender].failedTrades++;
            agentStates[msg.sender].consecutiveLosses++;
        }
    }
    
    @private
    async function executeSell(uint256 amount, uint256 price) {
        log::info("Executing SELL: " + string::from_int(amount) + " at " + 
                  string::from_int(price));
        
        bool success = simulateTrade("sell", amount, price);
        
        if (success) {
            agentStates[msg.sender].successfulTrades++;
            agentStates[msg.sender].consecutiveLosses = 0;
        } else {
            agentStates[msg.sender].failedTrades++;
            agentStates[msg.sender].consecutiveLosses++;
        }
    }
    
    // Integration with DEX (Uniswap example)
    @private
    function executeDEXSwap(
        address tokenIn,
        address tokenOut,
        uint256 amountIn,
        uint256 minAmountOut
    ) -> uint256 {
        // Call Uniswap router
        return defi::swap(tokenIn, tokenOut, amountIn, "uniswap_v3");
    }
```

---

## Step 6: Adaptive Learning & Strategy Adjustment

```dal
    // Agent learns and adjusts strategy based on performance
    @public
    function evaluatePerformance() {
        require(agentConfigs[msg.sender].active, "Agent not active");
        require(agentStates[msg.sender].totalTrades >= 10, "Need at least 10 trades");
        
        AgentState memory state = agentStates[msg.sender];
        AgentConfig memory config = agentConfigs[msg.sender];
        
        // Calculate success rate
        uint256 successRate = (state.successfulTrades * 100) / state.totalTrades;
        
        // Use AI to recommend strategy adjustment
        let performanceData = json::stringify({
            "total_trades": state.totalTrades,
            "success_rate": successRate,
            "consecutive_losses": state.consecutiveLosses,
            "total_profit": state.totalProfit,
            "current_strategy": config.strategy,
            "risk_level": config.riskLevel
        });
        
        let recommendation = ai::generate("strategy_optimizer",
            "Analyze this trading performance and recommend adjustments: " + 
            performanceData
        );
        
        // Parse and apply recommendations
        applyStrategyAdjustments(recommendation);
        
        log::info("Performance evaluated. Recommendations applied.");
    }
    
    @private
    function applyStrategyAdjustments(string memory recommendation) {
        // Parse AI recommendations
        if (string::contains(recommendation, "reduce_risk")) {
            // Lower risk level
            if (agentConfigs[msg.sender].riskLevel > 1) {
                agentConfigs[msg.sender].riskLevel--;
            }
        } else if (string::contains(recommendation, "increase_risk")) {
            // Increase risk level (if performing well)
            if (agentConfigs[msg.sender].riskLevel < 10 && 
                agentStates[msg.sender].successfulTrades > 
                agentStates[msg.sender].failedTrades) {
                agentConfigs[msg.sender].riskLevel++;
            }
        }
        
        if (string::contains(recommendation, "change_strategy")) {
            string memory oldStrategy = agentConfigs[msg.sender].strategy;
            
            // AI suggests strategy change
            if (string::contains(recommendation, "conservative")) {
                agentConfigs[msg.sender].strategy = "conservative";
                agentConfigs[msg.sender].maxTradeAmount = 
                    agentConfigs[msg.sender].budget / 20;
            } else if (string::contains(recommendation, "moderate")) {
                agentConfigs[msg.sender].strategy = "moderate";
                agentConfigs[msg.sender].maxTradeAmount = 
                    agentConfigs[msg.sender].budget / 10;
            }
            
            emit StrategyAdjusted(msg.sender, oldStrategy, 
                                 agentConfigs[msg.sender].strategy);
        }
    }
```

---

## Step 7: Control & Safety Features

```dal
    // Pause agent (emergency stop)
    @public
    function pauseAgent() {
        require(agentConfigs[msg.sender].active, "Agent not active");
        
        agentConfigs[msg.sender].active = false;
        emit AgentPaused(msg.sender);
        
        log::info("Agent paused by owner");
    }
    
    // Resume agent
    @public
    function resumeAgent() {
        require(!agentConfigs[msg.sender].active, "Agent already active");
        
        agentConfigs[msg.sender].active = true;
        
        // Reset consecutive losses on resume
        agentStates[msg.sender].consecutiveLosses = 0;
        
        emit AgentResumed(msg.sender);
        
        log::info("Agent resumed by owner");
    }
    
    // Withdraw funds
    @public
    function withdrawFunds(uint256 amount) {
        require(amount <= agentConfigs[msg.sender].budget, "Insufficient balance");
        
        agentConfigs[msg.sender].budget -= amount;
        payable(msg.sender).transfer(amount);
        
        log::info("Funds withdrawn: " + string::from_int(amount));
    }
    
    // Add funds
    @public
    payable
    function addFunds() {
        require(msg.value > 0, "Must send funds");
        
        agentConfigs[msg.sender].budget += msg.value;
        
        log::info("Funds added: " + string::from_int(msg.value));
    }
    
    // Update risk level
    @public
    function updateRiskLevel(uint256 newRiskLevel) {
        require(newRiskLevel >= 1 && newRiskLevel <= 10, "Invalid risk level");
        
        agentConfigs[msg.sender].riskLevel = newRiskLevel;
        
        log::info("Risk level updated to: " + string::from_int(newRiskLevel));
    }
    
    // Helper functions
    @private
    function parseTradeAction(string memory decision) -> string {
        if (string::contains(decision, "buy") || string::contains(decision, "BUY")) {
            return "buy";
        } else if (string::contains(decision, "sell") || string::contains(decision, "SELL")) {
            return "sell";
        } else {
            return "hold";
        }
    }
    
    @private
    function getCurrentPrice(string memory asset) -> uint256 {
        // Fetch from oracle
        return oracle::fetch("chainlink", asset + "/USD").data;
    }
    
    @private
    function calculateVolatility(string memory asset) -> uint256 {
        // Calculate from historical prices
        return 15;  // Mock: 15% volatility
    }
    
    @private
    function getLiquidity(string memory asset) -> uint256 {
        // Fetch from DEX
        return 1000000;  // Mock liquidity
    }
    
    @private
    function calculateExposure(address owner, string memory asset) -> uint256 {
        // Calculate portfolio exposure
        return 25;  // Mock: 25% exposure
    }
    
    @private
    function simulateTrade(
        string memory action,
        uint256 amount,
        uint256 price
    ) -> bool {
        // Simulate with 70% success rate
        uint256 random = uint256(keccak256(abi.encodePacked(
            block.timestamp, action, amount
        ))) % 100;
        
        return random < 70;
    }
}
```

---

## Step 8: Testing

Create `tests/ai_agent_tests.dal`:

```dal
@test_suite("AI Trading Agent Tests")
suite AITradingAgentTests {
    @test("Should create agent with correct config")
    async function testAgentCreation() {
        let agent = await deploy("AITradingAgent", []);
        
        await agent.createAgent(
            "Test Agent",
            "moderate",
            1000000,  // 1M wei budget
            5         // Risk level 5
        );
        
        let (config, state) = await agent.getAgentInfo(deployer);
        
        assert_eq(config.name, "Test Agent");
        assert_eq(config.strategy, "moderate");
        assert_eq(config.riskLevel, 5);
    }
    
    @test("Should analyze market sentiment")
    async function testSentimentAnalysis() {
        let agent = await deploy("AITradingAgent", []);
        
        await agent.createAgent("Test", "moderate", 1000000, 5);
        
        let sentiment = await agent.analyzeMarketSentiment("ETH");
        
        assert_true(
            sentiment == "bullish" || 
            sentiment == "bearish" || 
            sentiment == "neutral"
        );
    }
    
    @test("Should make trading decision")
    async function testTradeDecision() {
        let agent = await deploy("AITradingAgent", []);
        
        await agent.createAgent("Test", "moderate", 1000000, 5);
        
        await agent.makeTradeDecision();
        
        let (config, state) = await agent.getAgentInfo(deployer);
        
        assert_gt(state.totalTrades, 0);
    }
    
    @test("Should pause and resume agent")
    async function testPauseResume() {
        let agent = await deploy("AITradingAgent", []);
        
        await agent.createAgent("Test", "moderate", 1000000, 5);
        await agent.pauseAgent();
        
        let (config, _) = await agent.getAgentInfo(deployer);
        assert_false(config.active);
        
        await agent.resumeAgent();
        
        (config, _) = await agent.getAgentInfo(deployer);
        assert_true(config.active);
    }
}
```

---

## Step 9: Compile & Deploy

```bash
# Compile
dal compile TradingAgent.dal

# Test
dal test TradingAgent.dal

# Deploy to testnet
dal deploy TradingAgent.dal --network sepolia

# Deploy to mainnet (multi-chain)
dal deploy TradingAgent.dal --networks ethereum,polygon,arbitrum --verify
```

---

## Step 10: Interact with Your Agent

```javascript
// JavaScript example
const agent = await contract.connect(wallet);

// Create agent
await agent.createAgent(
  "My AI Trader",
  "moderate",
  ethers.parseEther("1.0"),  // 1 ETH budget
  6                          // Risk level 6
);

// Make trading decision
await agent.makeTradeDecision();

// Check performance
const [config, state] = await agent.getAgentInfo(wallet.address);
console.log("Total trades:", state.totalTrades);
console.log("Success rate:", (state.successfulTrades * 100) / state.totalTrades, "%");

// Evaluate and adjust
if (state.totalTrades >= 10) {
  await agent.evaluatePerformance();
}
```

---

## 🎉 Congratulations!

You've built an AI-powered trading agent with:
- ✅ AI market sentiment analysis
- ✅ ML-based price predictions
- ✅ Automated risk assessment
- ✅ Autonomous trading decisions
- ✅ Adaptive learning & strategy adjustment
- ✅ Multi-chain support
- ✅ Built-in safety features

---

## 📚 Next Steps

1. **Integrate with real DEX**: Connect to Uniswap/SushiSwap
2. **Add more AI models**: Technical analysis, volume prediction
3. **Implement portfolio management**: Multiple assets
4. **Add notifications**: Alert on trades via Discord/Telegram
5. **Build dashboard**: Visualize agent performance

---

## 💡 Pro Tips

1. **Start conservative**: Use low risk levels initially
2. **Monitor closely**: Check agent decisions regularly
3. **Test thoroughly**: Use testnet extensively
4. **Set limits**: Always have stop-loss mechanisms
5. **Diversify strategies**: Run multiple agents with different strategies

---

## 📖 Related Resources

- [AI Features Guide](../AI_FEATURES_GUIDE.md) - Complete AI capabilities
- [DeFi Integration](01_defi_token.md) - DeFi concepts
- [Best Practices](../BEST_PRACTICES.md) - Security & optimization
- [API Reference](../API_REFERENCE.md) - Complete API docs

---

**Full code**: [examples/ai_trading_agent/](../../examples/ai_trading_agent/)

**Next Tutorial**: [AI + Blockchain Integration →](03_ai_blockchain_integration.md)

