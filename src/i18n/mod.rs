//! Internationalization (i18n) module
//!
//! Provides multi-language support for the application.
//! Currently supports:
//! - English (en)
//! - Hungarian (hu)

use parking_lot::RwLock;
use std::sync::LazyLock;

/// Supported locales
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    /// English (default)
    #[default]
    English,
    /// Hungarian (Magyar)
    Hungarian,
}

impl Locale {
    /// Get locale code
    pub fn code(&self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Hungarian => "hu",
        }
    }

    /// Get locale display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Hungarian => "Magyar",
        }
    }

    /// Parse from string
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "en" | "en-us" | "en-gb" => Some(Self::English),
            "hu" | "hu-hu" => Some(Self::Hungarian),
            _ => None,
        }
    }

    /// Get all available locales
    pub fn available() -> &'static [Self] {
        &[Self::English, Self::Hungarian]
    }
}

/// Current locale
static CURRENT_LOCALE: LazyLock<RwLock<Locale>> = LazyLock::new(|| RwLock::new(Locale::English));

/// Get current locale
pub fn get_locale() -> Locale {
    *CURRENT_LOCALE.read()
}

/// Set current locale
pub fn set_locale(locale: Locale) {
    *CURRENT_LOCALE.write() = locale;
    rust_i18n::set_locale(locale.code());
}

/// Translate a key using rust-i18n
/// 
/// This is a wrapper around the rust_i18n::t! macro for use in code.
/// For compile-time translations, use the t! macro directly.
pub fn t(key: &str) -> String {
    rust_i18n::t!(key).to_string()
}

/// Translation keys for the application
pub mod keys {
    // Application
    pub const APP_NAME: &str = "app.name";
    pub const APP_VERSION: &str = "app.version";

    // Menu
    pub const MENU_FILE: &str = "menu.file";
    pub const MENU_EDIT: &str = "menu.edit";
    pub const MENU_VIEW: &str = "menu.view";
    pub const MENU_CONNECTION: &str = "menu.connection";
    pub const MENU_TOOLS: &str = "menu.tools";
    pub const MENU_HELP: &str = "menu.help";

    // File menu
    pub const MENU_NEW_SESSION: &str = "menu.new_session";
    pub const MENU_OPEN_SESSION: &str = "menu.open_session";
    pub const MENU_SAVE_SESSION: &str = "menu.save_session";
    pub const MENU_CLOSE_SESSION: &str = "menu.close_session";
    pub const MENU_EXIT: &str = "menu.exit";

    // Connection menu
    pub const MENU_CONNECT: &str = "menu.connect";
    pub const MENU_DISCONNECT: &str = "menu.disconnect";
    pub const MENU_QUICK_CONNECT: &str = "menu.quick_connect";

    // View menu
    pub const MENU_TEXT_VIEW: &str = "menu.text_view";
    pub const MENU_HEX_VIEW: &str = "menu.hex_view";
    pub const MENU_CHART_VIEW: &str = "menu.chart_view";

    // Tools menu
    pub const MENU_SEND_FILE: &str = "menu.send_file";
    pub const MENU_RECEIVE_FILE: &str = "menu.receive_file";
    pub const MENU_MACROS: &str = "menu.macros";
    pub const MENU_TRIGGERS: &str = "menu.triggers";
    pub const MENU_SETTINGS: &str = "menu.settings";

    // Connection dialog
    pub const DIALOG_CONNECTION_TYPE: &str = "dialog.connection_type";
    pub const DIALOG_SERIAL: &str = "dialog.serial";
    pub const DIALOG_TCP: &str = "dialog.tcp";
    pub const DIALOG_TELNET: &str = "dialog.telnet";
    pub const DIALOG_SSH: &str = "dialog.ssh";

    // Serial settings
    pub const SERIAL_PORT: &str = "serial.port";
    pub const SERIAL_BAUD_RATE: &str = "serial.baud_rate";
    pub const SERIAL_DATA_BITS: &str = "serial.data_bits";
    pub const SERIAL_STOP_BITS: &str = "serial.stop_bits";
    pub const SERIAL_PARITY: &str = "serial.parity";
    pub const SERIAL_FLOW_CONTROL: &str = "serial.flow_control";
    pub const SERIAL_PARITY_NONE: &str = "serial.parity_none";
    pub const SERIAL_PARITY_ODD: &str = "serial.parity_odd";
    pub const SERIAL_PARITY_EVEN: &str = "serial.parity_even";
    pub const SERIAL_FLOW_NONE: &str = "serial.flow_none";
    pub const SERIAL_FLOW_HARDWARE: &str = "serial.flow_hardware";
    pub const SERIAL_FLOW_SOFTWARE: &str = "serial.flow_software";

    // Network settings
    pub const NETWORK_HOST: &str = "network.host";
    pub const NETWORK_PORT: &str = "network.port";
    pub const NETWORK_TIMEOUT: &str = "network.timeout";

    // Buttons
    pub const BTN_CONNECT: &str = "btn.connect";
    pub const BTN_DISCONNECT: &str = "btn.disconnect";
    pub const BTN_CANCEL: &str = "btn.cancel";
    pub const BTN_OK: &str = "btn.ok";
    pub const BTN_APPLY: &str = "btn.apply";
    pub const BTN_SAVE: &str = "btn.save";
    pub const BTN_SEND: &str = "btn.send";
    pub const BTN_CLEAR: &str = "btn.clear";
    pub const BTN_REFRESH: &str = "btn.refresh";

    // Status
    pub const STATUS_CONNECTED: &str = "status.connected";
    pub const STATUS_DISCONNECTED: &str = "status.disconnected";
    pub const STATUS_CONNECTING: &str = "status.connecting";
    pub const STATUS_ERROR: &str = "status.error";
    pub const STATUS_BYTES_SENT: &str = "status.bytes_sent";
    pub const STATUS_BYTES_RECEIVED: &str = "status.bytes_received";

    // Logging
    pub const LOG_ENABLED: &str = "log.enabled";
    pub const LOG_FILE: &str = "log.file";
    pub const LOG_FORMAT: &str = "log.format";
    pub const LOG_TIMESTAMPS: &str = "log.timestamps";

    // AutoConnect
    pub const AUTOCONNECT_ENABLED: &str = "autoconnect.enabled";
    pub const AUTOCONNECT_DELAY: &str = "autoconnect.delay";
    pub const AUTOCONNECT_MAX_RETRIES: &str = "autoconnect.max_retries";

    // Errors
    pub const ERROR_CONNECTION_FAILED: &str = "error.connection_failed";
    pub const ERROR_PORT_NOT_FOUND: &str = "error.port_not_found";
    pub const ERROR_PERMISSION_DENIED: &str = "error.permission_denied";
    pub const ERROR_TIMEOUT: &str = "error.timeout";
    pub const ERROR_SEND_FAILED: &str = "error.send_failed";
}





