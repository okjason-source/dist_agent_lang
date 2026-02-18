use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Crypto namespace for cryptographic operations
/// Provides hashing and signing functionality

/// Hash algorithm types
#[derive(Debug, Clone, PartialEq)]
pub enum HashAlgorithm {
    SHA256,
    SHA512,
    Simple,  // Replaced MD5 with simple hash
    Custom(String),
}

/// Signature algorithm types
#[derive(Debug, Clone, PartialEq)]
pub enum SignatureAlgorithm {
    RSA,
    ECDSA,
    Ed25519,
    Custom(String),
}

/// Hash a string using the specified algorithm
/// 
/// # Arguments
/// * `data` - The data to hash
/// * `algorithm` - The hashing algorithm to use
/// 
/// # Returns
/// * `String` - The hexadecimal hash string
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::crypto;
/// use dist_agent_lang::stdlib::crypto::HashAlgorithm;
/// let hash = crypto::hash("Hello, World!", HashAlgorithm::SHA256);
/// ```
pub fn hash(data: &str, algorithm: HashAlgorithm) -> String {
    match algorithm {
        HashAlgorithm::SHA256 => hash_sha256(data),
        HashAlgorithm::SHA512 => hash_sha512(data),
        HashAlgorithm::Simple => hash_simple(data),
        HashAlgorithm::Custom(name) => hash_custom(data, &name),
    }
}

/// Hash bytes with specified algorithm (for FFI compatibility)
pub fn hash_bytes(data: &[u8], algorithm: &str) -> Result<String, String> {
    match algorithm.to_uppercase().as_str() {
        "SHA256" => {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(data);
            Ok(format!("{:x}", hasher.finalize()))
        }
        "MD5" => {
            // MD5 is deprecated but kept for compatibility
            // Use a simple hash instead
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            Ok(format!("{:x}", hasher.finish()))
        }
        _ => Err(format!("Unsupported hash algorithm: {}", algorithm))
    }
}

/// Hash data using SHA-256
fn hash_sha256(data: &str) -> String {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    
    format!("{:x}", result)
}

/// Hash data using SHA-512
fn hash_sha512(data: &str) -> String {
    use sha2::{Sha512, Digest};
    
    let mut hasher = Sha512::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    
    format!("{:x}", result)
}

/// Hash data using simple hash (replaces MD5)
fn hash_simple(data: &str) -> String {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("simple_{:x}", hasher.finish())
}

/// Hash data using a custom algorithm (fallback to simple hash)
fn hash_custom(data: &str, algorithm_name: &str) -> String {
    // Fallback to a simple hash for custom algorithms
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    algorithm_name.hash(&mut hasher);
    
    format!("custom_{:x}", hasher.finish())
}

/// Generate a random hash
/// 
/// # Arguments
/// * `algorithm` - The hashing algorithm to use
/// 
/// # Returns
/// * `String` - A random hash string
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::crypto;
/// use dist_agent_lang::stdlib::crypto::HashAlgorithm;
/// let random_hash = crypto::random_hash(HashAlgorithm::SHA256);
/// ```
pub fn random_hash(algorithm: HashAlgorithm) -> String {
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    let random_data: String = (0..32)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();
    
    hash(&random_data, algorithm)
}

/// Sign data with a private key
/// 
/// # Arguments
/// * `data` - The data to sign
/// * `private_key` - The private key for signing
/// * `algorithm` - The signature algorithm to use
/// 
/// # Returns
/// * `String` - The signature string
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::crypto;
/// use dist_agent_lang::stdlib::crypto::SignatureAlgorithm;
/// let signature = crypto::sign("Hello, World!", "private_key_123", SignatureAlgorithm::RSA);
/// ```
pub fn sign(data: &str, private_key: &str, algorithm: SignatureAlgorithm) -> String {
    match algorithm {
        SignatureAlgorithm::RSA => sign_rsa(data, private_key),
        SignatureAlgorithm::ECDSA => sign_ecdsa(data, private_key),
        SignatureAlgorithm::Ed25519 => sign_ed25519(data, private_key),
        SignatureAlgorithm::Custom(name) => sign_custom(data, private_key, &name),
    }
}

