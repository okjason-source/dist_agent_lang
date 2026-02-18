// HTTP Server Security Module
// Provides security controls for the HTTP server

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

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
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';"
            .parse()
            .unwrap(),
    );

    // X-Frame-Options
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());

    // X-Content-Type-Options
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());

    // X-XSS-Protection
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());

    // Referrer-Policy
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

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
            "'",       // Single quote (dangerous in SQL)
            "\"",      // Double quote (dangerous in SQL)
            "--",      // SQL comment
            "/*",      // SQL comment start
            "*/",      // SQL comment end
            "xp_",     // SQL Server extended procedures
            "sp_",     // SQL Server stored procedures
            "exec",    // SQL execution command
            "union",   // SQL UNION injection
            "select",  // SQL SELECT statement
            " or ",    // SQL boolean injection
            " and ",   // SQL boolean injection
            "drop ",   // SQL DROP command
            "delete ", // SQL DELETE command
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

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,              // Subject (user ID)
    pub exp: usize,               // Expiration time (timestamp)
    pub iat: usize,               // Issued at (timestamp)
    pub roles: Vec<String>,       // User roles
    pub permissions: Vec<String>, // User permissions
}

impl Claims {
    /// Create new claims with expiration
    pub fn new(
        user_id: String,
        roles: Vec<String>,
        permissions: Vec<String>,
        exp_hours: i64,
    ) -> Self {
        let now = chrono::Utc::now();
        let exp = (now + chrono::Duration::hours(exp_hours)).timestamp() as usize;
        let iat = now.timestamp() as usize;

        Self {
            sub: user_id,
            exp,
            iat,
            roles,
            permissions,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp() as usize;
        self.exp < now
    }
}

/// JWT Configuration
pub struct JwtConfig {
    secret: String,
    algorithm: Algorithm,
    expiration_hours: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            // Require JWT_SECRET in production; empty when unset (callers must check)
            secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| String::new()),
            algorithm: Algorithm::HS256,
            expiration_hours: 24, // 24 hours default
        }
    }
}

impl JwtConfig {
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            algorithm: Algorithm::HS256,
            expiration_hours: 24,
        }
    }

    pub fn with_expiration(mut self, hours: i64) -> Self {
        self.expiration_hours = hours;
        self
    }
}

/// Authentication token validation with JWT support
pub struct AuthValidator {
    config: JwtConfig,
}

impl Default for AuthValidator {
    fn default() -> Self {
        Self {
            config: JwtConfig::default(),
        }
    }
}

impl AuthValidator {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    /// Validate JWT token
    pub fn validate_api_key(&self, token: &str) -> Result<Claims, String> {
        if token.is_empty() {
            return Err("Empty token provided".to_string());
        }

        // Decode and validate JWT
        let decoding_key = DecodingKey::from_secret(self.config.secret.as_bytes());
        let mut validation = Validation::new(self.config.algorithm);
        validation.validate_exp = true; // Validate expiration
        validation.set_required_spec_claims(&["exp"]); // Reject tokens with malformed/missing exp (CVE-2026-25537)

        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                // Additional expiration check
                if token_data.claims.is_expired() {
                    return Err("Token has expired".to_string());
                }
                Ok(token_data.claims)
            }
            Err(e) => Err(format!("Invalid JWT token: {:?}", e)),
        }
    }

    /// Generate JWT token for a user (for testing/auth endpoints)
    pub fn generate_token(
        &self,
        user_id: String,
        roles: Vec<String>,
        permissions: Vec<String>,
    ) -> Result<String, String> {
        let claims = Claims::new(user_id, roles, permissions, self.config.expiration_hours);

        let encoding_key = EncodingKey::from_secret(self.config.secret.as_bytes());

        encode(&Header::new(self.config.algorithm), &claims, &encoding_key)
            .map_err(|e| format!("Failed to generate token: {:?}", e))
    }

    /// Validate token and check for specific role
    pub fn validate_role(&self, token: &str, required_role: &str) -> Result<bool, String> {
        let claims = self.validate_api_key(token)?;
        Ok(claims.roles.contains(&required_role.to_string()))
    }

    /// Validate token and check for specific permission
    pub fn validate_permission(
        &self,
        token: &str,
        required_permission: &str,
    ) -> Result<bool, String> {
        let claims = self.validate_api_key(token)?;
        Ok(claims
            .permissions
            .contains(&required_permission.to_string()))
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
            max_body_size: 1_000_000, // 1MB
            max_header_size: 8_192,   // 8KB
            max_url_length: 2_048,    // 2KB
        }
    }
}

