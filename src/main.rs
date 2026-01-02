//! 9Paste - Clipboard Transformer
//!
//! A Rust utility that automatically cleans, formats, and transforms clipboard text.
//! Create reusable "recipes" to standardize pasting.

use std::sync::{Arc, Mutex};
use std::process::Command;
use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

use ninepaste::{
    ClipboardManager,
    Config,
    RecipeManager,
    Recipe,
    dashboard::Dashboard,
    tray::TrayManager,
    hotkeys::{HotkeyManager, HotkeyAction},
    clipboard::ClipboardEvent,
    ipc::{IpcServer, IpcCommand},
};

#[derive(Parser)]
#[command(name = "9paste")]
#[command(author = "9Paste Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Clipboard transformer that automatically cleans, formats, and transforms clipboard text")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Open the dashboard GUI
    Dashboard,
    
    /// Start the background service (clipboard monitoring)
    Start,
    
    /// Apply a recipe to current clipboard content
    Apply {
        /// Recipe name or ID
        recipe: String,
    },
    
    /// List all recipes
    List,
    
    /// Show clipboard content
    Show,
    
    /// Run a quick transformation on clipboard
    Transform {
        /// Transformation to apply (e.g., "lowercase", "trim", "remove-duplicates")
        transformation: String,
    },
    
    /// Toggle transformation on/off
    Toggle,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Dashboard) => run_dashboard()?,
        Some(Commands::Start) => run_background_service().await?,
        Some(Commands::Apply { recipe }) => apply_recipe(&recipe)?,
        Some(Commands::List) => list_recipes()?,
        Some(Commands::Show) => show_clipboard()?,
        Some(Commands::Transform { transformation }) => quick_transform(&transformation)?,
        Some(Commands::Toggle) => toggle_transformation()?,
        None => {
            // Default: run dashboard
            run_dashboard()?;
        }
    }
    
    Ok(())
}

/// Run the dashboard GUI
fn run_dashboard() -> Result<()> {
    info!("Starting 9Paste dashboard...");
    
    let recipe_manager = Arc::new(Mutex::new(RecipeManager::new()?));
    let config = Arc::new(Mutex::new(Config::load()?));
    
    Dashboard::run(recipe_manager, config)
        .map_err(|e| anyhow::anyhow!("Dashboard error: {}", e))?;
    
    Ok(())
}

