//! User-controlled encryption for sensitive data storage
//! 
//! Provides AES-256-GCM encryption with user-managed keys for complete
//! privacy control over stored behavioral and screenshot data.

use crate::error::{Result, StorageError};
use serde::{Serialize, Deserialize};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::{debug, info, error};

/// User-controlled encryption service
pub struct EncryptionService {
    /// User's encryption keys by key ID
    keys: HashMap<String, EncryptionKey>,
    /// Default key ID for new encryptions
    default_key_id: Option<String>,
    /// Encryption configuration
    config: EncryptionConfig,
}

/// Encryption key with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKey {
    pub key_id: String,
    pub algorithm: EncryptionAlgorithm,
    pub key_data: Vec<u8>,
    pub created_at: u64,
    pub description: String,
    pub usage_count: u64,
}

/// Supported encryption algorithms
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM (recommended)
    Aes256Gcm,
    /// ChaCha20-Poly1305 (alternative)
    ChaCha20Poly1305,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Auto-encrypt new data
    pub auto_encrypt: bool,
    /// Key rotation interval (seconds)
    pub key_rotation_interval: u64,
    /// Maximum key age before warning
    pub max_key_age: u64,
    /// Require user password for key access
    pub require_user_password: bool,
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub key_id: String,
    pub algorithm: EncryptionAlgorithm,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
    pub encrypted_at: u64,
}

/// Key generation options
#[derive(Debug, Clone)]
pub struct KeyGenerationOptions {
    pub algorithm: EncryptionAlgorithm,
    pub description: String,
    pub user_password: Option<String>,
}

/// Encryption statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionStats {
    pub total_keys: usize,
    pub active_keys: usize,
    pub total_encrypted_items: u64,
    pub total_encrypted_bytes: u64,
    pub oldest_key_age_days: u64,
    pub encryption_operations_today: u64,
}

impl EncryptionService {
    /// Create new encryption service
    pub fn new(config: EncryptionConfig) -> Self {
        Self {
            keys: HashMap::new(),
            default_key_id: None,
            config,
        }
    }
    
    /// Generate a new encryption key
    pub fn generate_key(&mut self, options: KeyGenerationOptions) -> Result<String> {
        let key_id = format!("key_{}", self.generate_key_id());
        let key_data = self.generate_key_material(options.algorithm)?;
        
        let encryption_key = EncryptionKey {
            key_id: key_id.clone(),
            algorithm: options.algorithm,
            key_data,
            created_at: current_timestamp(),
            description: options.description,
            usage_count: 0,
        };
        
        // If user password is provided, derive key from password
        if let Some(password) = options.user_password {
            let derived_key = self.derive_key_from_password(&password, &encryption_key.key_data)?;
            let mut password_key = encryption_key.clone();
            password_key.key_data = derived_key;
            self.keys.insert(key_id.clone(), password_key);
        } else {
            self.keys.insert(key_id.clone(), encryption_key);
        }
        
        // Set as default key if it's the first one
        if self.default_key_id.is_none() {
            self.default_key_id = Some(key_id.clone());
        }
        
        info!("Generated new encryption key: {}", key_id);
        Ok(key_id)
    }
    
    /// Encrypt data with the default key
    pub fn encrypt(&mut self, data: &[u8]) -> Result<EncryptedData> {
        if let Some(default_key_id) = self.default_key_id.clone() {
            self.encrypt_with_key(data, &default_key_id)
        } else {
            Err(StorageError::Other("No encryption key available".to_string()))
        }
    }
    
    /// Encrypt data with a specific key
    pub fn encrypt_with_key(&mut self, data: &[u8], key_id: &str) -> Result<EncryptedData> {
        let key = self.keys.get(key_id)
            .ok_or_else(|| StorageError::Other(format!("Key not found: {}", key_id)))?
            .clone();
        
        let encrypted_data = match key.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes256_gcm(data, &key)?,
            EncryptionAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha20_poly1305(data, &key)?,
        };
        
        // Update usage count
        if let Some(mut_key) = self.keys.get_mut(key_id) {
            mut_key.usage_count += 1;
        }
        
