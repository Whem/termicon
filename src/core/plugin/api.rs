//! Plugin API definitions

use super::{PluginError, PluginType};
use bytes::Bytes;

/// Plugin information
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin type
    pub plugin_type: PluginType,
}

/// Context passed to plugins during hook calls
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Session ID (if in session context)
    pub session_id: Option<uuid::Uuid>,
    /// Data (for data-related hooks)
    pub data: Option<Bytes>,
    /// Custom properties
    pub properties: std::collections::HashMap<String, String>,
}

impl PluginContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            session_id: None,
            data: None,
            properties: std::collections::HashMap::new(),
        }
    }

    /// Create context with session ID
    pub fn with_session(session_id: uuid::Uuid) -> Self {
        Self {
            session_id: Some(session_id),
            data: None,
            properties: std::collections::HashMap::new(),
        }
    }

    /// Create context with data
    pub fn with_data(data: Bytes) -> Self {
        Self {
            session_id: None,
            data: Some(data),
            properties: std::collections::HashMap::new(),
        }
    }

    /// Set a property
    pub fn set_property(&mut self, key: &str, value: &str) {
        self.properties.insert(key.to_string(), value.to_string());
    }

    /// Get a property
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }
}

impl Default for PluginContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin API trait
pub trait PluginApi: Send + Sync {
    /// Get plugin information
    fn info(&self) -> &PluginInfo;

    /// Initialize the plugin
    fn initialize(&self) -> Result<(), PluginError>;

    /// Shutdown the plugin
    fn shutdown(&self) -> Result<(), PluginError>;

    /// Call a hook
    fn call_hook(&self, hook: &str, ctx: &PluginContext) -> Result<(), PluginError>;
}

/// Plugin hooks
pub mod hooks {
    /// Called when application starts
    pub const APP_START: &str = "app_start";
    /// Called when application is shutting down
    pub const APP_SHUTDOWN: &str = "app_shutdown";
    /// Called when a session is created
    pub const SESSION_CREATED: &str = "session_created";
    /// Called when a session is closed
    pub const SESSION_CLOSED: &str = "session_closed";
    /// Called when data is received
    pub const DATA_RECEIVED: &str = "data_received";
    /// Called when data is about to be sent
    pub const DATA_SENDING: &str = "data_sending";
    /// Called when a connection is established
    pub const CONNECTED: &str = "connected";
    /// Called when a connection is lost
    pub const DISCONNECTED: &str = "disconnected";
}





