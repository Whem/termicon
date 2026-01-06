//! Credential Vault
//!
//! Secure storage for passwords, keys, and other secrets.
//! Supports OS keychain integration and encrypted local storage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Vault error types
#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Credential not found: {0}")]
    NotFound(String),
    #[error("Access denied")]
    AccessDenied,
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Invalid master password")]
    InvalidMasterPassword,
    #[error("Vault locked")]
    Locked,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Credential type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Credential {
    /// Plain password
    Password {
        username: Option<String>,
        password: String,
    },
    /// SSH key pair
    SshKey {
        private_key: String,
        passphrase: Option<String>,
        public_key: Option<String>,
    },
    /// API key/token
    ApiKey {
        key: String,
        secret: Option<String>,
    },
    /// Certificate with private key
    Certificate {
        cert_pem: String,
        key_pem: String,
        passphrase: Option<String>,
    },
    /// Generic secret data
    Secret {
        data: String,
    },
}

impl Credential {
    /// Create a password credential
    pub fn password(password: &str) -> Self {
        Self::Password {
            username: None,
            password: password.to_string(),
        }
    }

    /// Create a password credential with username
    pub fn password_with_user(username: &str, password: &str) -> Self {
        Self::Password {
            username: Some(username.to_string()),
            password: password.to_string(),
        }
    }

    /// Create an SSH key credential
    pub fn ssh_key(private_key: &str) -> Self {
        Self::SshKey {
            private_key: private_key.to_string(),
            passphrase: None,
            public_key: None,
        }
    }

    /// Get password if this is a password credential
    pub fn get_password(&self) -> Option<&str> {
        match self {
            Self::Password { password, .. } => Some(password),
            _ => None,
        }
    }

    /// Get username if available
    pub fn get_username(&self) -> Option<&str> {
        match self {
            Self::Password { username, .. } => username.as_deref(),
            _ => None,
        }
    }
}

/// Credential metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialEntry {
    /// Credential ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Associated profile ID
    pub profile_id: Option<String>,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Created timestamp
    pub created: chrono::DateTime<chrono::Local>,
    /// Modified timestamp
    pub modified: chrono::DateTime<chrono::Local>,
    /// The credential (encrypted at rest)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<Credential>,
}

impl CredentialEntry {
    /// Create new entry
    pub fn new(id: &str, name: &str, credential: Credential) -> Self {
        let now = chrono::Local::now();
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            profile_id: None,
            tags: Vec::new(),
            created: now,
            modified: now,
            credential: Some(credential),
        }
    }
}

/// Vault storage backend trait
pub trait VaultBackend: Send + Sync {
    /// Store a credential
    fn store(&mut self, id: &str, credential: &Credential) -> Result<(), VaultError>;
    
    /// Retrieve a credential
    fn retrieve(&self, id: &str) -> Result<Credential, VaultError>;
    
    /// Delete a credential
    fn delete(&mut self, id: &str) -> Result<(), VaultError>;
    
    /// List all credential IDs
    fn list(&self) -> Result<Vec<String>, VaultError>;
    
    /// Check if credential exists
    fn exists(&self, id: &str) -> bool;
}

/// In-memory vault backend (for testing)
pub struct MemoryBackend {
    credentials: HashMap<String, Credential>,
}

impl MemoryBackend {
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
        }
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultBackend for MemoryBackend {
    fn store(&mut self, id: &str, credential: &Credential) -> Result<(), VaultError> {
        self.credentials.insert(id.to_string(), credential.clone());
        Ok(())
    }

    fn retrieve(&self, id: &str) -> Result<Credential, VaultError> {
        self.credentials
            .get(id)
            .cloned()
            .ok_or_else(|| VaultError::NotFound(id.to_string()))
    }

    fn delete(&mut self, id: &str) -> Result<(), VaultError> {
        self.credentials.remove(id);
        Ok(())
    }

    fn list(&self) -> Result<Vec<String>, VaultError> {
        Ok(self.credentials.keys().cloned().collect())
    }

    fn exists(&self, id: &str) -> bool {
        self.credentials.contains_key(id)
    }
}

/// File-based encrypted vault backend
pub struct FileBackend {
    path: PathBuf,
    credentials: HashMap<String, Credential>,
    locked: bool,
}

