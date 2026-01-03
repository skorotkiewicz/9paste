//! Quick Menu GUI
//!
//! A lightweight popup menu for quickly selecting and applying recipes.

use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::recipe::RecipeManager;
use crate::clipboard::ClipboardManager;

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
            Box::new(move |_cc| {
                Ok(Box::new(QuickMenu::new(recipe_manager)))
            }),
        )
    }
}

impl eframe::App for QuickMenu {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                ui.heading("🚀 Quick Menu");
            });
            
            ui.add_space(5.0);
            
            // Search box
            let search_edit = egui::TextEdit::singleline(&mut self.search_query)
                .hint_text("🔍 Search recipes...")
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
                        let icon = recipe.icon.as_deref().unwrap_or("📋");
                        let label = format!("{} {}", icon, recipe.name);
                        
                        if ui.button(label).clicked() {
                            if let Ok(text) = ClipboardManager::get_text() {
                                let transformed = recipe.apply(&text);
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
