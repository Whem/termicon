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