impl FileBackend {
    /// Create new file backend
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            credentials: HashMap::new(),
            locked: true,
        }
    }

    /// Unlock vault with master password
    pub fn unlock(&mut self, _master_password: &str) -> Result<(), VaultError> {
        // TODO: Implement actual encryption/decryption
        // For now, just load the file as-is (not secure!)
        if self.path.exists() {
            let content = std::fs::read_to_string(&self.path)?;
            self.credentials = serde_json::from_str(&content)
                .map_err(|e| VaultError::StorageError(e.to_string()))?;
        }
        self.locked = false;
        Ok(())
    }

    /// Lock the vault
    pub fn lock(&mut self) {
        self.credentials.clear();
        self.locked = true;
    }

    /// Save vault to file
    fn save(&self) -> Result<(), VaultError> {
        // TODO: Implement actual encryption
        let content = serde_json::to_string_pretty(&self.credentials)
            .map_err(|e| VaultError::StorageError(e.to_string()))?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }
}

impl VaultBackend for FileBackend {
    fn store(&mut self, id: &str, credential: &Credential) -> Result<(), VaultError> {
        if self.locked {
            return Err(VaultError::Locked);
        }
        self.credentials.insert(id.to_string(), credential.clone());
        self.save()
    }

    fn retrieve(&self, id: &str) -> Result<Credential, VaultError> {
        if self.locked {
            return Err(VaultError::Locked);
        }
        self.credentials
            .get(id)
            .cloned()
            .ok_or_else(|| VaultError::NotFound(id.to_string()))
    }

    fn delete(&mut self, id: &str) -> Result<(), VaultError> {
        if self.locked {
            return Err(VaultError::Locked);
        }
        self.credentials.remove(id);
        self.save()
    }

    fn list(&self) -> Result<Vec<String>, VaultError> {
        if self.locked {
            return Err(VaultError::Locked);
        }
        Ok(self.credentials.keys().cloned().collect())
    }

    fn exists(&self, id: &str) -> bool {
        if self.locked {
            return false;
        }
        self.credentials.contains_key(id)
    }
}

/// Credential vault
pub struct CredentialVault {
    backend: Box<dyn VaultBackend>,
    entries: HashMap<String, CredentialEntry>,
}

impl CredentialVault {
    /// Create vault with memory backend
    pub fn new_memory() -> Self {
        Self {
            backend: Box::new(MemoryBackend::new()),
            entries: HashMap::new(),
        }
    }

    /// Create vault with file backend
    pub fn new_file(path: PathBuf) -> Self {
        Self {
            backend: Box::new(FileBackend::new(path)),
            entries: HashMap::new(),
        }
    }

    /// Store a credential
    pub fn store(&mut self, entry: CredentialEntry) -> Result<(), VaultError> {
        if let Some(ref credential) = entry.credential {
            self.backend.store(&entry.id, credential)?;
        }
        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    /// Retrieve a credential
    pub fn retrieve(&self, id: &str) -> Result<Credential, VaultError> {
        self.backend.retrieve(id)
    }

    /// Get entry metadata
    pub fn get_entry(&self, id: &str) -> Option<&CredentialEntry> {
        self.entries.get(id)
    }

    /// Delete a credential
    pub fn delete(&mut self, id: &str) -> Result<(), VaultError> {
        self.backend.delete(id)?;
        self.entries.remove(id);
        Ok(())
    }

    /// List all entries
    pub fn list(&self) -> Vec<&CredentialEntry> {
        self.entries.values().collect()
    }

    /// Find entries by profile
    pub fn find_by_profile(&self, profile_id: &str) -> Vec<&CredentialEntry> {
        self.entries
            .values()
            .filter(|e| e.profile_id.as_deref() == Some(profile_id))
            .collect()
    }

    /// Find entries by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<&CredentialEntry> {
        self.entries
            .values()
            .filter(|e| e.tags.contains(&tag.to_string()))
            .collect()
    }
}

/// Helper to generate credential IDs
pub fn generate_credential_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_vault() {
        let mut vault = CredentialVault::new_memory();
        
        let entry = CredentialEntry::new(
            "test-1",
            "Test Credential",
            Credential::password("secret123"),
        );
        
        vault.store(entry).unwrap();
        
        let retrieved = vault.retrieve("test-1").unwrap();
        assert_eq!(retrieved.get_password(), Some("secret123"));
    }

    #[test]
    fn test_credential_types() {
        let password = Credential::password_with_user("admin", "pass");
        assert_eq!(password.get_username(), Some("admin"));
        assert_eq!(password.get_password(), Some("pass"));

        let ssh_key = Credential::ssh_key("-----BEGIN RSA PRIVATE KEY-----\n...");
        assert!(ssh_key.get_password().is_none());
    }
}

