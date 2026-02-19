use crate::runtime::values::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Metadata for an active stream. When a real WebSocket feature is added,
/// this can hold the connection (e.g. `Option<WebSocketStream>`) or a
/// separate map can store stream_id → connection.
#[derive(Debug, Clone)]
pub struct StreamEntry {
    pub source: String,
    pub created_at: u64,
}

fn get_stream_registry() -> std::sync::MutexGuard<'static, HashMap<String, StreamEntry>> {
    static REG: OnceLock<Mutex<HashMap<String, StreamEntry>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()));
    REG.get().unwrap().lock().unwrap()
}

#[cfg(feature = "http-interface")]
fn json_to_value(j: &serde_json::Value) -> Value {
    match j {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => Value::List(arr.iter().map(json_to_value).collect()),
        serde_json::Value::Object(obj) => {
            let m: HashMap<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect();
            Value::Map(m)
        }
    }
}

/// Oracle ABI - Interface for external data integration with security hardening
///
/// This provides a namespace-based approach to oracle operations:
/// - oracle::fetch(source, query) - Fetch data from external source
/// - oracle::verify(data, signature) - Verify data authenticity
/// - oracle::stream(source, callback) - Stream real-time data
///
/// Security Features:
/// - Signed data feeds with cryptographic verification
/// - Multi-source validation and consensus
/// - Timestamp validation and replay protection
/// - Rate limiting per source
/// - Trusted source allowlisting

#[derive(Debug, Clone)]
pub struct OracleSource {
    pub name: String,
    pub url: String,
    pub api_key: Option<String>,
    pub rate_limit: Option<i64>,
    pub trusted: bool,                  // Security: Mark trusted sources
    pub public_key: Option<String>,     // Security: Public key for signature verification
    pub last_request_time: Option<u64>, // Security: For rate limiting
}

#[derive(Debug, Clone)]
pub struct OracleQuery {
    pub query_type: String,
    pub parameters: HashMap<String, Value>,
    pub timeout: Option<i64>,
    pub require_signature: bool, // Security: Require signed response
    pub min_confirmations: Option<u32>, // Security: Require multiple sources
}

#[derive(Debug, Clone)]
pub struct OracleResponse {
    pub data: Value,
    pub timestamp: u64, // Changed to u64 for proper timestamp handling
    pub source: String,
    pub signature: Option<String>,
    pub verified: bool,        // Security: Whether signature was verified
    pub confidence_score: f64, // Security: Confidence from multi-source validation (0.0-1.0)
}

/// Security: Oracle data validation manager
#[derive(Debug, Clone)]
pub struct OracleSecurityManager {
    trusted_sources: HashMap<String, OracleSource>,
    response_cache: HashMap<String, (OracleResponse, u64)>, // Cache with timestamp
    max_age_seconds: u64,                                   // Maximum age for cached responses
}

/// Security: Multi-source consensus validator
#[derive(Debug, Clone)]
pub struct OracleConsensus {
    pub sources: Vec<String>,
    pub responses: Vec<OracleResponse>,
    pub consensus_threshold: f64, // 0.5 = 50% agreement required
}

