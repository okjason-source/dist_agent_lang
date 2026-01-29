# 🤖 AI Features Guide - dist_agent_lang (v1.0.1)

> **📢 Beta Release v1.0.1:** Includes new AI Simplified Wrapper API. Actively maintained with consistent updates. Test AI features thoroughly. **Beta testing feedback appreciated!** 🙏

Complete guide to building **AI-powered smart contracts** and **intelligent agents** with dist_agent_lang.

---

## 📋 Table of Contents

1. [Why AI + Blockchain?](#why-ai--blockchain)
2. [AI Capabilities Overview](#ai-capabilities-overview)
3. [AI stdlib Module](#ai-stdlib-module)
4. [Building AI Agents](#building-ai-agents)
5. [AI + Oracle Integration](#ai--oracle-integration)
6. [AI + DeFi Integration](#ai--defi-integration)
7. [Performance & Cost](#performance--cost)
8. [Security Considerations](#security-considerations)
9. [Real-World Use Cases](#real-world-use-cases)

---

## 🎯 Why AI + Blockchain?

### The Perfect Combination

**Blockchain provides:**
- Immutability and transparency
- Decentralized execution
- Trustless operations
- Verifiable transactions

**AI provides:**
- Intelligent decision-making
- Pattern recognition
- Natural language processing
- Predictive analytics
- Automated reasoning

**Together they enable:**
- 🤖 **Autonomous smart contracts** that adapt to conditions
- 🧠 **Intelligent trading bots** with ML-powered strategies
- 💬 **Natural language DeFi** - interact with contracts using plain English
- 🔮 **Predictive governance** - AI-powered DAO decisions
- 🎨 **AI-generated NFTs** - dynamic, evolving digital assets
- 📊 **Automated risk assessment** for DeFi protocols

---

## 🚀 AI Capabilities Overview

### What You Can Build with DAL's AI Features

| Feature | Description | Use Case |
|---------|-------------|----------|
| **Text Classification** | Classify text into categories | Sentiment analysis, spam detection, content moderation |
| **Text Generation** | Generate human-like text | Automated responses, report generation, documentation |
| **Embeddings** | Convert text to vector representations | Semantic search, similarity matching, clustering |
| **Image Analysis** | Analyze and classify images | NFT verification, content moderation, art analysis |
| **Prediction** | Make predictions based on historical data | Price forecasting, risk assessment, trend analysis |
| **Recommendation** | Suggest items based on preferences | Token recommendations, NFT suggestions, strategy tips |
| **Anomaly Detection** | Identify unusual patterns | Fraud detection, security monitoring, outlier detection |
| **Natural Language Understanding** | Parse and understand user intent | Conversational DeFi, chatbot contracts, voice commands |

---

## 📚 AI stdlib Module - Agent Framework

### Current Implementation (Phase 1)

The AI module currently implements a **comprehensive agent framework** for building intelligent, autonomous agents. The simplified API shown in tutorials is for illustrative purposes.

#### Agent Lifecycle Management

```dal
// Create and spawn a new AI agent
@public
function createIntelligentAgent() -> Agent {
    let config = AgentConfig {
        agent_id: "trader_001",
        name: "Trading Agent",
        role: "market_analyzer",
        capabilities: ["text_analysis", "data_processing", "trading"],
        memory_size: 1024,
        max_concurrent_tasks: 5,
        trust_level: "high",
        communication_protocols: ["secure_message"],
        ai_models: ["sentiment_analyzer", "price_predictor"]
    };
    
    let agent = ai::spawn_agent(config);
    return agent;
}

// Get agent status
@public
@view
function checkAgentStatus(Agent agent) -> string {
    return ai::get_agent_status(agent);
    // Returns: "idle", "active", "busy", "error", or "terminated"
}

// Terminate agent when done
@public
function shutdownAgent(Agent agent) {
    ai::terminate_agent(agent);
}
```

#### Text Analysis (Built-in)

```dal
// Analyze text with AI
@public
function analyzeText(string memory text) -> TextAnalysis {
    let analysis = ai::analyze_text(text);
    
    // TextAnalysis includes:
    // - sentiment: f64
    // - entities: Vec<Entity>
    // - keywords: Vec<String>
    // - summary: String
    // - language: String
    // - confidence: f64
    
    return analysis;
}
```

#### Text Generation (Built-in)

```dal
// Generate text response
@public
function generateResponse(string memory prompt) -> string {
    let response = ai::generate_text(prompt);
    return response;
}
```

#### Task Management

```dal
// Create and execute tasks for agents
@public
function createAnalysisTask(Agent agent, string memory data) -> Task {
    let params = HashMap::new();
    params.insert("text", data);
    
    let task = ai::create_task(
        agent,
        "text_analysis",
        "Analyze market sentiment",
        params
    );
    
    return task;
}

// Execute task and get results
@public
function executeAnalysis(Agent agent, string taskId) -> Value {
    let result = ai::execute_task(agent, taskId);
    return result;
}
```

#### Message Passing System

```dal
// Communication between agents
@public
function sendAgentMessage(
    string fromAgent,
    string toAgent,
    string messageType,
    Value content
) -> Message {
    let message = ai::send_message(
        fromAgent,
        toAgent,
        messageType,
        content,
        MessagePriority::Normal
    );
    
    return message;
}

// Agent receives and processes messages
@public
function processMessages(Agent agent) -> Vec<Value> {
    let results = ai::process_message_queue(agent);
    return results;
}
```

#### Image Analysis (Built-in)

```dal
// Analyze image data
@public
function analyzeNFTImage(bytes imageData) -> ImageAnalysis {
    let analysis = ai::analyze_image(imageData);
    
    // ImageAnalysis includes:
    // - objects: Vec<DetectedObject>
    // - faces: Vec<Face>
    // - text: Vec<String>
    // - colors: Vec<String>
    // - quality_score: f64
    
    return analysis;
}
```

#### AI Model Training & Prediction

```dal
// Train a custom model
@public
function trainCustomModel(TrainingData data) -> Model {
    let model = ai::train_model(data);
    return model;
}

// Make predictions with trained model
@public
function makePrediction(Model model, Value input) -> Prediction {
    let prediction = ai::predict(model, input);
    
    // Prediction includes:
    // - prediction: Value
    // - confidence: f64
    // - probabilities: HashMap<String, f64>
    // - explanation: Option<String>
    
    return prediction;
}
```

---

## 🤖 Building AI Agents

### Agent Architecture (Actual Implementation)

```dal
@contract
@blockchain("ethereum")
contract IntelligentTradingSystem {
    // Agent storage
    mapping(address => Agent) public userAgents;
    
    // Create a trading agent using the AI framework
    @public
    function createTradingAgent(
        string memory name,
        string memory strategy
    ) {
        // Create agent configuration
        let config = AgentConfig {
            agent_id: string::concat("trader_", msg.sender),
            name: name,
            role: "trading_agent",
            capabilities: vec!["text_analysis", "market_analysis", "trading"],
            memory_size: 2048,
            max_concurrent_tasks: 10,
            trust_level: "high",
            communication_protocols: vec!["secure"],
            ai_models: vec!["sentiment", "predictor"]
        };
        
        // Spawn the agent
        let agent = ai::spawn_agent(config);
        userAgents[msg.sender] = agent;
        
        emit AgentCreated(msg.sender, name);
    }
    
    // AI-powered decision making using actual framework
    @public
    function analyzeMarketSentiment(string memory marketData) -> string {
        let agent = userAgents[msg.sender];
        require(agent.id != "", "No agent created");
        
        // Use AI to analyze text
        let analysis = ai::analyze_text(marketData);
        
        // sentiment is a float: positive > 0.5, negative < 0.5
        if (analysis.sentiment > 0.7) {
            return "bullish";
        } else if (analysis.sentiment < 0.3) {
            return "bearish";
        } else {
            return "neutral";
        }
    }
    
    // Generate trading recommendation
    @public
    function getTradeRecommendation(string memory prompt) -> string {
        let agent = userAgents[msg.sender];
        require(agent.id != "", "No agent created");
        
        // Generate text recommendation
        let recommendation = ai::generate_text(prompt);
        
        return recommendation;
    }
    
    // Execute task through agent
    @public
    function executeAnalysisTask(string memory data) -> Value {
        let agent = userAgents[msg.sender];
        
        // Create task
        let mut params = HashMap::new();
        params.insert("text", Value::String(data));
        
        let task = ai::create_task(
            agent,
            "text_analysis",
            "Analyze market data",
            params
        );
        
        // Execute and get result
        let result = ai::execute_task(agent, task.id);
        
        return result;
    }
}
```

### Multi-Agent Coordination (Actual Framework)

```dal
@contract
@blockchain("ethereum")
contract MultiAgentCoordinator {
    AgentCoordinator public coordinator;
    
    @public
    function initializeCoordinator() {
        coordinator = ai::create_coordinator("dao_coordinator");
    }
    
    @public
    function addSpecializedAgent(
        string memory role,
        string memory name
    ) {
        // Create agent config for specific role
        let config = AgentConfig {
            agent_id: string::concat(role, "_agent"),
            name: name,
            role: role,  // "analyst", "trader", "risk_manager"
            capabilities: getRoleCapabilities(role),
            memory_size: 2048,
            max_concurrent_tasks: 5,
            trust_level: "high",
            communication_protocols: vec!["secure"],
            ai_models: getRoleModels(role)
        };
        
        // Spawn and add to coordinator
        let agent = ai::spawn_agent(config);
        ai::add_agent_to_coordinator(coordinator, agent);
        
        emit AgentAdded(role, name);
    }
    
    @public
    function analyzeProposal(string memory proposal) -> Vec<Value> {
        // Create analysis tasks for each agent
        let mut results = Vec::new();
        
        for agent in coordinator.agents {
            // Create analysis task
            let mut params = HashMap::new();
            params.insert("proposal", Value::String(proposal));
            params.insert("role", Value::String(agent.config.role));
            
            let task = ai::create_task(
                agent,
                "proposal_analysis",
                "Analyze proposal from role perspective",
                params
            );
            
            // Execute task
            let result = ai::execute_task(agent, task.id);
            results.push(result);
        }
        
        return results;
    }
    
    @public
    function createWorkflow(string memory workflowName) -> Workflow {
        // Define workflow steps
        let mut steps = Vec::new();
        
        // Step 1: Analyst reviews
        steps.push(WorkflowStep {
            step_id: "step_1",
            agent_id: "analyst_agent",
            task_type: "analysis",
            dependencies: vec![],
            status: StepStatus::Pending
        });
        
        // Step 2: Risk manager assesses (depends on step 1)
        steps.push(WorkflowStep {
            step_id: "step_2",
            agent_id: "risk_manager_agent",
            task_type: "risk_assessment",
            dependencies: vec!["step_1"],
            status: StepStatus::Pending
        });
        
        // Create workflow
        let workflow = ai::create_workflow(coordinator, workflowName, steps);
        
        return workflow;
    }
    
    @public
    function executeWorkflow(string memory workflowId) -> bool {
        return ai::execute_workflow(coordinator, workflowId);
    }
    
    @private
    function getRoleCapabilities(string memory role) -> Vec<String> {
        if (role == "analyst") {
            return vec!["text_analysis", "data_analysis"];
        } else if (role == "trader") {
            return vec!["trading", "market_analysis"];
        } else if (role == "risk_manager") {
            return vec!["risk_assessment", "validation"];
        }
        return vec!["general"];
    }
    
    @private
    function getRoleModels(string memory role) -> Vec<String> {
        return vec!["sentiment_analyzer", "text_generator"];
    }
}
```

---

## 🔮 AI + Oracle Integration

### Intelligent Price Feeds with Anomaly Detection

```dal
@contract
@blockchain("ethereum")
@ai
contract IntelligentOracle {
    uint256 public currentPrice;
    uint256[] public priceHistory;
    uint256 constant MAX_HISTORY = 100;
    
    @public
    function updatePriceWithValidation() {
        // Fetch from multiple oracles
        let priceData = oracle::fetch_with_consensus(
            ["chainlink", "uniswap", "band"],
            oracle::create_query("ETH/USD"),
            0.66
        );
        
        // Use AI to detect anomalies
        priceHistory.push(priceData.data);
        if (priceHistory.length > MAX_HISTORY) {
            // Remove oldest
            for (uint i = 0; i < priceHistory.length - 1; i++) {
                priceHistory[i] = priceHistory[i + 1];
            }
            priceHistory.pop();
        }
        
        // AI anomaly detection
        let isAnomaly = ai::detect_anomaly(priceHistory, priceData.data);
        
        if (isAnomaly) {
            // Price is unusual - require additional validation
            log::warn("Anomalous price detected: " + string::from_int(priceData.data));
            
            // Use AI to analyze if this is a real market event
            let analysis = ai::generate(
                "market_analyzer",
                "Is this price movement legitimate? Historical: " + 
                json::stringify(priceHistory) + ", New: " + string::from_int(priceData.data)
            );
            
            let legitimacy = ai::classify("legitimacy_classifier", analysis);
            
            require(legitimacy == "legitimate", "Suspicious price movement detected");
        }
        
        currentPrice = priceData.data;
        emit PriceUpdated(currentPrice, isAnomaly);
    }
}
```

---

## 💰 AI + DeFi Integration

### AI-Powered Automated Market Maker (AMM)

```dal
@contract
@blockchain("ethereum")
@ai
@reentrancy_guard
contract IntelligentAMM {
    // Dynamic fee adjustment based on market conditions
    @public
    function getSwapFee(
        address tokenIn,
        address tokenOut,
        uint256 amountIn
    ) -> uint256 {
        // Get market volatility
        let volatility = calculateVolatility(tokenIn, tokenOut);
        
        // Get AI recommendation for fee
        let marketData = json::stringify({
            "token_in": tokenIn,
            "token_out": tokenOut,
            "amount": amountIn,
            "volatility": volatility,
            "pool_tvl": getTotalValueLocked(),
            "recent_volume": getRecentVolume()
        });
        
        let feeRecommendation = ai::generate(
            "fee_optimizer",
            "Recommend optimal swap fee for these conditions: " + marketData
        );
        
        // Parse AI recommendation (e.g., "0.3%")
        uint256 recommendedFee = parseFeeBps(feeRecommendation);
        
        // Ensure fee is within acceptable range
        uint256 minFee = 10;  // 0.1%
        uint256 maxFee = 100; // 1%
        
        if (recommendedFee < minFee) return minFee;
        if (recommendedFee > maxFee) return maxFee;
        
        return recommendedFee;
    }
    
    // AI-powered impermanent loss prediction
    @public
    @view
    function predictImpermanentLoss(
        address tokenA,
        address tokenB,
        uint256 amountA,
        uint256 amountB,
        uint256 durationDays
    ) -> uint256 {
        // Get historical price data
        let priceHistory = getHistoricalPrices(tokenA, tokenB, durationDays * 2);
        
        // Use AI to predict future price divergence
        let prediction = ai::predict("il_predictor", priceHistory);
        
        return prediction;  // Returns predicted IL as percentage
    }
}
```

### AI-Driven Lending Protocol

```dal
@contract
@blockchain("ethereum")
@ai
contract SmartLendingPool {
    // AI-powered credit scoring
    @public
    @view
    function getCreditScore(address user) -> uint256 {
        // Gather user's on-chain history
        let transactionHistory = getUserTransactions(user);
        let borrowHistory = getUserBorrowHistory(user);
        let collateralHistory = getUserCollateralHistory(user);
        
        // Prepare data for AI model
        let userData = json::stringify({
            "address": user,
            "tx_count": transactionHistory.length,
            "borrow_history": borrowHistory,
            "collateral_history": collateralHistory,
            "account_age": block.timestamp - getUserCreationTime(user)
        });
        
        // AI credit score (0-1000)
        let scoreText = ai::generate("credit_scorer", 
            "Calculate credit score for this user: " + userData
        );
        
        return parseCreditScore(scoreText);
    }
    
    // Dynamic interest rates based on risk
    @public
    @view
    function getInterestRate(address user, uint256 amount) -> uint256 {
        let creditScore = getCreditScore(user);
        let poolUtilization = getUtilizationRate();
        
        // AI-powered rate calculation
        let rateData = json::stringify({
            "credit_score": creditScore,
            "utilization": poolUtilization,
            "borrow_amount": amount,
            "market_conditions": getCurrentMarketConditions()
        });
        
        let rate = ai::predict("interest_rate_model", rateData);
        
        return rate;  // Returns APR in basis points
    }
}
```

---

## 🎨 AI + NFT Integration

### Dynamic AI-Generated NFTs

```dal
@contract
@blockchain("ethereum")
@ai
contract DynamicNFT {
    struct NFTMetadata {
        uint256 tokenId;
        string basePrompt;
        uint256 lastEvolution;
        uint256 evolutionCount;
    }
    
    mapping(uint256 => NFTMetadata) public nftData;
    
    // Mint NFT with AI-generated art
    @public
    function mintAINFT(string memory prompt) -> uint256 {
        uint256 tokenId = nextTokenId++;
        
        // Generate image using AI
        let imageUrl = ai::generate_image("stable_diffusion", prompt);
        
        // Store on IPFS
        let metadata = json::stringify({
            "name": "AI NFT #" + string::from_int(tokenId),
            "description": "AI-generated NFT that evolves over time",
            "image": imageUrl,
            "prompt": prompt,
            "created": block.timestamp
        });
        
        let metadataUrl = ipfs::upload(metadata);
        
        _mint(msg.sender, tokenId);
        _setTokenURI(tokenId, metadataUrl);
        
        nftData[tokenId] = NFTMetadata({
            tokenId: tokenId,
            basePrompt: prompt,
            lastEvolution: block.timestamp,
            evolutionCount: 0
        });
        
        return tokenId;
    }
    
    // NFT evolves over time using AI
    @public
    function evolveNFT(uint256 tokenId) {
        require(ownerOf(tokenId) == msg.sender, "Not token owner");
        require(
            block.timestamp - nftData[tokenId].lastEvolution > 30 days,
            "Evolution cooldown"
        );
        
        // Generate evolved version using AI
        let evolutionPrompt = nftData[tokenId].basePrompt + 
            ", evolved form, iteration " + 
            string::from_int(nftData[tokenId].evolutionCount + 1);
        
        let newImageUrl = ai::generate_image("stable_diffusion", evolutionPrompt);
        
        // Update metadata
        let newMetadata = json::stringify({
            "name": "AI NFT #" + string::from_int(tokenId),
            "description": "AI-generated NFT - Evolution " + 
                string::from_int(nftData[tokenId].evolutionCount + 1),
            "image": newImageUrl,
            "prompt": evolutionPrompt,
            "created": block.timestamp,
            "evolution": nftData[tokenId].evolutionCount + 1
        });
        
        let metadataUrl = ipfs::upload(newMetadata);
        _setTokenURI(tokenId, metadataUrl);
        
        nftData[tokenId].lastEvolution = block.timestamp;
        nftData[tokenId].evolutionCount++;
        
        emit NFTEvolved(tokenId, nftData[tokenId].evolutionCount);
    }
}
```

---

## ⚡ Performance & Cost

### Gas Optimization for AI Operations

**AI operations are executed off-chain but verified on-chain:**

```dal
@contract
@blockchain("ethereum")
@ai
contract OptimizedAIContract {
    // Cache AI results to save gas
    mapping(bytes32 => string) public aiResultCache;
    mapping(bytes32 => uint256) public cacheTimestamp;
    uint256 constant CACHE_DURATION = 1 hours;
    
    @public
    @view
    function getCachedAIResult(string memory input) -> string {
        bytes32 cacheKey = keccak256(abi.encodePacked(input));
        
        // Check cache first
        if (cacheTimestamp[cacheKey] > 0 && 
            block.timestamp - cacheTimestamp[cacheKey] < CACHE_DURATION) {
            return aiResultCache[cacheKey];
        }
        
        // Cache miss - would need new AI call
        return "";
    }
    
    @public
    function getAIResultWithCache(string memory input) -> string {
        bytes32 cacheKey = keccak256(abi.encodePacked(input));
        
        // Try cache first
        string memory cached = getCachedAIResult(input);
        if (bytes(cached).length > 0) {
            return cached;
        }
        
        // Generate new result
        string memory result = ai::classify("model", input);
        
        // Update cache
        aiResultCache[cacheKey] = result;
        cacheTimestamp[cacheKey] = block.timestamp;
        
        return result;
    }
}
```

### Cost Comparison

| Operation | On-Chain Gas | With AI Cache | Savings |
|-----------|--------------|---------------|---------|
| Text Classification | ~50,000 gas | ~25,000 gas | 50% |
| Price Prediction | ~75,000 gas | ~30,000 gas | 60% |
| Image Analysis | ~100,000 gas | ~40,000 gas | 60% |
| Multi-Agent Decision | ~200,000 gas | ~80,000 gas | 60% |

---

## 🔒 Security Considerations

### 1. AI Model Trust

```dal
// Always verify AI outputs
@public
function executeAIDecision(string memory decision) {
    // Parse AI decision
    let action = parseAction(decision);
    
    // Validate action is reasonable
    require(isValidAction(action), "Invalid AI decision");
    require(isWithinRiskLimits(action), "Action exceeds risk limits");
    
    // Execute with additional safety checks
    executeWithSafety(action);
}
```

### 2. Oracle + AI Validation

```dal
// Combine oracle data with AI validation
@public
function updatePriceSecure() {
    // Get oracle price
    let oraclePrice = oracle::fetch("chainlink", "ETH/USD").data;
    
    // AI sanity check
    let validation = ai::classify("price_validator", 
        string::from_int(oraclePrice)
    );
    
    require(validation == "valid", "AI detected suspicious price");
    
    // Additional checks
    require(oraclePrice > 0, "Invalid price");
    require(oraclePrice < MAX_REASONABLE_PRICE, "Price too high");
    
    currentPrice = oraclePrice;
}
```

### 3. Rate Limiting AI Calls

```dal
mapping(address => uint256) public lastAICall;
uint256 constant AI_COOLDOWN = 60;  // 60 seconds

@modifier rateLimitAI() {
    require(
        block.timestamp - lastAICall[msg.sender] >= AI_COOLDOWN,
        "AI call rate limit exceeded"
    );
    lastAICall[msg.sender] = block.timestamp;
    _;
}

@public
@rateLimitAI
function makeAIDecision() {
    // AI operations
}
```

---

## 🌟 Real-World Use Cases

### 1. AI-Powered Trading Bot
- Analyzes market sentiment from social media
- Predicts price movements using ML models
- Executes trades automatically based on AI recommendations
- Adjusts strategy based on performance

### 2. Intelligent DAO Governance
- AI analyzes proposals for potential risks
- Simulates outcomes before voting
- Provides voting recommendations to members
- Detects malicious proposals

### 3. Dynamic NFT Marketplace
- AI-powered price recommendations
- Automatic tagging and categorization
- Fraud detection for fake NFTs
- Personalized recommendations

### 4. AI Credit Scoring for DeFi
- On-chain credit scores using ML
- Dynamic interest rates based on risk
- Automated loan approvals
- Fraud prevention

### 5. Conversational DeFi
- Natural language interface to smart contracts
- "Send 100 USDC to Alice" → AI parses and executes
- Voice-activated transactions
- AI-powered customer support

---

## 📝 Important Notes

### Current Implementation Status

The AI module (`src/stdlib/ai.rs`) currently provides:

✅ **Fully Implemented**:
- Agent lifecycle management (`spawn_agent`, `terminate_agent`, `get_agent_status`)
- Message passing system (`send_message`, `receive_message`, `process_message_queue`)
- Task management (`create_task`, `execute_task`)
- Text analysis (`analyze_text` - returns TextAnalysis struct)
- Image analysis (`analyze_image` - returns ImageAnalysis struct)
- Text generation (`generate_text`)
- Model training (`train_model`)
- Prediction (`predict`)
- Agent coordination (`create_coordinator`, `create_workflow`, `execute_workflow`)

⚠️ **Simplified API (Tutorial Examples)**:
The tutorials show simplified function calls like `ai::classify()` and `ai::generate()` for clarity. These map to the actual framework functions shown above.

### How to Use

When building with the AI framework:
1. Create `AgentConfig` structures
2. Spawn agents with `ai::spawn_agent()`
3. Create tasks with `ai::create_task()`
4. Execute tasks with `ai::execute_task()`
5. Use built-in analysis functions (`analyze_text`, `analyze_image`, `generate_text`)

For multi-agent systems:
1. Create coordinator with `ai::create_coordinator()`
2. Add agents with `ai::add_agent_to_coordinator()`
3. Define workflows with `ai::create_workflow()`
4. Execute with `ai::execute_workflow()`

---

## 📚 Next Steps

1. **[AI Agent Tutorial](tutorials/02_ai_trading_agent.md)** - Build using the agent framework
2. **[AI Best Practices](AI_BEST_PRACTICES.md)** - Security and optimization
3. **[Full AI Implementation](../dist_agent_lang/src/stdlib/ai.rs)** - See the actual code
4. **[API Reference](API_REFERENCE.md#6-ai-aiml-integration)** - Complete AI API docs

---

**Ready to build intelligent contracts? Start with [Tutorial: AI Trading Agent →](tutorials/02_ai_trading_agent.md)**

