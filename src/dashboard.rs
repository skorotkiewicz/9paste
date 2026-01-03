//! Dashboard GUI
//!
//! Provides a graphical interface for managing recipes and settings.

use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::recipe::{Recipe, RecipeManager, Transformation};
use crate::config::{Config, HistoryManager};
use crate::clipboard::ClipboardManager;
use crate::ipc::{IpcClient, IpcCommand};

/// Dashboard application state
pub struct Dashboard {
    /// Recipe manager
    recipe_manager: Arc<Mutex<RecipeManager>>,
    /// Configuration
    config: Arc<Mutex<Config>>,
    /// Currently selected recipe ID
    selected_recipe: Option<uuid::Uuid>,
    /// Current tab
    current_tab: DashboardTab,
    /// Test input for preview
    test_input: String,
    /// Test output (preview)
    test_output: String,
    /// Whether transformation is enabled
    transform_enabled: bool,
    /// Status message
    status_message: Option<(String, std::time::Instant)>,
    /// New recipe being created
    new_recipe_name: String,
    /// Current edit mode for recipe
    editing_recipe: bool,
    /// History manager
    history_manager: Option<HistoryManager>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardTab {
    Recipes,
    Settings,
    History,
    About,
}

impl Dashboard {
    /// Create a new dashboard
    pub fn new(
        recipe_manager: Arc<Mutex<RecipeManager>>,
        config: Arc<Mutex<Config>>,
    ) -> Self {
        let transform_enabled = config.lock().unwrap().auto_transform;
        let max_history_size = config.lock().unwrap().max_history_size;
        let history_manager = HistoryManager::new(max_history_size).ok();
        
        Self {
            recipe_manager,
            config,
            selected_recipe: None,
            current_tab: DashboardTab::Recipes,
            test_input: "Hello, World!\n\nThis is a   test with  extra   spaces.\n\n\"Smart quotes\" and 'apostrophes'.\n\nLine one\nLine one\nLine two".to_string(),
            test_output: String::new(),
            transform_enabled,
            status_message: None,
            new_recipe_name: String::new(),
            editing_recipe: false,
            history_manager,
        }
    }
    
    /// Run the dashboard
    pub fn run(
        recipe_manager: Arc<Mutex<RecipeManager>>,
        config: Arc<Mutex<Config>>,
    ) -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([900.0, 650.0])
                .with_min_inner_size([700.0, 500.0])
                .with_title("9Paste - Clipboard Transformer"),
            ..Default::default()
        };
        
        eframe::run_native(
            "9Paste",
            options,
            Box::new(move |_cc| {
                Ok(Box::new(Dashboard::new(recipe_manager, config)))
            }),
        )
    }
    
    fn show_status(&mut self, message: impl Into<String>) {
        self.status_message = Some((message.into(), std::time::Instant::now()));
    }
    
    fn update_preview(&mut self) {
        if let Some(recipe_id) = self.selected_recipe {
            let recipe_manager = self.recipe_manager.lock().unwrap();
            if let Some(recipe) = recipe_manager.get_recipe(recipe_id) {
                self.test_output = recipe.apply(&self.test_input);
            }
        }
    }
}

impl eframe::App for Dashboard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme from config
        {
            let config = self.config.lock().unwrap();
            match config.theme.as_str() {
                "light" => ctx.set_visuals(egui::Visuals::light()),
                "dark" => ctx.set_visuals(egui::Visuals::dark()),
                _ => {} // "system" - use default
            }
        }
        
        // Clear old status messages
        if let Some((_, time)) = &self.status_message {
            if time.elapsed() > std::time::Duration::from_secs(3) {
                self.status_message = None;
            }
        }
        
        // Top panel with tabs
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.heading("🔧 9Paste");
                ui.add_space(20.0);
                
                if ui.selectable_label(self.current_tab == DashboardTab::Recipes, "📝 Recipes").clicked() {
                    self.current_tab = DashboardTab::Recipes;
                }
                if ui.selectable_label(self.current_tab == DashboardTab::Settings, "⚙️ Settings").clicked() {
                    self.current_tab = DashboardTab::Settings;
                }
                if ui.selectable_label(self.current_tab == DashboardTab::History, "📋 History").clicked() {
                    self.current_tab = DashboardTab::History;
                }
                if ui.selectable_label(self.current_tab == DashboardTab::About, "ℹ️ About").clicked() {
                    self.current_tab = DashboardTab::About;
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Toggle transformation
                    let toggle_text = if self.transform_enabled { "🟢 Active" } else { "🔴 Inactive" };
                    if ui.toggle_value(&mut self.transform_enabled, toggle_text).changed() {
                        {
                            let mut config = self.config.lock().unwrap();
                            config.auto_transform = self.transform_enabled;
                            config.save().ok();
                        } // config guard dropped here

                        IpcClient::send(IpcCommand::ToggleTransformation);
                        self.show_status(if self.transform_enabled { 
                            "Transformation enabled" 
                        } else { 
                            "Transformation disabled" 
                        });
                    }
                });
            });
            ui.add_space(5.0);
        });
        
        // Status bar
        if let Some((message, _)) = &self.status_message {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("ℹ️");
                    ui.label(message);
                });
            });
        }
        
        // Main content
        match self.current_tab {
            DashboardTab::Recipes => self.show_recipes_tab(ctx),
            DashboardTab::Settings => self.show_settings_tab(ctx),
            DashboardTab::History => self.show_history_tab(ctx),
            DashboardTab::About => self.show_about_tab(ctx),
        }
    }
}

