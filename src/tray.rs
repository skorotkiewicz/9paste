//! System tray integration
//!
//! Provides system tray icon and menu for quick access to 9Paste features.

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use anyhow::{Result, Context};
use tokio::sync::mpsc;
use tracing::{error, info};

/// Commands that can be triggered from the tray menu
#[derive(Debug, Clone)]
pub enum TrayCommand {
    /// Open the dashboard
    OpenDashboard,
    /// Toggle transformation on/off
    ToggleTransformation,
    /// Apply a specific recipe
    ApplyRecipe(uuid::Uuid),
    /// Show quick menu
    ShowQuickMenu,
    /// Quit the application
    Quit,
}

/// System tray manager
pub struct TrayManager {
    /// Whether the tray is running
    running: Arc<AtomicBool>,
}

impl TrayManager {
    /// Create a new TrayManager
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Start the system tray
    /// Returns a receiver for tray commands
    pub fn start(&mut self) -> Result<mpsc::Receiver<TrayCommand>> {
        let (tx, rx) = mpsc::channel(100);
        self.running.store(true, Ordering::SeqCst);
        
        let running = Arc::clone(&self.running);
        
        std::thread::spawn(move || {
            // Note: tray-item requires being run on the main thread on some platforms
            // We'll use a message-passing approach to handle this
            
            match Self::run_tray_loop(tx, running) {
                Ok(()) => info!("Tray loop exited normally"),
                Err(e) => error!("Tray loop error: {}", e),
            }
        });
        
        Ok(rx)
    }
    
    /// Run the tray event loop (blocking)
    fn run_tray_loop(
        tx: mpsc::Sender<TrayCommand>,
        running: Arc<AtomicBool>,
    ) -> Result<()> {
        use tray_item::TrayItem;
        
        let mut tray = TrayItem::new("9Paste", tray_item::IconSource::Resource(""))
            .context("Failed to create tray item")?;
        
        // Dashboard
        let tx_clone = tx.clone();
        tray.add_menu_item("Dashboard", move || {
            let _ = tx_clone.blocking_send(TrayCommand::OpenDashboard);
        }).context("Failed to add Dashboard menu item")?;
        
        // Toggle
        let tx_clone = tx.clone();
        tray.add_menu_item("Toggle Transformation", move || {
            let _ = tx_clone.blocking_send(TrayCommand::ToggleTransformation);
        }).context("Failed to add Toggle menu item")?;
        
        // Quick Menu
        let tx_clone = tx.clone();
        tray.add_menu_item("Quick Menu", move || {
            let _ = tx_clone.blocking_send(TrayCommand::ShowQuickMenu);
        }).context("Failed to add Quick Menu item")?;
        
        // Add a visual separator using a label
        tray.add_menu_item("────────", || {}).ok();
        
        // Quit
        let tx_clone = tx.clone();
        tray.add_menu_item("Quit", move || {
            let _ = tx_clone.blocking_send(TrayCommand::Quit);
        }).context("Failed to add Quit menu item")?;
        
        // Keep the tray alive
        while running.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        Ok(())
    }
    
    /// Stop the tray
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}
