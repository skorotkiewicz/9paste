//! Clipboard management module
//!
//! Provides cross-platform clipboard access and monitoring.

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use anyhow::{Result, Context};
use arboard::Clipboard;
#[cfg(target_os = "linux")]
use arboard::SetExtLinux;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::recipe::Recipe;

/// Events emitted by the clipboard manager
#[derive(Debug, Clone)]
pub enum ClipboardEvent {
    /// Clipboard content changed
    Changed(String),
    /// Clipboard was transformed
    Transformed { original: String, result: String },
    /// Error occurred
    Error(String),
}

/// Clipboard manager for monitoring and transforming clipboard content
pub struct ClipboardManager {
    /// Whether the monitor is running
    running: Arc<AtomicBool>,
    /// Channel for clipboard events
    event_sender: Option<mpsc::Sender<ClipboardEvent>>,
    /// Last known clipboard content (to detect changes)
    last_content: Arc<std::sync::Mutex<String>>,
    /// Whether transformation is enabled
    transform_enabled: Arc<AtomicBool>,
}

impl ClipboardManager {
    /// Create a new ClipboardManager
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            event_sender: None,
            last_content: Arc::new(std::sync::Mutex::new(String::new())),
            transform_enabled: Arc::new(AtomicBool::new(true)),
        }
    }
    
    /// Check if clipboard access is available
    pub fn is_available() -> bool {
        Clipboard::new().is_ok()
    }
    
    /// Get the current clipboard text
    pub fn get_text() -> Result<String> {
        let mut clipboard = Clipboard::new()
            .context("Failed to access clipboard")?;
        clipboard.get_text()
            .context("Failed to get clipboard text")
    }
    
    /// Set the clipboard text
    /// On Linux, this waits for the clipboard manager to take ownership
    #[cfg(target_os = "linux")]
    pub fn set_text(text: &str) -> Result<()> {
        let mut clipboard = Clipboard::new()
            .context("Failed to access clipboard")?;
        
        // Use the Set builder with wait() for proper handover to clipboard manager on Linux
        clipboard.set()
            .wait()
            .text(text.to_string())
            .context("Failed to set clipboard text")
    }
    
    /// Set clipboard text without blocking (spawns background process)
    /// Use this when the calling process needs to exit immediately (e.g., Quick Menu)
    #[cfg(target_os = "linux")]
    pub fn set_text_background(text: &str) -> Result<()> {
        use std::process::{Command, Stdio};
        use std::io::Write;
        
        // Use xclip if available (most reliable for persistence)
        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(text.as_bytes())?;
            }
            return Ok(());
        }
        
        // Fallback to xsel
        if let Ok(mut child) = Command::new("xsel")
            .args(["--clipboard", "--input"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(text.as_bytes())?;
            }
            return Ok(());
        }
        
        // Last resort: blocking set (will freeze until clipboard is read)
        Self::set_text(text)
    }
    
    /// Set clipboard text without blocking (non-Linux - just use normal set)
    #[cfg(not(target_os = "linux"))]
    pub fn set_text_background(text: &str) -> Result<()> {
        Self::set_text(text)
    }
    
    /// Set the clipboard text (non-Linux platforms)
    #[cfg(not(target_os = "linux"))]
    pub fn set_text(text: &str) -> Result<()> {
        let mut clipboard = Clipboard::new()
            .context("Failed to access clipboard")?;
        clipboard.set_text(text)
            .context("Failed to set clipboard text")
    }
    
    /// Apply a recipe to the current clipboard content
    pub fn apply_recipe(recipe: &Recipe) -> Result<String> {
        let original = Self::get_text()?;
        let transformed = recipe.apply(&original);
        Self::set_text(&transformed)?;
        Ok(transformed)
    }
    
    /// Enable or disable automatic transformation
    pub fn set_transform_enabled(&self, enabled: bool) {
        self.transform_enabled.store(enabled, Ordering::SeqCst);
    }
    
    /// Check if transformation is enabled
    pub fn is_transform_enabled(&self) -> bool {
        self.transform_enabled.load(Ordering::SeqCst)
    }
    
    /// Start monitoring the clipboard for changes
    pub fn start_monitoring(
        &mut self,
        active_recipe: Option<Arc<std::sync::Mutex<Option<Recipe>>>>,
    ) -> mpsc::Receiver<ClipboardEvent> {
        let (tx, rx) = mpsc::channel(100);
        self.event_sender = Some(tx.clone());
        self.running.store(true, Ordering::SeqCst);
        
        let running = Arc::clone(&self.running);
        let last_content = Arc::clone(&self.last_content);
        let transform_enabled = Arc::clone(&self.transform_enabled);
        
        // Initialize with current clipboard content
        if let Ok(content) = Self::get_text() {
            *last_content.lock().unwrap() = content;
        }
        
        tokio::spawn(async move {
            let poll_interval = Duration::from_millis(250);
            
            while running.load(Ordering::SeqCst) {
                tokio::time::sleep(poll_interval).await;
                
                // Get current clipboard content
                let current = match Self::get_text() {
                    Ok(text) => text,
                    Err(e) => {
                        debug!("Failed to get clipboard: {}", e);
                        continue;
                    }
                };
                
                // Check if content changed
                let last = {
                    let guard = last_content.lock().unwrap();
                    guard.clone()
                };
                
                if current != last {
                    info!("Clipboard changed: {} chars", current.len());
                    
                    // Send change event
                    if tx.send(ClipboardEvent::Changed(current.clone())).await.is_err() {
                        break;
                    }
                    
                    // Apply transformation if enabled and we have a recipe
                    // Clone the recipe to avoid holding the mutex guard across await
                    let maybe_recipe = if transform_enabled.load(Ordering::SeqCst) {
                        if let Some(ref recipe_mutex) = active_recipe {
                            recipe_mutex.lock().unwrap().clone()
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    
                    if let Some(recipe) = maybe_recipe {
                        let transformed = recipe.apply(&current);
                        
                        if transformed != current {
                            // Update clipboard with transformed text (non-blocking to avoid delays)
                            if let Err(e) = Self::set_text_background(&transformed) {
                                error!("Failed to set transformed clipboard: {}", e);
                                let _ = tx.send(ClipboardEvent::Error(e.to_string())).await;
                            } else {
                                info!("Transformed clipboard: {} -> {} chars", 
                                      current.len(), transformed.len());
                                
                                // Update last content to transformed version
                                *last_content.lock().unwrap() = transformed.clone();
                                
                                let _ = tx.send(ClipboardEvent::Transformed {
                                    original: current,
                                    result: transformed,
                                }).await;
                                
                                continue;
                            }
                        }
                    }
                    
                    // Update last content
                    *last_content.lock().unwrap() = current;
                }
            }
            
            info!("Clipboard monitor stopped");
        });
        
        rx
    }
    
    /// Stop monitoring the clipboard
    pub fn stop_monitoring(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
    
    /// Check if monitoring is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ClipboardManager {
    fn drop(&mut self) {
        self.stop_monitoring();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_available() {
        // This might fail in headless environments, so we just check it doesn't panic
        let _ = ClipboardManager::is_available();
    }
}
