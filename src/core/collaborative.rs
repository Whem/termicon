//! Collaborative / Team Features
//! 
//! Provides:
//! - Profile sharing
//! - Team workspace
//! - Read-only observer mode
//! - Session sharing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// User role in a workspace
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    /// Full access - can modify everything
    Admin,
    /// Can create/modify own sessions and profiles
    Editor,
    /// Can connect and send commands
    Operator,
    /// Read-only - can only observe
    Observer,
}

impl UserRole {
    pub fn can_edit(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Editor)
    }

    pub fn can_send(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Editor | UserRole::Operator)
    }

    pub fn can_connect(&self) -> bool {
        !matches!(self, UserRole::Observer)
    }
}

/// User in a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceUser {
    /// User ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Email
    pub email: String,
    /// Role
    pub role: UserRole,
    /// Is currently online?
    #[serde(skip)]
    pub online: bool,
    /// Currently viewing session
    pub current_session: Option<String>,
    /// Last activity
    pub last_activity: String,
}

impl WorkspaceUser {
    pub fn new(id: &str, name: &str, email: &str, role: UserRole) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            email: email.to_string(),
            role,
            online: false,
            current_session: None,
            last_activity: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// Shared profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedProfile {
    /// Profile ID
    pub id: String,
    /// Profile name
    pub name: String,
    /// Owner user ID
    pub owner_id: String,
    /// Connection type (serial, tcp, ssh, etc.)
    pub connection_type: String,
    /// Connection settings (JSON)
    pub settings: String,
    /// Description
    pub description: String,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Who can access this profile
    pub shared_with: SharedAccess,
    /// Created timestamp
    pub created: String,
    /// Modified timestamp
    pub modified: String,
    /// Version number
    pub version: u32,
}

/// Access level for sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharedAccess {
    /// Only owner
    Private,
    /// Specific users
    Users(Vec<String>),
    /// Everyone in workspace
    Workspace,
    /// Public (with link)
    Public { link_id: String },
}

/// Shared session for observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedSession {
    /// Session ID
    pub id: String,
    /// Session name
    pub name: String,
    /// Owner user ID
    pub owner_id: String,
    /// Profile used
    pub profile_id: Option<String>,
    /// Observers currently watching
    pub observers: Vec<String>,
    /// Allow observers to send commands
    pub allow_observer_send: bool,
    /// Started timestamp
    pub started: String,
    /// Is session active?
    pub active: bool,
}

/// Team workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Workspace ID
    pub id: String,
    /// Workspace name
    pub name: String,
    /// Description
    pub description: String,
    /// Owner user ID
    pub owner_id: String,
    /// Members
    pub members: Vec<WorkspaceUser>,
    /// Shared profiles
    pub profiles: Vec<SharedProfile>,
    /// Active shared sessions
    #[serde(skip)]
    pub active_sessions: Vec<SharedSession>,
    /// Created timestamp
    pub created: String,
    /// Settings
    pub settings: WorkspaceSettings,
}

/// Workspace settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceSettings {
    /// Allow self-registration
    pub allow_join: bool,
    /// Require approval for new members
    pub require_approval: bool,
    /// Default role for new members
    pub default_role: UserRole,
    /// Allow profile sharing
    pub allow_profile_sharing: bool,
    /// Allow session sharing
    pub allow_session_sharing: bool,
    /// Max concurrent sessions
    pub max_sessions: usize,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Observer
    }
}

