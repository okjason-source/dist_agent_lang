use crate::runtime::values::Value;
use std::collections::HashMap;

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

/// Push data to target location
pub fn push(_data: HashMap<String, Value>, target: SyncTarget) -> Result<bool, String> {
    // Mock implementation - in real system this would transfer data
    match target.protocol.as_str() {
        "http" => {
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
        _ => Err(format!("Unsupported protocol: {}", target.protocol))
    }
}

/// Pull data from source
pub fn pull(source: &str, filters: SyncFilters) -> Result<(HashMap<String, Value>, bool), String> {
    // Mock implementation - in real system this would fetch data
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
        _ => Err(format!("Unknown source: {}", source))
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
