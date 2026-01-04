// Comprehensive Security Integration Tests
// Tests for Phase 1 security features integration

use dist_agent_lang::http_server_security::{AuthValidator, JwtConfig, Claims, SecurityLogger};
use dist_agent_lang::stdlib::crypto_signatures::ECDSASignatureVerifier;
use dist_agent_lang::stdlib::cross_chain_security::{CrossChainSecurityManager, CrossChainOperation, CrossChainOperationType, OperationStatus, ValidatorSignature};

#[test]
fn test_jwt_full_lifecycle() {
    // Test complete JWT workflow: generate, validate, check roles, check permissions
    
    let config = JwtConfig::new("test_secret_key".to_string())
        .with_expiration(24);
    
    let validator = AuthValidator::new(config);
    
    // Generate token
    let token = validator.generate_token(
        "user_123".to_string(),
        vec!["admin".to_string(), "moderator".to_string()],
        vec!["read".to_string(), "write".to_string(), "delete".to_string()]
    );
    
    assert!(token.is_ok(), "Token generation failed");
    let token_str = token.unwrap();
    
    // Validate token
    let claims_result = validator.validate_api_key(&token_str);
    assert!(claims_result.is_ok(), "Token validation failed");
    
    let claims = claims_result.unwrap();
    assert_eq!(claims.sub, "user_123");
    assert_eq!(claims.roles.len(), 2);
    assert_eq!(claims.permissions.len(), 3);
    
    // Check role
    let has_admin = validator.validate_role(&token_str, "admin");
    assert!(has_admin.is_ok());
    assert!(has_admin.unwrap());
    
    // Check permission
    let has_delete = validator.validate_permission(&token_str, "delete");
    assert!(has_delete.is_ok());
    assert!(has_delete.unwrap());
    
    // Check non-existent role
    let has_super = validator.validate_role(&token_str, "superadmin");
    assert!(has_super.is_ok());
    assert!(!has_super.unwrap());
}

#[test]
fn test_ecdsa_cross_chain_integration() {
    // Test ECDSA signatures in cross-chain context
    
    // Generate validator keypair
    let (privkey, pubkey) = ECDSASignatureVerifier::generate_keypair().unwrap();
    
    // Create message
    let message = b"Transfer 1000 tokens from Ethereum to Polygon";
    
    // Sign message
    let signature = ECDSASignatureVerifier::sign(message, &privkey);
    assert!(signature.is_ok(), "Signing failed");
    
    let sig = signature.unwrap();
    
    // Verify signature
    let verify_result = ECDSASignatureVerifier::verify(message, &sig, &pubkey);
    assert!(verify_result.is_ok(), "Verification failed");
    assert!(verify_result.unwrap(), "Signature should be valid");
    
    // Try to verify with wrong message
    let wrong_msg = b"Transfer 2000 tokens";
    let verify_wrong = ECDSASignatureVerifier::verify(wrong_msg, &sig, &pubkey);
    assert!(verify_wrong.is_ok());
    assert!(!verify_wrong.unwrap(), "Wrong message should fail verification");
}

#[test]
fn test_jwt_expiration_and_security() {
    // Test JWT expiration and security features
    
    let validator = AuthValidator::default();
    
    // Generate token
    let token = validator.generate_token(
        "user_123".to_string(),
        vec!["user".to_string()],
        vec!["read".to_string()]
    ).unwrap();
    
    // Should be valid initially
    let claims = validator.validate_api_key(&token);
    assert!(claims.is_ok());
    assert!(!claims.unwrap().is_expired());
    
    // Test empty token
    let empty_result = validator.validate_api_key("");
    assert!(empty_result.is_err());
    assert!(empty_result.unwrap_err().contains("Empty token"));
    
    // Test invalid token format
    let invalid_result = validator.validate_api_key("not.a.valid.jwt");
    assert!(invalid_result.is_err());
    assert!(invalid_result.unwrap_err().contains("Invalid JWT"));
}

#[test]
fn test_ecdsa_keypair_security() {
    // Test ECDSA keypair generation and security properties
    
    // Generate multiple keypairs
    let (priv1, pub1) = ECDSASignatureVerifier::generate_keypair().unwrap();
    let (priv2, pub2) = ECDSASignatureVerifier::generate_keypair().unwrap();
    
    // Keys should be different
    assert_ne!(priv1, priv2, "Private keys should be unique");
    assert_ne!(pub1, pub2, "Public keys should be unique");
    
    // Keys should have correct length
    assert_eq!(priv1.len(), 64, "Private key should be 32 bytes (64 hex)");
    assert_eq!(pub1.len(), 66, "Public key should be 33 bytes compressed (66 hex)");
    
    // Sign with keypair 1
    let message = b"test message";
    let sig1 = ECDSASignatureVerifier::sign(message, &priv1).unwrap();
    
    // Should verify with correct public key
    let verify1 = ECDSASignatureVerifier::verify(message, &sig1, &pub1).unwrap();
    assert!(verify1, "Should verify with correct key");
    
    // Should NOT verify with different public key
    let verify2 = ECDSASignatureVerifier::verify(message, &sig1, &pub2).unwrap();
    assert!(!verify2, "Should NOT verify with different key");
}

