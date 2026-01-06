//! Keyboard Navigation and Shortcuts
//!
//! Provides comprehensive keyboard navigation for accessibility
//! and power user productivity.

use egui::{self, Key, Modifiers, Context};
use std::collections::HashMap;

/// Keyboard action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyAction {
    // Navigation
    NextTab,
    PrevTab,
    NextPane,
    PrevPane,
    FocusTerminal,
    FocusSidebar,
    FocusCommandLine,
    
    // Splits
    SplitHorizontal,
    SplitVertical,
    ClosePane,
    MaximizePane,
    
    // Session
    NewConnection,
    CloseConnection,
    Reconnect,
    Disconnect,
    
    // View
    ToggleSidebar,
    ToggleHexView,
    ToggleChart,
    ToggleFullscreen,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    
    // Edit
    Copy,
    Paste,
    SelectAll,
    ClearTerminal,
    ClearScrollback,
    
    // Search
    Find,
    FindNext,
    FindPrev,
    
    // Tools
    CommandPalette,
    QuickConnect,
    Snippets,
    Macros,
    Settings,
    
    // File Transfer
    SendFile,
    ReceiveFile,
    
    // Macros
    RecordMacro,
    StopRecording,
    PlayMacro,
    QuickMacro(u8), // M1-M24
    
    // Misc
    ToggleTheme,
    Help,
    Quit,
}

/// Keyboard shortcut definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl Shortcut {
    pub const fn new(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }
    
    pub fn ctrl(key: Key) -> Self {
        Self::new(key, Modifiers::CTRL)
    }
    
    pub fn ctrl_shift(key: Key) -> Self {
        Self::new(key, Modifiers::CTRL.plus(Modifiers::SHIFT))
    }
    
    pub fn alt(key: Key) -> Self {
        Self::new(key, Modifiers::ALT)
    }
    
    pub fn none(key: Key) -> Self {
        Self::new(key, Modifiers::NONE)
    }
    
    /// Check if this shortcut is pressed
    pub fn pressed(&self, ctx: &Context) -> bool {
        ctx.input(|i| {
            i.key_pressed(self.key) && i.modifiers == self.modifiers
        })
    }
    
    /// Format as display string
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        
        if self.modifiers.ctrl {
            parts.push("Ctrl");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        if self.modifiers.mac_cmd || self.modifiers.command {
            parts.push("Cmd");
        }
        
        parts.push(key_name(self.key));
        parts.join("+")
    }
}

fn key_name(key: Key) -> &'static str {
    match key {
        Key::A => "A",
        Key::B => "B",
        Key::C => "C",
        Key::D => "D",
        Key::E => "E",
        Key::F => "F",
        Key::G => "G",
        Key::H => "H",
        Key::I => "I",
        Key::J => "J",
        Key::K => "K",
        Key::L => "L",
        Key::M => "M",
        Key::N => "N",
        Key::O => "O",
        Key::P => "P",
        Key::Q => "Q",
        Key::R => "R",
        Key::S => "S",
        Key::T => "T",
        Key::U => "U",
        Key::V => "V",
        Key::W => "W",
        Key::X => "X",
        Key::Y => "Y",
        Key::Z => "Z",
        Key::Num0 => "0",
        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::Num5 => "5",
        Key::Num6 => "6",
        Key::Num7 => "7",
        Key::Num8 => "8",
        Key::Num9 => "9",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::Escape => "Esc",
        Key::Tab => "Tab",
        Key::Backspace => "Backspace",
        Key::Enter => "Enter",
        Key::Space => "Space",
        Key::Insert => "Insert",
        Key::Delete => "Delete",
        Key::Home => "Home",
        Key::End => "End",
        Key::PageUp => "PageUp",
        Key::PageDown => "PageDown",
        Key::ArrowLeft => "Left",
        Key::ArrowRight => "Right",
        Key::ArrowUp => "Up",
        Key::ArrowDown => "Down",
        _ => "?",
    }
}

/// Keyboard shortcut manager
#[derive(Debug)]
pub struct KeyboardManager {
    shortcuts: HashMap<KeyAction, Shortcut>,
    custom_bindings: HashMap<Shortcut, KeyAction>,
}

impl Default for KeyboardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardManager {
    pub fn new() -> Self {
        let mut manager = Self {
            shortcuts: HashMap::new(),
            custom_bindings: HashMap::new(),
        };
        manager.load_defaults();
        manager
    }
    