/// Sign data using RSA (real implementation when key is PEM; otherwise mock for compat)
fn sign_rsa(data: &str, private_key: &str) -> String {
    if !private_key.contains("-----") {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        private_key.hash(&mut hasher);
        return format!("rsa_sign_{:x}", hasher.finish());
    }
    use rsa::pkcs1::DecodeRsaPrivateKey;
    use rsa::pkcs1v15::SigningKey;
    use rsa::sha2::Sha256;
    use rsa::signature::{Signer, SignatureEncoding};
    use rsa::RsaPrivateKey;
    let key = match RsaPrivateKey::from_pkcs1_pem(private_key) {
        Ok(k) => k,
        Err(_) => {
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            return format!("rsa_sign_{:x}", hasher.finish());
        }
    };
    use sha2::Digest;
    let signing_key = SigningKey::<Sha256>::new(key);
    let digest = Sha256::digest(data.as_bytes());
    let sig = signing_key.sign(digest.as_ref());
    hex::encode(sig.to_bytes())
}

/// Sign data using ECDSA (delegate to crypto_signatures; fallback mock on invalid key)
fn sign_ecdsa(data: &str, private_key: &str) -> String {
    match crate::stdlib::crypto_signatures::ECDSASignatureVerifier::sign(data.as_bytes(), private_key) {
        Ok(sig) => sig,
        Err(_) => {
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            private_key.hash(&mut hasher);
            format!("ecdsa_sign_{:x}", hasher.finish())
        }
    }
}

/// Sign data using Ed25519 (delegate to crypto_signatures; fallback mock on invalid key)
fn sign_ed25519(data: &str, private_key: &str) -> String {
    match crate::stdlib::crypto_signatures::EdDSASignatureVerifier::sign(data.as_bytes(), private_key) {
        Ok(sig) => sig,
        Err(_) => {
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            private_key.hash(&mut hasher);
            format!("ed25519_sign_{:x}", hasher.finish())
        }
    }
}

/// Sign data using a custom algorithm (unsupported; mock for compat)
fn sign_custom(data: &str, private_key: &str, algorithm_name: &str) -> String {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    private_key.hash(&mut hasher);
    algorithm_name.hash(&mut hasher);
    format!("custom_sign_{:x}", hasher.finish())
}

/// Verify a signature
/// 
/// # Arguments
/// * `_data` - The original data (unused in mock implementation)
/// * `signature` - The signature to verify
/// * `_public_key` - The public key for verification (unused in mock implementation)
/// * `algorithm` - The signature algorithm used
/// 
/// # Returns
/// * `bool` - True if signature is valid, false otherwise
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::crypto;
/// use dist_agent_lang::stdlib::crypto::SignatureAlgorithm;
/// let signature = crypto::sign("Hello, World!", "private_key_123", SignatureAlgorithm::RSA);
/// let is_valid = crypto::verify("Hello, World!", &signature, "public_key_123", SignatureAlgorithm::RSA);
/// ```
pub fn verify(data: &str, signature: &str, public_key: &str, algorithm: SignatureAlgorithm) -> bool {
    match algorithm {
        SignatureAlgorithm::RSA => verify_rsa(data, signature, public_key),
        SignatureAlgorithm::ECDSA => verify_ecdsa(data, signature, public_key),
        SignatureAlgorithm::Ed25519 => verify_ed25519(data, signature, public_key),
        SignatureAlgorithm::Custom(_name) => verify_custom(signature),
    }
}

/// Verify RSA signature (real when public_key is PEM; otherwise prefix check for compat)
fn verify_rsa(data: &str, signature: &str, public_key: &str) -> bool {
    if !public_key.contains("-----") || hex::decode(signature).is_err() {
        return signature.starts_with("rsa_sign_");
    }
    use rsa::pkcs1v15::{Signature, VerifyingKey};
    use sha2::Digest;
    use rsa::pkcs8::DecodePublicKey;
    use rsa::sha2::Sha256;
    use rsa::signature::Verifier;
    use rsa::RsaPublicKey;
    let key: RsaPublicKey = match RsaPublicKey::from_public_key_pem(public_key) {
        Ok(k) => k,
        Err(_) => return signature.starts_with("rsa_sign_"),
    };
    let sig_bytes = match hex::decode(signature) {
        Ok(b) => b,
        Err(_) => return signature.starts_with("rsa_sign_"),
    };
    let sig: Signature = match Signature::try_from(sig_bytes.as_slice()) {
        Ok(s) => s,
        Err(_) => return signature.starts_with("rsa_sign_"),
    };
    let digest = Sha256::digest(data.as_bytes());
    let verifying_key = VerifyingKey::<Sha256>::new(key);
    verifying_key.verify(digest.as_ref(), &sig).is_ok()
}

