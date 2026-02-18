# 🏗️ Oracle Development Setup for dist_agent_lang

Complete guide to setting up and using oracles in your dist_agent_lang development environment for xNFTs and Dynamic RWAs.

---

## 🚀 Quick Start (5 Minutes)

Get up and running with oracles in just 5 minutes:

### 1. One-Click Setup
```rust
// Run this in your dist_agent_lang environment
oracle_development_workflow();
```

This will:
- ✅ Start mock oracle server on port 3001
- ✅ Create sample price and weather oracles
- ✅ Generate test data with realistic patterns
- ✅ Setup monitoring and health checks
- ✅ Create sample xNFT integration

### 2. Test Your Setup
```bash
# Check server health
curl http://localhost:3001/health

# Test price oracle
curl http://localhost:3001/quick/price

# Test weather oracle
curl http://localhost:3001/quick/weather
```

### 3. Create Your First xNFT
```rust
@trust("hybrid")
service MyFirstXNFT {
    weather_oracle: any,

    fn initialize() {
        this.weather_oracle = {
            "endpoint": "http://localhost:3001/quick/weather",
            "type": "mock"
        };
    }

    fn update_based_on_weather() {
        let weather_data = web::get(this.weather_oracle.endpoint);
        let data = json::parse(weather_data.body);

        update_metadata({
            "current_temperature": data.temperature,
            "weather_condition": data.conditions,
            "last_update": data.timestamp
        });
    }

    schedule_execution("every_5_minutes", update_based_on_weather);
}
```

---

## 📋 Oracle Types Available

### Mock Oracles (Development)
Perfect for local development and testing:

| Oracle | Endpoint | Data Type | Update Interval |
|--------|----------|-----------|-----------------|
| **Price** | `/mock/price` | ETH, BTC, MATIC prices | 5 seconds |
| **Weather** | `/mock/weather` | Temperature, humidity, conditions | 10 seconds |
| **Social** | `/mock/social` | Sentiment scores, mentions | 15 seconds |
| **IoT** | `/mock/iot` | Sensor readings, device status | 2 seconds |
| **Sports** | `/mock/sports` | Team stats, game results | 30 seconds |

### Test Oracles (Integration)
For integration testing with realistic scenarios:

- **TestNet Price Feeds**: Real blockchain data from test networks
- **API-based Oracles**: External API integration with rate limiting
- **Hybrid Oracles**: Mix of mock and real data

### Production Oracles (Development Testing)
Test against real oracle networks:

- **Chainlink Price Feeds**: Mainnet price data
- **The Graph Protocol**: Decentralized data indexing
- **Custom Oracle Networks**: Your own oracle infrastructure

---

## 🛠️ Development Environment Setup

### Option 1: Automated Setup (Recommended)
```rust
// One-click setup for everything
let setup_result = setup_development_oracle_environment();

// Check setup status
println!("Setup complete: {}", setup_result.status);
println!("Server port: {}", setup_result.development_environment.config.mock_server.port);
```

### Option 2: Manual Setup
```rust
// Step 1: Setup oracle configurations
let oracle_setup = OracleDevelopmentSetup::new();

// Step 2: Configure mock oracles
oracle_setup.setup_mock_oracles();

// Step 3: Start development server
oracle_setup.start_oracle_development_server(3001);

// Step 4: Initialize monitoring
oracle_setup.setup_oracle_monitoring();
```

### Option 3: Custom Configuration
```rust
// Create custom oracle configuration
let custom_config = {
    "environment": "development",
    "mock_server": {
        "port": 3001,
        "custom_endpoints": [
            "/my-custom-oracle",
            "/specialized-data"
        ]
    },
    "oracle_instances": [
        {
            "name": "custom_price_oracle",
            "type": "mock",
            "custom_data_generator": my_custom_generator_function
        }
    ]
};

// Apply configuration
oracle_setup.apply_custom_configuration(custom_config);
```

---

## 🔧 Oracle Configuration

### Basic Configuration
```rust
let oracle_config = {
    "name": "MyPriceOracle",
    "type": "mock",              // "mock", "test", "development", "production"
    "endpoint": "http://localhost:3001/mock/price",
    "update_interval": 5000,    // 5 seconds
    "data_format": "json",
    "authentication": {
        "type": "none",          // "none", "api_key", "bearer_token"
        "credentials": null
    },
    "validation": {
        "enabled": true,
        "schema": "price_data_schema",
        "range_check": true
    }
};
```