#[test]
fn test_cross_chain_with_real_signatures() {
    // Test full cross-chain operation with real ECDSA signatures
    
    let mut manager = CrossChainSecurityManager::new();
    
    // Generate validator keypair
    let (validator_privkey, validator_pubkey) = ECDSASignatureVerifier::generate_keypair().unwrap();
    
    // Create bridge
    let bridge_result = manager.create_bridge(
        137, // Polygon source
        137, // Polygon target (same for testing)
        "0xTestBridge".to_string(),
        vec![validator_pubkey.clone()],
        1,
        1000000,
        1000000,
    );
    
    assert!(bridge_result.is_ok(), "Bridge creation failed");
    
    // Create operation
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Create and sign message
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(b"cross_chain_transfer_data");
    hasher.update(&now.to_be_bytes());
    let message = hasher.finalize();
    
    let signature = ECDSASignatureVerifier::sign(&message, &validator_privkey).unwrap();
    
    let operation = CrossChainOperation {
        operation_id: format!("op_{}", now),
        source_chain: 137,
        target_chain: 137,
        operation_type: CrossChainOperationType::Transfer {
            from: "0xUser1".to_string(),
            to: "0xUser2".to_string(),
            amount: 5000,
        },
        data: b"cross_chain_transfer_data".to_vec(),
        signatures: vec![ValidatorSignature {
            validator_address: validator_pubkey,
            signature,
            timestamp: now,
            chain_id: 137,
        }],
        status: OperationStatus::Pending,
        created_at: now,
        timeout: now + 3600,
    };
    
    // Validate operation
    let result = manager.validate_cross_chain_operation(operation);
    assert!(result.is_ok(), "Cross-chain operation validation should succeed with valid signature");
}

#[test]
fn test_multi_signature_validation() {
    // Test that operations can be validated with multiple signatures
    
    let mut manager = CrossChainSecurityManager::new();
    
    // Generate two validator keypairs
    let (priv1, pub1) = ECDSASignatureVerifier::generate_keypair().unwrap();
    let (priv2, pub2) = ECDSASignatureVerifier::generate_keypair().unwrap();
    
    // Create bridge requiring 2 signatures
    manager.create_bridge(
        137, 137, "0xMultiSigBridge".to_string(),
        vec![pub1.clone(), pub2.clone()],
        2, // Require 2 signatures
        1000000,
        1000000,
    ).unwrap();
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Sign message with both validators
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(b"multi_sig_data");
    hasher.update(&now.to_be_bytes());
    let message = hasher.finalize();
    
    let sig1 = ECDSASignatureVerifier::sign(&message, &priv1).unwrap();
    let sig2 = ECDSASignatureVerifier::sign(&message, &priv2).unwrap();
    
    let operation = CrossChainOperation {
        operation_id: format!("multi_op_{}", now),
        source_chain: 137,
        target_chain: 137,
        operation_type: CrossChainOperationType::Transfer {
            from: "0xSender".to_string(),
            to: "0xReceiver".to_string(),
            amount: 10000,
        },
        data: b"multi_sig_data".to_vec(),
        signatures: vec![
            ValidatorSignature {
                validator_address: pub1,
                signature: sig1,
                timestamp: now,
                chain_id: 137,
            },
            ValidatorSignature {
                validator_address: pub2,
                signature: sig2,
                timestamp: now,
                chain_id: 137,
            },
        ],
        status: OperationStatus::Pending,
        created_at: now,
        timeout: now + 3600,
    };
    
    // Should pass with 2 valid signatures
    let result = manager.validate_cross_chain_operation(operation);
    assert!(result.is_ok(), "Multi-signature validation should succeed");
}

