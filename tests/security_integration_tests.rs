// Security Integration Tests
// Tests for security middleware and FFI security integration

use dist_agent_lang::http_server_security::{
    RateLimiter, RequestSizeLimiter, InputValidator, SecurityLogger,
};
use dist_agent_lang::ffi::security::{FFIInputValidator, FFIResourceLimits};
use std::net::IpAddr;

/// Test: Rate limiter should reject excessive requests
#[tokio::test]
async fn test_rate_limiter_rejects_excessive_requests() {
    let limiter = RateLimiter::new(5, 60); // 5 requests per minute
    let ip: IpAddr = "127.0.0.1".parse().unwrap();
    
    // Make 5 requests - should all succeed
    for _ in 0..5 {
        assert!(limiter.check_rate_limit(ip).await.is_ok());
    }
    
    // 6th request should fail
    assert!(limiter.check_rate_limit(ip).await.is_err());
}

/// Test: Request size limiter should reject oversized requests
#[test]
fn test_request_size_limiter() {
    use axum::http::HeaderMap;
    
    let limiter = RequestSizeLimiter::default();
    
    // Valid request
    let mut headers = HeaderMap::new();
    headers.insert("Content-Length", "1000".parse().unwrap());
    assert!(limiter.validate_request(&headers, 1000, 100).is_ok());
    
    // Oversized body
    assert!(limiter.validate_request(&headers, 2_000_000, 100).is_err());
    
    // Oversized URL
    assert!(limiter.validate_request(&headers, 1000, 3_000).is_err());
}

/// Test: Input validator should reject SQL injection patterns
#[test]
fn test_input_validator_sql_injection() {
    let sql_injections = vec![
        "'; DROP TABLE users; --",
        "\"; DELETE FROM users; --",
        "admin' OR '1'='1",
        "'; EXEC xp_cmdshell('dir'); --",
    ];
    
    for injection in sql_injections {
        assert!(InputValidator::validate_string(injection, 1000).is_err());
    }
}

/// Test: Input validator should reject XSS patterns
#[test]
fn test_input_validator_xss() {
    let xss_patterns = vec![
        "<script>alert('XSS')</script>",
        "javascript:alert('XSS')",
        "<img src=x onerror=alert('XSS')>",
        "eval('malicious code')",
    ];
    
    for pattern in xss_patterns {
        assert!(InputValidator::validate_string(pattern, 1000).is_err());
    }
}

/// Test: Input validator should accept valid addresses
#[test]
fn test_input_validator_address() {
    // Valid Ethereum address: 0x + 40 hex chars = 42 total chars
    let valid_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
    assert!(InputValidator::validate_address(valid_address).is_ok());
    
    let invalid_addresses = vec![
        "0x123", // Too short
        "742d35Cc6634C0532925a3b844Bc9e7595f0bEb", // Missing 0x
        "0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG", // Invalid hex
    ];
    
    for addr in invalid_addresses {
        assert!(InputValidator::validate_address(addr).is_err());
    }
}

/// Test: FFI input validator should reject oversized inputs
#[test]
fn test_ffi_input_validator_size() {
    let limits = FFIResourceLimits::default();
    
    // Valid input
    let valid_input = "x".repeat(1_000_000);
    assert!(FFIInputValidator::validate_source(&valid_input, &limits).is_ok());
    
    // Oversized input
    let oversized_input = "x".repeat(11_000_000);
    assert!(FFIInputValidator::validate_source(&oversized_input, &limits).is_err());
}

/// Test: FFI input validator should reject null bytes
#[test]
fn test_ffi_input_validator_null_bytes() {
    let limits = FFIResourceLimits::default();
    let input_with_null = "valid code\0malicious code";
    
    assert!(FFIInputValidator::validate_source(input_with_null, &limits).is_err());
}

/// Test: FFI input validator should reject extremely long lines
#[test]
fn test_ffi_input_validator_long_lines() {
    let limits = FFIResourceLimits::default();
    let long_line = "x".repeat(2_000_000);
    
    assert!(FFIInputValidator::validate_source(&long_line, &limits).is_err());
}

/// Test: Security logger should log events
#[test]
fn test_security_logger() {
    // This test just verifies the logger doesn't panic
    SecurityLogger::log_event("TEST", "Test event", Some("127.0.0.1"));
    SecurityLogger::log_rate_limit("127.0.0.1");
    SecurityLogger::log_auth_failure("127.0.0.1", "Invalid token");
    SecurityLogger::log_invalid_input("127.0.0.1", "SQL injection attempt");
    
    assert!(true); // If we get here, logging worked
}

/// Test: Input sanitization should remove dangerous characters
#[test]
fn test_input_sanitization() {
    let dangerous = "test<script>alert('xss')</script>test";
    let sanitized = InputValidator::sanitize_string(dangerous);
    
    // Should remove script tags
    assert!(!sanitized.contains("<script>"));
}

/// Test: FFI input sanitization should remove null bytes
#[test]
fn test_ffi_input_sanitization() {
    let with_null = "test\0null\0bytes";
    let sanitized = FFIInputValidator::sanitize_string(with_null);
    
    assert!(!sanitized.contains('\0'));
    assert_eq!(sanitized, "testnullbytes");
}

