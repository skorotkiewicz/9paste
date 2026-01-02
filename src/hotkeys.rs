//! Global hotkey support
//!
//! Provides cross-platform global hotkey registration and handling.

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use anyhow::{Result, Context};
use global_hotkey::{
    GlobalHotKeyManager, GlobalHotKeyEvent,
    hotkey::{HotKey, Modifiers, Code},
};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Hotkey actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    /// Toggle transformation on/off
    ToggleTransformation,
    /// Open quick recipe menu
    OpenQuickMenu,
    /// Open dashboard
    OpenDashboard,
}

/// Hotkey manager for registering and handling global hotkeys
pub struct HotkeyManager {
    manager: Option<GlobalHotKeyManager>,
    running: Arc<AtomicBool>,
    registered_hotkeys: Vec<(HotKey, HotkeyAction)>,
}

impl HotkeyManager {
    /// Create a new HotkeyManager
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new()
            .context("Failed to create global hotkey manager")?;
        
        Ok(Self {
            manager: Some(manager),
            running: Arc::new(AtomicBool::new(false)),
            registered_hotkeys: Vec::new(),
        })
    }
    
    /// Parse a hotkey string like "Ctrl+Shift+T" into a HotKey
    pub fn parse_hotkey(hotkey_str: &str) -> Result<HotKey> {
        let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();
        
        let mut modifiers = Modifiers::empty();
        let mut key_code = None;
        
        for part in parts {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
                "shift" => modifiers |= Modifiers::SHIFT,
                "alt" => modifiers |= Modifiers::ALT,
                "super" | "win" | "cmd" | "meta" => modifiers |= Modifiers::META,
                
                // Letters
                "a" => key_code = Some(Code::KeyA),
                "b" => key_code = Some(Code::KeyB),
                "c" => key_code = Some(Code::KeyC),
                "d" => key_code = Some(Code::KeyD),
                "e" => key_code = Some(Code::KeyE),
                "f" => key_code = Some(Code::KeyF),
                "g" => key_code = Some(Code::KeyG),
                "h" => key_code = Some(Code::KeyH),
                "i" => key_code = Some(Code::KeyI),
                "j" => key_code = Some(Code::KeyJ),
                "k" => key_code = Some(Code::KeyK),
                "l" => key_code = Some(Code::KeyL),
                "m" => key_code = Some(Code::KeyM),
                "n" => key_code = Some(Code::KeyN),
                "o" => key_code = Some(Code::KeyO),
                "p" => key_code = Some(Code::KeyP),
                "q" => key_code = Some(Code::KeyQ),
                "r" => key_code = Some(Code::KeyR),
                "s" => key_code = Some(Code::KeyS),
                "t" => key_code = Some(Code::KeyT),
                "u" => key_code = Some(Code::KeyU),
                "v" => key_code = Some(Code::KeyV),
                "w" => key_code = Some(Code::KeyW),
                "x" => key_code = Some(Code::KeyX),
                "y" => key_code = Some(Code::KeyY),
                "z" => key_code = Some(Code::KeyZ),
                
                // Numbers
                "0" => key_code = Some(Code::Digit0),
                "1" => key_code = Some(Code::Digit1),
                "2" => key_code = Some(Code::Digit2),
                "3" => key_code = Some(Code::Digit3),
                "4" => key_code = Some(Code::Digit4),
                "5" => key_code = Some(Code::Digit5),
                "6" => key_code = Some(Code::Digit6),
                "7" => key_code = Some(Code::Digit7),
                "8" => key_code = Some(Code::Digit8),
                "9" => key_code = Some(Code::Digit9),
                
                // Function keys
                "f1" => key_code = Some(Code::F1),
                "f2" => key_code = Some(Code::F2),
                "f3" => key_code = Some(Code::F3),
                "f4" => key_code = Some(Code::F4),
                "f5" => key_code = Some(Code::F5),
                "f6" => key_code = Some(Code::F6),
                "f7" => key_code = Some(Code::F7),
                "f8" => key_code = Some(Code::F8),
                "f9" => key_code = Some(Code::F9),
                "f10" => key_code = Some(Code::F10),
                "f11" => key_code = Some(Code::F11),
                "f12" => key_code = Some(Code::F12),
                
                // Special keys
                "space" => key_code = Some(Code::Space),
                "enter" | "return" => key_code = Some(Code::Enter),
                "tab" => key_code = Some(Code::Tab),
                "escape" | "esc" => key_code = Some(Code::Escape),
                "backspace" => key_code = Some(Code::Backspace),
                "delete" => key_code = Some(Code::Delete),
                "home" => key_code = Some(Code::Home),
                "end" => key_code = Some(Code::End),
                "pageup" => key_code = Some(Code::PageUp),
                "pagedown" => key_code = Some(Code::PageDown),
                "up" => key_code = Some(Code::ArrowUp),
                "down" => key_code = Some(Code::ArrowDown),
                "left" => key_code = Some(Code::ArrowLeft),
                "right" => key_code = Some(Code::ArrowRight),
                
                _ => warn!("Unknown key: {}", part),
            }
        }
        
        let code = key_code.context("No valid key found in hotkey string")?;
        Ok(HotKey::new(Some(modifiers), code))
    }
    
    /// Register a hotkey
    pub fn register(&mut self, hotkey_str: &str, action: HotkeyAction) -> Result<()> {
        let hotkey = Self::parse_hotkey(hotkey_str)?;
        
        if let Some(ref manager) = self.manager {
            manager.register(hotkey)
                .context("Failed to register hotkey")?;
            
            self.registered_hotkeys.push((hotkey, action));
            info!("Registered hotkey: {} -> {:?}", hotkey_str, action);
        }
        
        Ok(())
    }
    
    /// Unregister all hotkeys
    pub fn unregister_all(&mut self) -> Result<()> {
        if let Some(ref manager) = self.manager {
            for (hotkey, _) in &self.registered_hotkeys {
                manager.unregister(*hotkey).ok();
            }
        }
        self.registered_hotkeys.clear();
        Ok(())
    }
    
    /// Start listening for hotkey events
    pub fn start(&mut self) -> mpsc::Receiver<HotkeyAction> {
        let (tx, rx) = mpsc::channel(100);
        self.running.store(true, Ordering::SeqCst);
        
        let running = Arc::clone(&self.running);
        let hotkeys = self.registered_hotkeys.clone();
        
        std::thread::spawn(move || {
            let receiver = GlobalHotKeyEvent::receiver();
            
            while running.load(Ordering::SeqCst) {
                if let Ok(event) = receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                    // Find the action for this hotkey
                    for (hotkey, action) in &hotkeys {
                        if hotkey.id() == event.id {
                            info!("Hotkey triggered: {:?}", action);
                            if tx.blocking_send(*action).is_err() {
                                error!("Failed to send hotkey action");
                                return;
                            }
                            break;
                        }
                    }
                }
            }
        });
        
        rx
    }
    
    /// Stop listening for hotkeys
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new().expect("Failed to create HotkeyManager")
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        self.unregister_all().ok();
    }
}
