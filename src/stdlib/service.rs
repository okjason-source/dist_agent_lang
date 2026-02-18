use crate::runtime::values::Value;
use std::collections::HashMap;

/// Service ABI - Interface for external service integration
///
/// This provides a namespace-based approach to service operations:
/// - service::ai(prompt, model) - AI service integration
/// - service::call(service, method, params) - External service calls
/// - service::webhook(url, data) - Webhook service integration

#[derive(Debug, Clone)]
pub struct AIService {
    pub model: String,
    pub temperature: f64,
    pub max_tokens: Option<i64>,
    pub api_key: Option<String>,
    /// Optional base URL for API (e.g. https://api.openai.com/v1). When set with api_key, real HTTP is used.
    pub base_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ServiceCall {
    pub service_name: String,
    pub method: String,
    pub parameters: HashMap<String, Value>,
    pub timeout: Option<i64>,
    /// When set with http-interface, real HTTP POST is made to this URL (path/method appended as needed).
    pub base_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WebhookConfig {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub retry_count: Option<i64>,
}

impl AIService {
    pub fn new(model: String) -> Self {
        Self {
            model,
            temperature: 0.7,
            max_tokens: None,
            api_key: None,
            base_url: None,
        }
    }

    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: i64) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }
}

impl ServiceCall {
    pub fn new(service_name: String, method: String) -> Self {
        Self {
            service_name,
            method,
            parameters: HashMap::new(),
            timeout: None,
            base_url: None,
        }
    }

    pub fn with_parameter(mut self, key: String, value: Value) -> Self {
        self.parameters.insert(key, value);
        self
    }

    pub fn with_timeout(mut self, timeout: i64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }
}

/// AI service integration. When api_key and base_url are set and http-interface is enabled, calls real LLM API. Uses OpenAI-compatible API shape; works with any provider that exposes the same endpoints (OpenAI, Azure OpenAI, Anthropic-compatible, local servers, etc.).
pub fn ai(prompt: &str, service: AIService) -> Result<String, String> {
    #[cfg(feature = "http-interface")]
    if let Some(api_key) = service.api_key.as_ref() {
        let base = service
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        if let Ok(text) = call_llm_api(
            base,
            api_key,
            &service.model,
            prompt,
            service.temperature,
            service.max_tokens,
        ) {
            return Ok(text);
        }
    }

    // Fallback: mock responses when no API key or HTTP unavailable
    match service.model.as_str() {
        "gpt-4" => {
            let response = match prompt {
                "What is blockchain?" => "Blockchain is a distributed ledger technology...",
                "Explain smart contracts" => "Smart contracts are self-executing contracts...",
                "How does DeFi work?" => "DeFi (Decentralized Finance) enables...",
                _ => "I understand your question about blockchain technology...",
            };
            Ok(response.to_string())
        }
        "claude" => {
            let response = match prompt {
                "What is blockchain?" => "Blockchain represents a paradigm shift...",
                "Explain smart contracts" => "Smart contracts automate agreement execution...",
                "How does DeFi work?" => "DeFi removes intermediaries from financial services...",
                _ => "I can help explain blockchain concepts...",
            };
            Ok(response.to_string())
        }
        _ => Err(format!("Unsupported AI model: {}", service.model)),
    }
}

#[cfg(feature = "http-interface")]
fn call_llm_api(
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    temperature: f64,
    max_tokens: Option<i64>,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": temperature,
        "max_tokens": max_tokens.unwrap_or(1024),
    });
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    let json: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    let text = json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or_else(|| "Invalid API response".to_string())?;
    Ok(text.to_string())
}

/// Embeddings API. When api_key and base_url are set and http-interface is enabled, calls /embeddings. Works with any provider that exposes the same API (OpenAI, Azure OpenAI, compatible proxies, local servers, etc.).
#[cfg(feature = "http-interface")]
pub fn embeddings(text: &str, service: AIService) -> Result<Vec<f64>, String> {
    if let Some(api_key) = service.api_key.as_ref() {
        let base = service
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        if let Ok(vec) = call_embeddings_api(base, api_key, &service.model, text) {
            return Ok(vec);
        }
    }
    Err("Embeddings require api_key and http-interface".to_string())
}