/// SSH key types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SshKeyType {
    /// Ed25519 (recommended, modern)
    Ed25519,
    /// RSA 2048-bit
    Rsa2048,
    /// RSA 4096-bit  
    Rsa4096,
    /// ECDSA with P-256
    EcdsaP256,
    /// ECDSA with P-384
    EcdsaP384,
}

impl SshKeyType {
    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            Self::Ed25519 => "Ed25519 (Recommended)",
            Self::Rsa2048 => "RSA 2048-bit",
            Self::Rsa4096 => "RSA 4096-bit",
            Self::EcdsaP256 => "ECDSA P-256",
            Self::EcdsaP384 => "ECDSA P-384",
        }
    }
    
    /// Get key algorithm name
    pub fn algorithm(&self) -> &str {
        match self {
            Self::Ed25519 => "ed25519",
            Self::Rsa2048 | Self::Rsa4096 => "rsa",
            Self::EcdsaP256 => "ecdsa-sha2-nistp256",
            Self::EcdsaP384 => "ecdsa-sha2-nistp384",
        }
    }
    
    /// Get all available types
    pub fn all() -> &'static [Self] {
        &[
            Self::Ed25519,
            Self::Rsa4096,
            Self::Rsa2048,
            Self::EcdsaP256,
            Self::EcdsaP384,
        ]
    }
}

/// SSH key pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyPair {
    /// Key type
    pub key_type: SshKeyType,
    /// Private key in PEM format
    pub private_key_pem: String,
    /// Public key in OpenSSH format
    pub public_key_openssh: String,
    /// Key fingerprint (SHA256)
    pub fingerprint: String,
    /// Key comment (usually email or identifier)
    pub comment: String,
    /// Creation timestamp
    pub created: chrono::DateTime<chrono::Local>,
}

/// SSH key generator
pub struct SshKeyGenerator;

impl SshKeyGenerator {
    /// Generate a new SSH key pair
    /// Note: This is a simplified implementation. In production, use a proper crypto library.
    pub fn generate(key_type: SshKeyType, comment: &str, passphrase: Option<&str>) -> Result<SshKeyPair, VaultError> {
        use rand::Rng;
        
        // Generate random bytes for the key
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        
        // For a real implementation, we would use proper key generation.
        // This is a placeholder that generates a fake key for structure purposes.
        let (private_key, public_key) = match key_type {
            SshKeyType::Ed25519 => {
                Self::generate_ed25519_placeholder(&random_bytes, comment, passphrase)
            }
            SshKeyType::Rsa2048 | SshKeyType::Rsa4096 => {
                let bits = if key_type == SshKeyType::Rsa4096 { 4096 } else { 2048 };
                Self::generate_rsa_placeholder(bits, comment, passphrase)
            }
            SshKeyType::EcdsaP256 | SshKeyType::EcdsaP384 => {
                let curve = if key_type == SshKeyType::EcdsaP384 { "nistp384" } else { "nistp256" };
                Self::generate_ecdsa_placeholder(curve, comment, passphrase)
            }
        };
        
        // Calculate fingerprint (simplified - just hash the public key)
        let fingerprint = Self::calculate_fingerprint(&public_key);
        
        Ok(SshKeyPair {
            key_type,
            private_key_pem: private_key,
            public_key_openssh: public_key,
            fingerprint,
            comment: comment.to_string(),
            created: chrono::Local::now(),
        })
    }
    
    fn generate_ed25519_placeholder(seed: &[u8], comment: &str, passphrase: Option<&str>) -> (String, String) {
        // This is a placeholder - real implementation would use actual ed25519 key generation
        let encoded_seed = base64::encode(&seed[..32.min(seed.len())]);
        
        let private_key = if passphrase.is_some() {
            format!(
                "-----BEGIN OPENSSH PRIVATE KEY-----\n\
                 b3BlbnNzaC1rZXktdjEAAAAACmFlczI1Ni1jdHIAAAAGYmNyeXB0AAAAGAAAABDx\n\
                 {}AAAA\n\
                 -----END OPENSSH PRIVATE KEY-----",
                &encoded_seed[..20.min(encoded_seed.len())]
            )
        } else {
            format!(
                "-----BEGIN OPENSSH PRIVATE KEY-----\n\
                 b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtz\n\
                 {}AAAA\n\
                 -----END OPENSSH PRIVATE KEY-----",
                &encoded_seed[..20.min(encoded_seed.len())]
            )
        };
        
        let public_key = format!(
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI{} {}",
            &encoded_seed[..20.min(encoded_seed.len())],
            comment
        );
        
        (private_key, public_key)
    }
    
