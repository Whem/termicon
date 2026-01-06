//! SFTP File Browser Panel
//!
//! GUI component for browsing and transferring files over SFTP

use eframe::egui::{self, Color32, RichText, ScrollArea, Vec2};
use std::path::PathBuf;

/// File entry in SFTP browser
#[derive(Debug, Clone)]
pub struct SftpFileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
    pub permissions: String,
}

impl SftpFileEntry {
    pub fn new_dir(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_dir: true,
            size: 0,
            modified: String::new(),
            permissions: "drwxr-xr-x".to_string(),
        }
    }

    pub fn new_file(name: &str, size: u64) -> Self {
        Self {
            name: name.to_string(),
            is_dir: false,
            size,
            modified: String::new(),
            permissions: "-rw-r--r--".to_string(),
        }
    }

    pub fn icon(&self) -> &str {
        if self.is_dir {
            "[D]"
        } else {
            match self.name.rsplit('.').next() {
                Some("txt") | Some("md") | Some("log") => "[T]",
                Some("rs") | Some("py") | Some("js") | Some("c") | Some("h") => "[<>]",
                Some("jpg") | Some("png") | Some("gif") | Some("bmp") => "[I]",
                Some("zip") | Some("tar") | Some("gz") | Some("7z") => "[Z]",
                Some("exe") | Some("bin") | Some("sh") => "[X]",
                _ => "[F]",
            }
        }
    }

    pub fn size_str(&self) -> String {
        if self.is_dir {
            "-".to_string()
        } else if self.size < 1024 {
            format!("{} B", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{:.1} KB", self.size as f64 / 1024.0)
        } else if self.size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", self.size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

/// Transfer status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferStatus {
    Idle,
    Uploading,
    Downloading,
    Complete,
    Error(String),
}

/// Transfer progress
#[derive(Debug, Clone)]
pub struct TransferProgress {
    pub file_name: String,
    pub status: TransferStatus,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
}

impl TransferProgress {
    pub fn percent(&self) -> f32 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.bytes_transferred as f32 / self.total_bytes as f32) * 100.0
        }
    }
}

/// SFTP Panel state
pub struct SftpPanel {
    /// Current remote path
    pub remote_path: String,
    /// Current local path
    pub local_path: String,
    /// Remote file listing
    pub remote_files: Vec<SftpFileEntry>,
    /// Local file listing
    pub local_files: Vec<SftpFileEntry>,
    /// Selected remote files
    pub selected_remote: Vec<usize>,
    /// Selected local files
    pub selected_local: Vec<usize>,
    /// Transfer progress
    pub transfer: Option<TransferProgress>,
    /// Error message
    pub error: Option<String>,
    /// Show hidden files
    pub show_hidden: bool,
    /// Sort by (name, size, date)
    pub sort_by: String,
    /// Connected
    pub connected: bool,
}