/// Spawn dashboard as a separate process (used when running in background mode)
fn spawn_dashboard() {
    // Get the current executable path
    match std::env::current_exe() {
        Ok(exe_path) => {
            match Command::new(&exe_path)
                .arg("dashboard")
                .spawn()
            {
                Ok(_child) => {
                    info!("Dashboard spawned successfully");
                }
                Err(e) => {
                    error!("Failed to spawn dashboard: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to get executable path: {}", e);
        }
    }
}

/// Run the background clipboard monitoring service
async fn run_background_service() -> Result<()> {
    info!("Starting 9Paste background service...");
    
    let config = Config::load()?;
    let recipe_manager = Arc::new(Mutex::new(RecipeManager::new()?));
    
    // Get the active recipe
    let active_recipe: Arc<Mutex<Option<Recipe>>> = {
        let rm = recipe_manager.lock().unwrap();
        Arc::new(Mutex::new(rm.get_active_recipe().cloned()))
    };
    
    // Start clipboard monitoring
    let mut clipboard_manager = ClipboardManager::new();
    let mut clipboard_rx = clipboard_manager.start_monitoring(Some(Arc::clone(&active_recipe)));
    
    // Set up hotkeys if configured
    let mut hotkey_manager = HotkeyManager::new().ok();
    let mut hotkey_rx = None;
    
    if let Some(ref mut hm) = hotkey_manager {
        if let Some(ref hotkey) = config.toggle_hotkey {
            if hm.register(hotkey, HotkeyAction::ToggleTransformation).is_ok() {
                info!("Registered toggle hotkey: {}", hotkey);
            }
        }
        if let Some(ref hotkey) = config.quick_menu_hotkey {
            if hm.register(hotkey, HotkeyAction::OpenQuickMenu).is_ok() {
                info!("Registered quick menu hotkey: {}", hotkey);
            }
        }
        if let Some(ref hotkey) = config.dashboard_hotkey {
            if hm.register(hotkey, HotkeyAction::OpenDashboard).is_ok() {
                info!("Registered dashboard hotkey: {}", hotkey);
            }
        }
        hotkey_rx = Some(hm.start());
    }
    
    // Start system tray
    let mut tray_manager = TrayManager::new();
    let mut tray_result = tray_manager.start();
    
    // Start IPC server for dashboard communication
    let ipc_server = IpcServer::new();
    let mut ipc_rx = ipc_server.start();
    let active_recipe_for_ipc = Arc::clone(&active_recipe);
    
    println!("9Paste is running in the background.");
    println!("Press Ctrl+C to stop.");
    
    if let Some(ref recipe) = *active_recipe.lock().unwrap() {
        println!("Active recipe: {}", recipe.name);
    } else {
        println!("No active recipe. Set one in the dashboard.");
    }
    
    // Debounce for hotkeys to prevent double-firing
    let mut last_hotkey_time = std::time::Instant::now() - std::time::Duration::from_secs(1);
    const HOTKEY_DEBOUNCE_MS: u128 = 300;
    
    // Main event loop
    loop {
        tokio::select! {
            // Handle clipboard events
            Some(event) = clipboard_rx.recv() => {
                match event {
                    ClipboardEvent::Changed(text) => {
                        info!("Clipboard changed: {} chars", text.len());
                    }
                    ClipboardEvent::Transformed { original, result } => {
                        info!("Transformed: {} -> {} chars", original.len(), result.len());
                        if config.show_notifications {
                            println!("✨ Clipboard transformed!");
                        }
                    }
                    ClipboardEvent::Error(err) => {
                        error!("Clipboard error: {}", err);
                    }
                }
            }
            
            // Handle IPC commands from dashboard
            Some(cmd) = async {
                if let Some(ref mut rx) = ipc_rx {
                    rx.recv().await
                } else {
                    std::future::pending().await
                }
            } => {
                match cmd {
                    IpcCommand::ReloadRecipe => {
                        // Reload active recipe from disk
                        if let Ok(rm) = RecipeManager::new() {
                            let new_active = rm.get_active_recipe().cloned();
                            let mut current = active_recipe_for_ipc.lock().unwrap();
                            
                            if let Some(ref recipe) = new_active {
                                println!("📝 Active recipe: {}", recipe.name);
                            } else {
                                println!("📝 Recipe deactivated");
                            }
                            *current = new_active;
                        }
                    }
                    IpcCommand::Ping => {
                        // Just a ping, nothing to do
                    }
                }
            }
            
            // Handle hotkey events
            Some(action) = async {
                if let Some(ref mut rx) = hotkey_rx {
                    rx.recv().await
                } else {
                    std::future::pending().await
                }
            } => {
                // Debounce: ignore if triggered too quickly
                let now = std::time::Instant::now();
                if now.duration_since(last_hotkey_time).as_millis() < HOTKEY_DEBOUNCE_MS {
                    continue;
                }
                last_hotkey_time = now;
                
                match action {
                    HotkeyAction::ToggleTransformation => {
                        let enabled = clipboard_manager.is_transform_enabled();
                        clipboard_manager.set_transform_enabled(!enabled);
                        println!("Transformation: {}", if !enabled { "enabled" } else { "disabled" });
                    }
                    HotkeyAction::OpenQuickMenu => {
                        println!("Quick menu not yet implemented");
                    }
                    HotkeyAction::OpenDashboard => {
                        spawn_dashboard();
                    }
                }
            }
            
            // Handle tray events
            Some(cmd) = async {
                if let Ok(ref mut rx) = tray_result {
                    rx.recv().await
                } else {
                    std::future::pending().await
                }
            } => {
                use ninepaste::tray::TrayCommand;
                match cmd {
                    TrayCommand::Quit => {
                        info!("Quit requested from tray");
                        break;
                    }
                    TrayCommand::ToggleTransformation => {
                        let enabled = clipboard_manager.is_transform_enabled();
                        clipboard_manager.set_transform_enabled(!enabled);
                    }
                    TrayCommand::OpenDashboard => {
                        spawn_dashboard();
                    }
                    _ => {}
                }
            }
            
            // Handle Ctrl+C
            _ = tokio::signal::ctrl_c() => {
                info!("Shutting down...");
                break;
            }
        }
    }
    
    // Graceful shutdown - stop components in order
    println!("Shutting down...");
    
    // Stop clipboard first to prevent any clipboard operations blocking us
    clipboard_manager.stop_monitoring();
    
    if let Some(ref hm) = hotkey_manager {
        hm.stop();
    }
    
    ipc_server.stop();
    tray_manager.stop();
    
    println!("9Paste stopped.");
    
    std::process::exit(0);
}

/// Apply a recipe to clipboard content
fn apply_recipe(recipe_name: &str) -> Result<()> {
    let recipe_manager = RecipeManager::new()?;
    
    // Find recipe by name or ID
    let recipe = recipe_manager.recipes.iter()
        .find(|r| r.name.eq_ignore_ascii_case(recipe_name) || r.id.to_string() == recipe_name)
        .context(format!("Recipe not found: {}", recipe_name))?;
    
    let original = ClipboardManager::get_text()?;
    let transformed = recipe.apply(&original);
    ClipboardManager::set_text(&transformed)?;
    
    println!("✨ Applied recipe: {}", recipe.name);
    println!("   {} chars → {} chars", original.len(), transformed.len());
    
    Ok(())
}

/// List all recipes
fn list_recipes() -> Result<()> {
    let recipe_manager = RecipeManager::new()?;
    
    println!("📋 Recipes:\n");
    
    for recipe in &recipe_manager.recipes {
        let icon = recipe.icon.as_deref().unwrap_or("📋");
        let active = if recipe.is_active { " [ACTIVE]" } else { "" };
        println!("  {} {}{}", icon, recipe.name, active);
        
        if let Some(ref desc) = recipe.description {
            println!("    {}", desc);
        }
        
        println!("    Transformations: {}", recipe.transformations.len());
        println!();
    }
    
    Ok(())
}

/// Show current clipboard content
fn show_clipboard() -> Result<()> {
    let text = ClipboardManager::get_text()?;
    
    if text.is_empty() {
        println!("(clipboard is empty)");
    } else {
        println!("📋 Clipboard ({} chars):\n", text.len());
        println!("{}", text);
    }
    
    Ok(())
}

/// Quick transformation
fn quick_transform(transformation: &str) -> Result<()> {
    use ninepaste::recipe::Transformation;
    
    let transform = match transformation.to_lowercase().as_str() {
        "lowercase" | "lower" => Transformation::ToLowercase,
        "uppercase" | "upper" => Transformation::ToUppercase,
        "titlecase" | "title" => Transformation::ToTitleCase,
        "sentencecase" | "sentence" => Transformation::ToSentenceCase,
        "camelcase" | "camel" => Transformation::ToCamelCase,
        "pascalcase" | "pascal" => Transformation::ToPascalCase,
        "snakecase" | "snake" => Transformation::ToSnakeCase,
        "kebabcase" | "kebab" => Transformation::ToKebabCase,
        "trim" => Transformation::TrimLines,
        "normalize" | "whitespace" => Transformation::NormalizeWhitespace,
        "remove-empty" | "no-empty" => Transformation::RemoveEmptyLines,
        "remove-duplicates" | "unique" | "dedup" => Transformation::RemoveDuplicateLines,
        "sort" => Transformation::SortLines,
        "reverse" => Transformation::ReverseLines,
        "smartquotes" | "fix-quotes" | "quotes" => Transformation::FixSmartQuotes,
        "remove-emojis" | "no-emoji" => Transformation::RemoveEmojis,
        "strip" | "plain" => Transformation::StripFormatting,
        "slugify" | "slug" => Transformation::Slugify,
        "html-encode" => Transformation::EncodeHtmlEntities,
        "html-decode" => Transformation::DecodeHtmlEntities,
        "unix" | "lf" => Transformation::ToUnixLineEndings,
        "windows" | "crlf" => Transformation::ToWindowsLineEndings,
        _ => {
            println!("Unknown transformation: {}", transformation);
            println!("\nAvailable transformations:");
            println!("  lowercase, uppercase, titlecase, sentencecase");
            println!("  camelcase, pascalcase, snakecase, kebabcase");
            println!("  trim, normalize, remove-empty, remove-duplicates");
            println!("  sort, reverse, smartquotes, remove-emojis");
            println!("  strip, slugify, html-encode, html-decode");
            println!("  unix, windows");
            return Ok(());
        }
    };
    
    let original = ClipboardManager::get_text()?;
    let result = transform.apply(&original);
    ClipboardManager::set_text(&result)?;
    
    println!("✨ Applied: {}", transform.display_name());
    println!("   {} chars → {} chars", original.len(), result.len());
    
    Ok(())
}

/// Toggle transformation on/off
fn toggle_transformation() -> Result<()> {
    let mut config = Config::load()?;
    config.auto_transform = !config.auto_transform;
    config.save()?;
    
    println!("Transformation: {}", if config.auto_transform { "enabled" } else { "disabled" });
    
    Ok(())
}
