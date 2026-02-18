# 🤖 AI Best Practices for Smart Contracts (v1.0.5)

> **📢 Beta Release v1.0.5:** Follow these best practices for AI features. Test thoroughly before production. **Beta testing contributions welcome!** 🙏

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
service UnsafeAIContract {
    fn execute_ai_decision() {
        let decision = ai::generate_text("What should I do?");
        
        // Dangerous! Executing without validation
        execute_trade(decision);
    }
}
```

**✅ DO:**
```dal
@secure
service SafeAIContract {
    fn execute_ai_decision() {
        let decision = ai::generate_text("What should I do?");
        
        // Validate AI output
        if !is_valid_decision(decision) {
            log::error("ai", "Invalid AI decision");
            return;
        }
        if !is_within_risk_limits(decision) {
            log::error("ai", "Exceeds risk limits");
            return;
        }
        if !has_required_approvals(decision) {
            log::error("ai", "Requires approval");
            return;
        }
        
        // Log for audit
        log::info("ai", "AI Decision: " + decision);
        
        // Execute with safety checks
        execute_trade_with_safety(decision);
    }
    
    fn is_valid_decision(decision: string) -> bool {
        // Validate decision format
        if decision == "" {
            return false;
        }
        
        // Check if decision is one of allowed actions
        return string::contains(decision, "buy") || 
               string::contains(decision, "sell") ||
               string::contains(decision, "hold");
    }
    
    fn is_within_risk_limits(decision: string) -> bool {
        // Parse decision amount
        let amount = parse_amount(decision);
        
        // Check against risk limits
        return amount <= max_trade_amount && 
               amount >= min_trade_amount;
    }
}
```

### 2. Implement Circuit Breakers

**✅ DO:**
```dal
@ai
@secure
service SafeAIContract {
    // Circuit breaker state
    consecutive_ai_failures: int = 0;
    max_failures: int = 3;
    ai_circuit_breaker_tripped: bool = false;
    
    fn ai_operation(input: string) {
        if ai_circuit_breaker_tripped {
            log::error("ai", "AI circuit breaker tripped");
            return;
        }
        
        let result = ai::classify("model", input);
        if result.is_ok() {
            process_result(result.unwrap());
            // Reset on success
            consecutive_ai_failures = 0;
        } else {
            consecutive_ai_failures = consecutive_ai_failures + 1;
            
            if consecutive_ai_failures >= max_failures {
                ai_circuit_breaker_tripped = true;
                log::error("ai", "Circuit breaker tripped");
            }
        }
    }
    
    fn reset_circuit_breaker() {
        ai_circuit_breaker_tripped = false;
        consecutive_ai_failures = 0;
    }
}
```

### 3. Rate Limit AI Calls

**✅ DO:**
```dal
service RateLimitedAI {
    last_ai_call: map<string, int> = {};
    ai_call_count: map<string, int> = {};
    
    ai_call_cooldown: int = 60; // seconds
    max_ai_calls_per_day: int = 100;
    
    fn make_ai_decision(user_id: string, input: string) {
        // Time-based rate limiting
        let current_time = time::now();
        let last_call = last_ai_call.get(user_id, 0);
        
        if current_time - last_call < ai_call_cooldown {
            log::error("ai", "AI call cooldown active");
            return;
        }
        
        // Daily limit
        if current_time - last_call > 86400 { // 1 day
            ai_call_count[user_id] = 0; // Reset daily count
        }
        
        if ai_call_count.get(user_id, 0) >= max_ai_calls_per_day {
            log::error("ai", "Daily AI call limit exceeded");
            return;
        }
        
        last_ai_call[user_id] = current_time;
        ai_call_count[user_id] = ai_call_count.get(user_id, 0) + 1;
        
        // Perform AI operation
        let result = ai::classify("model", input);
        process_result(result);
    }
}
```

### 4. Validate AI Model Outputs

**✅ DO:**
```dal
service ValidatedAIOracle {
    fn get_price_with_ai_validation() -> int {
        // Get oracle price
        let oracle_result = oracle::fetch("chainlink", "ETH/USD");
        let oracle_price = oracle_result.data;
        
        // AI sanity check
        let historical_prices = get_recent_prices(10);
        let price_data = json::stringify({
            "current": oracle_price,
            "historical": historical_prices
        });
        
        let ai_validation = ai::classify("price_validator", price_data);
        
        if ai_validation != "valid" {
            log::error("ai", "AI flagged suspicious price");
            return 0;
        }
        
        // Additional bounds check
        if oracle_price <= 0 {
            log::error("ai", "Invalid price");
            return 0;
        }
        if oracle_price > 10000 {
            log::error("ai", "Price unreasonably high");
            return 0;
        }
        
        return oracle_price;
    }
}
```

### 5. Use Multi-Model Consensus

**✅ DO:**
```dal
service MultiModelConsensus {
    fn get_ai_consensus(input: string) -> string {
        // Get predictions from multiple models
        let model1 = ai::classify("model_a", input);
        let model2 = ai::classify("model_b", input);
        let model3 = ai::classify("model_c", input);
        
        // Require 2 out of 3 consensus
        let agreement_count = 0;
        let consensus = "";
        
        if model1 == model2 {
            agreement_count = agreement_count + 1;
            consensus = model1;
        }
        if model2 == model3 {
            agreement_count = agreement_count + 1;
            consensus = model2;
        }
        if model1 == model3 {
            agreement_count = agreement_count + 1;
            consensus = model1;
        }
        
        if agreement_count < 2 {
            log::error("ai", "No model consensus");
            return "";
        }
        
        return consensus;
    }
}
```

---

## ⚡ Performance Optimization

### 1. Cache AI Results

**✅ DO:**
```dal
service CachedAI {
    ai_cache: map<string, string> = {};
    cache_timestamp: map<string, int> = {};
    cache_duration: int = 3600; // 1 hour
    
    fn get_ai_result_cached(input: string) -> string {
        let cache_key = crypto::hash(input);
        let cached = ai_cache.get(cache_key, "");
        let timestamp = cache_timestamp.get(cache_key, 0);
        let current_time = time::now();
        
        // Check cache validity
        if cached != "" && current_time - timestamp < cache_duration {
            // Cache hit
            return cached;
        }
        
        // Cache miss - generate new result
        let result = ai::classify("model", input);
        if result.is_ok() {
            let result_value = result.unwrap();
            
            // Update cache
            ai_cache[cache_key] = result_value;
            cache_timestamp[cache_key] = current_time;
            
            return result_value;
        }
        
        return "";
    }
}
```

### 2. Batch AI Operations

**✅ DO:**
```dal
service BatchAI {
    fn batch_classify(inputs: list<string>) -> list<string> {
        if inputs.len() > 10 {
            log::error("ai", "Max 10 items per batch");
            return [];
        }
        
        let results: list<string> = [];
        
        // Single AI call for batch
        let batch_input = json::stringify(inputs);
        let batch_result = ai::classify("batch_model", batch_input);
        
        if batch_result.is_ok() {
            // Parse results
            let parsed = json::parse(batch_result.unwrap());
            if parsed.is_ok() {
                return parsed.unwrap();
            }
        }
        
        return results;
    }
}
```

### 3. Use Async for Non-Critical AI

**✅ DO:**
```dal
service AsyncAI {
    fn analyze_in_background(data: string) {
        // Non-blocking AI operation
        spawn {
            let analysis = ai::analyze_text(data);
            if analysis.is_ok() {
                process_analysis(analysis.unwrap());
            }
        };
    }
    
    // Critical path doesn't wait
    fn critical_operation() {
        // Start analysis in background
        analyze_in_background("data");
        
        // Continue with critical operations
        execute_critical_logic();
    }
}
```

---

## 💰 Cost Management

### 1. Estimate Costs Before AI Calls

**✅ DO:**
```dal
service CostAwareAI {
    fn estimate_ai_cost(operation: string) -> int {
        // Cost varies by operation
        if operation == "classify" {
            return 1000; // Classification cost
        } else if operation == "generate" {
            return 10000; // Generation cost (higher)
        } else if operation == "embed" {
            return 500; // Embedding cost
        }
        
        return 5000; // Default
    }
    
    fn perform_ai_operation(user_id: string, operation: string, payment: int) {
        let estimated_cost = estimate_ai_cost(operation);
        
        if payment < estimated_cost {
            log::error("ai", "Insufficient payment");
            return;
        }
        
        // Perform operation
        if operation == "classify" {
            let result = ai::classify("model", "input");
            process_result(result);
        }
        
        // Refund excess
        if payment > estimated_cost {
            refund(user_id, payment - estimated_cost);
        }
    }
}
```

### 2. Use Cheaper Models for Simple Tasks

**✅ DO:**
```dal
service OptimalModelSelection {
    fn classify_with_optimal_model(input: string) -> string {
        // Use simple model for short inputs
        if input.len() < 100 {
            return ai::classify("fast_model", input).unwrap_or(""); // Cheaper
        }
        
        // Use advanced model for complex inputs
        return ai::classify("advanced_model", input).unwrap_or(""); // More expensive but accurate
    }
}
```

### 3. Implement Cost Limits

**✅ DO:**
```dal
service BudgetLimitedAI {
    daily_ai_budget: int = 1000000; // 1 ETH equivalent
    daily_ai_spent: int = 0;
    last_budget_reset: int = 0;
    
    fn within_budget(cost: int) -> bool {
        let current_time = time::now();
        
        // Reset daily budget
        if current_time - last_budget_reset > 86400 { // 1 day
            daily_ai_spent = 0;
            last_budget_reset = current_time;
        }
        
        if daily_ai_spent + cost > daily_ai_budget {
            log::error("ai", "Daily AI budget exceeded");
            return false;
        }
        
        daily_ai_spent = daily_ai_spent + cost;
        return true;
    }
    
    fn expensive_ai_operation() {
        let cost = 10000; // 0.01 ETH equivalent
        
        if !within_budget(cost) {
            return;
        }
        
        // Operation that costs 0.01 ETH
        let result = ai::generate_text("complex prompt");
        process_result(result);
    }
}
```

---

## 🛡️ Reliability & Error Handling

### 1. Implement Fallback Mechanisms

**✅ DO:**
```dal
service FallbackAI {
    fn get_decision_with_fallback(input: string) -> string {
        // Try AI first
        let decision = ai::generate_text("primary_model: " + input);
        if decision.is_ok() {
            let decision_value = decision.unwrap();
            if is_valid_decision(decision_value) {
                return decision_value;
            }
        }
        
        log::warn("ai", "Primary AI model failed");
        
        // Fallback to secondary model
        let decision2 = ai::generate_text("fallback_model: " + input);
        if decision2.is_ok() {
            let decision_value = decision2.unwrap();
            if is_valid_decision(decision_value) {
                return decision_value;
            }
        }
        
        log::warn("ai", "Fallback AI model failed");
        
        // Ultimate fallback: rule-based decision
        return get_rule_based_decision(input);
    }
    
    fn get_rule_based_decision(input: string) -> string {
        // Simple rule-based logic as last resort
        let current_price = get_current_price();
        let average_price = get_average_price();
        
        if current_price > average_price {
            return "sell";
        } else {
            return "buy";
        }
    }
}
```

### 2. Handle AI Timeouts

**✅ DO:**
```dal
service TimeoutAwareAI {
    fn ai_operation_with_timeout(input: string) -> string {
        let start_time = time::now();
        let timeout = 30; // 30 seconds
        
        // Start AI operation
        let result = ai::classify("model", input);
        
        if result.is_ok() {
            let elapsed = time::now() - start_time;
            
            // Check if operation completed in time
            if elapsed < timeout {
                return result.unwrap();
            } else {
                log::error("ai", "AI operation timed out");
                return "";
            }
        }
        
        return "";
    }
}
```

### 3. Log AI Failures for Debugging

**✅ DO:**
```dal
service LoggedAI {
    fn ai_operation_with_logging(input: string) {
        let result = ai::classify("model", input);
        
        if result.is_ok() {
            process_result(result.unwrap());
        } else {
            // Log failure
            log::error("ai", "AI operation failed: " + result.unwrap_err());
            
            // Use fallback
            use_fallback_logic();
        }
    }
}
```

---

## 🧪 Testing AI Features

### 1. Mock AI Responses for Testing

**✅ DO:**
```dal
service TestableAI {
    test_mode: bool = false;
    mock_responses: map<string, string> = {};
    
    fn set_test_mode(enabled: bool) {
        test_mode = enabled;
    }
    
    fn set_mock_response(input: string, response: string) {
        mock_responses[input] = response;
    }
    
    fn get_ai_result(input: string) -> string {
        if test_mode {
            // Return mock response in test mode
            return mock_responses.get(input, "");
        }
        
        // Real AI call in production
        let result = ai::classify("model", input);
        return result.unwrap_or("");
    }
}
```

### 2. Property-Based Testing for AI

**✅ DO:**
```dal
service TestableAIConsistency {
    fn test_ai_consistency(input: string) -> bool {
        // Same input should give same output (within cache period)
        let result1 = ai::classify("model", input);
        let result2 = ai::classify("model", input);
        
        if result1.is_ok() && result2.is_ok() {
            return result1.unwrap() == result2.unwrap();
        }
        
        return false;
    }
    
    fn test_ai_valid_output(input: string) -> bool {
        let result = ai::classify("sentiment_model", input);
        
        if result.is_ok() {
            let classification = result.unwrap();
            
            // Result should be one of valid sentiments
            return classification == "positive" || 
                   classification == "negative" || 
                   classification == "neutral";
        }
        
        return false;
    }
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
service OptimalModelSelection {
    fn select_optimal_model(task_type: string, urgency: int) -> string {
        if task_type == "sentiment" {
            return "sentiment_model"; // Fast and cheap
        } else if task_type == "generation" {
            if urgency > 8 {
                return "llama-3"; // Faster, cheaper
            } else {
                return "gpt-4"; // Better quality
            }
        } else if task_type == "prediction" {
            return "price_model";
        }
        
        return "default_model";
    }
}
```

---

## ❌ Common Pitfalls

### Pitfall 1: Unbounded AI Loops

**❌ DON'T:**
```dal
service DangerousAI {
    fn dangerous_ai_loop() {
        loop {
            let decision = ai::generate_text("What next?");
            execute(decision);
            // Infinite loop! No exit condition!
        }
    }
}
```

**✅ DO:**
```dal
service SafeAI {
    fn safe_ai_loop() {
        let max_iterations = 10;
        let iterations = 0;
        
        while iterations < max_iterations {
            let decision = ai::generate_text("What next?");
            
            if decision == "stop" {
                break; // Exit condition
            }
            
            execute(decision);
            iterations = iterations + 1;
        }
        
        if iterations >= max_iterations {
            log::error("ai", "Max iterations reached");
        }
    }
}
```

### Pitfall 2: Ignoring AI Confidence Scores

**❌ DON'T:**
```dal
service IgnoreConfidence {
    fn ignore_confidence(input: string) {
        let result = ai::classify("model", input);
        // Using result without checking confidence!
        execute(result);
    }
}
```

**✅ DO:**
```dal
service CheckConfidence {
    fn check_confidence(input: string) {
        let analysis = ai::analyze_text(input);
        
        if analysis.is_ok() {
            let analysis_value = analysis.unwrap();
            
            if analysis_value.confidence > 0.8 {
                // Only use high-confidence results
                let classification = ai::classify("model", input);
                if classification.is_ok() {
                    execute(classification.unwrap());
                }
            } else {
                log::warn("ai", "Low confidence prediction");
            }
        }
    }
}
```

### Pitfall 3: Not Handling Model Updates

**✅ DO:**
```dal
service ModelVersioning {
    current_model_version: string = "v1.0";
    approved_models: map<string, bool> = {};
    
    fn approve_model(version: string) {
        approved_models[version] = true;
    }
    
    fn use_approved_model(input: string) -> string {
        if !approved_models.get(current_model_version, false) {
            log::error("ai", "Model version not approved");
            return "";
        }
        
        let result = ai::classify(current_model_version, input);
        return result.unwrap_or("");
    }
}
```

---

## 📚 Additional Resources

- [AI Features Guide](AI_FEATURES_GUIDE.md) - Complete AI capabilities overview
- [Best Practices](../BEST_PRACTICES.md) - General smart contract best practices
- [Security Guide](../SECURITY_GUIDE.md) - Security deep dive
- [Standard Library Reference](../STDLIB_REFERENCE.md#ai-module) - AI module API

---

**Next:** [AI Features Guide →](AI_FEATURES_GUIDE.md)