    /// Load default keyboard shortcuts
    fn load_defaults(&mut self) {
        // Navigation
        self.bind(KeyAction::NextTab, Shortcut::ctrl(Key::Tab));
        self.bind(KeyAction::PrevTab, Shortcut::ctrl_shift(Key::Tab));
        self.bind(KeyAction::NextPane, Shortcut::alt(Key::ArrowRight));
        self.bind(KeyAction::PrevPane, Shortcut::alt(Key::ArrowLeft));
        
        // Splits
        self.bind(KeyAction::SplitHorizontal, Shortcut::ctrl_shift(Key::H));
        self.bind(KeyAction::SplitVertical, Shortcut::ctrl_shift(Key::V));
        self.bind(KeyAction::ClosePane, Shortcut::ctrl_shift(Key::W));
        
        // Session
        self.bind(KeyAction::NewConnection, Shortcut::ctrl(Key::N));
        self.bind(KeyAction::CloseConnection, Shortcut::ctrl(Key::W));
        self.bind(KeyAction::Reconnect, Shortcut::ctrl(Key::R));
        
        // View
        self.bind(KeyAction::ToggleSidebar, Shortcut::ctrl(Key::B));
        self.bind(KeyAction::ToggleHexView, Shortcut::ctrl(Key::H));
        self.bind(KeyAction::ToggleChart, Shortcut::ctrl_shift(Key::C));
        self.bind(KeyAction::ToggleFullscreen, Shortcut::none(Key::F11));
        self.bind(KeyAction::ZoomIn, Shortcut::ctrl(Key::Plus));
        self.bind(KeyAction::ZoomOut, Shortcut::ctrl(Key::Minus));
        
        // Edit
        self.bind(KeyAction::Copy, Shortcut::ctrl(Key::C));
        self.bind(KeyAction::Paste, Shortcut::ctrl(Key::V));
        self.bind(KeyAction::SelectAll, Shortcut::ctrl(Key::A));
        self.bind(KeyAction::ClearTerminal, Shortcut::ctrl(Key::L));
        
        // Search
        self.bind(KeyAction::Find, Shortcut::ctrl(Key::F));
        self.bind(KeyAction::FindNext, Shortcut::none(Key::F3));
        self.bind(KeyAction::FindPrev, Shortcut::ctrl_shift(Key::G));
        
        // Tools
        self.bind(KeyAction::CommandPalette, Shortcut::ctrl(Key::K));
        self.bind(KeyAction::QuickConnect, Shortcut::ctrl_shift(Key::N));
        self.bind(KeyAction::Settings, Shortcut::ctrl(Key::Comma));
        
        // Macros
        self.bind(KeyAction::RecordMacro, Shortcut::ctrl_shift(Key::R));
        
        // Quick macros F1-F12
        self.bind(KeyAction::QuickMacro(1), Shortcut::none(Key::F1));
        self.bind(KeyAction::QuickMacro(2), Shortcut::none(Key::F2));
        self.bind(KeyAction::QuickMacro(3), Shortcut::none(Key::F3));
        self.bind(KeyAction::QuickMacro(4), Shortcut::none(Key::F4));
        self.bind(KeyAction::QuickMacro(5), Shortcut::none(Key::F5));
        self.bind(KeyAction::QuickMacro(6), Shortcut::none(Key::F6));
        self.bind(KeyAction::QuickMacro(7), Shortcut::none(Key::F7));
        self.bind(KeyAction::QuickMacro(8), Shortcut::none(Key::F8));
        self.bind(KeyAction::QuickMacro(9), Shortcut::none(Key::F9));
        self.bind(KeyAction::QuickMacro(10), Shortcut::none(Key::F10));
        self.bind(KeyAction::QuickMacro(11), Shortcut::none(Key::F11));
        self.bind(KeyAction::QuickMacro(12), Shortcut::none(Key::F12));
        
        // Shift+F1-F12 for M13-M24
        self.bind(KeyAction::QuickMacro(13), Shortcut::new(Key::F1, Modifiers::SHIFT));
        self.bind(KeyAction::QuickMacro(14), Shortcut::new(Key::F2, Modifiers::SHIFT));
        self.bind(KeyAction::QuickMacro(15), Shortcut::new(Key::F3, Modifiers::SHIFT));
        self.bind(KeyAction::QuickMacro(16), Shortcut::new(Key::F4, Modifiers::SHIFT));
        self.bind(KeyAction::QuickMacro(17), Shortcut::new(Key::F5, Modifiers::SHIFT));
        self.bind(KeyAction::QuickMacro(18), Shortcut::new(Key::F6, Modifiers::SHIFT));
        self.bind(KeyAction::QuickMacro(19), Shortcut::new(Key::F7, Modifiers::SHIFT));
        self.bind(KeyAction::QuickMacro(20), Shortcut::new(Key::F8, Modifiers::SHIFT));
        self.bind(KeyAction::QuickMacro(21), Shortcut::new(Key::F9, Modifiers::SHIFT));
        self.bind(KeyAction::QuickMacro(22), Shortcut::new(Key::F10, Modifiers::SHIFT));
        self.bind(KeyAction::QuickMacro(23), Shortcut::new(Key::F11, Modifiers::SHIFT));
        self.bind(KeyAction::QuickMacro(24), Shortcut::new(Key::F12, Modifiers::SHIFT));
        
        // Misc
        self.bind(KeyAction::ToggleTheme, Shortcut::ctrl_shift(Key::T));
        self.bind(KeyAction::Help, Shortcut::none(Key::F1));
        self.bind(KeyAction::Quit, Shortcut::ctrl(Key::Q));
    }
    
