//! Termicon - Professional Terminal Application
//!
//! A modern, cross-platform terminal application supporting:
//! - Serial port communication
//! - TCP connections
//! - Telnet protocol
//! - SSH-2 protocol

// Initialize i18n for the binary - translations are loaded from i18n folder (TOML)
rust_i18n::i18n!("i18n", fallback = "en");

use eframe::egui;
use image::GenericImageView;

mod gui;

/// Load application icon from embedded PNG
fn load_icon() -> Option<egui::IconData> {
    // Embed the icon at compile time
    let icon_bytes = include_bytes!("../termicon.png");
    
    match image::load_from_memory(icon_bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            Some(egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            })
        }
        Err(e) => {
            tracing::warn!("Failed to load icon: {}", e);
            None
        }
    }
}

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Set default locale to English
    rust_i18n::set_locale("en");

    tracing::info!("Starting Termicon v{}", env!("CARGO_PKG_VERSION"));

    // Load application icon
    let icon = load_icon();

    // Native options
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1024.0, 768.0])
        .with_min_inner_size([800.0, 600.0])
        .with_title("Termicon");
    
    // Set icon if loaded successfully
    if let Some(icon_data) = icon {
        viewport = viewport.with_icon(std::sync::Arc::new(icon_data));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "Termicon",
        native_options,
        Box::new(|cc| Ok(Box::new(gui::TermiconApp::new(cc)))),
    )
}