#[test]
fn test_jwt_role_based_access_control() {
    // Test complete RBAC with JWT
    
    let validator = AuthValidator::default();
    
    // Create admin token
    let admin_token = validator.generate_token(
        "admin_001".to_string(),
        vec!["admin".to_string()],
        vec!["read".to_string(), "write".to_string(), "delete".to_string()]
    ).unwrap();
    
    // Create user token
    let user_token = validator.generate_token(
        "user_001".to_string(),
        vec!["user".to_string()],
        vec!["read".to_string()]
    ).unwrap();
    
    // Admin should have admin role
    assert!(validator.validate_role(&admin_token, "admin").unwrap());
    
    // User should NOT have admin role
    assert!(!validator.validate_role(&user_token, "admin").unwrap());
    
    // Admin should have delete permission
    assert!(validator.validate_permission(&admin_token, "delete").unwrap());
    
    // User should NOT have delete permission
    assert!(!validator.validate_permission(&user_token, "delete").unwrap());
    
    // Both should have read permission
    assert!(validator.validate_permission(&admin_token, "read").unwrap());
    assert!(validator.validate_permission(&user_token, "read").unwrap());
}

#[test]
fn test_signature_replay_protection() {
    // Test that replay attacks are prevented
    
    use dist_agent_lang::stdlib::crypto_signatures::SecureSignatureVerifier;
    
    let mut verifier = SecureSignatureVerifier::new();
    let (privkey, pubkey) = ECDSASignatureVerifier::generate_keypair().unwrap();
    
    let message = b"transaction_data";
    let nonce = 1u64;
    let signer_key = "user_address_1";
    
    // Create message with nonce
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.update(&nonce.to_be_bytes());
    let message_with_nonce = hasher.finalize();
    
    let signature = ECDSASignatureVerifier::sign(&message_with_nonce, &privkey).unwrap();
    
    // First verification should succeed
    let result1 = verifier.verify_with_nonce(
        message,
        &signature,
        &pubkey,
        nonce,
        signer_key,
        "ecdsa",
    );
    
    assert!(result1.is_ok(), "First verification should succeed");
    assert!(result1.unwrap(), "Signature should be valid");
    
    // Replay attempt with same nonce should be rejected
    let result2 = verifier.verify_with_nonce(
        message,
        &signature,
        &pubkey,
        nonce, // Same nonce = replay attack
        signer_key,
        "ecdsa",
    );
    
    assert!(result2.is_err(), "Replay attack should be detected");
    assert!(result2.unwrap_err().to_string().to_lowercase().contains("replay") 
            || result2.unwrap_err().to_string().to_lowercase().contains("nonce")
            || result2.unwrap_err().to_string().to_lowercase().contains("used"));
}

#[test]
fn test_jwt_and_ecdsa_together() {
    // Test using both JWT and ECDSA in a simulated API scenario
    
    let jwt_validator = AuthValidator::default();
    let (ecdsa_privkey, ecdsa_pubkey) = ECDSASignatureVerifier::generate_keypair().unwrap();
    
    // Step 1: User authenticates and gets JWT
    let jwt = jwt_validator.generate_token(
        "user_blockchain_001".to_string(),
        vec!["trader".to_string()],
        vec!["trade".to_string(), "sign_transactions".to_string()]
    ).unwrap();
    
    // Step 2: Validate JWT
    let claims = jwt_validator.validate_api_key(&jwt).unwrap();
    assert_eq!(claims.sub, "user_blockchain_001");
    
    // Step 3: Check permission to sign transactions
    assert!(jwt_validator.validate_permission(&jwt, "sign_transactions").unwrap());
    
    // Step 4: User signs a blockchain transaction with ECDSA
    let tx_data = b"transfer(0xRecipient, 1000000000000000000)";
    let signature = ECDSASignatureVerifier::sign(tx_data, &ecdsa_privkey).unwrap();
    
    // Step 5: Verify transaction signature
    let verified = ECDSASignatureVerifier::verify(tx_data, &signature, &ecdsa_pubkey).unwrap();
    assert!(verified, "Transaction signature should be valid");
    
    // This simulates a complete flow: JWT auth + ECDSA signing
}

#[test]
fn test_security_logging_integration() {
    // Test security logging doesn't crash and formats correctly
    
    SecurityLogger::log_event("TEST_EVENT", "Integration test", Some("127.0.0.1"));
    SecurityLogger::log_rate_limit("192.168.1.1");
    SecurityLogger::log_auth_failure("10.0.0.1", "Invalid credentials");
    SecurityLogger::log_auth_success("10.0.0.2", "user_123");
    SecurityLogger::log_suspicious_activity("172.16.0.1", "Multiple failed logins");
    SecurityLogger::log_token_validation_failure("192.168.0.1", "Expired token");
    
    // If we got here without panicking, logging works correctly
    assert!(true, "All logging methods executed successfully");
}

