# 🤖 AI Best Practices for Smart Contracts (v1.0.1)

> **📢 Beta Release v1.0.1:** Follow these best practices for AI features. Test thoroughly before production. **Beta testing contributions welcome!** 🙏

Security, performance, and reliability best practices for **AI-powered smart contracts**.

---

## 📋 Table of Contents

1. [Security Best Practices](#security-best-practices)
2. [Performance Optimization](#performance-optimization)
3. [Cost Management](#cost-management)
4. [Reliability & Error Handling](#reliability--error-handling)
5. [Testing AI Features](#testing-ai-features)
6. [Model Selection](#model-selection)
7. [Common Pitfalls](#common-pitfalls)

---

## 🔒 Security Best Practices

### 1. Never Trust AI Output Blindly

**❌ DON'T:**
```dal
@public
function executeAIDecision() {
    let decision = ai::generate("model", "What should I do?");
    
    // Dangerous! Executing without validation
    executeTrade(decision);
}
```

**✅ DO:**
```dal
@public
function executeAIDecision() {
    let decision = ai::generate("model", "What should I do?");
    
    // Validate AI output
    require(isValidDecision(decision), "Invalid AI decision");
    require(isWithinRiskLimits(decision), "Exceeds risk limits");
    require(hasRequiredApprovals(decision), "Requires approval");
    
    // Log for audit
    log::info("AI Decision: " + decision);
    
    // Execute with safety checks
    executeTradeWithSafety(decision);
}

@private
function isValidDecision(string memory decision) -> bool {
    // Validate decision format
    if (bytes(decision).length == 0) return false;
    
    // Check if decision is one of allowed actions
    return string::contains(decision, "buy") || 
           string::contains(decision, "sell") ||
           string::contains(decision, "hold");
}

@private
function isWithinRiskLimits(string memory decision) -> bool {
    // Parse decision amount
    uint256 amount = parseAmount(decision);
    
    // Check against risk limits
    return amount <= maxTradeAmount && 
           amount >= minTradeAmount;
}
```

### 2. Implement Circuit Breakers

**✅ DO:**
```dal
@contract
@ai
contract SafeAIContract {
    // Circuit breaker state
    uint256 public consecutiveAIFailures = 0;
    uint256 constant MAX_FAILURES = 3;
    bool public aiCircuitBreakerTripped = false;
    
    @modifier
    modifier aiCircuitBreaker() {
        require(!aiCircuitBreakerTripped, "AI circuit breaker tripped");
        _;
        
        // Reset on success
        consecutiveAIFailures = 0;
    }
    
    @public
    @aiCircuitBreaker
    function aiOperation() {
        try {
            let result = ai::classify("model", input);
            processResult(result);
        } catch {
            consecutiveAIFailures++;
            
            if (consecutiveAIFailures >= MAX_FAILURES) {
                aiCircuitBreakerTripped = true;
                emit CircuitBreakerTripped("AI");
            }
            
            revert("AI operation failed");
        }
    }
    
    @public
    @onlyOwner
    function resetCircuitBreaker() {
        aiCircuitBreakerTripped = false;
        consecutiveAIFailures = 0;
    }
}
```

### 3. Rate Limit AI Calls

**✅ DO:**
```dal
mapping(address => uint256) public lastAICall;
mapping(address => uint256) public aiCallCount;

uint256 constant AI_CALL_COOLDOWN = 60 seconds;
uint256 constant MAX_AI_CALLS_PER_DAY = 100;

@modifier
modifier rateLimitAI() {
    // Time-based rate limiting
    require(
        block.timestamp - lastAICall[msg.sender] >= AI_CALL_COOLDOWN,
        "AI call cooldown active"
    );
    
    // Daily limit
    if (block.timestamp - lastAICall[msg.sender] > 1 days) {
        aiCallCount[msg.sender] = 0;  // Reset daily count
    }
    
    require(
        aiCallCount[msg.sender] < MAX_AI_CALLS_PER_DAY,
        "Daily AI call limit exceeded"
    );
    
    lastAICall[msg.sender] = block.timestamp;
    aiCallCount[msg.sender]++;
    _;
}

@public
@rateLimitAI
function makeAIDecision() {
    // AI operations
}
```

### 4. Validate AI Model Outputs

**✅ DO:**
```dal
@public
function getPriceWithAIValidation() -> uint256 {
    // Get oracle price
    let oraclePrice = oracle::fetch("chainlink", "ETH/USD").data;
    
    // AI sanity check
    let historicalPrices = getRecentPrices(10);
    let aiValidation = ai::classify(
        "price_validator",
        json::stringify({
            "current": oraclePrice,
            "historical": historicalPrices
        })
    );
    
    require(aiValidation == "valid", "AI flagged suspicious price");
    
    // Additional bounds check
    require(oraclePrice > 0, "Invalid price");
    require(oraclePrice < 10000 * 1e18, "Price unreasonably high");
    
    return oraclePrice;
}
```

### 5. Use Multi-Model Consensus

**✅ DO:**
```dal
@public
function getAIConsensus(string memory input) -> string {
    // Get predictions from multiple models
    let model1 = ai::classify("model_a", input);
    let model2 = ai::classify("model_b", input);
    let model3 = ai::classify("model_c", input);
    
    // Require 2 out of 3 consensus
    uint256 agreementCount = 0;
    string memory consensus;
    
    if (string::equals(model1, model2)) {
        agreementCount++;
        consensus = model1;
    }
    if (string::equals(model2, model3)) {
        agreementCount++;
        consensus = model2;
    }
    if (string::equals(model1, model3)) {
        agreementCount++;
        consensus = model1;
    }
    
    require(agreementCount >= 2, "No model consensus");
    
    return consensus;
}
```

---

## ⚡ Performance Optimization

### 1. Cache AI Results

**✅ DO:**
```dal
// Cache structure
struct CachedAIResult {
    string result;
    uint256 timestamp;
    uint256 hitCount;
}

mapping(bytes32 => CachedAIResult) public aiCache;
uint256 constant CACHE_DURATION = 1 hours;

@public
@view
function getAIResultCached(string memory input) -> string {
    bytes32 cacheKey = keccak256(abi.encodePacked(input));
    CachedAIResult memory cached = aiCache[cacheKey];
    
    // Check cache validity
    if (cached.timestamp > 0 && 
        block.timestamp - cached.timestamp < CACHE_DURATION) {
        // Cache hit
        aiCache[cacheKey].hitCount++;
        return cached.result;
    }
    
    // Cache miss - generate new result
    string memory result = ai::classify("model", input);
    
    // Update cache
    aiCache[cacheKey] = CachedAIResult({
        result: result,
        timestamp: block.timestamp,
        hitCount: 1
    });
    
    return result;
}
```

### 2. Batch AI Operations

**✅ DO:**
```dal
@public
function batchClassify(string[] memory inputs) -> string[] {
    require(inputs.length <= 10, "Max 10 items per batch");
    
    string[] memory results = new string[](inputs.length);
    
    // Single AI call for batch
    let batchInput = json::stringify(inputs);
    let batchResult = ai::classify("batch_model", batchInput);
    
    // Parse results
    results = json::parse(batchResult);
    
    return results;
}
```

### 3. Use Async for Non-Critical AI

**✅ DO:**
```dal
@public
@async
async function analyzeInBackground(string memory data) {
    // Non-blocking AI operation
    let analysis = await ai::analyze("model", data);
    
    // Process results asynchronously
    await processAnalysis(analysis);
}

// Critical path doesn't wait
@public
function criticalOperation() {
    // Start analysis in background
    analyzeInBackground(data);
    
    // Continue with critical operations
    executeCriticalLogic();
}
```

---

## 💰 Cost Management

### 1. Estimate Costs Before AI Calls

**✅ DO:**
```dal
@public
@view
function estimateAICost(string memory operation) -> uint256 {
    // Cost varies by operation
    if (string::equals(operation, "classify")) {
        return 0.001 ether;  // Classification cost
    } else if (string::equals(operation, "generate")) {
        return 0.01 ether;   // Generation cost (higher)
    } else if (string::equals(operation, "embed")) {
        return 0.0005 ether; // Embedding cost
    }
    
    return 0.005 ether;  // Default
}

@public
payable
function performAIOperation(string memory operation) {
    uint256 estimatedCost = estimateAICost(operation);
    require(msg.value >= estimatedCost, "Insufficient payment");
    
    // Perform operation
    performOperation(operation);
    
    // Refund excess
    if (msg.value > estimatedCost) {
        payable(msg.sender).transfer(msg.value - estimatedCost);
    }
}
```

### 2. Use Cheaper Models for Simple Tasks

**✅ DO:**
```dal
@public
function classifyWithOptimalModel(string memory input) -> string {
    // Use simple model for short inputs
    if (bytes(input).length < 100) {
        return ai::classify("fast_model", input);  // Cheaper
    }
    
    // Use advanced model for complex inputs
    return ai::classify("advanced_model", input);   // More expensive but accurate
}
```

### 3. Implement Cost Limits

**✅ DO:**
```dal
uint256 public dailyAIBudget = 1 ether;
uint256 public dailyAISpent = 0;
uint256 public lastBudgetReset;

@modifier
modifier withinBudget(uint256 cost) {
    // Reset daily budget
    if (block.timestamp - lastBudgetReset > 1 days) {
        dailyAISpent = 0;
        lastBudgetReset = block.timestamp;
    }
    
    require(
        dailyAISpent + cost <= dailyAIBudget,
        "Daily AI budget exceeded"
    );
    
    dailyAISpent += cost;
    _;
}

@public
@withinBudget(0.01 ether)
function expensiveAIOperation() {
    // Operation that costs 0.01 ETH
}
```

---

## 🛡️ Reliability & Error Handling

### 1. Implement Fallback Mechanisms

**✅ DO:**
```dal
@public
function getDecisionWithFallback() -> string {
    // Try AI first
    try {
        let decision = ai::generate("primary_model", input);
        if (isValidDecision(decision)) {
            return decision;
        }
    } catch {
        log::warn("Primary AI model failed");
    }
    
    // Fallback to secondary model
    try {
        let decision = ai::generate("fallback_model", input);
        if (isValidDecision(decision)) {
            return decision;
        }
    } catch {
        log::warn("Fallback AI model failed");
    }
    
    // Ultimate fallback: rule-based decision
    return getRuleBasedDecision(input);
}

@private
function getRuleBasedDecision(string memory input) -> string {
    // Simple rule-based logic as last resort
    if (getCurrentPrice() > getAveragePrice()) {
        return "sell";
    } else {
        return "buy";
    }
}
```

### 2. Handle AI Timeouts

**✅ DO:**
```dal
@public
function aiOperationWithTimeout(string memory input) -> string {
    uint256 startTime = block.timestamp;
    uint256 timeout = 30 seconds;
    
    // Start AI operation
    let result = ai::classify("model", input);
    
    // Check if operation completed in time
    require(
        block.timestamp - startTime < timeout,
        "AI operation timed out"
    );
    
    return result;
}
```

### 3. Log AI Failures for Debugging

**✅ DO:**
```dal
event AIOperationFailed(
    string operation,
    string input,
    string error,
    uint256 timestamp
);

@public
function aiOperationWithLogging(string memory input) {
    try {
        let result = ai::classify("model", input);
        processResult(result);
    } catch Error(string memory reason) {
        // Log failure
        emit AIOperationFailed(
            "classify",
            input,
            reason,
            block.timestamp
        );
        
        log::error("AI operation failed: " + reason);
        
        // Use fallback
        useFallbackLogic();
    }
}
```

---

## 🧪 Testing AI Features

### 1. Mock AI Responses for Testing

**✅ DO:**
```dal
// Test mode flag
bool public testMode = false;
mapping(string => string) public mockResponses;

@public
@onlyOwner
function setTestMode(bool enabled) {
    testMode = enabled;
}

@public
@onlyOwner
function setMockResponse(string memory input, string memory response) {
    mockResponses[input] = response;
}

@public
function getAIResult(string memory input) -> string {
    if (testMode) {
        // Return mock response in test mode
        return mockResponses[input];
    }
    
    // Real AI call in production
    return ai::classify("model", input);
}
```

### 2. Property-Based Testing for AI

**✅ DO:**
```dal
@property_test("AI results should be consistent")
function prop_aiConsistency(string memory input) {
    // Same input should give same output (within cache period)
    let result1 = ai::classify("model", input);
    let result2 = ai::classify("model", input);
    
    assert_eq(result1, result2, "AI results should be consistent");
}

@property_test("AI results should be valid")
function prop_aiValidOutput(string memory input) {
    let result = ai::classify("sentiment_model", input);
    
    // Result should be one of valid sentiments
    assert_true(
        result == "positive" || 
        result == "negative" || 
        result == "neutral",
        "AI should return valid sentiment"
    );
}
```

---

## 🎯 Model Selection

### Choosing the Right Model

| Task | Recommended Model | Cost | Speed | Accuracy |
|------|------------------|------|-------|----------|
| Sentiment Analysis | `sentiment_model` | 💰 Low | ⚡ Fast | 🎯 Good |
| Text Generation | `gpt-4` | 💰💰💰 High | 🐌 Slow | 🎯🎯🎯 Excellent |
| Classification | `fast_classifier` | 💰 Low | ⚡⚡ Very Fast | 🎯 Good |
| Price Prediction | `price_model` | 💰💰 Medium | ⚡ Fast | 🎯🎯 Very Good |
| Risk Assessment | `risk_model` | 💰💰 Medium | ⚡ Fast | 🎯🎯 Very Good |

**✅ DO:**
```dal
@public
function selectOptimalModel(string memory taskType, uint256 urgency) -> string {
    if (string::equals(taskType, "sentiment")) {
        return "sentiment_model";  // Fast and cheap
    } else if (string::equals(taskType, "generation")) {
        if (urgency > 8) {
            return "llama-3";  // Faster, cheaper
        } else {
            return "gpt-4";    // Better quality
        }
    } else if (string::equals(taskType, "prediction")) {
        return "price_model";
    }
    
    return "default_model";
}
```

---

## ❌ Common Pitfalls

### Pitfall 1: Unbounded AI Loops

**❌ DON'T:**
```dal
@public
function dangerousAILoop() {
    while (true) {
        let decision = ai::generate("model", "What next?");
        execute(decision);
        // Infinite loop! No exit condition!
    }
}
```

**✅ DO:**
```dal
@public
function safeAILoop() {
    uint256 maxIterations = 10;
    uint256 iterations = 0;
    
    while (iterations < maxIterations) {
        let decision = ai::generate("model", "What next?");
        
        if (string::equals(decision, "stop")) {
            break;  // Exit condition
        }
        
        execute(decision);
        iterations++;
    }
    
    require(iterations < maxIterations, "Max iterations reached");
}
```

### Pitfall 2: Ignoring AI Confidence Scores

**❌ DON'T:**
```dal
@public
function ignoreConfidence() {
    let result = ai::classify("model", input);
    // Using result without checking confidence!
    execute(result);
}
```

**✅ DO:**
```dal
@public
function checkConfidence() {
    let result = ai::classify_with_confidence("model", input);
    
    require(result.confidence > 0.8, "Low confidence prediction");
    
    // Only use high-confidence results
    execute(result.prediction);
}
```

### Pitfall 3: Not Handling Model Updates

**✅ DO:**
```dal
string public currentModelVersion = "v1.0";
mapping(string => bool) public approvedModels;

@public
@onlyOwner
function approveModel(string memory version) {
    approvedModels[version] = true;
}

@public
function useApprovedModel(string memory input) -> string {
    require(
        approvedModels[currentModelVersion],
        "Model version not approved"
    );
    
    return ai::classify(currentModelVersion, input);
}
```

---

## 📚 Additional Resources

- [AI Features Guide](AI_FEATURES_GUIDE.md) - Complete AI capabilities overview
- [AI Trading Agent Tutorial](tutorials/02_ai_trading_agent.md) - Build an AI agent
- [Best Practices](BEST_PRACTICES.md) - General smart contract best practices
- [Security Guide](SECURITY_GUIDE.md) - Security deep dive

---

**Next:** [Build Your First AI Agent →](tutorials/02_ai_trading_agent.md)

