// HTTP Server Security Module
// Provides security controls for the HTTP server

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use std::net::IpAddr;
use chrono;

/// Rate limiter for IP addresses
#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<IpAddr, Vec<Instant>>>>,
    max_requests: usize,
    window_seconds: u64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_seconds,
        }
    }

    /// Check if an IP address is rate limited
    pub async fn check_rate_limit(&self, ip: IpAddr) -> Result<(), StatusCode> {
        let mut requests = self.requests.write().await;
        
        // Clean up old requests
        let now = Instant::now();
        let window = Duration::from_secs(self.window_seconds);
        
        if let Some(timestamps) = requests.get_mut(&ip) {
            timestamps.retain(|&timestamp| now.duration_since(timestamp) < window);
            
            if timestamps.len() >= self.max_requests {
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }
            
            timestamps.push(now);
        } else {
            requests.insert(ip, vec![now]);
        }
        
        Ok(())
    }
}

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';".parse().unwrap(),
    );
    
    // X-Frame-Options
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    
    // X-Content-Type-Options
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    
    // X-XSS-Protection
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    
    // Referrer-Policy
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    
    response
}

/// Input validation utilities
pub struct InputValidator;

impl InputValidator {
    /// Validate string input (prevent injection attacks)
    pub fn validate_string(input: &str, max_length: usize) -> Result<(), String> {
        if input.len() > max_length {
            return Err(format!("Input too long (max {} chars)", max_length));
        }
        
        // Check for SQL injection patterns
        let sql_patterns = vec![
            "'",        // Single quote (dangerous in SQL)
            "\"",       // Double quote (dangerous in SQL)  
            "--",       // SQL comment
            "/*",       // SQL comment start
            "*/",       // SQL comment end
            "xp_",      // SQL Server extended procedures
            "sp_",      // SQL Server stored procedures
            "exec",     // SQL execution command
            "union",    // SQL UNION injection
            "select",   // SQL SELECT statement
            " or ",     // SQL boolean injection
            " and ",    // SQL boolean injection
            "drop ",    // SQL DROP command
            "delete ",  // SQL DELETE command
        ];
        
        let input_lower = input.to_lowercase();
        for pattern in sql_patterns {
            if input_lower.contains(pattern) {
                return Err("Invalid input pattern detected".to_string());
            }
        }
        
        // Check for XSS patterns
        let xss_patterns = vec![
            "<script",
            "javascript:",
            "onerror=",
            "onload=",
            "onclick=",
            "eval(",
        ];
        
        for pattern in xss_patterns {
            if input_lower.contains(pattern) {
                return Err("Invalid input pattern detected".to_string());
            }
        }
        
        Ok(())
    }
    
    /// Validate numeric input
    pub fn validate_number(value: i64, min: i64, max: i64) -> Result<(), String> {
        if value < min || value > max {
            return Err(format!("Value must be between {} and {}", min, max));
        }
        Ok(())
    }
    
    /// Validate address format (Ethereum address)
    pub fn validate_address(address: &str) -> Result<(), String> {
        if !address.starts_with("0x") {
            return Err("Address must start with 0x".to_string());
        }
        
        if address.len() != 42 {
            return Err("Address must be 42 characters".to_string());
        }
        
        if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Address must be hexadecimal".to_string());
        }
        
        Ok(())
    }
    
    /// Sanitize string (remove dangerous characters)
    pub fn sanitize_string(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.' || *c == '@')
            .collect()
    }
}

/// CORS configuration
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["http://localhost:3000".to_string()], // Default: localhost only
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            allow_credentials: false,
        }
    }
}

/// Authentication token validation (placeholder for JWT/API key)
pub struct AuthValidator;

impl AuthValidator {
    /// Validate API key (placeholder - should use proper JWT validation)
    pub fn validate_api_key(_token: &str) -> Result<bool, String> {
        // TODO: Implement proper JWT validation
        // For now, return true for any non-empty token
        Ok(!_token.is_empty())
    }
    
    /// Extract token from Authorization header
    pub fn extract_token(headers: &HeaderMap) -> Option<String> {
        headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| {
                if s.starts_with("Bearer ") {
                    Some(s[7..].to_string())
                } else {
                    None
                }
            })
    }
}

/// Request size limiter
pub struct RequestSizeLimiter {
    pub max_body_size: usize,
    pub max_header_size: usize,
    pub max_url_length: usize,
}

impl Default for RequestSizeLimiter {
    fn default() -> Self {
        Self {
            max_body_size: 1_000_000,      // 1MB
            max_header_size: 8_192,        // 8KB
            max_url_length: 2_048,         // 2KB
        }
    }
}

impl RequestSizeLimiter {
    pub fn validate_request(&self, headers: &HeaderMap, body_size: usize, url_length: usize) -> Result<(), StatusCode> {
        // Check body size
        if body_size > self.max_body_size {
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }
        
        // Check URL length
        if url_length > self.max_url_length {
            return Err(StatusCode::URI_TOO_LONG);
        }
        
        // Check header size (approximate)
        let header_size: usize = headers
            .iter()
            .map(|(name, value)| name.as_str().len() + value.len())
            .sum();
        
        if header_size > self.max_header_size {
            return Err(StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE);
        }
        
        Ok(())
    }
}

/// Security event logger
pub struct SecurityLogger;

impl SecurityLogger {
    /// Log security event
    pub fn log_event(event_type: &str, details: &str, ip: Option<&str>) {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let ip_str = ip.unwrap_or("unknown");
        
        eprintln!(
            "[SECURITY] [{}] {} - IP: {} - Details: {}",
            timestamp, event_type, ip_str, details
        );
        
        // TODO: Send to proper logging system
    }
    
    /// Log rate limit violation
    pub fn log_rate_limit(ip: &str) {
        Self::log_event("RATE_LIMIT", "Too many requests", Some(ip));
    }
    
    /// Log authentication failure
    pub fn log_auth_failure(ip: &str, reason: &str) {
        Self::log_event("AUTH_FAILURE", reason, Some(ip));
    }
    
    /// Log invalid input
    pub fn log_invalid_input(ip: &str, input: &str) {
        Self::log_event("INVALID_INPUT", input, Some(ip));
    }
}