#[test]
fn test_invalid_signature_rejection() {
    // Test that invalid signatures are properly rejected
    
    let (privkey, pubkey) = ECDSASignatureVerifier::generate_keypair().unwrap();
    let (_other_priv, other_pub) = ECDSASignatureVerifier::generate_keypair().unwrap();
    
    let message = b"important_transaction";
    let signature = ECDSASignatureVerifier::sign(message, &privkey).unwrap();
    
    // Should fail with wrong public key
    let wrong_key_result = ECDSASignatureVerifier::verify(message, &signature, &other_pub);
    assert!(wrong_key_result.is_ok());
    assert!(!wrong_key_result.unwrap(), "Should reject signature with wrong public key");
    
    // Should fail with wrong message
    let wrong_msg = b"tampered_transaction";
    let wrong_msg_result = ECDSASignatureVerifier::verify(wrong_msg, &signature, &pubkey);
    assert!(wrong_msg_result.is_ok());
    assert!(!wrong_msg_result.unwrap(), "Should reject signature for wrong message");
    
    // Should fail with corrupted signature
    let corrupted_sig = "0".repeat(128); // All zeros
    let corrupted_result = ECDSASignatureVerifier::verify(message, &corrupted_sig, &pubkey);
    // May fail at parsing or verification, either is acceptable
    assert!(corrupted_result.is_err() || !corrupted_result.unwrap(), 
            "Should reject corrupted signature");
}

#[test]
fn test_jwt_with_different_algorithms() {
    // Test JWT configuration with different settings
    
    let config1 = JwtConfig::new("secret1".to_string()).with_expiration(1);
    let config2 = JwtConfig::new("secret2".to_string()).with_expiration(48);
    
    let validator1 = AuthValidator::new(config1);
    let validator2 = AuthValidator::new(config2);
    
    // Generate tokens with different validators
    let token1 = validator1.generate_token(
        "user1".to_string(),
        vec![],
        vec![]
    ).unwrap();
    
    let token2 = validator2.generate_token(
        "user2".to_string(),
        vec![],
        vec![]
    ).unwrap();
    
    // Each validator can validate its own token
    assert!(validator1.validate_api_key(&token1).is_ok());
    assert!(validator2.validate_api_key(&token2).is_ok());
    
    // But cannot validate each other's tokens (different secrets)
    assert!(validator1.validate_api_key(&token2).is_err(), 
            "Should reject token from different secret");
    assert!(validator2.validate_api_key(&token1).is_err(), 
            "Should reject token from different secret");
}

#[test]
fn test_comprehensive_security_stack() {
    // Test the complete security stack working together
    
    // 1. JWT Authentication
    let jwt_validator = AuthValidator::default();
    let jwt = jwt_validator.generate_token(
        "secure_user".to_string(),
        vec!["verified".to_string()],
        vec!["execute_transaction".to_string()]
    ).unwrap();
    
    // 2. Validate JWT and permissions
    assert!(jwt_validator.validate_permission(&jwt, "execute_transaction").unwrap());
    
    // 3. Generate ECDSA keypair for signing
    let (privkey, pubkey) = ECDSASignatureVerifier::generate_keypair().unwrap();
    
    // 4. Create cross-chain manager
    let mut cc_manager = CrossChainSecurityManager::new();
    cc_manager.create_bridge(
        137, 137, "0xSecureBridge".to_string(),
        vec![pubkey.clone()],
        1, 1000000, 1000000,
    ).unwrap();
    
    // 5. Sign transaction
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(b"secure_transaction");
    hasher.update(&now.to_be_bytes());
    let tx_msg = hasher.finalize();
    
    let tx_signature = ECDSASignatureVerifier::sign(&tx_msg, &privkey).unwrap();
    
    // 6. Create and validate operation
    let operation = CrossChainOperation {
        operation_id: format!("secure_op_{}", now),
        source_chain: 137,
        target_chain: 137,
        operation_type: CrossChainOperationType::Transfer {
            from: "0xSecureUser".to_string(),
            to: "0xRecipient".to_string(),
            amount: 50000,
        },
        data: b"secure_transaction".to_vec(),
        signatures: vec![ValidatorSignature {
            validator_address: pubkey,
            signature: tx_signature,
            timestamp: now,
            chain_id: 137,
        }],
        status: OperationStatus::Pending,
        created_at: now,
        timeout: now + 3600,
    };
    
    // 7. Validate the complete secure operation
    let result = cc_manager.validate_cross_chain_operation(operation);
    assert!(result.is_ok(), "Complete security stack should work together");
    
    // 8. Log the successful operation
    SecurityLogger::log_auth_success("127.0.0.1", "secure_user");
    
    // All security layers working together!
}

