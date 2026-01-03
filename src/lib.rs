//! 9Paste - Clipboard Transformer
//!
//! A Rust utility that automatically cleans, formats, and transforms clipboard text.
//! Create reusable "recipes" to standardize pasting with Ctrl+V.

pub mod clipboard;
pub mod config;
pub mod recipe;
pub mod transformers;
pub mod tray;
pub mod dashboard;
pub mod hotkeys;
pub mod ipc;
pub mod quick_menu;

pub use clipboard::ClipboardManager;
pub use config::Config;
pub use recipe::{Recipe, RecipeManager};