### Advanced Configuration
```rust
let advanced_config = {
    "name": "ProductionPriceOracle",
    "type": "chainlink",
    "networks": {
        "ethereum": {
            "rpc_url": "https://eth-mainnet.alchemyapi.io/v2/YOUR_API_KEY",
            "contracts": {
                "ETH_USD": "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419",
                "BTC_USD": "0xF4030086522a5bEEa4988F8cA5B36dbC97BeE88c"
            }
        }
    },
    "gas_optimization": {
        "max_gas_price": 100000000000,  // 100 gwei
        "gas_limit": 200000
    },
    "monitoring": {
        "health_checks": true,
        "performance_metrics": true,
        "error_alerts": true
    }
};
```

---

## 📊 Testing Your Oracle Setup

### Automated Testing
```rust
// Run complete test suite
let test_suite = oracle_setup.create_oracle_test_suite();
let test_results = oracle_setup.run_oracle_tests(test_suite);

// Check results
println!("Tests passed: {}/{}", test_results.passed, test_results.total);
```

### Manual Testing
```bash
# Health check
curl http://localhost:3001/health

# Price data
curl http://localhost:3001/mock/price | jq .

# Weather data
curl http://localhost:3001/mock/weather | jq .

# Social data
curl http://localhost:3001/mock/social | jq .

# IoT data
curl http://localhost:3001/mock/iot | jq .

# Sports data
curl http://localhost:3001/mock/sports | jq .
```

### Performance Testing
```rust
// Test oracle response times
let performance_test = oracle_setup.run_performance_test({
    "duration": 60000,  // 1 minute
    "concurrency": 10,  // 10 concurrent requests
    "oracle_types": ["price", "weather", "social"]
});

// View results
println!("Average response time: {}ms", performance_test.avg_response_time);
println!("Requests per second: {}", performance_test.rps);
println!("Error rate: {}%", performance_test.error_rate * 100);
```

---

## 🔗 Integrating Oracles with xNFTs

### Basic xNFT with Oracle Integration
```rust
@trust("hybrid")
service OraclePoweredNFT {
    price_oracle: any,
    current_value: Float,

    fn initialize() {
        this.price_oracle = oracle::create_price_feed({
            "endpoint": "http://localhost:3001/quick/price",
            "assets": ["ETH"],
            "update_interval": 30000
        });
        this.current_value = 100.0;
    }

    fn update_value_based_on_price() {
        // Get price data from oracle
        let price_data = oracle::get_price_data(this.price_oracle);

        // Calculate new NFT value based on ETH price
        let eth_price = price_data.ETH;
        let new_value = this.current_value * (eth_price / 2500.0); // Base ETH price

        // Update NFT metadata
        update_metadata({
            "current_value": new_value,
            "eth_price": eth_price,
            "last_price_update": price_data.timestamp,
            "value_change_percentage": ((new_value - this.current_value) / this.current_value) * 100
        });

        this.current_value = new_value;

        // Trigger events based on value changes
        if new_value > this.current_value * 1.05 {
            trigger_action("value_increased", {
                "new_value": new_value,
                "change_percentage": 5.0
            });
        }
    }

    schedule_execution("every_30_seconds", update_value_based_on_price);
}
```

### Advanced xNFT with Multiple Oracles
```rust
@trust("hybrid")
@ai
service MultiOracleNFT {
    price_oracle: any,
    weather_oracle: any,
    social_oracle: any,
    ai_analyzer: any,

    fn initialize() {
        // Setup multiple oracles
        this.price_oracle = oracle::create_price_feed({
            "endpoint": "http://localhost:3001/mock/price"
        });

        this.weather_oracle = oracle::create_weather_feed({
            "endpoint": "http://localhost:3001/mock/weather"
        });

        this.social_oracle = oracle::create_social_feed({
            "endpoint": "http://localhost:3001/mock/social"
        });

        // Setup AI analyzer
        this.ai_analyzer = ai::create_analyzer({
            "model": "multi_factor_analysis",
            "factors": ["price", "weather", "social_sentiment"]
        });
    }

    fn intelligent_nft_update() {
        // Gather data from all oracles
        let price_data = oracle::get_price_data(this.price_oracle);
        let weather_data = oracle::get_weather_data(this.weather_oracle);
        let social_data = oracle::get_social_data(this.social_oracle);

        // Use AI to analyze combined factors
        let analysis = ai::analyze_multi_factor_data(this.ai_analyzer, {
            "price_data": price_data,
            "weather_data": weather_data,
            "social_data": social_data
        });

        // Determine NFT appearance and value
        let appearance = this.calculate_optimal_appearance(analysis);
        let value = this.calculate_dynamic_value(analysis);

        // Update NFT with AI-driven insights
        update_metadata({
            "appearance": appearance,
            "current_value": value,
            "ai_confidence": analysis.confidence_score,
            "influencing_factors": analysis.key_factors,
            "last_ai_update": chain::get_block_timestamp(),
            "recommendations": analysis.recommendations
        });

        // Execute AI recommendations
        this.execute_ai_recommendations(analysis.recommendations);
    }

    fn calculate_optimal_appearance(analysis: any) -> String {
        // Determine appearance based on AI analysis
        if analysis.market_sentiment > 0.7 && analysis.weather_positive {
            return "golden_bullish";
        } else if analysis.social_sentiment < 0.3 {
            return "cautious_blue";
        } else {
            return "balanced_neutral";
        }
    }

    fn calculate_dynamic_value(analysis: any) -> Float {
        let base_value = 100.0;
        let market_multiplier = analysis.market_sentiment;
        let weather_multiplier = analysis.weather_positive ? 1.1 : 0.9;
        let social_multiplier = analysis.social_sentiment;

        return base_value * market_multiplier * weather_multiplier * social_multiplier;
    }

    fn execute_ai_recommendations(recommendations: List<any>) -> Unit {
        for recommendation in recommendations {
            match recommendation.action {
                "increase_liquidity" => {
                    this.increase_nft_liquidity(recommendation.amount);
                },
                "hedge_position" => {
                    this.hedge_nft_position(recommendation.instrument);
                },
                "take_profit" => {
                    this.take_nft_profit(recommendation.percentage);
                }
            }
        }
    }

    schedule_execution("every_5_minutes", intelligent_nft_update);
}
```

