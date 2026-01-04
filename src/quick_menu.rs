//! Quick Menu GUI
//!
//! A lightweight popup menu for quickly selecting and applying recipes.

use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::recipe::RecipeManager;
use crate::clipboard::ClipboardManager;
use crate::config::{Config, HistoryManager, HistoryEntry};
use egui_phosphor::regular::*;

/// Quick Menu application state
pub struct QuickMenu {
    /// Recipe manager
    recipe_manager: Arc<Mutex<RecipeManager>>,
    /// Search query
    search_query: String,
    /// Whether to close the application
    should_close: bool,
    /// First frame flag for focus
    first_frame: bool,
}

impl QuickMenu {
    /// Create a new Quick Menu
    pub fn new(recipe_manager: Arc<Mutex<RecipeManager>>) -> Self {
        Self {
            recipe_manager,
            search_query: String::new(),
            should_close: false,
            first_frame: true,
        }
    }
    
    /// Run the Quick Menu
    pub fn run(recipe_manager: Arc<Mutex<RecipeManager>>) -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([300.0, 400.0])
                .with_always_on_top()
                .with_decorations(false) // Frameless
                .with_resizable(false)
                .with_title("9Paste Quick Menu"),
            ..Default::default()
        };
        
        eframe::run_native(
            "9Paste Quick Menu",
            options,
            Box::new(move |cc| {
                // Initialize phosphor fonts
                let mut fonts = egui::FontDefinitions::default();
                egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
                cc.egui_ctx.set_fonts(fonts);
                
                Ok(Box::new(QuickMenu::new(recipe_manager)))
            }),
        )
    }
}

impl eframe::App for QuickMenu {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme from config
        if let Ok(config) = Config::load() {
            match config.theme.as_str() {
                "light" => ctx.set_visuals(egui::Visuals::light()),
                "dark" => ctx.set_visuals(egui::Visuals::dark()),
                _ => {} // "system" - use default
            }
        }
        
        if self.should_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Close on Escape
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.should_close = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(format!("{} Quick Menu", ROCKET_LAUNCH));
            });
            
            ui.add_space(5.0);
            
            // Search box
            let search_edit = egui::TextEdit::singleline(&mut self.search_query)
                .hint_text(format!("{} Search recipes...", MAGNIFYING_GLASS))
                .desired_width(f32::INFINITY);
            
            let response = ui.add(search_edit);
            
            // Focus on first frame
            if self.first_frame {
                response.request_focus();
                self.first_frame = false;
            }
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);
            
            // Recipe list
            egui::ScrollArea::vertical().show(ui, |ui| {
                let recipe_manager = self.recipe_manager.lock().unwrap();
                let search_lower = self.search_query.to_lowercase();
                
                let filtered_recipes: Vec<_> = recipe_manager.recipes.iter()
                    .filter(|r| r.name.to_lowercase().contains(&search_lower))
                    .cloned()
                    .collect();
                
                if filtered_recipes.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("No recipes found");
                    });
                } else {
                    for recipe in filtered_recipes {
                        let icon = recipe.icon.as_deref().unwrap_or(CLIPBOARD);
                        let label = format!("{} {}", icon, recipe.name);
                        
                        if ui.button(label).clicked() {
                            if let Ok(text) = ClipboardManager::get_text() {
                                let transformed = recipe.apply(&text);
                                
                                // Save to history
                                if let Ok(config) = Config::load() {
                                    if config.keep_history {
                                        if let Ok(mut hm) = HistoryManager::new(config.max_history_size) {
                                            let entry = HistoryEntry {
                                                original: text.clone(),
                                                transformed: Some(transformed.clone()),
                                                recipe_id: Some(recipe.id.to_string()),
                                                recipe_name: Some(recipe.name.clone()),
                                                timestamp: chrono::Utc::now(),
                                            };
                                            let _ = hm.add(entry);
                                        }
                                    }
                                }
                                
                                // Use non-blocking clipboard set so we can close immediately
                                let _ = ClipboardManager::set_text_background(&transformed);
                            }
                            self.should_close = true;
                        }
                    }
                }
            });
            
            ui.add_space(5.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        self.should_close = true;
                    }
                });
            });
        });

        // Repaint to keep focus/input snappy if needed
        ctx.request_repaint();
    }
}