impl Workspace {
    pub fn new(id: &str, name: &str, owner_id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            owner_id: owner_id.to_string(),
            members: Vec::new(),
            profiles: Vec::new(),
            active_sessions: Vec::new(),
            created: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            settings: WorkspaceSettings::default(),
        }
    }

    /// Add a member
    pub fn add_member(&mut self, user: WorkspaceUser) {
        self.members.push(user);
    }

    /// Remove a member
    pub fn remove_member(&mut self, user_id: &str) {
        self.members.retain(|m| m.id != user_id);
    }

    /// Get member by ID
    pub fn get_member(&self, user_id: &str) -> Option<&WorkspaceUser> {
        self.members.iter().find(|m| m.id == user_id)
    }

    /// Get member by ID (mutable)
    pub fn get_member_mut(&mut self, user_id: &str) -> Option<&mut WorkspaceUser> {
        self.members.iter_mut().find(|m| m.id == user_id)
    }

    /// Share a profile
    pub fn share_profile(&mut self, profile: SharedProfile) {
        self.profiles.push(profile);
    }

    /// Get profiles visible to a user
    pub fn get_visible_profiles(&self, user_id: &str) -> Vec<&SharedProfile> {
        self.profiles.iter()
            .filter(|p| {
                p.owner_id == user_id || match &p.shared_with {
                    SharedAccess::Private => false,
                    SharedAccess::Users(users) => users.contains(&user_id.to_string()),
                    SharedAccess::Workspace => true,
                    SharedAccess::Public { .. } => true,
                }
            })
            .collect()
    }

    /// Start a shared session
    pub fn start_shared_session(&mut self, session: SharedSession) {
        self.active_sessions.push(session);
    }

    /// End a shared session
    pub fn end_shared_session(&mut self, session_id: &str) {
        self.active_sessions.retain(|s| s.id != session_id);
    }

    /// Join as observer
    pub fn join_session(&mut self, session_id: &str, user_id: &str) -> Result<(), String> {
        if let Some(session) = self.active_sessions.iter_mut().find(|s| s.id == session_id) {
            if !session.observers.contains(&user_id.to_string()) {
                session.observers.push(user_id.to_string());
            }
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Leave as observer
    pub fn leave_session(&mut self, session_id: &str, user_id: &str) {
        if let Some(session) = self.active_sessions.iter_mut().find(|s| s.id == session_id) {
            session.observers.retain(|o| o != user_id);
        }
    }
}

/// Workspace manager
#[derive(Debug, Default)]
pub struct WorkspaceManager {
    /// Current user ID
    pub current_user_id: Option<String>,
    /// Workspaces
    pub workspaces: HashMap<String, Workspace>,
    /// Active workspace ID
    pub active_workspace: Option<String>,
    /// Config path
    config_path: Option<PathBuf>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        let mut manager = Self::default();
        
        // Try to load from config
        if let Some(path) = Self::config_file_path() {
            manager.config_path = Some(path.clone());
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(loaded) = serde_json::from_str::<Vec<Workspace>>(&data) {
                    for ws in loaded {
                        manager.workspaces.insert(ws.id.clone(), ws);
                    }
                }
            }
        }

        manager
    }

    fn config_file_path() -> Option<PathBuf> {
        std::env::var("APPDATA").ok()
            .map(|p| PathBuf::from(p).join("termicon").join("workspaces.json"))
    }

    pub fn save(&self) {
        if let Some(ref path) = self.config_path {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let workspaces: Vec<_> = self.workspaces.values().collect();
            if let Ok(json) = serde_json::to_string_pretty(&workspaces) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    /// Create a workspace
    pub fn create_workspace(&mut self, id: &str, name: &str, owner_id: &str) -> &mut Workspace {
        let workspace = Workspace::new(id, name, owner_id);
        self.workspaces.insert(id.to_string(), workspace);
        self.save();
        self.workspaces.get_mut(id).unwrap()
    }

    /// Get workspace
    pub fn get_workspace(&self, id: &str) -> Option<&Workspace> {
        self.workspaces.get(id)
    }

    /// Get workspace (mutable)
    pub fn get_workspace_mut(&mut self, id: &str) -> Option<&mut Workspace> {
        self.workspaces.get_mut(id)
    }

    /// Get active workspace
    pub fn active(&self) -> Option<&Workspace> {
        self.active_workspace.as_ref()
            .and_then(|id| self.workspaces.get(id))
    }

    /// Get active workspace (mutable)
    pub fn active_mut(&mut self) -> Option<&mut Workspace> {
        let id = self.active_workspace.clone()?;
        self.workspaces.get_mut(&id)
    }

    /// Set active workspace
    pub fn set_active(&mut self, id: &str) {
        if self.workspaces.contains_key(id) {
            self.active_workspace = Some(id.to_string());
        }
    }

    /// Export profile to shareable format
    pub fn export_profile(&self, workspace_id: &str, profile_id: &str) -> Option<String> {
        let workspace = self.workspaces.get(workspace_id)?;
        let profile = workspace.profiles.iter().find(|p| p.id == profile_id)?;
        serde_json::to_string_pretty(profile).ok()
    }

    /// Import profile from shareable format
    pub fn import_profile(&mut self, workspace_id: &str, json: &str) -> Result<(), String> {
        let profile: SharedProfile = serde_json::from_str(json)
            .map_err(|e| format!("Invalid profile format: {}", e))?;
        
        if let Some(workspace) = self.workspaces.get_mut(workspace_id) {
            workspace.share_profile(profile);
            self.save();
            Ok(())
        } else {
            Err("Workspace not found".to_string())
        }
    }
}

/// Message for real-time collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollabMessage {
    /// User joined session
    UserJoined { user_id: String, user_name: String },
    /// User left session
    UserLeft { user_id: String },
    /// Terminal output
    TerminalOutput { data: Vec<u8> },
    /// Command sent
    CommandSent { user_id: String, command: String },
    /// Session ended
    SessionEnded,
    /// Chat message
    Chat { user_id: String, message: String },
    /// Cursor position update
    CursorUpdate { user_id: String, row: u32, col: u32 },
}