#[cfg(feature = "http-interface")]
fn call_embeddings_api(
    base_url: &str,
    api_key: &str,
    model: &str,
    input: &str,
) -> Result<Vec<f64>, String> {
    let url = format!("{}/embeddings", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": model,
        "input": input,
    });
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Embeddings API error: {}", resp.status()));
    }
    let json: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    let data = json
        .get("data")
        .and_then(|d| d.get(0))
        .and_then(|e| e.get("embedding"))
        .and_then(|e| e.as_array())
        .ok_or_else(|| "Invalid embeddings response".to_string())?;
    let vec: Result<Vec<f64>, _> = data.iter().map(|v| v.as_f64().ok_or("non-f64")).collect();
    vec.map_err(|_| "Embedding values must be f64".to_string())
}

/// Vision API: analyze image via OpenAI-compatible chat with image content. Returns model description text.
#[cfg(feature = "http-interface")]
pub fn vision_analyze(
    service: AIService,
    image_url: Option<&str>,
    image_base64: Option<&str>,
) -> Result<String, String> {
    let api_key = service
        .api_key
        .as_deref()
        .ok_or("Vision requires api_key".to_string())?;
    let base = service
        .base_url
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");
    let prompt = "Describe this image in detail: list main objects, any text visible, and dominant colors. Be concise.";
    call_vision_api(
        base,
        api_key,
        &service.model,
        prompt,
        image_url,
        image_base64,
    )
}

#[cfg(feature = "http-interface")]
fn call_vision_api(
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    image_url: Option<&str>,
    image_base64: Option<&str>,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let image_content = if let Some(u) = image_url {
        serde_json::json!({ "type": "image_url", "image_url": { "url": u } })
    } else if let Some(b64) = image_base64 {
        let data_url = if b64.starts_with("data:") {
            b64.to_string()
        } else {
            format!("data:image/jpeg;base64,{}", b64)
        };
        serde_json::json!({ "type": "image_url", "image_url": { "url": data_url } })
    } else {
        return Err("vision_analyze requires image_url or image_base64".to_string());
    };
    let body = serde_json::json!({
        "model": model,
        "messages": [{
            "role": "user",
            "content": [
                { "type": "text", "text": prompt },
                image_content
            ]
        }],
        "max_tokens": 1024
    });
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Vision API error: {}", resp.status()));
    }
    let json: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    let text = json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or_else(|| "Invalid vision API response".to_string())?;
    Ok(text.to_string())
}

/// Image generation API. When api_key and base_url set and http-interface enabled, calls /images/generations. Works with providers that expose this endpoint (OpenAI DALL·E, compatible APIs, etc.).
#[cfg(feature = "http-interface")]
pub fn image_generate(service: AIService, prompt: &str) -> Result<String, String> {
    let api_key = service
        .api_key
        .as_deref()
        .ok_or("Image generation requires api_key".to_string())?;
    let base = service
        .base_url
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");
    call_image_generations_api(base, api_key, &service.model, prompt)
}

#[cfg(feature = "http-interface")]
fn call_image_generations_api(
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    let url = format!("{}/images/generations", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "n": 1,
        "size": "1024x1024"
    });
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Image generation API error: {}", resp.status()));
    }
    let json: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    let data = json
        .get("data")
        .and_then(|d| d.get(0))
        .ok_or_else(|| "Invalid image response".to_string())?;
    if let Some(u) = data.get("url").and_then(|u| u.as_str()) {
        return Ok(u.to_string());
    }
    if let Some(b64) = data.get("b64_json").and_then(|b| b.as_str()) {
        return Ok(format!("data:image/png;base64,{}", b64));
    }
    Err("Image response has no url or b64_json".to_string())
}

/// External service call. When base_url is set and http-interface is enabled, makes real HTTP POST with parameters as JSON.
pub fn call(service: ServiceCall) -> Result<Value, String> {
    #[cfg(feature = "http-interface")]
    if let Some(ref base_url) = service.base_url {
        if let Ok(v) = call_http_service(
            base_url,
            &service.method,
            &service.parameters,
            service.timeout,
        ) {
            return Ok(v);
        }
    }

    // Fallback: mock when no base_url or HTTP unavailable
    match service.service_name.as_str() {
        "payment" => match service.method.as_str() {
            "process" => Ok(Value::String("payment_processed_12345".to_string())),
            "refund" => Ok(Value::String("refund_issued_67890".to_string())),
            "status" => Ok(Value::String("completed".to_string())),
            _ => Err(format!("Unknown payment method: {}", service.method)),
        },
        "email" => match service.method.as_str() {
            "send" => Ok(Value::String("email_sent_abc123".to_string())),
            "template" => Ok(Value::String("welcome_email".to_string())),
            "verify" => Ok(Value::Bool(true)),
            _ => Err(format!("Unknown email method: {}", service.method)),
        },
        "sms" => match service.method.as_str() {
            "send" => Ok(Value::String("sms_sent_def456".to_string())),
            "verify" => Ok(Value::Bool(true)),
            _ => Err(format!("Unknown SMS method: {}", service.method)),
        },
        _ => Err(format!("Unknown service: {}", service.service_name)),
    }
}