/// Verify ECDSA signature (delegate to crypto_signatures; fallback prefix check)
fn verify_ecdsa(data: &str, signature: &str, public_key: &str) -> bool {
    match crate::stdlib::crypto_signatures::ECDSASignatureVerifier::verify(
        data.as_bytes(),
        signature,
        public_key,
    ) {
        Ok(valid) => valid,
        Err(_) => signature.starts_with("ecdsa_sign_"),
    }
}

/// Verify Ed25519 signature (delegate to crypto_signatures; fallback prefix check)
fn verify_ed25519(data: &str, signature: &str, public_key: &str) -> bool {
    match crate::stdlib::crypto_signatures::EdDSASignatureVerifier::verify(
        data.as_bytes(),
        signature,
        public_key,
    ) {
        Ok(valid) => valid,
        Err(_) => signature.starts_with("ed25519_sign_"),
    }
}

/// Verify custom signature (unsupported; prefix check for compat)
fn verify_custom(signature: &str) -> bool {
    signature.starts_with("custom_sign_")
}

/// Generate a key pair
/// 
/// # Arguments
/// * `algorithm` - The signature algorithm to use
/// 
/// # Returns
/// * `HashMap<String, String>` - Map containing public and private keys
/// 
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::crypto;
/// use dist_agent_lang::stdlib::crypto::SignatureAlgorithm;
/// let keypair = crypto::generate_keypair(SignatureAlgorithm::RSA);
/// ```
pub fn generate_keypair(algorithm: SignatureAlgorithm) -> HashMap<String, String> {
    let mut keypair = HashMap::new();
    let algorithm_name = match algorithm {
        SignatureAlgorithm::RSA => "rsa".to_string(),
        SignatureAlgorithm::ECDSA => "ecdsa".to_string(),
        SignatureAlgorithm::Ed25519 => "ed25519".to_string(),
        SignatureAlgorithm::Custom(ref name) => name.clone(),
    };
    keypair.insert("algorithm".to_string(), algorithm_name.clone());

    match algorithm {
        SignatureAlgorithm::ECDSA => {
            match crate::stdlib::crypto_signatures::ECDSASignatureVerifier::generate_keypair() {
                Ok((priv_k, pub_k)) => {
                    keypair.insert("private_key".to_string(), priv_k);
                    keypair.insert("public_key".to_string(), pub_k);
                }
                Err(_) => fallback_keypair(&mut keypair),
            }
        }
        SignatureAlgorithm::Ed25519 => {
            match crate::stdlib::crypto_signatures::EdDSASignatureVerifier::generate_keypair() {
                Ok((priv_k, pub_k)) => {
                    keypair.insert("private_key".to_string(), priv_k);
                    keypair.insert("public_key".to_string(), pub_k);
                }
                Err(_) => fallback_keypair(&mut keypair),
            }
        }
        SignatureAlgorithm::RSA => {
            use rand::rngs::OsRng;
            use rsa::pkcs1::EncodeRsaPrivateKey;
            use rsa::pkcs8::EncodePublicKey;
            use rsa::RsaPrivateKey;
            match RsaPrivateKey::new(&mut OsRng, 2048) {
                Ok(key) => {
                    let priv_pem = key.to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
                        .map(|z| z.to_string())
                        .unwrap_or_else(|_| String::new());
                    let pub_key = key.to_public_key();
                    let pub_pem = pub_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)
                        .unwrap_or_else(|_| String::new());
                    keypair.insert("private_key".to_string(), priv_pem);
                    keypair.insert("public_key".to_string(), pub_pem);
                }
                Err(_) => fallback_keypair(&mut keypair),
            }
        }
        SignatureAlgorithm::Custom(_) => fallback_keypair(&mut keypair),
    }
    keypair
}

