//! Glyphgen - High-Performance Terminal Art Rendering Studio
//!
//! A TUI application for converting images to ASCII/Unicode art and stylizing text.

pub mod color_space;
pub mod config;
pub mod image_loader;
pub mod input;
pub mod perf_monitor;
pub mod render_engines;
pub mod state;
pub mod terminal_capabilities;
pub mod ui;
pub mod unicode_handler;
pub mod worker;

// Re-export commonly used types
pub use config::Config;
pub use state::{AppState, RenderMode};