        Ok(encrypted_data)
    }
    
    /// Decrypt data
    pub fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>> {
        let key = self.keys.get(&encrypted_data.key_id)
            .ok_or_else(|| StorageError::Other(format!("Key not found: {}", encrypted_data.key_id)))?;
        
        match encrypted_data.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes256_gcm(encrypted_data, key),
            EncryptionAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha20_poly1305(encrypted_data, key),
        }
    }
    
    /// Set the default encryption key
    pub fn set_default_key(&mut self, key_id: String) -> Result<()> {
        if self.keys.contains_key(&key_id) {
            self.default_key_id = Some(key_id);
            Ok(())
        } else {
            Err(StorageError::Other(format!("Key not found: {}", key_id)))
        }
    }
    
    /// Remove an encryption key
    pub fn remove_key(&mut self, key_id: &str) -> Result<()> {
        if self.default_key_id.as_ref() == Some(&key_id.to_string()) {
            return Err(StorageError::Other("Cannot remove default key".to_string()));
        }
        
        self.keys.remove(key_id)
            .ok_or_else(|| StorageError::Other(format!("Key not found: {}", key_id)))?;
        
        info!("Removed encryption key: {}", key_id);
        Ok(())
    }
    
    /// Rotate keys (generate new default key)
    pub fn rotate_keys(&mut self, options: KeyGenerationOptions) -> Result<String> {
        let new_key_id = self.generate_key(options)?;
        self.default_key_id = Some(new_key_id.clone());
        
        info!("Rotated encryption keys, new default: {}", new_key_id);
        Ok(new_key_id)
    }
    
    /// Get encryption statistics
    pub fn get_stats(&self) -> EncryptionStats {
        let total_keys = self.keys.len();
        let active_keys = if self.default_key_id.is_some() { 1 } else { 0 };
        
        let total_encrypted_items = self.keys.values()
            .map(|key| key.usage_count)
            .sum();
        
        let oldest_key_age = self.keys.values()
            .map(|key| current_timestamp() - key.created_at)
            .max()
            .unwrap_or(0);
        
        let oldest_key_age_days = oldest_key_age / (24 * 60 * 60);
        
        EncryptionStats {
            total_keys,
            active_keys,
            total_encrypted_items,
            total_encrypted_bytes: 0, // Would track this in real implementation
            oldest_key_age_days,
            encryption_operations_today: 0, // Would track this in real implementation
        }
    }
    
    /// List available keys
    pub fn list_keys(&self) -> Vec<&EncryptionKey> {
        self.keys.values().collect()
    }
    
    /// Check if keys need rotation
    pub fn needs_key_rotation(&self) -> bool {
        if let Some(default_key_id) = &self.default_key_id {
            if let Some(key) = self.keys.get(default_key_id) {
                let age = current_timestamp() - key.created_at;
                return age > self.config.key_rotation_interval;
            }
        }
        false
    }
    
    /// Generate key material for the specified algorithm
    fn generate_key_material(&self, algorithm: EncryptionAlgorithm) -> Result<Vec<u8>> {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        
        let key_size = match algorithm {
            EncryptionAlgorithm::Aes256Gcm => 32, // 256 bits
            EncryptionAlgorithm::ChaCha20Poly1305 => 32, // 256 bits
        };
        
        let mut key_data = vec![0u8; key_size];
        rng.fill_bytes(&mut key_data);
        
        Ok(key_data)
    }
    
    /// Generate a unique key ID
    fn generate_key_id(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let id: u64 = rng.gen();
        format!("{:016x}", id)
    }
    
    /// Derive key from user password using PBKDF2
    fn derive_key_from_password(&self, password: &str, salt: &[u8]) -> Result<Vec<u8>> {
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;
        
        let mut derived_key = vec![0u8; 32]; // 256 bits
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut derived_key);
        
        Ok(derived_key)
    }
    
    /// Encrypt data using AES-256-GCM
    fn encrypt_aes256_gcm(&self, data: &[u8], key: &EncryptionKey) -> Result<EncryptedData> {
        use aes_gcm::{Aes256Gcm, Key, Nonce, AeadInPlace, KeyInit};
        use rand::RngCore;
        
        let cipher_key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key.key_data);
        let cipher = Aes256Gcm::new(cipher_key);
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt data
        let mut buffer = data.to_vec();
        let tag = cipher.encrypt_in_place_detached(nonce, b"", &mut buffer)
            .map_err(|e| StorageError::Other(format!("Encryption failed: {}", e)))?;
        
        Ok(EncryptedData {
            key_id: key.key_id.clone(),
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            nonce: nonce_bytes.to_vec(),
            ciphertext: buffer,
            tag: tag.to_vec(),
            encrypted_at: current_timestamp(),
        })
    }
    
    /// Decrypt data using AES-256-GCM
    fn decrypt_aes256_gcm(&self, encrypted_data: &EncryptedData, key: &EncryptionKey) -> Result<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, Key, Nonce, AeadInPlace, KeyInit};
        
        let cipher_key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key.key_data);
        let cipher = Aes256Gcm::new(cipher_key);
        
        let nonce = Nonce::from_slice(&encrypted_data.nonce);
        let mut buffer = encrypted_data.ciphertext.clone();
        let tag = aes_gcm::Tag::from_slice(&encrypted_data.tag);
        
        cipher.decrypt_in_place_detached(nonce, b"", &mut buffer, tag)
            .map_err(|e| StorageError::Other(format!("Decryption failed: {}", e)))?;
        
        Ok(buffer)
    }
    
    /// Encrypt data using ChaCha20-Poly1305
    fn encrypt_chacha20_poly1305(&self, data: &[u8], key: &EncryptionKey) -> Result<EncryptedData> {
        use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, AeadInPlace, KeyInit};
        use rand::RngCore;
        
        let cipher_key = chacha20poly1305::Key::from_slice(&key.key_data);
        let cipher = ChaCha20Poly1305::new(cipher_key);
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt data
        let mut buffer = data.to_vec();
        let tag = cipher.encrypt_in_place_detached(nonce, b"", &mut buffer)
            .map_err(|e| StorageError::Other(format!("Encryption failed: {}", e)))?;
        
        Ok(EncryptedData {
            key_id: key.key_id.clone(),
            algorithm: EncryptionAlgorithm::ChaCha20Poly1305,
            nonce: nonce_bytes.to_vec(),
            ciphertext: buffer,
            tag: tag.to_vec(),
            encrypted_at: current_timestamp(),
        })
    }
    
    /// Decrypt data using ChaCha20-Poly1305
    fn decrypt_chacha20_poly1305(&self, encrypted_data: &EncryptedData, key: &EncryptionKey) -> Result<Vec<u8>> {
        use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, AeadInPlace, KeyInit};
        
        let cipher_key = chacha20poly1305::Key::from_slice(&key.key_data);
        let cipher = ChaCha20Poly1305::new(cipher_key);
        
        let nonce = Nonce::from_slice(&encrypted_data.nonce);
        let mut buffer = encrypted_data.ciphertext.clone();
        let tag = chacha20poly1305::Tag::from_slice(&encrypted_data.tag);
        
        cipher.decrypt_in_place_detached(nonce, b"", &mut buffer, tag)
            .map_err(|e| StorageError::Other(format!("Decryption failed: {}", e)))?;
        
        Ok(buffer)
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            auto_encrypt: true,
            key_rotation_interval: 30 * 24 * 60 * 60, // 30 days
            max_key_age: 90 * 24 * 60 * 60, // 90 days
            require_user_password: false,
        }
    }
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// User interface for managing encryption
pub struct UserEncryptionManager {
    service: EncryptionService,
}