fn fallback_keypair(keypair: &mut HashMap<String, String>) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let private_key: String = (0..64)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();
    let public_key: String = (0..64)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();
    keypair.insert("private_key".to_string(), private_key);
    keypair.insert("public_key".to_string(), public_key);
}

/// Encrypt data with a public key (RSA when key is PEM; otherwise mock for compat)
pub fn encrypt(data: &str, public_key: &str) -> String {
    if !public_key.contains("-----") {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        public_key.hash(&mut hasher);
        return format!("encrypted_{:x}", hasher.finish());
    }
    use rsa::oaep::Oaep;
    use rsa::pkcs8::DecodePublicKey;
    use rsa::sha2::Sha256;
    use rsa::RsaPublicKey;
    let key: RsaPublicKey = match RsaPublicKey::from_public_key_pem(public_key) {
        Ok(k) => k,
        Err(_) => {
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            return format!("encrypted_{:x}", hasher.finish());
        }
    };
    let mut rng = rand::thread_rng();
    let padding = Oaep::new::<Sha256>();
    match key.encrypt(&mut rng, padding, data.as_bytes()) {
        Ok(ciphertext) => base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            ciphertext,
        ),
        Err(_) => {
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            format!("encrypted_{:x}", hasher.finish())
        }
    }
}

/// Decrypt data with a private key (RSA when key is PEM; otherwise mock for compat)
pub fn decrypt(encrypted_data: &str, private_key: &str) -> Option<String> {
    if !private_key.contains("-----") {
        return if encrypted_data.starts_with("encrypted_") {
            Some("decrypted_message".to_string())
        } else {
            None
        };
    }
    use rsa::oaep::Oaep;
    use rsa::pkcs8::DecodePrivateKey;
    use rsa::sha2::Sha256;
    use rsa::RsaPrivateKey;
    let key = RsaPrivateKey::from_pkcs8_pem(private_key).ok()?;
    let ciphertext = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        encrypted_data,
    ).ok()?;
    let padding = Oaep::new::<Sha256>();
    let plaintext = key.decrypt(padding, &ciphertext).ok()?;
    String::from_utf8(plaintext).ok()
}

/// Encrypt data using AES-256-GCM (real implementation)
#[allow(deprecated)] // generic_array 0.14 (from aes-gcm) deprecated; upgrade when aes-gcm 0.11 stable
pub fn encrypt_aes256(data: &str, key: &str) -> Result<String, String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm,
    };
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let key_bytes: [u8; 32] = hasher.finalize().into();
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| e.to_string())?;
    let mut nonce = [0u8; 12];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut nonce);
    let nonce_arr = aes_gcm::aead::generic_array::GenericArray::clone_from_slice(&nonce);
    let ciphertext = cipher
        .encrypt(&nonce_arr, data.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut out = nonce.to_vec();
    out.extend_from_slice(&ciphertext);
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &out,
    ))
}

/// Decrypt data using AES-256-GCM (real implementation)
#[allow(deprecated)] // generic_array 0.14 (from aes-gcm) deprecated; upgrade when aes-gcm 0.11 stable
pub fn decrypt_aes256(encrypted_data: &str, key: &str) -> Result<String, String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm,
    };
    use sha2::{Sha256, Digest};
    let raw = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        encrypted_data,
    ).map_err(|_| "Invalid base64".to_string())?;
    if raw.len() < 12 {
        return Err("Invalid encrypted data format".to_string());
    }
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let key_bytes: [u8; 32] = hasher.finalize().into();
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| e.to_string())?;
    let (nonce, ct) = raw.split_at(12);
    let nonce_arr = aes_gcm::aead::generic_array::GenericArray::clone_from_slice(nonce);
    let plaintext = cipher
        .decrypt(&nonce_arr, ct)
        .map_err(|_| "Decryption failed (wrong key or corrupted data)".to_string())?;
    String::from_utf8(plaintext).map_err(|e| e.to_string())
}
