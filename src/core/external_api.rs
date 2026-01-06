//! External Tool Hooks / API
//! 
//! Provides:
//! - REST API for control
//! - WebSocket for real-time data
//! - External trigger outputs
//! - CI/CD integration hooks

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc;

/// API endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    /// Endpoint path (e.g., "/session/create")
    pub path: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Description
    pub description: String,
    /// Required parameters
    pub parameters: Vec<ApiParameter>,
    /// Response type
    pub response_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiParameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

/// API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    /// Request ID
    pub id: String,
    /// Endpoint path
    pub path: String,
    /// Method
    pub method: HttpMethod,
    /// Parameters
    pub params: HashMap<String, serde_json::Value>,
    /// Auth token
    pub auth_token: Option<String>,
}

/// API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    /// Request ID
    pub request_id: String,
    /// Success
    pub success: bool,
    /// HTTP status code
    pub status_code: u16,
    /// Response data
    pub data: serde_json::Value,
    /// Error message
    pub error: Option<String>,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// Subscribe to session output
    Subscribe { session_id: String },
    /// Unsubscribe from session
    Unsubscribe { session_id: String },
    /// Send data to session
    Send { session_id: String, data: String },
    /// Terminal output
    Output { session_id: String, data: String, timestamp: String },
    /// Session status change
    Status { session_id: String, status: String },
    /// Error
    Error { message: String },
    /// Ping/Pong
    Ping,
    Pong,
}

/// External trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerOutput {
    /// Trigger name
    pub name: String,
    /// Trigger type
    pub trigger_type: TriggerType,
    /// Enabled
    pub enabled: bool,
    /// Configuration
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerType {
    /// Webhook (HTTP POST)
    Webhook { url: String },
    /// GPIO pin (for embedded systems)
    GPIO { pin: u8, active_high: bool },
    /// Write to file
    File { path: String },
    /// Execute command
    Command { cmd: String },
    /// MQTT publish
    MQTT { broker: String, topic: String },
    /// Serial DTR/RTS toggle
    SerialSignal { port: String, signal: String },
}

/// External trigger event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEvent {
    /// Event name
    pub name: String,
    /// Event type
    pub event_type: EventType,
    /// Session ID
    pub session_id: Option<String>,
    /// Data
    pub data: HashMap<String, String>,
    /// Timestamp
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    SessionCreated,
    SessionConnected,
    SessionDisconnected,
    SessionError,
    DataReceived,
    DataSent,
    PatternMatched,
    ProtocolEvent,
    Custom(String),
}

/// API server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiServerConfig {
    /// Enable REST API
    pub rest_enabled: bool,
    /// REST API port
    pub rest_port: u16,
    /// Enable WebSocket
    pub websocket_enabled: bool,
    /// WebSocket port
    pub websocket_port: u16,
    /// Enable external triggers
    pub triggers_enabled: bool,
    /// Auth required
    pub auth_required: bool,
    /// API key
    pub api_key: Option<String>,
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        Self {
            rest_enabled: false,
            rest_port: 8080,
            websocket_enabled: false,
            websocket_port: 8081,
            triggers_enabled: false,
            auth_required: false,
            api_key: None,
            cors_origins: vec!["*".to_string()],
        }
    }
}

/// External API manager
#[derive(Debug)]
pub struct ExternalApiManager {
    /// Configuration
    pub config: ApiServerConfig,
    /// Registered endpoints
    pub endpoints: Vec<ApiEndpoint>,
    /// Trigger outputs
    pub triggers: Vec<TriggerOutput>,
    /// Event subscribers (channel senders)
    #[allow(dead_code)]
    subscribers: Vec<mpsc::Sender<TriggerEvent>>,
    /// Pending events
    pub pending_events: Vec<TriggerEvent>,
}

impl Default for ExternalApiManager {
    fn default() -> Self {
        let mut manager = Self {
            config: ApiServerConfig::default(),
            endpoints: Vec::new(),
            triggers: Vec::new(),
            subscribers: Vec::new(),
            pending_events: Vec::new(),
        };
        manager.setup_default_endpoints();
        manager
    }
}