impl UserEncryptionManager {
    /// Create new user encryption manager
    pub fn new(config: EncryptionConfig) -> Self {
        Self {
            service: EncryptionService::new(config),
        }
    }
    
    /// User-friendly key generation
    pub fn create_encryption_key(&mut self, description: &str, password: Option<&str>) -> Result<String> {
        let options = KeyGenerationOptions {
            algorithm: EncryptionAlgorithm::Aes256Gcm, // Default to AES-256-GCM
            description: description.to_string(),
            user_password: password.map(|p| p.to_string()),
        };
        
        self.service.generate_key(options)
    }
    
    /// Encrypt user data
    pub fn protect_data(&mut self, data: &[u8]) -> Result<EncryptedData> {
        self.service.encrypt(data)
    }
    
    /// Decrypt user data
    pub fn unprotect_data(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>> {
        self.service.decrypt(encrypted_data)
    }
    
    /// Get user-friendly encryption status
    pub fn get_encryption_status(&self) -> EncryptionStatus {
        let stats = self.service.get_stats();
        let has_keys = stats.total_keys > 0;
        let needs_rotation = self.service.needs_key_rotation();
        
        EncryptionStatus {
            enabled: has_keys,
            total_keys: stats.total_keys,
            needs_key_rotation: needs_rotation,
            oldest_key_age_days: stats.oldest_key_age_days,
            encryption_strength: if has_keys { "AES-256" } else { "None" }.to_string(),
        }
    }
    
    /// Delete encryption key with confirmation
    pub fn delete_key(&mut self, key_id: &str, confirmation: &str) -> Result<()> {
        if confirmation != "DELETE" {
            return Err(StorageError::Other("Invalid confirmation".to_string()));
        }
        
        self.service.remove_key(key_id)
    }
}

/// User-friendly encryption status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionStatus {
    pub enabled: bool,
    pub total_keys: usize,
    pub needs_key_rotation: bool,
    pub oldest_key_age_days: u64,
    pub encryption_strength: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_service_creation() {
        let config = EncryptionConfig::default();
        let service = EncryptionService::new(config);
        
        let stats = service.get_stats();
        assert_eq!(stats.total_keys, 0);
    }
    
