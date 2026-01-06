//! GUI module for the graphical user interface

mod accessibility;
mod ansi_parser;
mod app;
mod ble_panel;
mod chart_panel;
mod command_palette;
mod font_config;
mod keyboard;
mod macros_panel;
mod profiles;
mod session_tab;
mod sftp_panel;
mod split_view;
mod themes;

pub use app::TermiconApp;
