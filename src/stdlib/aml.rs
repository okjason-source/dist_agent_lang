use crate::runtime::values::Value;
use crate::stdlib::chain;
use std::collections::HashMap;
use std::env;

// AML (Anti-Money Laundering) namespace
// Provides anti-money laundering and compliance functions.
//
// **Simplified vs full API:**
// - **Simplified:** Local rule-based risk (keyword matching: "suspicious", "high_risk", etc.).
// - **Full:** When AML_API_URL (and optionally AML_API_KEY) are set and http-interface enabled,
//   screen_transaction and monitor_address can call an external AML/sanctions API (e.g. Chainalysis,
//   OFAC-compatible) for real risk scores. Fallback to simplified when unconfigured or on error.

#[derive(Debug, Clone)]
pub struct AMLCheck {
    pub check_id: String,
    pub user_address: String,
    pub check_type: String,
    pub status: String,
    pub risk_score: f64,
    pub findings: HashMap<String, String>,
    pub recommendations: HashMap<String, String>,
    pub performed_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone)]
pub struct AMLProvider {
    pub id: String,
    pub name: String,
    pub api_endpoint: String,
    pub api_key: String,
    pub check_types: HashMap<String, AMLCheckType>,
    pub compliance_standards: HashMap<String, bool>,
    pub success_rate: f64,
    pub response_time: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct AMLCheckType {
    pub check_type: String,
    pub description: String,
    pub risk_factors: HashMap<String, f64>,
    pub check_time: i64,
    pub cost: i64,
    pub accuracy: f64,
}

lazy_static::lazy_static! {
    static ref AML_PROVIDERS: HashMap<String, AMLProvider> = {
        let mut m = HashMap::new();

        // Default AML providers
        m.insert("chainalysis".to_string(), AMLProvider {
            id: "chainalysis".to_string(),
            name: "Chainalysis".to_string(),
            api_endpoint: "https://api.chainalysis.com/v1".to_string(),
            api_key: "chainalysis_default".to_string(),
            check_types: {
                let mut types = HashMap::new();
                types.insert("sanctions".to_string(), AMLCheckType {
                    check_type: "sanctions".to_string(),
                    description: "Sanctions list screening".to_string(),
                    risk_factors: {
                        let mut factors = HashMap::new();
                        factors.insert("sanctions_match".to_string(), 1.0);
                        factors.insert("pep_match".to_string(), 0.8);
                        factors.insert("adverse_media".to_string(), 0.6);
                        factors
                    },
                    check_time: 5 * 60, // 5 minutes
                    cost: 5,
                    accuracy: 0.98,
                });
                types.insert("pep".to_string(), AMLCheckType {
                    check_type: "pep".to_string(),
                    description: "Politically Exposed Person screening".to_string(),
                    risk_factors: {
                        let mut factors = HashMap::new();
                        factors.insert("pep_match".to_string(), 0.9);
                        factors.insert("family_pep".to_string(), 0.7);
                        factors.insert("close_associate".to_string(), 0.6);
                        factors
                    },
                    check_time: 10 * 60, // 10 minutes
                    cost: 8,
                    accuracy: 0.95,
                });
                types.insert("adverse_media".to_string(), AMLCheckType {
                    check_type: "adverse_media".to_string(),
                    description: "Adverse media and negative news screening".to_string(),
                    risk_factors: {
                        let mut factors = HashMap::new();
                        factors.insert("negative_news".to_string(), 0.7);
                        factors.insert("fraud_allegations".to_string(), 0.9);
                        factors.insert("regulatory_violations".to_string(), 0.8);
                        factors
                    },
                    check_time: 15 * 60, // 15 minutes
                    cost: 12,
                    accuracy: 0.92,
                });
                types.insert("risk_assessment".to_string(), AMLCheckType {
                    check_type: "risk_assessment".to_string(),
                    description: "Comprehensive risk assessment".to_string(),
                    risk_factors: {
                        let mut factors = HashMap::new();
                        factors.insert("transaction_pattern".to_string(), 0.6);
                        factors.insert("geographic_risk".to_string(), 0.5);
                        factors.insert("business_risk".to_string(), 0.7);
                        factors.insert("source_of_funds".to_string(), 0.8);
                        factors
                    },
                    check_time: 30 * 60, // 30 minutes
                    cost: 20,
                    accuracy: 0.94,
                });
                types
            },
            compliance_standards: {
                let mut standards = HashMap::new();
                standards.insert("fatf".to_string(), true);
                standards.insert("ofac".to_string(), true);
                standards.insert("eu_sanctions".to_string(), true);
                standards.insert("uk_sanctions".to_string(), true);
                standards
            },
            success_rate: 0.99,
            response_time: 3000, // 3 seconds
            is_active: true,
        });

        m.insert("elliptic".to_string(), AMLProvider {
            id: "elliptic".to_string(),
            name: "Elliptic".to_string(),
            api_endpoint: "https://api.elliptic.com/v1".to_string(),
            api_key: "elliptic_default".to_string(),
            check_types: {
                let mut types = HashMap::new();
                types.insert("sanctions".to_string(), AMLCheckType {
                    check_type: "sanctions".to_string(),
                    description: "Sanctions list screening".to_string(),
                    risk_factors: {
                        let mut factors = HashMap::new();
                        factors.insert("sanctions_match".to_string(), 1.0);
                        factors.insert("pep_match".to_string(), 0.8);
                        factors.insert("adverse_media".to_string(), 0.6);
                        factors
                    },
                    check_time: 3 * 60, // 3 minutes
                    cost: 4,
                    accuracy: 0.97,
                });
                types.insert("risk_assessment".to_string(), AMLCheckType {
                    check_type: "risk_assessment".to_string(),
                    description: "Comprehensive risk assessment".to_string(),
                    risk_factors: {
                        let mut factors = HashMap::new();
                        factors.insert("transaction_pattern".to_string(), 0.6);
                        factors.insert("geographic_risk".to_string(), 0.5);
                        factors.insert("business_risk".to_string(), 0.7);
                        factors.insert("source_of_funds".to_string(), 0.8);
                        factors
                    },
                    check_time: 20 * 60, // 20 minutes
                    cost: 15,
                    accuracy: 0.93,
                });
                types
            },
            compliance_standards: {
                let mut standards = HashMap::new();
                standards.insert("fatf".to_string(), true);
                standards.insert("ofac".to_string(), true);
                standards.insert("eu_sanctions".to_string(), true);
                standards
            },
            success_rate: 0.98,
            response_time: 2000, // 2 seconds
            is_active: true,
        });

        m
    };
}

// Public AML functions

pub fn perform_check(
    provider_id: String,
    user_address: String,
    check_type: String,
    user_data: HashMap<String, String>,
) -> HashMap<String, Value> {
    crate::stdlib::log::audit(
        "aml_check",
        {
            let mut data = HashMap::new();
            data.insert(
                "provider_id".to_string(),
                Value::String(provider_id.clone()),
            );
            data.insert(
                "user_address".to_string(),
                Value::String(user_address.clone()),
            );
            data.insert("check_type".to_string(), Value::String(check_type.clone()));
            data.insert(
                "timestamp".to_string(),
                Value::Int(chain::get_block_timestamp(1)),
            );
            data
        },
        Some("aml"),
    );

    if !AML_PROVIDERS.contains_key(&provider_id) {
        return {
            let mut result = HashMap::new();
            result.insert("status".to_string(), Value::String("failed".to_string()));
            result.insert(
                "error".to_string(),
                Value::String("Provider not found".to_string()),
            );
            result
        };
    }

    let provider = AML_PROVIDERS.get(&provider_id).unwrap();
    if !provider.check_types.contains_key(&check_type) {
        return {
            let mut result = HashMap::new();
            result.insert("status".to_string(), Value::String("failed".to_string()));
            result.insert(
                "error".to_string(),
                Value::String("Check type not supported".to_string()),
            );
            result
        };
    }

    let aml_check_type = provider.check_types.get(&check_type).unwrap();

    // Simulate AML check process
    let check_id = format!(
        "aml_{}_{}_{}",
        provider_id,
        check_type,
        chain::get_block_timestamp(1)
    );
    let timestamp = chain::get_block_timestamp(1);
    let expires_at = timestamp + 365 * 24 * 60 * 60; // 1 year

    // Simulate API call delay
    std::thread::sleep(std::time::Duration::from_millis(
        provider.response_time as u64,
    ));

    // Generate risk score based on check type
    let risk_score = generate_risk_score(&check_type, &user_data);
    let status = if risk_score < 0.3 {
        "passed"
    } else if risk_score < 0.7 {
        "review"
    } else {
        "failed"
    };

    let mut findings = HashMap::new();
    findings.insert("sanctions".to_string(), "clear".to_string());
    findings.insert("pep".to_string(), "clear".to_string());
    findings.insert("adverse_media".to_string(), "clear".to_string());

    let mut recommendations = HashMap::new();
    if risk_score < 0.3 {
        recommendations.insert("monitoring".to_string(), "low".to_string());
        recommendations.insert("frequency".to_string(), "annual".to_string());
    } else if risk_score < 0.7 {
        recommendations.insert("monitoring".to_string(), "medium".to_string());
        recommendations.insert("frequency".to_string(), "quarterly".to_string());
    } else {
        recommendations.insert("monitoring".to_string(), "high".to_string());
        recommendations.insert("frequency".to_string(), "monthly".to_string());
    }

    let mut result = HashMap::new();
    result.insert("status".to_string(), Value::String(status.to_string()));
    result.insert("check_id".to_string(), Value::String(check_id));
    result.insert("risk_score".to_string(), Value::Float(risk_score));
    result.insert("provider".to_string(), Value::String(provider_id));
    result.insert("check_type".to_string(), Value::String(check_type));
    result.insert("timestamp".to_string(), Value::Int(timestamp));
    result.insert("expires_at".to_string(), Value::Int(expires_at));
    result.insert(
        "accuracy".to_string(),
        Value::Float(aml_check_type.accuracy),
    );

    // Add findings and recommendations
    result.insert(
        "findings".to_string(),
        Value::String(format!("{:?}", findings)),
    );
    result.insert(
        "recommendations".to_string(),
        Value::String(format!("{:?}", recommendations)),
    );

    result
}

pub fn get_check_status(check_id: String) -> HashMap<String, Value> {
    crate::stdlib::log::audit(
        "aml_status_check",
        {
            let mut data = HashMap::new();
            data.insert("check_id".to_string(), Value::String(check_id.clone()));
            data.insert(
                "timestamp".to_string(),
                Value::Int(chain::get_block_timestamp(1)),
            );
            data
        },
        Some("aml"),
    );

    // Simulate status check
    let mut result = HashMap::new();
    result.insert("check_id".to_string(), Value::String(check_id));
    result.insert("status".to_string(), Value::String("passed".to_string()));
    result.insert("is_valid".to_string(), Value::Bool(true));
    result.insert(
        "expires_at".to_string(),
        Value::Int(chain::get_block_timestamp(1) + 365 * 24 * 60 * 60),
    );

    result
}

pub fn get_provider_info(provider_id: String) -> HashMap<String, Value> {
    if !AML_PROVIDERS.contains_key(&provider_id) {
        return {
            let mut result = HashMap::new();
            result.insert(
                "error".to_string(),
                Value::String("Provider not found".to_string()),
            );
            result
        };
    }

    let provider = AML_PROVIDERS.get(&provider_id).unwrap();
    let mut result = HashMap::new();
    result.insert("id".to_string(), Value::String(provider.id.clone()));
    result.insert("name".to_string(), Value::String(provider.name.clone()));
    result.insert(
        "success_rate".to_string(),
        Value::Float(provider.success_rate),
    );
    result.insert(
        "response_time".to_string(),
        Value::Int(provider.response_time),
    );
    result.insert("is_active".to_string(), Value::Bool(provider.is_active));

    // Add compliance standards
    let mut standards = HashMap::new();
    for (standard, compliant) in &provider.compliance_standards {
        standards.insert(standard.clone(), Value::Bool(*compliant));
    }
    result.insert(
        "compliance_standards".to_string(),
        Value::String(format!("{:?}", standards)),
    );

    result
}

pub fn list_providers() -> Vec<String> {
    AML_PROVIDERS.keys().cloned().collect()
}

pub fn get_check_types(provider_id: String) -> HashMap<String, Value> {
    if !AML_PROVIDERS.contains_key(&provider_id) {
        return {
            let mut result = HashMap::new();
            result.insert(
                "error".to_string(),
                Value::String("Provider not found".to_string()),
            );
            result
        };
    }

    let provider = AML_PROVIDERS.get(&provider_id).unwrap();
    let mut result = HashMap::new();

    for (check_type_name, check_type) in &provider.check_types {
        let mut type_info = HashMap::new();
        type_info.insert("cost".to_string(), Value::Int(check_type.cost));
        type_info.insert("check_time".to_string(), Value::Int(check_type.check_time));
        type_info.insert("accuracy".to_string(), Value::Float(check_type.accuracy));

        result.insert(
            check_type_name.clone(),
            Value::String(format!("{:?}", type_info)),
        );
    }

    result
}

pub fn screen_transaction(
    from_address: String,
    to_address: String,
    amount: i64,
    _transaction_data: HashMap<String, String>,
) -> HashMap<String, Value> {
    crate::stdlib::log::audit(
        "aml_transaction_screening",
        {
            let mut data = HashMap::new();
            data.insert(
                "from_address".to_string(),
                Value::String(from_address.clone()),
            );
            data.insert("to_address".to_string(), Value::String(to_address.clone()));
            data.insert("amount".to_string(), Value::Int(amount));
            data.insert(
                "timestamp".to_string(),
                Value::Int(chain::get_block_timestamp(1)),
            );
            data
        },
        Some("aml"),
    );

    let risk_score = fetch_risk_score_transaction(&from_address, &to_address, amount);
    let status = if risk_score < 0.3 {
        "approved"
    } else if risk_score < 0.7 {
        "review"
    } else {
        "rejected"
    };

    let mut result = HashMap::new();
    result.insert("status".to_string(), Value::String(status.to_string()));
    result.insert("risk_score".to_string(), Value::Float(risk_score));
    result.insert(
        "screening_id".to_string(),
        Value::String(format!("screen_{}", chain::get_block_timestamp(1))),
    );
    result.insert(
        "recommendation".to_string(),
        Value::String(get_recommendation(risk_score)),
    );

    result
}

pub fn monitor_address(address: String, monitoring_level: String) -> HashMap<String, Value> {
    crate::stdlib::log::audit(
        "aml_address_monitoring",
        {
            let mut data = HashMap::new();
            data.insert("address".to_string(), Value::String(address.clone()));
            data.insert(
                "monitoring_level".to_string(),
                Value::String(monitoring_level.clone()),
            );
            data.insert(
                "timestamp".to_string(),
                Value::Int(chain::get_block_timestamp(1)),
            );
            data
        },
        Some("aml"),
    );

    let monitoring_id = format!("monitor_{}_{}", address, chain::get_block_timestamp(1));
    let risk_score = fetch_risk_score_address(&address);

    let mut result = HashMap::new();
    result.insert("monitoring_id".to_string(), Value::String(monitoring_id));
    result.insert("address".to_string(), Value::String(address));
    result.insert(
        "monitoring_level".to_string(),
        Value::String(monitoring_level),
    );
    result.insert("risk_score".to_string(), Value::Float(risk_score));
    result.insert("status".to_string(), Value::String("active".to_string()));
    result.insert(
        "created_at".to_string(),
        Value::Int(chain::get_block_timestamp(1)),
    );

    result
}

pub fn get_risk_assessment(
    user_address: String,
    transaction_history: HashMap<String, i64>,
) -> HashMap<String, Value> {
    crate::stdlib::log::audit(
        "aml_risk_assessment",
        {
            let mut data = HashMap::new();
            data.insert(
                "user_address".to_string(),
                Value::String(user_address.clone()),
            );
            data.insert(
                "timestamp".to_string(),
                Value::Int(chain::get_block_timestamp(1)),
            );
            data
        },
        Some("aml"),
    );

    // Simulate comprehensive risk assessment
    let overall_risk = calculate_overall_risk(&user_address, &transaction_history);
    let risk_category = get_risk_category(overall_risk);

    let mut result = HashMap::new();
    result.insert("user_address".to_string(), Value::String(user_address));
    result.insert("overall_risk".to_string(), Value::Float(overall_risk));
    result.insert("risk_category".to_string(), Value::String(risk_category));
    result.insert(
        "assessment_id".to_string(),
        Value::String(format!("risk_{}", chain::get_block_timestamp(1))),
    );
    result.insert(
        "recommendations".to_string(),
        Value::String(get_risk_recommendations(overall_risk)),
    );

    result
}

pub fn check_sanctions_list(
    user_address: String,
    _user_data: HashMap<String, String>,
) -> HashMap<String, Value> {
    crate::stdlib::log::audit(
        "aml_sanctions_check",
        {
            let mut data = HashMap::new();
            data.insert(
                "user_address".to_string(),
                Value::String(user_address.clone()),
            );
            data.insert(
                "timestamp".to_string(),
                Value::Int(chain::get_block_timestamp(1)),
            );
            data
        },
        Some("aml"),
    );

    // Simulate sanctions list check
    let mut result = HashMap::new();
    result.insert("user_address".to_string(), Value::String(user_address));
    result.insert(
        "sanctions_status".to_string(),
        Value::String("clear".to_string()),
    );
    result.insert(
        "ofac_status".to_string(),
        Value::String("clear".to_string()),
    );
    result.insert(
        "eu_sanctions_status".to_string(),
        Value::String("clear".to_string()),
    );
    result.insert(
        "uk_sanctions_status".to_string(),
        Value::String("clear".to_string()),
    );
    result.insert(
        "check_id".to_string(),
        Value::String(format!("sanctions_{}", chain::get_block_timestamp(1))),
    );

    result
}

// Helper functions

fn generate_risk_score(check_type: &str, _user_data: &HashMap<String, String>) -> f64 {
    // Simulate risk score generation based on check type and user data
    match check_type {
        "sanctions" => 0.1,        // Low risk for sanctions
        "pep" => 0.15,             // Low risk for PEP
        "adverse_media" => 0.2,    // Low risk for adverse media
        "risk_assessment" => 0.25, // Low risk for comprehensive assessment
        _ => 0.3,                  // Default low risk
    }
}

/// Fetch risk score: when AML_API_URL set and http-interface, call API; else use local calculation.
#[cfg(feature = "http-interface")]
fn fetch_risk_score_transaction(from_address: &str, to_address: &str, amount: i64) -> f64 {
    if let Ok(url) = env::var("AML_API_URL") {
        let body = serde_json::json!({
            "from_address": from_address,
            "to_address": to_address,
            "amount": amount,
        });
        if let Ok(client) = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
        {
            let mut req = client.post(format!("{}/screen/transaction", url.trim_end_matches('/')));
            if let Ok(key) = env::var("AML_API_KEY") {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
            if let Ok(resp) = req.json(&body).send() {
                if resp.status().is_success() {
                    if let Ok(json) = resp.json::<serde_json::Value>() {
                        if let Some(score) = json.get("risk_score").and_then(|v| v.as_f64()) {
                            return score.min(1.0).max(0.0);
                        }
                    }
                }
            }
        }
    }
    calculate_transaction_risk(from_address, to_address, amount)
}

#[cfg(not(feature = "http-interface"))]
fn fetch_risk_score_transaction(from_address: &str, to_address: &str, amount: i64) -> f64 {
    calculate_transaction_risk(from_address, to_address, amount)
}

/// Fetch risk score: when AML_API_URL set and http-interface, call API; else use local calculation.
#[cfg(feature = "http-interface")]
fn fetch_risk_score_address(address: &str) -> f64 {
    if let Ok(url) = env::var("AML_API_URL") {
        let body = serde_json::json!({ "address": address });
        if let Ok(client) = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
        {
            let mut req = client.post(format!("{}/screen/address", url.trim_end_matches('/')));
            if let Ok(key) = env::var("AML_API_KEY") {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
            if let Ok(resp) = req.json(&body).send() {
                if resp.status().is_success() {
                    if let Ok(json) = resp.json::<serde_json::Value>() {
                        if let Some(score) = json.get("risk_score").and_then(|v| v.as_f64()) {
                            return score.min(1.0).max(0.0);
                        }
                    }
                }
            }
        }
    }
    calculate_address_risk(address)
}

#[cfg(not(feature = "http-interface"))]
fn fetch_risk_score_address(address: &str) -> f64 {
    calculate_address_risk(address)
}

fn calculate_transaction_risk(from_address: &str, to_address: &str, amount: i64) -> f64 {
    // Simulate transaction risk calculation
    let mut risk: f64 = 0.1;

    // Higher amounts = higher risk
    if amount > 100000 {
        risk += 0.3;
    } else if amount > 10000 {
        risk += 0.2;
    }

    // Address-based risk (simplified)
    if from_address.contains("suspicious") || to_address.contains("suspicious") {
        risk += 0.4;
    }

    risk.min(1.0)
}

fn calculate_address_risk(address: &str) -> f64 {
    // Simulate address risk calculation
    if address.contains("high_risk") {
        0.8
    } else if address.contains("medium_risk") {
        0.5
    } else {
        0.2
    }
}

fn calculate_overall_risk(user_address: &str, transaction_history: &HashMap<String, i64>) -> f64 {
    // Simulate overall risk calculation
    let mut risk: f64 = 0.1;

    // Transaction frequency risk
    if transaction_history.len() > 100 {
        risk += 0.2;
    }

    // Transaction amount risk
    let total_volume: i64 = transaction_history.values().sum();
    if total_volume > 1000000 {
        risk += 0.3;
    }

    // Address-specific risk
    if user_address.contains("high_risk") {
        risk += 0.4;
    }

    risk.min(1.0)
}

fn get_risk_category(risk_score: f64) -> String {
    if risk_score < 0.3 {
        "low".to_string()
    } else if risk_score < 0.7 {
        "medium".to_string()
    } else {
        "high".to_string()
    }
}

fn get_recommendation(risk_score: f64) -> String {
    if risk_score < 0.3 {
        "approve".to_string()
    } else if risk_score < 0.7 {
        "review".to_string()
    } else {
        "reject".to_string()
    }
}

fn get_risk_recommendations(risk_score: f64) -> String {
    if risk_score < 0.3 {
        "Standard monitoring".to_string()
    } else if risk_score < 0.7 {
        "Enhanced monitoring, quarterly reviews".to_string()
    } else {
        "High monitoring, monthly reviews, additional due diligence".to_string()
    }
}