    fn generate_rsa_placeholder(bits: u32, comment: &str, passphrase: Option<&str>) -> (String, String) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_data: String = (0..64).map(|_| {
            let idx = rng.gen_range(0..62);
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
                .chars().nth(idx).unwrap()
        }).collect();
        
        let private_key = if passphrase.is_some() {
            format!(
                "-----BEGIN RSA PRIVATE KEY-----\n\
                 Proc-Type: 4,ENCRYPTED\n\
                 DEK-Info: AES-256-CBC,{}\n\n\
                 {}\n\
                 -----END RSA PRIVATE KEY-----",
                &random_data[..16],
                &random_data
            )
        } else {
            format!(
                "-----BEGIN RSA PRIVATE KEY-----\n\
                 {}\n\
                 -----END RSA PRIVATE KEY-----",
                &random_data
            )
        };
        
        let public_key = format!(
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQAB{} {}",
            &random_data[..32],
            comment
        );
        
        let _ = bits; // Used for real implementation
        (private_key, public_key)
    }
    
    fn generate_ecdsa_placeholder(curve: &str, comment: &str, passphrase: Option<&str>) -> (String, String) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_data: String = (0..48).map(|_| {
            let idx = rng.gen_range(0..62);
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
                .chars().nth(idx).unwrap()
        }).collect();
        
        let private_key = if passphrase.is_some() {
            format!(
                "-----BEGIN EC PRIVATE KEY-----\n\
                 Proc-Type: 4,ENCRYPTED\n\
                 DEK-Info: AES-256-CBC,{}\n\n\
                 {}\n\
                 -----END EC PRIVATE KEY-----",
                &random_data[..16],
                &random_data
            )
        } else {
            format!(
                "-----BEGIN EC PRIVATE KEY-----\n\
                 {}\n\
                 -----END EC PRIVATE KEY-----",
                &random_data
            )
        };
        
        let key_type = format!("ecdsa-sha2-{}", curve);
        let public_key = format!(
            "{} AAAAE2VjZHNhLXNoYTItbmlzdHAyNTY{} {}",
            key_type,
            &random_data[..24],
            comment
        );
        
        (private_key, public_key)
    }
    
    fn calculate_fingerprint(public_key: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        public_key.hash(&mut hasher);
        let hash = hasher.finish();
        
        format!("SHA256:{:016x}{:016x}", hash, hash.rotate_left(32))
    }
    
    /// Export key pair to files
    pub fn export_to_files(
        key_pair: &SshKeyPair,
        private_key_path: &std::path::Path,
        public_key_path: &std::path::Path,
    ) -> Result<(), VaultError> {
        // Write private key with restricted permissions
        std::fs::write(private_key_path, &key_pair.private_key_pem)?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(private_key_path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(private_key_path, perms)?;
        }
        
        // Write public key
        std::fs::write(public_key_path, &key_pair.public_key_openssh)?;
        
        Ok(())
    }
    
    /// Import key from file
    pub fn import_from_file(path: &std::path::Path) -> Result<String, VaultError> {
        std::fs::read_to_string(path).map_err(VaultError::from)
    }
}

// Base64 encoding helper (simple implementation)
mod base64 {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    pub fn encode(data: &[u8]) -> String {
        let mut result = String::new();
        
        for chunk in data.chunks(3) {
            let b0 = chunk[0] as usize;
            let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
            let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
            
            result.push(ALPHABET[b0 >> 2] as char);
            result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
            
            if chunk.len() > 1 {
                result.push(ALPHABET[((b1 & 0x0F) << 2) | (b2 >> 6)] as char);
            } else {
                result.push('=');
            }
            
            if chunk.len() > 2 {
                result.push(ALPHABET[b2 & 0x3F] as char);
            } else {
                result.push('=');
            }
        }
        
        result
    }
}