impl ExternalApiManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: ApiServerConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Setup default API endpoints
    fn setup_default_endpoints(&mut self) {
        // Session endpoints
        self.endpoints.push(ApiEndpoint {
            path: "/api/v1/sessions".to_string(),
            method: HttpMethod::GET,
            description: "List all sessions".to_string(),
            parameters: Vec::new(),
            response_type: "Session[]".to_string(),
        });

        self.endpoints.push(ApiEndpoint {
            path: "/api/v1/sessions".to_string(),
            method: HttpMethod::POST,
            description: "Create a new session".to_string(),
            parameters: vec![
                ApiParameter {
                    name: "type".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Connection type (serial, tcp, ssh)".to_string(),
                },
                ApiParameter {
                    name: "config".to_string(),
                    param_type: "object".to_string(),
                    required: true,
                    description: "Connection configuration".to_string(),
                },
            ],
            response_type: "Session".to_string(),
        });

        self.endpoints.push(ApiEndpoint {
            path: "/api/v1/sessions/{id}".to_string(),
            method: HttpMethod::GET,
            description: "Get session details".to_string(),
            parameters: vec![
                ApiParameter {
                    name: "id".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Session ID".to_string(),
                },
            ],
            response_type: "Session".to_string(),
        });

        self.endpoints.push(ApiEndpoint {
            path: "/api/v1/sessions/{id}/send".to_string(),
            method: HttpMethod::POST,
            description: "Send data to session".to_string(),
            parameters: vec![
                ApiParameter {
                    name: "data".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Data to send (string or base64)".to_string(),
                },
            ],
            response_type: "SendResult".to_string(),
        });

        self.endpoints.push(ApiEndpoint {
            path: "/api/v1/sessions/{id}/disconnect".to_string(),
            method: HttpMethod::POST,
            description: "Disconnect session".to_string(),
            parameters: Vec::new(),
            response_type: "Result".to_string(),
        });

        // Profile endpoints
        self.endpoints.push(ApiEndpoint {
            path: "/api/v1/profiles".to_string(),
            method: HttpMethod::GET,
            description: "List all profiles".to_string(),
            parameters: Vec::new(),
            response_type: "Profile[]".to_string(),
        });

        // Macro endpoints
        self.endpoints.push(ApiEndpoint {
            path: "/api/v1/macros/{index}/execute".to_string(),
            method: HttpMethod::POST,
            description: "Execute a macro".to_string(),
            parameters: vec![
                ApiParameter {
                    name: "index".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    description: "Macro index (1-24)".to_string(),
                },
            ],
            response_type: "Result".to_string(),
        });

        // Health check
        self.endpoints.push(ApiEndpoint {
            path: "/api/v1/health".to_string(),
            method: HttpMethod::GET,
            description: "Health check".to_string(),
            parameters: Vec::new(),
            response_type: "HealthStatus".to_string(),
        });
    }

    /// Add a trigger output
    pub fn add_trigger(&mut self, trigger: TriggerOutput) {
        self.triggers.push(trigger);
    }

    /// Emit an event
    pub fn emit_event(&mut self, event: TriggerEvent) {
        // Store pending event
        self.pending_events.push(event.clone());

        // Process triggers
        for trigger in &self.triggers {
            if !trigger.enabled {
                continue;
            }
            self.execute_trigger(trigger, &event);
        }

        // Keep only last 1000 events
        if self.pending_events.len() > 1000 {
            self.pending_events.drain(0..500);
        }
    }

    /// Execute a trigger
    fn execute_trigger(&self, trigger: &TriggerOutput, event: &TriggerEvent) {
        match &trigger.trigger_type {
            TriggerType::Webhook { url } => {
                // Would make HTTP POST in async context
                eprintln!("Webhook trigger: {} -> {}", event.name, url);
            }
            TriggerType::GPIO { pin, active_high } => {
                eprintln!("GPIO trigger: pin {} = {}", pin, active_high);
            }
            TriggerType::File { path } => {
                if let Ok(json) = serde_json::to_string_pretty(event) {
                    let _ = std::fs::write(path, json);
                }
            }
            TriggerType::Command { cmd } => {
                let _ = std::process::Command::new("cmd")
                    .arg("/C")
                    .arg(cmd)
                    .spawn();
            }
            TriggerType::MQTT { broker: _, topic: _ } => {
                eprintln!("MQTT trigger: {}", event.name);
            }
            TriggerType::SerialSignal { port: _, signal: _ } => {
                eprintln!("Serial signal trigger: {}", event.name);
            }
        }
    }

    /// Handle API request
    pub fn handle_request(&self, request: &ApiRequest) -> ApiResponse {
        // Auth check
        if self.config.auth_required {
            if request.auth_token.as_ref() != self.config.api_key.as_ref() {
                return ApiResponse {
                    request_id: request.id.clone(),
                    success: false,
                    status_code: 401,
                    data: serde_json::Value::Null,
                    error: Some("Unauthorized".to_string()),
                };
            }
        }

        // Route request
        match (request.method.clone(), request.path.as_str()) {
            (HttpMethod::GET, "/api/v1/health") => {
                ApiResponse {
                    request_id: request.id.clone(),
                    success: true,
                    status_code: 200,
                    data: serde_json::json!({
                        "status": "ok",
                        "version": env!("CARGO_PKG_VERSION"),
                    }),
                    error: None,
                }
            }
            _ => {
                ApiResponse {
                    request_id: request.id.clone(),
                    success: false,
                    status_code: 404,
                    data: serde_json::Value::Null,
                    error: Some("Not found".to_string()),
                }
            }
        }
    }

    /// Get OpenAPI spec
    pub fn openapi_spec(&self) -> serde_json::Value {
        serde_json::json!({
            "openapi": "3.0.0",
            "info": {
                "title": "Termicon API",
                "version": env!("CARGO_PKG_VERSION"),
                "description": "External API for Termicon terminal emulator"
            },
            "servers": [
                {
                    "url": format!("http://localhost:{}", self.config.rest_port)
                }
            ],
            "paths": self.endpoints.iter().map(|e| {
                (e.path.clone(), serde_json::json!({
                    e.method.to_string().to_lowercase(): {
                        "summary": e.description,
                        "parameters": e.parameters.iter().map(|p| {
                            serde_json::json!({
                                "name": p.name,
                                "in": "query",
                                "required": p.required,
                                "schema": { "type": p.param_type }
                            })
                        }).collect::<Vec<_>>()
                    }
                }))
            }).collect::<HashMap<_, _>>()
        })
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::PATCH => write!(f, "PATCH"),
        }
    }
}

