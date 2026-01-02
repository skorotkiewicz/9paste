//! IPC (Inter-Process Communication) module
//!
//! Provides communication between the dashboard and background service
//! using a simple TCP socket on localhost.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info};

/// Default port for IPC communication
pub const IPC_PORT: u16 = 9549;

/// Commands that can be sent via IPC
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcCommand {
    /// Reload the active recipe from disk
    ReloadRecipe,
    /// Toggle transformation
    ToggleTransformation,
    /// Ping to check if service is running
    Ping,
}

/// IPC Server - runs in the background service
pub struct IpcServer {
    running: Arc<AtomicBool>,
}

impl IpcServer {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Start the IPC server, returns a receiver for commands
    pub fn start(&self) -> Option<mpsc::Receiver<IpcCommand>> {
        let (tx, rx) = mpsc::channel(32);
        let running = Arc::clone(&self.running);
        running.store(true, Ordering::SeqCst);
        
        // Try to bind to the port
        let listener = match TcpListener::bind(format!("127.0.0.1:{}", IPC_PORT)) {
            Ok(l) => {
                // Set non-blocking so we can check the running flag
                l.set_nonblocking(true).ok();
                l
            }
            Err(e) => {
                debug!("Could not start IPC server on port {}: {}", IPC_PORT, e);
                return None;
            }
        };
        
        info!("IPC server listening on port {}", IPC_PORT);
        
        std::thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _addr)) => {
                        // Read command
                        let mut buf = [0u8; 32];
                        if let Ok(n) = stream.read(&mut buf) {
                            if n > 0 {
                                let cmd_str = String::from_utf8_lossy(&buf[..n]);
                                let command = match cmd_str.trim() {
                                    "RELOAD" => Some(IpcCommand::ReloadRecipe),
                                    "TRANSFORM" => Some(IpcCommand::ToggleTransformation),
                                    "PING" => {
                                        // Respond to ping
                                        let _ = stream.write_all(b"PONG");
                                        Some(IpcCommand::Ping)
                                    }
                                    _ => None,
                                };
                                
                                if let Some(cmd) = command {
                                    let _ = tx.blocking_send(cmd);
                                }
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No connection waiting, sleep a bit
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    Err(e) => {
                        debug!("IPC accept error: {}", e);
                    }
                }
            }
            info!("IPC server stopped");
        });
        
        Some(rx)
    }
    
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Default for IpcServer {
    fn default() -> Self {
        Self::new()
    }
}

/// IPC Client - used by the dashboard to send commands
pub struct IpcClient;

impl IpcClient {
    /// Send a command to the background service
    /// Returns true if successful, false if service not running
    pub fn send(command: IpcCommand) -> bool {
        let cmd_str = match command {
            IpcCommand::ReloadRecipe => "RELOAD",
            IpcCommand::ToggleTransformation => "TRANSFORM",
            IpcCommand::Ping => "PING",
        };
        
        match TcpStream::connect(format!("127.0.0.1:{}", IPC_PORT)) {
            Ok(mut stream) => {
                stream.set_write_timeout(Some(std::time::Duration::from_millis(100))).ok();
                if stream.write_all(cmd_str.as_bytes()).is_ok() {
                    debug!("Sent IPC command: {:?}", command);
                    true
                } else {
                    false
                }
            }
            Err(_) => {
                // Service not running, that's OK
                false
            }
        }
    }
    
    /// Check if the background service is running
    pub fn is_service_running() -> bool {
        if let Ok(mut stream) = TcpStream::connect(format!("127.0.0.1:{}", IPC_PORT)) {
            stream.set_read_timeout(Some(std::time::Duration::from_millis(100))).ok();
            stream.set_write_timeout(Some(std::time::Duration::from_millis(100))).ok();
            
            if stream.write_all(b"PING").is_ok() {
                let mut buf = [0u8; 4];
                if stream.read(&mut buf).is_ok() {
                    return &buf == b"PONG";
                }
            }
        }
        false
    }
}