---

## 🎯 Dynamic RWA Examples

### Real Estate RWA with Oracle Integration
```rust
@trust("hybrid")
service RealEstateRWA {
    property_feed: any,
    economic_feed: any,

    fn initialize() {
        this.property_feed = oracle::create_real_estate_feed({
            "location": "New York",
            "property_type": "commercial"
        });

        this.economic_feed = oracle::create_economic_feed({
            "region": "northeast",
            "indicators": ["gdp_growth", "interest_rates", "employment"]
        });
    }

    fn update_property_valuation() {
        let property_data = oracle::get_real_estate_data(this.property_feed);
        let economic_data = oracle::get_economic_data(this.economic_feed);

        let valuation = this.calculate_property_valuation(property_data, economic_data);
        let risk_assessment = this.assess_property_risk(property_data, economic_data);

        update_metadata({
            "current_valuation": valuation.value,
            "valuation_confidence": valuation.confidence,
            "risk_rating": risk_assessment.rating,
            "economic_indicators": economic_data,
            "last_valuation_update": chain::get_block_timestamp()
        });
    }

    schedule_execution("daily", update_property_valuation);
}
```

### Carbon Credit RWA
```rust
@trust("hybrid")
service CarbonCreditRWA {
    carbon_feed: any,
    market_feed: any,

    fn initialize() {
        this.carbon_feed = oracle::create_carbon_feed({
            "project_id": "amazon_reforestation_001",
            "standard": "verra"
        });

        this.market_feed = oracle::create_carbon_market_feed({
            "trading_platforms": ["xeprex", "aircarbon"]
        });
    }

    fn update_carbon_credit_value() {
        let carbon_data = oracle::get_carbon_data(this.carbon_feed);
        let market_data = oracle::get_carbon_market_data(this.market_feed);

        let credit_value = carbon_data.sequestered_tons * market_data.price_per_ton;

        update_metadata({
            "total_credits": carbon_data.sequestered_tons,
            "market_price": market_data.price_per_ton,
            "total_value": credit_value,
            "verification_status": carbon_data.verification_status,
            "last_update": chain::get_block_timestamp()
        });
    }

    schedule_execution("weekly", update_carbon_credit_value);
}
```

---

## 📈 Monitoring & Debugging

### Health Monitoring
```rust
// Monitor oracle health
let health_report = oracle_setup.monitor_oracle_health();

println!("Oracle Health Report:");
println!("  Price Oracle: {}", health_report.price_oracle.status);
println!("  Weather Oracle: {}", health_report.weather_oracle.status);
println!("  Overall Status: {}", health_report.overall_status);
```

### Performance Monitoring
```rust
// Monitor oracle performance
let performance_report = oracle_setup.monitor_oracle_performance();

println!("Oracle Performance:");
println!("  Average Response Time: {}ms", performance_report.avg_response_time);
println!("  Requests Per Second: {}", performance_report.rps);
println!("  Error Rate: {}%", performance_report.error_rate * 100);
```

### Data Quality Monitoring
```rust
// Monitor data quality
let quality_report = oracle_setup.monitor_data_quality();

println!("Data Quality:");
println!("  Freshness Score: {}%", quality_report.freshness_score);
println!("  Accuracy Score: {}%", quality_report.accuracy_score);
println!("  Completeness Score: {}%", quality_report.completeness_score);
```