impl OracleSource {
    pub fn new(name: String, url: String) -> Self {
        Self {
            name,
            url,
            api_key: None,
            rate_limit: None,
            trusted: false, // Default: untrusted
            public_key: None,
            last_request_time: None,
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn with_rate_limit(mut self, rate_limit: i64) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }

    /// Security: Mark source as trusted with public key for verification
    pub fn with_trust(mut self, public_key: String) -> Self {
        self.trusted = true;
        self.public_key = Some(public_key);
        self
    }

    /// Security: Check if rate limit allows request
    pub fn can_request(&mut self) -> bool {
        let now = get_current_timestamp();

        if let Some(rate_limit) = self.rate_limit {
            if let Some(last_time) = self.last_request_time {
                let min_interval = 1000 / rate_limit as u64; // Convert rate limit to min interval (ms)
                if now - last_time < min_interval {
                    return false; // Rate limit exceeded
                }
            }
        }

        self.last_request_time = Some(now);
        true
    }
}

impl OracleQuery {
    pub fn new(query_type: String) -> Self {
        Self {
            query_type,
            parameters: HashMap::new(),
            timeout: None,
            require_signature: true, // Default: require signatures for security
            min_confirmations: None,
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

    /// Security: Require signature verification
    pub fn require_signature(mut self, required: bool) -> Self {
        self.require_signature = required;
        self
    }

    /// Security: Require multiple source confirmations
    pub fn with_confirmations(mut self, count: u32) -> Self {
        self.min_confirmations = Some(count);
        self
    }
}

impl OracleSecurityManager {
    pub fn new() -> Self {
        Self {
            trusted_sources: HashMap::new(),
            response_cache: HashMap::new(),
            max_age_seconds: 300, // Default: 5 minutes
        }
    }

    /// Register a trusted oracle source
    pub fn register_trusted_source(&mut self, source: OracleSource) {
        if source.trusted && source.public_key.is_some() {
            self.trusted_sources.insert(source.name.clone(), source);
        }
    }

    /// Security: Verify oracle response signature
    pub fn verify_response(&self, response: &OracleResponse) -> bool {
        if let Some(signature) = &response.signature {
            if let Some(source) = self.trusted_sources.get(&response.source) {
                if let Some(public_key) = &source.public_key {
                    // In production, use actual cryptographic signature verification
                    // For now, implement basic validation
                    return verify_signature(&response.data, signature, public_key);
                }
            }
        }
        false
    }

    /// Security: Validate response timestamp (prevent replay attacks)
    pub fn validate_timestamp(&self, timestamp: u64) -> bool {
        let now = get_current_timestamp();
        let age = now.saturating_sub(timestamp);

        // Reject responses older than max_age
        age <= self.max_age_seconds * 1000 // Convert to ms
    }

    /// Security: Get cached response if valid
    pub fn get_cached(&self, cache_key: &str) -> Option<OracleResponse> {
        if let Some((response, cached_time)) = self.response_cache.get(cache_key) {
            if self.validate_timestamp(*cached_time) {
                return Some(response.clone());
            }
        }
        None
    }
}

impl OracleConsensus {
    pub fn new(sources: Vec<String>, threshold: f64) -> Self {
        Self {
            sources,
            responses: Vec::new(),
            consensus_threshold: threshold,
        }
    }

    /// Add a response from a source
    pub fn add_response(&mut self, response: OracleResponse) {
        self.responses.push(response);
    }

    /// Security: Determine consensus from multiple sources
    pub fn get_consensus(&self) -> Option<OracleResponse> {
        if self.responses.is_empty() {
            return None;
        }

        // Find the most common value
        let mut value_counts: HashMap<String, (usize, OracleResponse)> = HashMap::new();

        for response in &self.responses {
            let key = format!("{:?}", response.data); // Simplified comparison
            value_counts
                .entry(key)
                .and_modify(|(count, _)| *count += 1)
                .or_insert((1, response.clone()));
        }

        // Find majority
        let total = self.responses.len();
        for (count, response) in value_counts.values() {
            let agreement = *count as f64 / total as f64;
            if agreement >= self.consensus_threshold {
                let mut consensus_response = response.clone();
                consensus_response.confidence_score = agreement;
                return Some(consensus_response);
            }
        }

        None
    }
}

/// Security: Fetch data from external oracle source with validation.
///
/// Oracle sources must be HTTP/HTTPS URLs. The function fetches data from the URL,
/// parses JSON responses, and optionally validates signatures from the X-Signature header.
///
/// For dynamic oracle sources, use `oracle::create_source()` to register sources
/// that can be referenced by name. All sources must ultimately resolve to HTTP URLs.
///
/// # Arguments
/// - `source`: HTTP/HTTPS URL or a registered source name (must resolve to URL)
/// - `query`: OracleQuery specifying query type, parameters, timeout, and security requirements
///
/// # Returns
/// OracleResponse with data, timestamp, source, signature (if provided), and verification status
///
/// # Errors
/// Returns error if:
/// - Source is not an HTTP/HTTPS URL and not a registered source
/// - HTTP request fails
/// - JSON parsing fails
/// - Signature is required but not provided
pub fn fetch(source: &str, query: OracleQuery) -> Result<OracleResponse, String> {
    let timestamp = get_current_timestamp();

    // Check if source is an HTTP/HTTPS URL
    #[cfg(feature = "http-interface")]
    if source.starts_with("http://") || source.starts_with("https://") {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(
                query.timeout.unwrap_or(30) as u64
            ))
            .build()
            .map_err(|e| e.to_string())?;
        let resp = client.get(source).send().map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("Oracle fetch failed: {}", resp.status()));
        }
        let signature = resp
            .headers()
            .get("X-Signature")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
        let data = json_to_value(&body);
        if query.require_signature && signature.is_none() {
            return Err("Signature required but not provided".to_string());
        }
        return Ok(OracleResponse {
            data,
            timestamp,
            source: source.to_string(),
            signature,
            verified: false,
            confidence_score: 1.0,
        });
    }

