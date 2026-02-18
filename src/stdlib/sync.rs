use crate::runtime::values::Value;
use std::collections::HashMap;

#[cfg(feature = "http-interface")]
fn value_to_serde_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Int(n) => serde_json::json!(n),
        Value::Float(f) => serde_json::json!(f),
        Value::String(s) => serde_json::json!(s),
        Value::Bool(b) => serde_json::json!(b),
        Value::Null => serde_json::Value::Null,
        Value::List(arr) => serde_json::Value::Array(arr.iter().map(value_to_serde_json).collect()),
        Value::Map(m) => serde_json::Value::Object(
            m.iter().map(|(k, v)| (k.clone(), value_to_serde_json(v))).collect(),
        ),
        Value::Struct(_, m) => serde_json::Value::Object(
            m.iter().map(|(k, v)| (k.clone(), value_to_serde_json(v))).collect(),
        ),
        Value::Array(arr) => serde_json::Value::Array(arr.iter().map(value_to_serde_json).collect()),
        _ => serde_json::Value::String(v.to_string()),
    }
}

#[cfg(feature = "http-interface")]
fn serde_json_to_value(j: &serde_json::Value) -> Value {
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
        serde_json::Value::Array(arr) => {
            Value::List(arr.iter().map(serde_json_to_value).collect())
        }
        serde_json::Value::Object(obj) => {
            let m: HashMap<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), serde_json_to_value(v)))
                .collect();
            Value::Map(m)
        }
    }
}

/// Sync ABI - Interface for data synchronization
/// 
/// This provides a namespace-based approach to sync operations:
/// - sync::push(data, target) - Push data to target location
/// - sync::pull(source, filters) - Pull data from source

#[derive(Debug, Clone)]
pub struct SyncTarget {
    pub location: String,
    pub protocol: String,
    pub credentials: Option<HashMap<String, String>>,
    pub compression: bool,
}

#[derive(Debug, Clone)]
pub struct SyncFilters {
    pub data_type: Option<String>,
    pub date_range: Option<(i64, i64)>,
    pub tags: Vec<String>,
    pub max_size: Option<i64>,
}

impl SyncTarget {
    pub fn new(location: String, protocol: String) -> Self {
        Self {
            location,
            protocol,
            credentials: None,
            compression: false,
        }
    }
    
    pub fn with_credentials(mut self, credentials: HashMap<String, String>) -> Self {
        self.credentials = Some(credentials);
        self
    }
    
    pub fn with_compression(mut self, compression: bool) -> Self {
        self.compression = compression;
        self
    }
}

impl SyncFilters {
    pub fn new() -> Self {
        Self {
            data_type: None,
            date_range: None,
            tags: Vec::new(),
            max_size: None,
        }
    }
    
    pub fn with_data_type(mut self, data_type: String) -> Self {
        self.data_type = Some(data_type);
        self
    }
    
    pub fn with_date_range(mut self, start: i64, end: i64) -> Self {
        self.date_range = Some((start, end));
        self
    }
    
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }
    
    pub fn with_max_size(mut self, max_size: i64) -> Self {
        self.max_size = Some(max_size);
        self
    }
}

impl Default for SyncFilters {
    fn default() -> Self {
        Self::new()
    }
}

/// Push data to target location. When http-interface and protocol is http/https, POSTs data as JSON.
pub fn push(data: HashMap<String, Value>, target: SyncTarget) -> Result<bool, String> {
    #[cfg(feature = "http-interface")]
    if target.protocol == "http" || target.protocol == "https" {
        let body = serde_json::Value::Object(
            data.iter()
                .map(|(k, v)| (k.clone(), value_to_serde_json(v)))
                .collect(),
        );
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;
        let resp = client
            .post(&target.location)
            .json(&body)
            .send()
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            return Ok(true);
        }
        return Err(format!("HTTP push failed: {}", resp.status()));
    }

    // Fallback: mock by protocol
    match target.protocol.as_str() {
        "http" | "https" => {
            if target.location.contains("api.example.com") {
                Ok(true)
            } else {
                Err("HTTP push failed: Invalid endpoint".to_string())
            }
        }
        "ftp" => {
            if target.location.contains("ftp.example.com") {
                Ok(true)
            } else {
                Err("FTP push failed: Invalid endpoint".to_string())
            }
        }
        "s3" => {
            if target.location.contains("s3.amazonaws.com") {
                Ok(true)
            } else {
                Err("S3 push failed: Invalid endpoint".to_string())
            }
        }
        _ => Err(format!("Unsupported protocol: {}", target.protocol)),
    }
}

/// Pull data from source. When http-interface and source is a URL (http/https), GETs and parses JSON to map.
pub fn pull(source: &str, filters: SyncFilters) -> Result<(HashMap<String, Value>, bool), String> {
    #[cfg(feature = "http-interface")]
    if source.starts_with("http://") || source.starts_with("https://") {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;
        let resp = client.get(source).send().map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP pull failed: {}", resp.status()));
        }
        let json: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
        let value = serde_json_to_value(&json);
        if let Value::Map(m) = value {
            return Ok((m, true));
        }
        // Single value: wrap in a map
        let mut data = HashMap::new();
        data.insert("data".to_string(), value);
        return Ok((data, true));
    }

    // Fallback: mock sources
    match source {
        "database" => {
            let mut data = HashMap::new();
            data.insert("user_123".to_string(), Value::String("John Doe".to_string()));
            data.insert("user_456".to_string(), Value::String("Jane Smith".to_string()));

            if let Some(data_type) = &filters.data_type {
                if data_type == "users" {
                    Ok((data, true))
                } else {
                    Err("Data type filter not supported".to_string())
                }
            } else {
                Ok((data, true))
            }
        }
        "api" => {
            let mut data = HashMap::new();
            data.insert("price_btc".to_string(), Value::Int(45000));
            data.insert("price_eth".to_string(), Value::Int(3200));

            Ok((data, true))
        }
        _ => Err(format!("Unknown source: {}", source)),
    }
}

/// Create a new sync target
pub fn create_sync_target(location: String, protocol: String) -> SyncTarget {
    SyncTarget::new(location, protocol)
}

/// Create new sync filters
pub fn create_sync_filters() -> SyncFilters {
    SyncFilters::new()
}