### Debugging Tools
```rust
// Enable debug logging
oracle_setup.enable_debug_logging();

// View oracle logs
let logs = oracle_setup.get_oracle_logs({
    "oracle_type": "price",
    "time_range": "last_hour",
    "log_level": "debug"
});

// Test oracle connectivity
let connectivity_test = oracle_setup.test_oracle_connectivity();
println!("Connectivity Test Results: {}", connectivity_test);
```

---

## 🚀 Production Deployment

### Moving from Development to Production
```rust
// 1. Switch from mock to real oracles
let production_config = {
    "environment": "production",
    "oracles": {
        "price": {
            "type": "chainlink",
            "network": "ethereum_mainnet",
            "contracts": {
                "ETH_USD": "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419"
            }
        },
        "weather": {
            "type": "api",
            "provider": "openweathermap",
            "api_key": process.env.OPENWEATHER_API_KEY
        }
    }
};

// 2. Update xNFT configurations
oracle_setup.apply_production_config(production_config);

// 3. Enable production monitoring
oracle_setup.enable_production_monitoring();

// 4. Run production tests
let production_tests = oracle_setup.run_production_tests();
```

### Scaling Considerations
```rust
// Configure for high throughput
let scaling_config = {
    "load_balancing": {
        "enabled": true,
        "algorithm": "least_connections",
        "max_connections": 1000
    },
    "caching": {
        "enabled": true,
        "ttl": 300000,  // 5 minutes
        "max_size": 10000
    },
    "rate_limiting": {
        "enabled": true,
        "requests_per_minute": 1000,
        "burst_limit": 100
    }
};

oracle_setup.apply_scaling_config(scaling_config);
```

---

## 🔧 Troubleshooting

### Common Issues & Solutions

#### Oracle Server Not Starting
```bash
# Check if port is available
lsof -i :3001

# Kill process using port
kill -9 $(lsof -t -i:3001)

# Restart server
oracle_setup.start_oracle_development_server(3001);
```

#### Mock Data Not Updating
```rust
// Check data generator status
let generator_status = oracle_setup.check_data_generators();

// Restart data generators
oracle_setup.restart_data_generators();
```

#### Oracle Connection Failures
```rust
// Test connectivity
let connectivity_test = oracle_setup.test_oracle_connectivity();

// Check network configuration
let network_config = oracle_setup.get_network_config();

// Reset connections
oracle_setup.reset_oracle_connections();
```

#### Performance Issues
```rust
// Enable performance monitoring
oracle_setup.enable_performance_monitoring();

// Get performance report
let performance_report = oracle_setup.get_performance_report();

// Optimize configuration
oracle_setup.optimize_performance();
```

---

## 📚 Resources & Examples

- 📄 **[oracle_development_setup.rs](./examples/oracle_development_setup.rs)** - Complete oracle setup guide
- 📄 **[oracle_quick_start.rs](./examples/oracle_quick_start.rs)** - Quick start guide
- 📄 **[xnft_implementation.rs](./examples/xnft_implementation.rs)** - xNFT system with oracle integration
- 📄 **[dynamic_nft_examples.rs](./examples/dynamic_nft_examples.rs)** - Dynamic NFT examples
- 📄 **[dynamic_rwa_examples.rs](./examples/dynamic_rwa_examples.rs)** - Dynamic RWA examples

---

## 🎯 Best Practices

### Development
1. **Use mock oracles** for local development
2. **Test with realistic data** patterns
3. **Monitor oracle health** continuously
4. **Implement proper error handling**
5. **Use caching** to reduce oracle calls

### Testing
1. **Test error scenarios** thoroughly
2. **Validate data freshness** and accuracy
3. **Test with various network conditions**
4. **Monitor performance metrics**
5. **Use automated testing** suites

### Production
1. **Use real oracle networks** for production
2. **Implement redundancy** and failover
3. **Monitor costs** and optimize usage
4. **Regular security audits**
5. **Plan for scaling** and high availability

---

## 💡 Pro Tips

- **Start with mock oracles** for fastest development iteration
- **Combine multiple oracles** for robust data sources
- **Use AI for data analysis** and decision making
- **Implement caching** to reduce costs and improve performance
- **Monitor everything** - data quality, performance, errors
- **Test failure scenarios** before going to production
- **Plan for oracle costs** in your budget
- **Keep oracle logic simple** and focused

---

**Ready to build amazing xNFTs and Dynamic RWAs?** 🚀

The oracle development setup for dist_agent_lang provides everything you need to create intelligent, dynamic digital assets that respond to real-world data. Start with the quick setup guide and build from there! 🎯
