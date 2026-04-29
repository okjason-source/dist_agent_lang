// HTTP Server Security Module
// Provides security controls for the HTTP server

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
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

/// Failed HTTP Basic Auth attempts per IP (sliding window). Mitigates online brute force when
/// Basic Auth is enabled for `dal serve`. Configure with **`DAL_HTTP_AUTH_MAX_FAILS_PER_IP`**
/// (default **15**) and **`DAL_HTTP_AUTH_FAIL_WINDOW_SECS`** (default **300**). Set
/// **`DAL_HTTP_AUTH_DISABLE_BRUTE=1`** to disable (tests / special cases only).
#[derive(Debug, Clone)]
pub struct DalServeBasicAuthBruteForce {
    fails: Arc<RwLock<HashMap<IpAddr, Vec<Instant>>>>,
    max_attempts: usize,
    window: Duration,
    disabled: bool,
}

impl DalServeBasicAuthBruteForce {
    pub fn from_env() -> Self {
        let disabled = std::env::var("DAL_HTTP_AUTH_DISABLE_BRUTE")
            .ok()
            .map(|v| {
                matches!(
                    v.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false);
        let max_attempts = std::env::var("DAL_HTTP_AUTH_MAX_FAILS_PER_IP")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|&n| n > 0)
            .unwrap_or(15);
        let window_secs = std::env::var("DAL_HTTP_AUTH_FAIL_WINDOW_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|&n| n > 0)
            .unwrap_or(300);
        Self {
            fails: Arc::new(RwLock::new(HashMap::new())),
            max_attempts,
            window: Duration::from_secs(window_secs),
            disabled,
        }
    }

    /// Returns true when this IP has reached the failure threshold (still within the window).
    pub async fn is_locked_out(&self, ip: IpAddr) -> bool {
        if self.disabled {
            return false;
        }
        let mut guard = self.fails.write().await;
        Self::prune_map(&mut guard, ip, self.window);
        guard
            .get(&ip)
            .map(|v| v.len() >= self.max_attempts)
            .unwrap_or(false)
    }

    pub async fn record_failure(&self, ip: IpAddr) {
        if self.disabled {
            return;
        }
        let mut guard = self.fails.write().await;
        let now = Instant::now();
        let e = guard.entry(ip).or_insert_with(Vec::new);
        e.retain(|t| now.duration_since(*t) < self.window);
        e.push(now);
    }

    pub async fn clear(&self, ip: IpAddr) {
        let mut guard = self.fails.write().await;
        guard.remove(&ip);
    }

    fn prune_map(guard: &mut HashMap<IpAddr, Vec<Instant>>, ip: IpAddr, window: Duration) {
        let now = Instant::now();
        if let Some(v) = guard.get_mut(&ip) {
            v.retain(|t| now.duration_since(*t) < window);
            if v.is_empty() {
                guard.remove(&ip);
            }
        }
    }
}

/// Security headers middleware
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Static header names/values — use `HeaderName`/`HeaderValue` constructors instead of
    // infallible `.parse().unwrap()` on the request path.
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';",
        ),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
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

#[derive(Default)]
/// Authentication token validation with JWT support
pub struct AuthValidator {
    config: JwtConfig,
}

impl AuthValidator {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    /// Validate JWT token.
    /// Requires JWT_SECRET to be set; returns an error if secret is empty (production safety).
    pub fn validate_api_key(&self, token: &str) -> Result<Claims, String> {
        if token.is_empty() {
            return Err("Empty token provided".to_string());
        }
        if self.config.secret.is_empty() {
            return Err(
                "JWT_SECRET is not set; required for authentication. Set JWT_SECRET in production."
                    .to_string(),
            );
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

    /// Generate JWT token for a user (for testing/auth endpoints).
    /// Requires JWT_SECRET to be set.
    pub fn generate_token(
        &self,
        user_id: String,
        roles: Vec<String>,
        permissions: Vec<String>,
    ) -> Result<String, String> {
        if self.config.secret.is_empty() {
            return Err(
                "JWT_SECRET is not set; required to generate tokens. Set JWT_SECRET in production."
                    .to_string(),
            );
        }
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
                s.strip_prefix("Bearer ")
                    .map(|stripped| stripped.to_string())
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

    /// AuthValidator with a fixed secret so tests never depend on JWT_SECRET env.
    fn test_validator() -> AuthValidator {
        AuthValidator::new(JwtConfig::new("test_jwt_secret_for_unit_tests".to_string()))
    }

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
        let validator = test_validator();

        // Generate a token
        let token_result = validator.generate_token(
            "user123".to_string(),
            vec!["admin".to_string()],
            vec!["read".to_string(), "write".to_string()],
        );

        let token = token_result.expect("generate_token in test");
        assert!(!token.is_empty());

        let claims = validator
            .validate_api_key(&token)
            .expect("validate_api_key in test");
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.roles, vec!["admin"]);
    }

    #[test]
    fn test_jwt_role_validation() {
        let validator = test_validator();

        let token = validator
            .generate_token(
                "user123".to_string(),
                vec!["admin".to_string(), "moderator".to_string()],
                vec![],
            )
            .expect("generate_token in test");

        assert_eq!(validator.validate_role(&token, "admin"), Ok(true));
        assert_eq!(validator.validate_role(&token, "moderator"), Ok(true));
        assert_eq!(validator.validate_role(&token, "user"), Ok(false));
    }

    #[test]
    fn test_jwt_permission_validation() {
        let validator = test_validator();

        let token = validator
            .generate_token(
                "user123".to_string(),
                vec![],
                vec!["read".to_string(), "write".to_string()],
            )
            .expect("generate_token in test");

        assert_eq!(validator.validate_permission(&token, "read"), Ok(true));
        assert_eq!(validator.validate_permission(&token, "delete"), Ok(false));
    }

    #[test]
    fn test_jwt_empty_token() {
        let validator = test_validator();

        let result = validator.validate_api_key("");
        let err = result.expect_err("empty token should fail");
        assert!(err.contains("Empty token"));
    }

    #[test]
    fn test_jwt_invalid_token() {
        let validator = test_validator();

        let result = validator.validate_api_key("invalid.jwt.token");
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_extract_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::try_from("Bearer my_token_123").expect("test Authorization header"),
        );

        let token = AuthValidator::extract_token(&headers);
        assert_eq!(token.as_deref(), Some("my_token_123"));
    }

    #[test]
    fn test_jwt_extract_no_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_static("my_token_123"),
        );

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

        let rt = Runtime::new().expect("tokio runtime for test");
        rt.block_on(async {
            let limiter = RateLimiter::new(3, 60); // 3 requests per 60 seconds
            let ip: IpAddr = "127.0.0.1".parse().expect("loopback test IP");

            // First 3 requests should succeed
            assert!(limiter.check_rate_limit(ip).await.is_ok());
            assert!(limiter.check_rate_limit(ip).await.is_ok());
            assert!(limiter.check_rate_limit(ip).await.is_ok());

            // 4th request should be rate limited
            let result = limiter.check_rate_limit(ip).await;
            assert_eq!(result, Err(StatusCode::TOO_MANY_REQUESTS));
        });
    }
}