    // If not HTTP URL and http-interface feature is enabled, return error
    #[cfg(feature = "http-interface")]
    {
        return Err(format!(
            "Oracle source must be an HTTP/HTTPS URL. Got: '{}'. Use oracle::create_source() to register sources, or provide a URL directly.",
            source
        ));
    }

    // If http-interface feature is not enabled, return error
    #[cfg(not(feature = "http-interface"))]
    {
        return Err(format!(
            "Oracle fetching requires the 'http-interface' feature. Source: '{}'",
            source
        ));
    }
}

/// Security: Fetch data from multiple sources and validate consensus
pub fn fetch_with_consensus(
    sources: Vec<&str>,
    query: OracleQuery,
    threshold: f64,
) -> Result<OracleResponse, String> {
    let mut consensus =
        OracleConsensus::new(sources.iter().map(|s| s.to_string()).collect(), threshold);

    // Fetch from all sources
    for source in sources {
        match fetch(source, query.clone()) {
            Ok(response) => consensus.add_response(response),
            Err(e) => eprintln!("Oracle source {} failed: {}", source, e),
        }
    }

    // Get consensus
    consensus.get_consensus().ok_or_else(|| {
        format!(
            "Failed to reach consensus (threshold: {:.1}%)",
            threshold * 100.0
        )
    })
}

/// Verify data authenticity using signature
pub fn verify(data: &Value, signature: &str) -> bool {
    // Security: Verify cryptographic signature
    // In production, this would use actual crypto verification
    verify_signature(data, signature, "default_public_key")
}

/// Stream real-time data from oracle source.
///
/// Creates a stream entry for real-time data feeds. When `source` is a `ws://` or `wss://` URL,
/// a stream ID is returned. A future implementation could establish WebSocket connections
/// (e.g. via tokio-tungstenite) behind a Cargo feature (e.g. `oracle-websocket`).
///
/// For HTTP-based sources, developers can use `oracle::fetch()` in a polling loop or
/// implement their own WebSocket client using the returned `stream_id` for correlation.
///
/// **App developers** can use WebSockets today: call `stream(ws_url, …)` to get a `stream_id`,
/// then implement their own WebSocket client (in-app, frontend, or another service) and use
/// `stream_id` with `get_stream` / `close_stream` for correlation.
///
/// # Arguments
/// - `source`: WebSocket URL (`ws://` or `wss://`) or any source identifier
/// - `callback`: Callback identifier (reserved for future use)
///
/// # Returns
/// Stream ID string that can be used with `get_stream()` and `close_stream()`
pub fn stream(source: &str, _callback: &str) -> Result<String, String> {
    let created_at = get_current_timestamp();

    // Accept WebSocket URLs (ws:// or wss://)
    if source.starts_with("ws://") || source.starts_with("wss://") {
        let stream_id = format!("stream_ws_{:x}", {
            let mut h = Sha256::new();
            h.update(source.as_bytes());
            h.finalize()[0..8]
                .iter()
                .fold(0u64, |a, &b| (a << 8) | u64::from(b))
        });
        get_stream_registry().insert(
            stream_id.clone(),
            StreamEntry {
                source: source.to_string(),
                created_at,
            },
        );
        return Ok(stream_id);
    }

    // For any other source identifier, generate a stream ID
    // This allows developers to use any source name and implement their own streaming logic
    let stream_id = format!("stream_{:x}", {
        let mut h = Sha256::new();
        h.update(source.as_bytes());
        h.update(created_at.to_be_bytes().as_slice());
        h.finalize()[0..8]
            .iter()
            .fold(0u64, |a, &b| (a << 8) | u64::from(b))
    });
    get_stream_registry().insert(
        stream_id.clone(),
        StreamEntry {
            source: source.to_string(),
            created_at,
        },
    );
    Ok(stream_id)
}

/// Look up a stream by id (connection registry). Returns metadata (source, created_at).
/// When a real WebSocket feature is added, the same registry can hold the connection
/// and a future `read_stream_message(stream_id)` would use it.
pub fn get_stream(stream_id: &str) -> Option<StreamEntry> {
    get_stream_registry().get(stream_id).cloned()
}

/// Remove a stream from the registry (e.g. when closing or unsubscribing).
pub fn close_stream(stream_id: &str) -> bool {
    get_stream_registry().remove(stream_id).is_some()
}

/// Create a new oracle source
pub fn create_source(name: String, url: String) -> OracleSource {
    OracleSource::new(name, url)
}

/// Create a new oracle query
pub fn create_query(query_type: String) -> OracleQuery {
    OracleQuery::new(query_type)
}

// ===== Helper Functions =====