impl Default for SftpPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl SftpPanel {
    pub fn new() -> Self {
        Self {
            remote_path: "/home".to_string(),
            local_path: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string()),
            remote_files: Vec::new(),
            local_files: Vec::new(),
            selected_remote: Vec::new(),
            selected_local: Vec::new(),
            transfer: None,
            error: None,
            show_hidden: false,
            sort_by: "name".to_string(),
            connected: false,
        }
    }

    /// Set connected state and load initial listing
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
        if connected {
            // Load demo remote files for now
            self.remote_files = vec![
                SftpFileEntry::new_dir(".."),
                SftpFileEntry::new_dir("Documents"),
                SftpFileEntry::new_dir("Downloads"),
                SftpFileEntry::new_dir(".config"),
                SftpFileEntry::new_file("README.md", 1234),
                SftpFileEntry::new_file("script.sh", 567),
            ];
        } else {
            self.remote_files.clear();
        }
        self.refresh_local();
    }

    /// Refresh local file listing
    pub fn refresh_local(&mut self) {
        self.local_files.clear();
        self.local_files.push(SftpFileEntry::new_dir(".."));

        if let Ok(entries) = std::fs::read_dir(&self.local_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                
                if !self.show_hidden && name.starts_with('.') {
                    continue;
                }

                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        self.local_files.push(SftpFileEntry::new_dir(&name));
                    } else {
                        self.local_files.push(SftpFileEntry::new_file(&name, metadata.len()));
                    }
                }
            }
        }

        Self::sort_files_by(&mut self.local_files, &self.sort_by);
    }

    /// Sort files
    fn sort_files_by(files: &mut [SftpFileEntry], sort_by: &str) {
        files.sort_by(|a, b| {
            // ".." always first
            if a.name == ".." { return std::cmp::Ordering::Less; }
            if b.name == ".." { return std::cmp::Ordering::Greater; }
            
            // Directories before files
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => {
                    match sort_by {
                        "size" => a.size.cmp(&b.size),
                        "date" => a.modified.cmp(&b.modified),
                        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    }
                }
            }
        });
    }

    /// Navigate to remote directory
    pub fn navigate_remote(&mut self, dir: &str) {
        if dir == ".." {
            if let Some(parent) = PathBuf::from(&self.remote_path).parent() {
                self.remote_path = parent.to_string_lossy().to_string();
                if self.remote_path.is_empty() {
                    self.remote_path = "/".to_string();
                }
            }
        } else {
            let new_path = PathBuf::from(&self.remote_path).join(dir);
            self.remote_path = new_path.to_string_lossy().to_string();
        }
        self.selected_remote.clear();
        // In real implementation, would call SFTP list here
    }

    /// Navigate to local directory
    pub fn navigate_local(&mut self, dir: &str) {
        if dir == ".." {
            if let Some(parent) = PathBuf::from(&self.local_path).parent() {
                self.local_path = parent.to_string_lossy().to_string();
            }
        } else {
            let new_path = PathBuf::from(&self.local_path).join(dir);
            self.local_path = new_path.to_string_lossy().to_string();
        }
        self.selected_local.clear();
        self.refresh_local();
    }

    /// Render the SFTP panel
    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Toolbar
        ui.horizontal(|ui| {
            ui.label(RichText::new("📁").size(16.0));
            ui.label(RichText::new("SFTP File Browser").strong());
            
            ui.separator();
            
            if ui.button("🔄 Refresh").clicked() {
                self.refresh_local();
            }
            
            ui.checkbox(&mut self.show_hidden, "Show hidden");
            
            ui.label("Sort:");
            egui::ComboBox::from_id_salt("sort_by")
                .selected_text(&self.sort_by)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort_by, "name".to_string(), "Name");
                    ui.selectable_value(&mut self.sort_by, "size".to_string(), "Size");
                    ui.selectable_value(&mut self.sort_by, "date".to_string(), "Date");
                });
        });

        ui.separator();

        // Error display
        let error_to_show = self.error.clone();
        let mut clear_error = false;
        if let Some(error) = error_to_show {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚠").color(Color32::YELLOW));
                ui.label(RichText::new(&error).color(Color32::YELLOW));
                if ui.small_button("✕").clicked() {
                    clear_error = true;
                }
            });
            ui.separator();
        }
        if clear_error {
            self.error = None;
        }

        // Transfer progress
        if let Some(ref transfer) = self.transfer {
            ui.horizontal(|ui| {
                let status_icon = match transfer.status {
                    TransferStatus::Uploading => "⬆",
                    TransferStatus::Downloading => "⬇",
                    TransferStatus::Complete => "✓",
                    TransferStatus::Error(_) => "✕",
                    TransferStatus::Idle => "",
                };
                ui.label(status_icon);
                ui.label(&transfer.file_name);
                ui.add(egui::ProgressBar::new(transfer.percent() / 100.0).show_percentage());
            });
            ui.separator();
        }

        // Collect navigation actions
        let mut navigate_local_to: Option<String> = None;
        let mut navigate_remote_to: Option<String> = None;
        let mut toggle_local: Option<usize> = None;
        let mut toggle_remote: Option<usize> = None;
        let mut local_path_changed: Option<String> = None;
        let mut remote_path_changed: Option<String> = None;

        // Two-panel file browser
        ui.horizontal(|ui| {
            let panel_width = (ui.available_width() - 40.0) / 2.0;
            
            // Local panel
            ui.vertical(|ui| {
                ui.set_min_width(panel_width);
                ui.set_max_width(panel_width);
                
                ui.label(RichText::new("Local").strong());
                
                // Path bar
                ui.horizontal(|ui| {
                    ui.label("📂");
                    let mut path = self.local_path.clone();
                    if ui.text_edit_singleline(&mut path).changed() {
                        local_path_changed = Some(path);
                    }
                });
                
                // File list
                let local_files: Vec<_> = self.local_files.iter().enumerate()
                    .map(|(i, f)| (i, f.icon().to_string(), f.name.clone(), f.size_str(), f.is_dir))
                    .collect();
                    
                ScrollArea::vertical()
                    .max_height(ui.available_height() - 60.0)
                    .show(ui, |ui| {
                        for (i, icon, name, size, is_dir) in &local_files {
                            let is_selected = self.selected_local.contains(i);
                            let response = ui.selectable_label(
                                is_selected,
                                format!("{} {} {}", icon, name, size)
                            );
                            
                            if response.clicked() {
                                toggle_local = Some(*i);
                            }
                            
                            if response.double_clicked() && *is_dir {
                                navigate_local_to = Some(name.clone());
                            }
                        }
                    });
                
                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("⬆ Upload").on_hover_text("Upload selected to remote").clicked() {
                        // Upload logic
                    }
                });
            });
            
            // Transfer buttons
            ui.vertical(|ui| {
                ui.add_space(100.0);
                if ui.button("➡").on_hover_text("Upload").clicked() {
                    // Upload
                }
                if ui.button("⬅").on_hover_text("Download").clicked() {
                    // Download
                }
            });
            
            // Remote panel
            ui.vertical(|ui| {
                ui.set_min_width(panel_width);
                ui.set_max_width(panel_width);
                
                ui.label(RichText::new("Remote (SFTP)").strong());
                
                if !self.connected {
                    ui.label(RichText::new("Not connected").color(Color32::GRAY));
                    ui.label("Connect via SSH to enable SFTP");
                } else {
                    // Path bar
                    ui.horizontal(|ui| {
                        ui.label("📂");
                        let mut path = self.remote_path.clone();
                        if ui.text_edit_singleline(&mut path).changed() {
                            remote_path_changed = Some(path);
                        }
                    });
                    
                    // File list
                    let remote_files: Vec<_> = self.remote_files.iter().enumerate()
                        .map(|(i, f)| (i, f.icon().to_string(), f.name.clone(), f.size_str(), f.is_dir))
                        .collect();
                        
                    ScrollArea::vertical()
                        .max_height(ui.available_height() - 60.0)
                        .show(ui, |ui| {
                            for (i, icon, name, size, is_dir) in &remote_files {
                                let is_selected = self.selected_remote.contains(i);
                                let response = ui.selectable_label(
                                    is_selected,
                                    format!("{} {} {}", icon, name, size)
                                );
                                
                                if response.clicked() {
                                    toggle_remote = Some(*i);
                                }
                                
                                if response.double_clicked() && *is_dir {
                                    navigate_remote_to = Some(name.clone());
                                }
                            }
                        });
                    
                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("⬇ Download").on_hover_text("Download selected to local").clicked() {
                            // Download logic
                        }
                        if ui.button("🗑 Delete").on_hover_text("Delete selected").clicked() {
                            // Delete logic
                        }
                        if ui.button("📁 New Folder").clicked() {
                            // Create folder
                        }
                    });
                }
            });
        });

        // Apply deferred actions
        if let Some(path) = local_path_changed {
            self.local_path = path;
            self.refresh_local();
        }
        if let Some(path) = remote_path_changed {
            self.remote_path = path;
        }
        if let Some(i) = toggle_local {
            if self.selected_local.contains(&i) {
                self.selected_local.retain(|&x| x != i);
            } else {
                self.selected_local.push(i);
            }
        }
        if let Some(i) = toggle_remote {
            if self.selected_remote.contains(&i) {
                self.selected_remote.retain(|&x| x != i);
            } else {
                self.selected_remote.push(i);
            }
        }
        if let Some(dir) = navigate_local_to {
            self.navigate_local(&dir);
        }
        if let Some(dir) = navigate_remote_to {
            self.navigate_remote(&dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_entry() {
        let file = SftpFileEntry::new_file("test.txt", 1500);
        assert!(!file.is_dir);
        assert_eq!(file.size_str(), "1.5 KB");
        
        let dir = SftpFileEntry::new_dir("folder");
        assert!(dir.is_dir);
    }
}