    #[test]
    fn test_key_generation() {
        let config = EncryptionConfig::default();
        let mut service = EncryptionService::new(config);
        
        let options = KeyGenerationOptions {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            description: "Test key".to_string(),
            user_password: None,
        };
        
        let key_id = service.generate_key(options).unwrap();
        assert!(!key_id.is_empty());
        
        let stats = service.get_stats();
        assert_eq!(stats.total_keys, 1);
    }
    
    #[test]
    fn test_encryption_decryption() {
        let config = EncryptionConfig::default();
        let mut service = EncryptionService::new(config);
        
        // Generate key
        let options = KeyGenerationOptions {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            description: "Test key".to_string(),
            user_password: None,
        };
        service.generate_key(options).unwrap();
        
        // Test data
        let original_data = b"This is sensitive test data";
        
        // Encrypt
        let encrypted = service.encrypt(original_data).unwrap();
        assert_ne!(encrypted.ciphertext, original_data);
        
        // Decrypt
        let decrypted = service.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, original_data);
    }
    
    #[test]
    fn test_password_derived_key() {
        let config = EncryptionConfig::default();
        let mut service = EncryptionService::new(config);
        
        let options = KeyGenerationOptions {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            description: "Password-protected key".to_string(),
            user_password: Some("strong_password_123".to_string()),
        };
        
        let key_id = service.generate_key(options).unwrap();
        assert!(!key_id.is_empty());
        
        // Test encryption/decryption still works
        let test_data = b"Password-protected data";
        let encrypted = service.encrypt(test_data).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, test_data);
    }
    
    #[test]
    fn test_user_encryption_manager() {
        let config = EncryptionConfig::default();
        let mut manager = UserEncryptionManager::new(config);
        
        // Create key
        let key_id = manager.create_encryption_key("My personal key", None).unwrap();
        assert!(!key_id.is_empty());
        
        // Check status
        let status = manager.get_encryption_status();
        assert!(status.enabled);
        assert_eq!(status.total_keys, 1);
        assert_eq!(status.encryption_strength, "AES-256");
        
        // Test data protection
        let sensitive_data = b"Personal behavioral data";
        let protected = manager.protect_data(sensitive_data).unwrap();
        let unprotected = manager.unprotect_data(&protected).unwrap();
        assert_eq!(unprotected, sensitive_data);
    }
    
    #[test]
    fn test_key_rotation() {
        let config = EncryptionConfig {
            key_rotation_interval: 1, // 1 second for testing
            ..Default::default()
        };
        let mut service = EncryptionService::new(config);
        
        // Generate initial key
        let options = KeyGenerationOptions {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            description: "Initial key".to_string(),
            user_password: None,
        };
        service.generate_key(options).unwrap();
        
        // Wait for key to age
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        // Check if rotation is needed
        assert!(service.needs_key_rotation());
        
        // Rotate keys
        let rotation_options = KeyGenerationOptions {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            description: "Rotated key".to_string(),
            user_password: None,
        };
        let new_key_id = service.rotate_keys(rotation_options).unwrap();
        assert!(!new_key_id.is_empty());
        
        let stats = service.get_stats();
        assert_eq!(stats.total_keys, 2); // Old key + new key
    }
}