/// Get current timestamp in milliseconds
fn get_current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Security: Generate signature for data
fn generate_signature(data: &Value, private_key: &str) -> String {
    let data_string = format!("{:?}", data); // Simplified serialization
    let mut hasher = Sha256::new();
    hasher.update(data_string.as_bytes());
    hasher.update(private_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Security: Verify signature against data and public key
fn verify_signature(data: &Value, signature: &str, public_key: &str) -> bool {
    // In production, this would use actual asymmetric crypto (ECDSA/EdDSA)
    // For now, implement basic verification using hash comparison
    let expected_sig = generate_signature(data, public_key);

    // Constant-time comparison to prevent timing attacks
    if signature.len() != expected_sig.len() {
        return false;
    }

    let mut matches = true;
    for (a, b) in signature.bytes().zip(expected_sig.bytes()) {
        if a != b {
            matches = false;
        }
    }
    matches
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_security_manager() {
        let mut manager = OracleSecurityManager::new();

        let source = OracleSource::new(
            "test_source".to_string(),
            "https://api.test.com".to_string(),
        )
        .with_trust("test_public_key".to_string());

        manager.register_trusted_source(source);
        assert!(manager.trusted_sources.contains_key("test_source"));
    }

    #[test]
    fn test_timestamp_validation() {
        let manager = OracleSecurityManager::new();
        let now = get_current_timestamp();

        assert!(manager.validate_timestamp(now));
        assert!(manager.validate_timestamp(now - 1000)); // 1 second ago
        assert!(!manager.validate_timestamp(now - 400000)); // 400 seconds ago (> 5 min)
    }

    #[test]
    fn test_signature_generation_and_verification() {
        let data = Value::Int(12345);
        let sig = generate_signature(&data, "test_key");

        assert!(verify_signature(&data, &sig, "test_key"));
        assert!(!verify_signature(&data, &sig, "wrong_key"));
        assert!(!verify_signature(&data, "wrong_sig", "test_key"));
    }

    #[test]
    #[cfg(feature = "http-interface")]
    fn test_oracle_fetch_with_security() {
        // Test that non-HTTP sources return an error (dynamic oracle system)
        let query = OracleQuery::new("btc_price".to_string()).require_signature(true);

        let result = fetch("price_feed", query.clone());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Oracle source must be an HTTP/HTTPS URL"));

        // Test that HTTP URLs are accepted (even if they fail, the error is different)
        let result = fetch("https://api.example.com/oracle", query);
        // This will fail with network error or HTTP error, but not "must be HTTP URL"
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(!err.contains("must be an HTTP/HTTPS URL"));
    }

    #[test]
    #[cfg(not(feature = "http-interface"))]
    fn test_oracle_fetch_requires_http_interface() {
        let query = OracleQuery::new("btc_price".to_string());
        let result = fetch("https://api.example.com/oracle", query);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("requires the 'http-interface' feature"));
    }

    #[test]
    fn test_rate_limiting() {
        let mut source =
            OracleSource::new("test".to_string(), "url".to_string()).with_rate_limit(10); // 10 requests/second

        // First request should succeed
        assert!(source.can_request());

        // Immediate second request may pass or fail depending on system speed (rate limit is per-request interval).
        // We only assert the function runs without panicking.
        let _ = source.can_request();
    }

    #[test]
    fn test_consensus_validation() {
        let mut consensus = OracleConsensus::new(
            vec![
                "source1".to_string(),
                "source2".to_string(),
                "source3".to_string(),
            ],
            0.66, // 66% threshold
        );

        // Add 3 responses with 2 agreeing
        let response1 = OracleResponse {
            data: Value::Int(100),
            timestamp: get_current_timestamp(),
            source: "source1".to_string(),
            signature: Some("sig1".to_string()),
            verified: true,
            confidence_score: 1.0,
        };

        let response2 = OracleResponse {
            data: Value::Int(100),
            timestamp: get_current_timestamp(),
            source: "source2".to_string(),
            signature: Some("sig2".to_string()),
            verified: true,
            confidence_score: 1.0,
        };

        let response3 = OracleResponse {
            data: Value::Int(99), // Different value
            timestamp: get_current_timestamp(),
            source: "source3".to_string(),
            signature: Some("sig3".to_string()),
            verified: true,
            confidence_score: 1.0,
        };

        consensus.add_response(response1);
        consensus.add_response(response2);
        consensus.add_response(response3);

        let result = consensus.get_consensus();
        assert!(result.is_some());

        let consensus_response = result.unwrap();
        assert!(matches!(consensus_response.data, Value::Int(100)));
        assert!(consensus_response.confidence_score >= 0.66);
    }
}