impl Dashboard {
    fn show_recipes_tab(&mut self, ctx: &egui::Context) {
        // Left panel - recipe list
        egui::SidePanel::left("recipe_list")
            .resizable(true)
            .default_width(250.0)
            .min_width(200.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.heading("📝 Recipes");
                ui.add_space(10.0);
                
                // New recipe button
                ui.label("Create New Recipe:");
                ui.horizontal(|ui| {
                    let available_width = ui.available_width();
                    let edit = egui::TextEdit::singleline(&mut self.new_recipe_name)
                        .hint_text("Recipe name...");
                    
                    ui.add_sized([available_width - 60.0, 20.0], edit);
                    
                    if ui.button("➕").clicked() && !self.new_recipe_name.trim().is_empty() {
                        let recipe = Recipe::new(self.new_recipe_name.trim());
                        let id = recipe.id;
                        self.recipe_manager.lock().unwrap().add_recipe(recipe).ok();
                        self.selected_recipe = Some(id);
                        self.new_recipe_name.clear();
                        self.show_status("Recipe created");
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);
                
                // Recipe list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let recipe_manager = self.recipe_manager.lock().unwrap();
                    let recipes: Vec<_> = recipe_manager.recipes.iter()
                        .map(|r| (r.id, r.name.clone(), r.icon.clone(), r.is_active))
                        .collect();
                    drop(recipe_manager);
                    
                    for (id, name, icon, is_active) in recipes {
                        let selected = self.selected_recipe == Some(id);
                        let icon_str = icon.unwrap_or_else(|| "📋".to_string());
                        let label = format!("{} {} {}", 
                            icon_str, 
                            name,
                            if is_active { "✓" } else { "" }
                        );
                        
                        if ui.selectable_label(selected, label).clicked() {
                            self.selected_recipe = Some(id);
                            self.editing_recipe = false;
                            self.update_preview();
                        }
                    }
                });
            });
        