impl RequestSizeLimiter {
    pub fn validate_request(
        &self,
        headers: &HeaderMap,
        body_size: usize,
        url_length: usize,
    ) -> Result<(), StatusCode> {
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

/// Security event logger with structured logging
pub struct SecurityLogger;

impl SecurityLogger {
    /// Log security event with structured logging
    pub fn log_event(event_type: &str, details: &str, ip: Option<&str>) {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let ip_str = ip.unwrap_or("unknown");

        // Use log crate for proper logging (can be configured to send to various backends)
        log::warn!(
            target: "security",
            "[{}] event={} ip={} details={}",
            timestamp, event_type, ip_str, details
        );

        // Also emit to stderr for immediate visibility in development
        eprintln!(
            "[SECURITY] [{}] {} - IP: {} - Details: {}",
            timestamp, event_type, ip_str, details
        );
    }

    /// Log rate limit violation
    pub fn log_rate_limit(ip: &str) {
        Self::log_event("RATE_LIMIT", "Too many requests", Some(ip));
        log::warn!(target: "security::rate_limit", "Rate limit exceeded for IP: {}", ip);
    }

    /// Log authentication failure
    pub fn log_auth_failure(ip: &str, reason: &str) {
        Self::log_event("AUTH_FAILURE", reason, Some(ip));
        log::error!(target: "security::auth", "Authentication failed for IP {}: {}", ip, reason);
    }

    /// Log invalid input
    pub fn log_invalid_input(ip: &str, input: &str) {
        Self::log_event("INVALID_INPUT", input, Some(ip));
        log::warn!(target: "security::input", "Invalid input from IP {}: {}", ip, input);
    }

    /// Log successful authentication (user_id is not logged to avoid cleartext exposure)
    pub fn log_auth_success(ip: &str, _user_id: &str) {
        log::info!(target: "security::auth", "Successful authentication from IP {}", ip);
    }

    /// Log suspicious activity
    pub fn log_suspicious_activity(ip: &str, activity: &str) {
        Self::log_event("SUSPICIOUS_ACTIVITY", activity, Some(ip));
        log::error!(target: "security::suspicious", "Suspicious activity from IP {}: {}", ip, activity);
    }

    /// Log token validation failure
    pub fn log_token_validation_failure(ip: &str, reason: &str) {
        log::warn!(target: "security::token", "Token validation failed from IP {}: {}", ip, reason);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_claims_creation() {
        let claims = Claims::new(
            "user123".to_string(),
            vec!["admin".to_string()],
            vec!["read".to_string(), "write".to_string()],
            24, // 24 hours
        );

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.roles, vec!["admin"]);
        assert_eq!(claims.permissions.len(), 2);
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_jwt_generation_and_validation() {
        let validator = AuthValidator::default();

        // Generate a token
        let token_result = validator.generate_token(
            "user123".to_string(),
            vec!["admin".to_string()],
            vec!["read".to_string(), "write".to_string()],
        );

        assert!(token_result.is_ok());
        let token = token_result.unwrap();

        // Token should not be empty
        assert!(!token.is_empty());

        // Validate the token
        let claims_result = validator.validate_api_key(&token);
        assert!(claims_result.is_ok());

        let claims = claims_result.unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.roles, vec!["admin"]);
    }

    #[test]
    fn test_jwt_role_validation() {
        let validator = AuthValidator::default();

        let token = validator
            .generate_token(
                "user123".to_string(),
                vec!["admin".to_string(), "moderator".to_string()],
                vec![],
            )
            .unwrap();

        // Should have admin role
        let has_admin = validator.validate_role(&token, "admin");
        assert!(has_admin.is_ok());
        assert!(has_admin.unwrap());

        // Should have moderator role
        let has_mod = validator.validate_role(&token, "moderator");
        assert!(has_mod.is_ok());
        assert!(has_mod.unwrap());

        // Should not have user role
        let has_user = validator.validate_role(&token, "user");
        assert!(has_user.is_ok());
        assert!(!has_user.unwrap());
    }

    #[test]
    fn test_jwt_permission_validation() {
        let validator = AuthValidator::default();

        let token = validator
            .generate_token(
                "user123".to_string(),
                vec![],
                vec!["read".to_string(), "write".to_string()],
            )
            .unwrap();

        // Should have read permission
        let has_read = validator.validate_permission(&token, "read");
        assert!(has_read.is_ok());
        assert!(has_read.unwrap());

        // Should not have delete permission
        let has_delete = validator.validate_permission(&token, "delete");
        assert!(has_delete.is_ok());
        assert!(!has_delete.unwrap());
    }

    #[test]
    fn test_jwt_empty_token() {
        let validator = AuthValidator::default();

        let result = validator.validate_api_key("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Empty token"));
    }

    #[test]
    fn test_jwt_invalid_token() {
        let validator = AuthValidator::default();

        let result = validator.validate_api_key("invalid.jwt.token");
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_extract_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer my_token_123".parse().unwrap());

        let token = AuthValidator::extract_token(&headers);
        assert!(token.is_some());
        assert_eq!(token.unwrap(), "my_token_123");
    }

    #[test]
    fn test_jwt_extract_no_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "my_token_123".parse().unwrap());

        let token = AuthValidator::extract_token(&headers);
        assert!(token.is_none());
    }

    #[test]
    fn test_jwt_config_custom() {
        let config = JwtConfig::new("my_secret_key".to_string()).with_expiration(48);

        assert_eq!(config.expiration_hours, 48);
    }

    #[test]
    fn test_rate_limiter() {
        use tokio::runtime::Runtime;

        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let limiter = RateLimiter::new(3, 60); // 3 requests per 60 seconds
            let ip: IpAddr = "127.0.0.1".parse().unwrap();

            // First 3 requests should succeed
            assert!(limiter.check_rate_limit(ip).await.is_ok());
            assert!(limiter.check_rate_limit(ip).await.is_ok());
            assert!(limiter.check_rate_limit(ip).await.is_ok());

            // 4th request should be rate limited
            let result = limiter.check_rate_limit(ip).await;
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), StatusCode::TOO_MANY_REQUESTS);
        });
    }
}
