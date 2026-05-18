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
            m.iter()
                .map(|(k, v)| (k.clone(), value_to_serde_json(v)))
                .collect(),
        ),
        Value::Struct(_, m) => serde_json::Value::Object(
            m.iter()
                .map(|(k, v)| (k.clone(), value_to_serde_json(v)))
                .collect(),
        ),
        Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(value_to_serde_json).collect())
        }
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
        serde_json::Value::Array(arr) => Value::List(arr.iter().map(serde_json_to_value).collect()),
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
            data.insert(
                "user_123".to_string(),
                Value::String("John Doe".to_string()),
            );
            data.insert(
                "user_456".to_string(),
                Value::String("Jane Smith".to_string()),
            );

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_fallback_protocol_matrix() {
        let mut data = HashMap::new();
        data.insert("k".to_string(), Value::Int(1));

        let ftp_ok = create_sync_target("ftp.example.com/data".to_string(), "ftp".to_string());
        assert_eq!(push(data.clone(), ftp_ok).unwrap(), true);

        let ftp_bad = create_sync_target("ftp.invalid.local/data".to_string(), "ftp".to_string());
        assert!(push(data.clone(), ftp_bad)
            .unwrap_err()
            .contains("FTP push failed"));

        let s3_ok = create_sync_target(
            "s3.amazonaws.com/example-bucket".to_string(),
            "s3".to_string(),
        );
        assert_eq!(push(data.clone(), s3_ok).unwrap(), true);

        let s3_bad = create_sync_target("s3.invalid.local/bucket".to_string(), "s3".to_string());
        assert!(push(data.clone(), s3_bad)
            .unwrap_err()
            .contains("S3 push failed"));

        let unsupported = create_sync_target("wherever".to_string(), "gopher".to_string());
        assert!(push(data, unsupported)
            .unwrap_err()
            .contains("Unsupported protocol"));
    }

    #[test]
    fn test_pull_database_and_api_filters() {
        let (db_all, complete_all) = pull("database", create_sync_filters()).unwrap();
        assert_eq!(complete_all, true);
        assert!(matches!(
            db_all.get("user_123"),
            Some(Value::String(v)) if v == "John Doe"
        ));
        assert!(matches!(
            db_all.get("user_456"),
            Some(Value::String(v)) if v == "Jane Smith"
        ));

        let users_only = create_sync_filters().with_data_type("users".to_string());
        let (db_users, complete_users) = pull("database", users_only).unwrap();
        assert_eq!(complete_users, true);
        assert_eq!(db_users.len(), 2);

        let bad_filter = create_sync_filters().with_data_type("orders".to_string());
        assert!(pull("database", bad_filter)
            .unwrap_err()
            .contains("Data type filter not supported"));

        let (api_data, complete_api) = pull("api", create_sync_filters()).unwrap();
        assert_eq!(complete_api, true);
        assert!(matches!(api_data.get("price_btc"), Some(Value::Int(45000))));
        assert!(matches!(api_data.get("price_eth"), Some(Value::Int(3200))));

        assert!(pull("unknown-source", create_sync_filters())
            .unwrap_err()
            .contains("Unknown source"));
    }

    #[test]
    fn test_builder_and_factory_methods_preserve_values() {
        let mut creds = HashMap::new();
        creds.insert("token".to_string(), "secret".to_string());
        let target = create_sync_target("api.example.com".to_string(), "https".to_string())
            .with_credentials(creds.clone())
            .with_compression(true);
        assert_eq!(target.location, "api.example.com");
        assert_eq!(target.protocol, "https");
        assert_eq!(target.compression, true);
        assert!(matches!(
            target.credentials.as_ref().and_then(|m| m.get("token")),
            Some(v) if v == "secret"
        ));

        let filters = create_sync_filters()
            .with_data_type("users".to_string())
            .with_date_range(10, 20)
            .with_tag("a".to_string())
            .with_tag("b".to_string())
            .with_max_size(99);
        assert_eq!(filters.data_type.as_deref(), Some("users"));
        assert_eq!(filters.date_range, Some((10, 20)));
        assert_eq!(filters.tags, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(filters.max_size, Some(99));
    }
}