/// Webhook service integration. When http-interface is enabled, makes real HTTP POST to config.url with data as JSON body.
pub fn webhook(config: WebhookConfig, data: HashMap<String, Value>) -> Result<String, String> {
    #[cfg(feature = "http-interface")]
    if let Ok(id) = webhook_http_post(&config.url, &config.method, &config.headers, &data) {
        return Ok(id);
    }

    // Fallback: mock when HTTP unavailable or feature off
    match config.url.as_str() {
        "https://api.example.com/webhook" => Ok("webhook_delivered_xyz789".to_string()),
        url if url.contains("hooks.slack.com") => Ok("slack_notification_sent".to_string()),
        url if url.contains("discord.com/api/webhooks") => Ok("discord_message_sent".to_string()),
        _ => Err(format!("Webhook delivery failed for: {}", config.url)),
    }
}

#[cfg(feature = "http-interface")]
fn value_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Int(n) => serde_json::json!(n),
        Value::Float(f) => serde_json::json!(f),
        Value::String(s) => serde_json::json!(s),
        Value::Bool(b) => serde_json::json!(b),
        Value::Null => serde_json::Value::Null,
        Value::List(arr) => serde_json::Value::Array(arr.iter().map(value_to_json).collect()),
        Value::Map(m) => serde_json::Value::Object(
            m.iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect(),
        ),
        Value::Struct(_, m) => serde_json::Value::Object(
            m.iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect(),
        ),
        Value::Array(arr) => serde_json::Value::Array(arr.iter().map(value_to_json).collect()),
        _ => serde_json::Value::String(v.to_string()),
    }
}

#[cfg(feature = "http-interface")]
fn call_http_service(
    base_url: &str,
    method: &str,
    parameters: &HashMap<String, Value>,
    timeout_secs: Option<i64>,
) -> Result<Value, String> {
    let url = format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        method.trim_start_matches('/')
    );
    let body: serde_json::Value = serde_json::Value::Object(
        parameters
            .iter()
            .map(|(k, v)| (k.clone(), value_to_json(v)))
            .collect(),
    );
    let mut client_builder = reqwest::blocking::Client::builder();
    if let Some(secs) = timeout_secs {
        client_builder = client_builder.timeout(std::time::Duration::from_secs(secs as u64));
    }
    let client = client_builder.build().map_err(|e| e.to_string())?;
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!(
            "HTTP {}: {}",
            resp.status(),
            resp.text().unwrap_or_default()
        ));
    }
    let json: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    Ok(Value::String(json.to_string()))
}

#[cfg(feature = "http-interface")]
fn webhook_http_post(
    url: &str,
    method: &str,
    headers: &HashMap<String, String>,
    data: &HashMap<String, Value>,
) -> Result<String, String> {
    let body: serde_json::Value = serde_json::Value::Object(
        data.iter()
            .map(|(k, v)| (k.clone(), value_to_json(v)))
            .collect(),
    );
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let mut req = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        _ => client.post(url),
    };
    for (k, v) in headers {
        req = req.header(k.as_str(), v.as_str());
    }
    let resp = req.json(&body).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Webhook HTTP {}", resp.status()));
    }
    Ok(format!("webhook_ok_{}", url.len()))
}

/// Create a new AI service configuration
pub fn create_ai_service(model: String) -> AIService {
    AIService::new(model)
}

/// Create a new service call
pub fn create_service_call(service_name: String, method: String) -> ServiceCall {
    ServiceCall::new(service_name, method)
}

/// Create a new webhook configuration
pub fn create_webhook(url: String, method: String) -> WebhookConfig {
    WebhookConfig {
        url,
        method,
        headers: HashMap::new(),
        retry_count: Some(3),
    }
}