    /// Bind a shortcut to an action
    pub fn bind(&mut self, action: KeyAction, shortcut: Shortcut) {
        self.shortcuts.insert(action, shortcut);
        self.custom_bindings.insert(shortcut, action);
    }
    
    /// Get shortcut for action
    pub fn get_shortcut(&self, action: KeyAction) -> Option<Shortcut> {
        self.shortcuts.get(&action).copied()
    }
    
    /// Get action for shortcut
    pub fn get_action(&self, shortcut: Shortcut) -> Option<KeyAction> {
        self.custom_bindings.get(&shortcut).copied()
    }
    
    /// Check all shortcuts and return triggered action
    pub fn check(&self, ctx: &Context) -> Option<KeyAction> {
        for (action, shortcut) in &self.shortcuts {
            if shortcut.pressed(ctx) {
                return Some(*action);
            }
        }
        None
    }
    
    /// Get shortcut display string for action
    pub fn shortcut_text(&self, action: KeyAction) -> String {
        self.shortcuts
            .get(&action)
            .map(|s| s.display())
            .unwrap_or_default()
    }
    
    /// Export bindings to JSON
    pub fn export_json(&self) -> String {
        serde_json::to_string_pretty(&self.shortcuts_to_strings()).unwrap_or_default()
    }
    
    fn shortcuts_to_strings(&self) -> HashMap<String, String> {
        self.shortcuts
            .iter()
            .map(|(action, shortcut)| {
                (format!("{:?}", action), shortcut.display())
            })
            .collect()
    }
}

/// Focus navigation helper
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTarget {
    Terminal,
    Sidebar,
    CommandLine,
    Toolbar,
    StatusBar,
    Dialog,
}

/// Focus manager for keyboard navigation
#[derive(Debug)]
pub struct FocusManager {
    current: FocusTarget,
    history: Vec<FocusTarget>,
    tab_order: Vec<FocusTarget>,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            current: FocusTarget::Terminal,
            history: Vec::new(),
            tab_order: vec![
                FocusTarget::Sidebar,
                FocusTarget::Terminal,
                FocusTarget::CommandLine,
            ],
        }
    }
    
    /// Get current focus
    pub fn current(&self) -> FocusTarget {
        self.current
    }
    
    /// Set focus
    pub fn set_focus(&mut self, target: FocusTarget) {
        if self.current != target {
            self.history.push(self.current);
            self.current = target;
        }
    }
    
    /// Go back to previous focus
    pub fn go_back(&mut self) -> FocusTarget {
        if let Some(prev) = self.history.pop() {
            self.current = prev;
        }
        self.current
    }
    
    /// Move to next in tab order
    pub fn tab_next(&mut self) -> FocusTarget {
        if let Some(pos) = self.tab_order.iter().position(|&t| t == self.current) {
            let next = (pos + 1) % self.tab_order.len();
            self.set_focus(self.tab_order[next]);
        }
        self.current
    }
    
    /// Move to previous in tab order
    pub fn tab_prev(&mut self) -> FocusTarget {
        if let Some(pos) = self.tab_order.iter().position(|&t| t == self.current) {
            let prev = if pos == 0 { self.tab_order.len() - 1 } else { pos - 1 };
            self.set_focus(self.tab_order[prev]);
        }
        self.current
    }
    
    /// Check if target is focused
    pub fn is_focused(&self, target: FocusTarget) -> bool {
        self.current == target
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shortcut_display() {
        let shortcut = Shortcut::ctrl(Key::K);
        assert_eq!(shortcut.display(), "Ctrl+K");
        
        let shortcut = Shortcut::ctrl_shift(Key::R);
        assert_eq!(shortcut.display(), "Ctrl+Shift+R");
    }
    
    #[test]
    fn test_keyboard_manager() {
        let manager = KeyboardManager::new();
        
        let shortcut = manager.get_shortcut(KeyAction::CommandPalette);
        assert!(shortcut.is_some());
        
        let text = manager.shortcut_text(KeyAction::Copy);
        assert_eq!(text, "Ctrl+C");
    }
    
    #[test]
    fn test_focus_manager() {
        let mut focus = FocusManager::new();
        
        assert_eq!(focus.current(), FocusTarget::Terminal);
        
        focus.set_focus(FocusTarget::Sidebar);
        assert_eq!(focus.current(), FocusTarget::Sidebar);
        
        focus.go_back();
        assert_eq!(focus.current(), FocusTarget::Terminal);
    }
}