        // Right panel - recipe editor
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(recipe_id) = self.selected_recipe {
                self.show_recipe_editor(ui, recipe_id);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a recipe to edit, or create a new one");
                });
            }
        });
    }
    
    fn show_recipe_editor(&mut self, ui: &mut egui::Ui, recipe_id: uuid::Uuid) {
        let recipe_manager = self.recipe_manager.lock().unwrap();
        
        let recipe_clone = recipe_manager.get_recipe(recipe_id).cloned();
        drop(recipe_manager);
        
        if let Some(mut recipe) = recipe_clone {
            // Header
            ui.horizontal(|ui| {
                ui.heading(format!("{} {}", 
                    recipe.icon.as_deref().unwrap_or("📋"), 
                    &recipe.name
                ));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑 Delete").clicked() {
                        self.recipe_manager.lock().unwrap().remove_recipe(recipe_id).ok();
                        self.selected_recipe = None;
                        self.show_status("Recipe deleted");
                        return;
                    }
                    
                    if recipe.is_active {
                        if ui.button("⏹ Deactivate").clicked() {
                            self.recipe_manager.lock().unwrap().deactivate_all().ok();
                            // Notify background service to reload recipe
                            IpcClient::send(IpcCommand::ReloadRecipe);
                            self.show_status("Recipe deactivated");
                        }
                    } else {
                        if ui.button("▶ Activate").clicked() {
                            self.recipe_manager.lock().unwrap().set_active(recipe_id).ok();
                            // Notify background service to reload recipe
                            IpcClient::send(IpcCommand::ReloadRecipe);
                            self.show_status("Recipe activated - transformations will be applied automatically");
                        }
                    }
                    
                    if ui.button("💾 Apply Now").clicked() {
                        match ClipboardManager::apply_recipe(&recipe) {
                            Ok(_) => self.show_status("Clipboard transformed!"),
                            Err(e) => self.show_status(format!("Error: {}", e)),
                        }
                    }
                });
            });
            
            ui.separator();
            
            // Recipe details
            ui.horizontal(|ui| {
                ui.label("Name:");
                if ui.text_edit_singleline(&mut recipe.name).changed() {
                    self.recipe_manager.lock().unwrap().update_recipe(recipe.clone()).ok();
                }
                
                ui.label("Icon:");
                let mut icon = recipe.icon.clone().unwrap_or_default();
                if ui.add(egui::TextEdit::singleline(&mut icon).desired_width(40.0)).changed() {
                    recipe.icon = if icon.is_empty() { None } else { Some(icon) };
                    self.recipe_manager.lock().unwrap().update_recipe(recipe.clone()).ok();
                }
            });
            
            let mut desc = recipe.description.clone().unwrap_or_default();
            ui.horizontal(|ui| {
                ui.label("Description:");
                if ui.text_edit_singleline(&mut desc).changed() {
                    recipe.description = if desc.is_empty() { None } else { Some(desc) };
                    self.recipe_manager.lock().unwrap().update_recipe(recipe.clone()).ok();
                }
            });
            
            ui.add_space(10.0);
            ui.separator();
            
            // Two-column layout for transformations and preview
            ui.columns(2, |columns| {
                // Left column - transformations
                columns[0].heading("Transformations");
                columns[0].add_space(5.0);
                
                egui::ScrollArea::vertical()
                    .id_salt("transformations_scroll")
                    .max_height(200.0)
                    .show(&mut columns[0], |ui| {
                        let mut to_remove = None;
                        
                        for (i, transformation) in recipe.transformations.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}.", i + 1));
                                ui.label(transformation.display_name());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("🗑").clicked() {
                                        to_remove = Some(i);
                                    }
                                });
                            });
                        }
                        
                        if let Some(i) = to_remove {
                            recipe.transformations.remove(i);
                            self.recipe_manager.lock().unwrap().update_recipe(recipe.clone()).ok();
                            self.update_preview();
                            self.show_status("Transformation removed");
                        }
                    });
                
                columns[0].add_space(10.0);
                columns[0].heading("➕ Add Transformation");
                columns[0].add_space(5.0);
                
                egui::ScrollArea::vertical()
                    .id_salt("add_transform_scroll")
                    .max_height(300.0)
                    .auto_shrink([false, false])
                    .show(&mut columns[0], |ui| {
                        ui.set_width(ui.available_width());
                        
                        // Group by category
                        for category in &[
                            "Whitespace", "Case Conversion", "Line Operations",
                            "Character Cleanup", "Content Removal", "HTML", "URL"
                        ] {
                            ui.collapsing(*category, |ui| {
                                ui.vertical_centered_justified(|ui| {
                                    for t in Self::get_transformations_for_category(category) {
                                        if ui.button(t.display_name()).clicked() {
                                            recipe.add_transformation(t.clone());
                                            self.recipe_manager.lock().unwrap().update_recipe(recipe.clone()).ok();
                                            self.update_preview();
                                            self.show_status(format!("Added: {}", t.display_name()));
                                        }
                                    }
                                });
                            });
                        }
                    });
                
                // Right column - preview
                columns[1].heading("Preview");
                columns[1].add_space(5.0);
                
                columns[1].label("Input:");
                egui::ScrollArea::vertical()
                    .id_salt("input_scroll")
                    .max_height(200.0)
                    .auto_shrink([false, false])
                    .show(&mut columns[1], |ui| {
                        let edit = egui::TextEdit::multiline(&mut self.test_input)
                            .desired_width(f32::INFINITY);
                        if ui.add(edit).changed() {
                            self.update_preview();
                        }
                    });
                
                columns[1].add_space(10.0);
                columns[1].label("Output:");
                egui::ScrollArea::vertical()
                    .id_salt("output_scroll")
                    .max_height(200.0)
                    .auto_shrink([false, false])
                    .show(&mut columns[1], |ui| {
                        ui.add(egui::TextEdit::multiline(&mut self.test_output)
                            .desired_width(f32::INFINITY)
                            .interactive(false));
                    });
            });
        }
    }
    
    fn get_transformations_for_category(category: &str) -> Vec<Transformation> {
        match category {
            "Whitespace" => vec![
                Transformation::NormalizeWhitespace,
                Transformation::TrimLines,
                Transformation::RemoveEmptyLines,
            ],
            "Case Conversion" => vec![
                Transformation::ToLowercase,
                Transformation::ToUppercase,
                Transformation::ToTitleCase,
                Transformation::ToSentenceCase,
                Transformation::ToCamelCase,
                Transformation::ToPascalCase,
                Transformation::ToSnakeCase,
                Transformation::ToScreamingSnakeCase,
                Transformation::ToKebabCase,
            ],
            "Line Operations" => vec![
                Transformation::RemoveDuplicateLines,
                Transformation::SortLines,
                Transformation::SortLinesReverse,
                Transformation::ReverseLines,
                Transformation::AddLineNumbers,
                Transformation::RemoveLineNumbers,
                Transformation::ToUnixLineEndings,
                Transformation::ToWindowsLineEndings,
            ],
            "Character Cleanup" => vec![
                Transformation::FixSmartQuotes,
                Transformation::RemoveNonAscii,
                Transformation::NormalizeUnicode,
                Transformation::RemoveEmojis,
                Transformation::StripFormatting,
            ],
            "Content Removal" => vec![
                Transformation::RemoveUrls,
                Transformation::RemoveEmails,
                Transformation::RemovePhoneNumbers,
                Transformation::RemoveMarkdown,
            ],
            "HTML" => vec![
                Transformation::EncodeHtmlEntities,
                Transformation::DecodeHtmlEntities,
            ],
            "URL" => vec![
                Transformation::Slugify,
            ],
            _ => vec![],
        }
    }
    
    fn show_settings_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("⚙️ Settings");
            ui.add_space(20.0);
            
            let mut config = self.config.lock().unwrap();
            
            egui::Grid::new("settings_grid")
                .num_columns(2)
                .spacing([40.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Start with system:");
                    if ui.checkbox(&mut config.start_with_system, "").changed() {
                        config.save().ok();
                    }
                    ui.end_row();
                    
                    ui.label("Start minimized:");
                    if ui.checkbox(&mut config.start_minimized, "").changed() {
                        config.save().ok();
                    }
                    ui.end_row();
                    
                    ui.label("Show notifications:");
                    if ui.checkbox(&mut config.show_notifications, "").changed() {
                        config.save().ok();
                    }
                    ui.end_row();
                    
                    ui.label("Auto-transform clipboard:");
                    if ui.checkbox(&mut config.auto_transform, "").changed() {
                        self.transform_enabled = config.auto_transform;
                        config.save().ok();
                    }
                    ui.end_row();
                    
                    ui.label("Keep clipboard history:");
                    if ui.checkbox(&mut config.keep_history, "").changed() {
                        config.save().ok();
                    }
                    ui.end_row();
                    
                    ui.label("Max history size:");
                    let mut size = config.max_history_size as i32;
                    if ui.add(egui::Slider::new(&mut size, 10..=500)).changed() {
                        config.max_history_size = size as usize;
                        config.save().ok();
                    }
                    ui.end_row();
                    
                    ui.label("Theme:");
                    egui::ComboBox::from_id_salt("theme_combo")
                        .selected_text(&config.theme)
                        .show_ui(ui, |ui| {
                            for theme in ["system", "dark", "light"] {
                                if ui.selectable_value(&mut config.theme, theme.to_string(), theme).changed() {
                                    config.save().ok();
                                }
                            }
                        });
                    ui.end_row();
                });
            
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            
            ui.heading("Hotkeys");
            ui.add_space(10.0);
            
            egui::Grid::new("hotkeys_grid")
                .num_columns(2)
                .spacing([40.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Toggle transformation:");
                    let mut toggle = config.toggle_hotkey.clone().unwrap_or_default();
                    if ui.text_edit_singleline(&mut toggle).changed() {
                        config.toggle_hotkey = if toggle.is_empty() { None } else { Some(toggle) };
                        config.save().ok();
                    }
                    ui.end_row();
                    
                    ui.label("Quick menu:");
                    let mut quick = config.quick_menu_hotkey.clone().unwrap_or_default();
                    if ui.text_edit_singleline(&mut quick).changed() {
                        config.quick_menu_hotkey = if quick.is_empty() { None } else { Some(quick) };
                        config.save().ok();
                    }
                    ui.end_row();
                    
                    ui.label("Open dashboard:");
                    let mut dash = config.dashboard_hotkey.clone().unwrap_or_default();
                    if ui.text_edit_singleline(&mut dash).changed() {
                        config.dashboard_hotkey = if dash.is_empty() { None } else { Some(dash) };
                        config.save().ok();
                    }
                    ui.end_row();
                });
        });
    }
    
    fn show_history_tab(&mut self, ctx: &egui::Context) {
        // Track actions to perform after UI rendering
        let mut action_clear = false;
        let mut action_refresh = false;
        let mut copy_text: Option<String> = None;
        let mut remove_index: Option<usize> = None;
        
        // Clone entries to avoid borrow issues
        let entries: Vec<_> = self.history_manager
            .as_ref()
            .map(|hm| hm.get_all().to_vec())
            .unwrap_or_default();
        let has_history = self.history_manager.is_some();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("📋 Clipboard History");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("� Clear All").clicked() {
                        action_clear = true;
                    }
                    if ui.button("🔄 Refresh").clicked() {
                        action_refresh = true;
                    }
                });
            });
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);
            
            if !has_history {
                ui.centered_and_justified(|ui| {
                    ui.label("Failed to load history.");
                });
            } else if entries.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("No history entries yet.\nTransformations will appear here.");
                });
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, entry) in entries.iter().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                // Timestamp and recipe name
                                let time_str = entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                                ui.label(format!("🕐 {}", time_str));
                                
                                if let Some(ref name) = entry.recipe_name {
                                    ui.label(format!("📝 {}", name));
                                }
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Delete button
                                    if ui.small_button("🗑").clicked() {
                                        remove_index = Some(i);
                                    }
                                    
                                    // Copy transformed button
                                    if let Some(ref transformed) = entry.transformed {
                                        if ui.small_button("📋 Copy Result").clicked() {
                                            copy_text = Some(transformed.clone());
                                        }
                                    }
                                    
                                    // Copy original button
                                    if ui.small_button("📄 Copy Original").clicked() {
                                        copy_text = Some(entry.original.clone());
                                    }
                                });
                            });
                            
                            ui.add_space(5.0);
                            
                            // Show preview of original text
                            let preview_len = 100.min(entry.original.len());
                            let preview = if entry.original.len() > 100 {
                                format!("{}...", &entry.original[..preview_len])
                            } else {
                                entry.original.clone()
                            };
                            
                            ui.horizontal(|ui| {
                                ui.label("Original:");
                                ui.add(egui::Label::new(preview).wrap());
                            });
                            
                            // Show preview of transformed text if different
                            if let Some(ref transformed) = entry.transformed {
                                if transformed != &entry.original {
                                    let t_preview_len = 100.min(transformed.len());
                                    let t_preview = if transformed.len() > 100 {
                                        format!("{}...", &transformed[..t_preview_len])
                                    } else {
                                        transformed.clone()
                                    };
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("Result:");
                                        ui.add(egui::Label::new(t_preview).wrap());
                                    });
                                }
                            }
                        });
                        
                        if i < entries.len() - 1 {
                            ui.add_space(5.0);
                        }
                    }
                });
            }
        });
        
        // Handle actions after UI
        if action_clear {
            if let Some(ref mut hm) = self.history_manager {
                if hm.clear().is_ok() {
                    self.show_status("History cleared");
                }
            }
        }
        
        if action_refresh {
            let max_size = self.config.lock().unwrap().max_history_size;
            self.history_manager = HistoryManager::new(max_size).ok();
            self.show_status("History refreshed");
        }
        
        if let Some(text) = copy_text {
            if ClipboardManager::set_text_background(&text).is_ok() {
                self.show_status("Copied to clipboard");
            }
        }
        
        if let Some(idx) = remove_index {
            if let Some(ref mut hm) = self.history_manager {
                if hm.remove(idx).is_ok() {
                    self.show_status("Entry removed");
                }
            }
        }
    }
    
    fn show_about_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.heading("🔧 9Paste");
                ui.label("Clipboard Transformer");
                ui.add_space(10.0);
                ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                ui.add_space(20.0);
                
                ui.label("A privacy-focused clipboard utility that automatically");
                ui.label("cleans, formats, and transforms clipboard text.");
                
                ui.add_space(30.0);
                ui.separator();
                ui.add_space(20.0);
                
                ui.heading("Features");
                ui.add_space(10.0);
                
                let features = [
                    "✨ 40+ text transformations",
                    "📝 Create reusable recipes",
                    "🔄 Auto-transform on paste",
                    "⌨️ Global hotkey support",
                    "🔒 100% local processing",
                    "🖥️ Cross-platform (Linux, macOS, Windows)",
                ];
                
                for feature in features {
                    ui.label(feature);
                }
                
                ui.add_space(30.0);
                ui.separator();
                ui.add_space(20.0);
                
                ui.label("🔒 Privacy First");
                ui.label("All processing happens locally on your device.");
                ui.label("No data is ever sent to any server.");
            });
        });
    }
}
