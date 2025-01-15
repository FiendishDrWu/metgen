use eframe::egui::{self, Color32, RichText, Rounding, Stroke, Vec2};
use serde_json::Value;

use crate::config::{get_user_airports, save_user_airport, delete_user_airport, UserAirport};
use crate::metar_generator;
use crate::one_call_metar;
use crate::input_handler;

// Retro color scheme
const CYAN_GLOW: Color32 = Color32::from_rgb(0, 255, 255);
const MAGENTA_GLOW: Color32 = Color32::from_rgb(255, 0, 255);
const BACKGROUND: Color32 = Color32::from_rgb(5, 5, 10);
const TEXT_COLOR: Color32 = Color32::from_rgb(220, 220, 240);
const ACCENT_COLOR: Color32 = Color32::from_rgb(128, 0, 255);
const PANEL_BACKGROUND: Color32 = Color32::from_rgb(10, 10, 15);
const TAB_ACTIVE: Color32 = Color32::from_rgb(5, 5, 10);
const TAB_INACTIVE: Color32 = Color32::from_rgb(5, 5, 10);
const BORDER_GREY: Color32 = Color32::from_gray(64);
const GENERATE_BUTTON_COLOR: Color32 = Color32::from_rgb(0, 255, 0);
const GENERATE_BUTTON_TEXT: Color32 = Color32::BLACK;

#[derive(Default, PartialEq, Clone, Copy)]
enum Units {
    #[default]
    Metric,
    Imperial,
}

pub struct MetGenApp {
    input_icao: String,
    input_lat: String,
    input_lon: String,
    input_location: String,
    generated_metar: String,
    error_message: Option<String>,
    success_message: Option<String>,
    config: Option<Value>,
    selected_api: ApiType,
    selected_tab: Tab,
    selected_units: Units,
    existing_metar: Option<String>,  // Store existing METAR when found
}

impl Default for MetGenApp {
    fn default() -> Self {
        Self {
            input_icao: String::new(),
            input_lat: String::new(),
            input_lon: String::new(),
            input_location: String::new(),
            generated_metar: String::new(),
            error_message: None,
            success_message: None,
            config: None,
            selected_api: ApiType::default(),
            selected_tab: Tab::default(),
            selected_units: Units::default(),
            existing_metar: None,
        }
    }
}

#[derive(Default, PartialEq, Clone)]
enum Tab {
    #[default]
    GenerateMetar,
    SavedAirports,
    Configuration,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum ApiType {
    #[default]
    Standard,
    OneCall,
}

impl MetGenApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Value) -> Self {
        // Set up custom fonts and theme
        let mut fonts = egui::FontDefinitions::default();
        // TODO: Add custom retro font if desired
        
        cc.egui_ctx.set_fonts(fonts);
        
        // Set up retro theme
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.window_rounding = Rounding::default();
        style.visuals.window_fill = BACKGROUND;
        style.visuals.window_stroke = Stroke::new(1.0, CYAN_GLOW);
        style.visuals.widgets.noninteractive.bg_fill = PANEL_BACKGROUND;
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(0.0, Color32::TRANSPARENT);
        style.visuals.widgets.inactive.bg_fill = PANEL_BACKGROUND;
        style.visuals.widgets.hovered.bg_fill = ACCENT_COLOR;
        style.visuals.widgets.active.bg_fill = MAGENTA_GLOW;
        style.visuals.panel_fill = PANEL_BACKGROUND;
        cc.egui_ctx.set_style(style);
        
        // Initialize selected_units from config
        let selected_units = if let Some(units) = config.get("units").and_then(|u| u.as_str()) {
            match units {
                "imperial" => Units::Imperial,
                _ => Units::Metric,
            }
        } else {
            Units::default()
        };
        
        Self {
            config: Some(config),
            selected_units,
            ..Default::default()
        }
    }
}

impl eframe::App for MetGenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let total_height = ctx.screen_rect().height();
        let total_width = ctx.screen_rect().width();
        
        // Fixed proportions
        let header_height = total_height * 0.15;  // 15% for header
        let content_height = total_height * 0.52; // 52% for middle section
        let output_height = total_height * 0.33;  // 33% for output section
        let half_width = total_width * 0.5;       // 50% of width for each side

        // Header panel
        egui::TopBottomPanel::top("header")
            .exact_height(header_height)
            .frame(egui::Frame::none()
                .inner_margin(egui::style::Margin::symmetric(10.0, 10.0))
                .fill(TAB_ACTIVE))
            .show(ctx, |ui| {
                self.draw_header(ui);
            });

        // Main content area (middle section)
        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(TAB_ACTIVE))
            .show(ctx, |ui| {
                ui.set_min_height(content_height);
                ui.set_max_height(content_height);
                
                ui.horizontal(|ui| {
                    // Left half - Tab content with proper frame
                    ui.allocate_ui_with_layout(
                        Vec2::new(half_width, content_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            egui::Frame::none()
                                .fill(TAB_ACTIVE)
                                .inner_margin(egui::style::Margin::symmetric(10.0, 10.0))
                                .show(ui, |ui| {
                                    ui.set_min_width(half_width);
                                    ui.set_max_width(half_width);
                                    ui.set_min_height(content_height - 20.0); // Account for margins
                                    ui.set_max_height(content_height - 20.0);
                                    
                                    ui.vertical(|ui| {
                                        self.draw_tab_bar(ui);
                                        match self.selected_tab {
                                            Tab::GenerateMetar => self.draw_generate_metar(ui),
                                            Tab::SavedAirports => self.draw_saved_airports(ui),
                                            Tab::Configuration => self.draw_configuration(ui),
                                        }
                                    });
                                });
                        }
                    );

                    // Right half - Reserved for future use with proper frame
                    ui.allocate_ui_with_layout(
                        Vec2::new(half_width, content_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            egui::Frame::none()
                                .fill(TAB_ACTIVE)
                                .inner_margin(egui::style::Margin::symmetric(10.0, 10.0))
                                .show(ui, |ui| {
                                    ui.set_min_width(half_width);
                                    ui.set_max_width(half_width);
                                    ui.set_min_height(content_height - 20.0); // Account for margins
                                    ui.set_max_height(content_height - 20.0);
                                    // Reserved for future use
                                });
                        }
                    );
                });
            });

        // Bottom output panel
        egui::TopBottomPanel::bottom("output")
            .exact_height(output_height)
            .frame(egui::Frame::none()
                .inner_margin(egui::style::Margin::symmetric(10.0, 10.0))
                .fill(TAB_ACTIVE))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Display Results
                    if let Some(existing) = &self.existing_metar.clone() {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                egui::Frame::none()
                                    .inner_margin(egui::style::Margin::same(8.0))
                                    .stroke(Stroke::new(1.0, CYAN_GLOW))
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            ui.heading(RichText::new("Existing METAR Found").color(MAGENTA_GLOW));
                                            ui.label(RichText::new(existing).color(TEXT_COLOR).size(16.0));
                                            
                                            ui.add_space(10.0);
                                            ui.horizontal(|ui| {
                                                let existing = existing.clone();
                                                if ui.button("Use Existing METAR").clicked() {
                                                    self.generated_metar = existing;
                                                    self.existing_metar = None;
                                                    self.success_message = Some("Using existing METAR from NOAA".to_string());
                                                }
                                                ui.add_space(20.0);
                                                if ui.add(egui::Button::new(RichText::new("Generate Synthesized METAR")
                                                    .color(GENERATE_BUTTON_TEXT))
                                                    .fill(GENERATE_BUTTON_COLOR))
                                                    .clicked() {
                                                    if let Some((lat, lon)) = input_handler::resolve_icao_to_lat_lon(&self.input_icao) {
                                                        self.generate_metar_with_coordinates(lat, lon);
                                                        self.existing_metar = None;
                                                    }
                                                }
                                            });
                                        });
                                    });
                            });
                        });
                    } else if !self.generated_metar.is_empty() {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                egui::Frame::none()
                                    .inner_margin(egui::style::Margin::same(8.0))
                                    .stroke(Stroke::new(1.0, CYAN_GLOW))
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            ui.heading(RichText::new("Generated METAR").color(MAGENTA_GLOW));
                                            ui.label(RichText::new(&self.generated_metar).color(TEXT_COLOR).size(16.0));
                                            
                                            // Add warning statement
                                            ui.add_space(10.0);
                                            ui.horizontal(|ui| {
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    ui.label(RichText::new("Not for aviation purposes").color(MAGENTA_GLOW).size(14.0));
                                                    ui.label(RichText::new("For simulator use only.").color(CYAN_GLOW).size(14.0));
                                                });
                                            });
                                            
                                            // Only show save button for custom location METARs
                                            if !self.input_icao.is_empty() && 
                                               (!self.input_lat.is_empty() || !self.input_location.is_empty()) {
                                                ui.add_space(10.0);
                                                ui.horizontal(|ui| {
                                                    if ui.button("Save Airport").clicked() {
                                                        if !self.input_lat.is_empty() {
                                                            // Save from lat/lon logic...
                                                            if let Ok(lat) = self.input_lat.parse::<f64>() {
                                                                if let Ok(lon) = self.input_lon.parse::<f64>() {
                                                                    if let Some((lat, lon)) = input_handler::validate_lat_lon(lat, lon) {
                                                                        if let Err(e) = save_user_airport(self.input_icao.to_uppercase(), lat, lon) {
                                                                            self.error_message = Some(format!("Failed to save airport: {}", e));
                                                                        } else {
                                                                            self.success_message = Some(format!("Saved airport {}", self.input_icao.to_uppercase()));
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        } else {
                                                            // Save from location search logic...
                                                            if let Some(config) = &self.config {
                                                                if let Some((lat, lon)) = input_handler::resolve_freeform_input(
                                                                    &self.input_location,
                                                                    config["decrypted_api_key"].as_str().unwrap(),
                                                                ) {
                                                                    if let Err(e) = save_user_airport(self.input_icao.to_uppercase(), lat, lon) {
                                                                        self.error_message = Some(format!("Failed to save airport: {}", e));
                                                                    } else {
                                                                        self.success_message = Some(format!("Saved airport {}", self.input_icao.to_uppercase()));
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                });
                                            }
                                        });
                                    });
                            });
                        });
                    }
                    
                    // Error/Success Messages
                    ui.add_space(8.0);
                    if let Some(error) = &self.error_message {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            ui.add_space(8.0);
                            ui.colored_label(Color32::RED, RichText::new(error).size(16.0));
                        });
                    }
                    if let Some(success) = &self.success_message {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            ui.add_space(8.0);
                            ui.colored_label(Color32::GREEN, RichText::new(success).size(16.0));
                        });
                    }
                });
            });
    }
}

impl MetGenApp {
    fn draw_header(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading(RichText::new("METGen").color(CYAN_GLOW).size(32.0));
            ui.label(RichText::new("Synthesized METAR Generation").color(MAGENTA_GLOW).size(16.0));
            ui.label(
                RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .color(TEXT_COLOR)
                    .size(14.0)
            );
        });
    }

    fn draw_tab_bar(&mut self, ui: &mut egui::Ui) {
        let tab_height = 30.0;
        let tab_padding = Vec2::new(20.0, 5.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 1.0;  // Minimal spacing between tabs
            
            for tab in [Tab::GenerateMetar, Tab::SavedAirports, Tab::Configuration] {
                let is_selected = self.selected_tab == tab;
                let text = match tab {
                    Tab::GenerateMetar => "Generate METAR",
                    Tab::SavedAirports => "Saved Airports",
                    Tab::Configuration => "Configuration",
                };

                let button = egui::Button::new(
                    RichText::new(text)
                        .color(if is_selected { MAGENTA_GLOW } else { CYAN_GLOW })
                )
                .fill(if is_selected { Color32::from_rgb(40, 40, 40) } else { Color32::BLACK });

                // Create a custom frame for the button with our desired styling
                let frame = egui::Frame::none()
                    .fill(if is_selected { TAB_ACTIVE } else { TAB_INACTIVE })
                    .inner_margin(tab_padding)
                    .show(ui, |ui| {
                        ui.add_sized(Vec2::new(0.0, tab_height), button)
                    });

                if frame.inner.clicked() {
                    self.selected_tab = tab.clone();
                }
            }
        });

        // Draw separator line below tabs
        ui.add_space(1.0);
        let rect = ui.max_rect();
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(1.0, CYAN_GLOW),
        );
    }

    fn draw_generate_metar(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(0.0);

            // API Selection
            ui.horizontal(|ui| {
                ui.add_space(40.0);  // Same left margin as other elements
                ui.selectable_value(&mut self.selected_api, ApiType::Standard, "Standard API");
                ui.add_space(20.0);
                ui.selectable_value(&mut self.selected_api, ApiType::OneCall, "One Call API");
            });
            
            ui.add_space(15.0);
            
            // Input Methods - all left-aligned with consistent spacing
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add_space(40.0);  // Left margin
                ui.vertical(|ui| {
                    // ICAO Input
                    ui.horizontal(|ui| {
                        ui.set_width(300.0);
                        ui.label("ICAO Lookup:");
                        ui.add_space(10.0);
                        let icao_edit = egui::TextEdit::singleline(&mut self.input_icao)
                            .desired_width(40.0);
                        ui.add(icao_edit);
                        ui.add_space(10.0);
                        if ui.add(egui::Button::new(RichText::new("Generate")
                            .color(GENERATE_BUTTON_TEXT))
                            .fill(GENERATE_BUTTON_COLOR))
                            .clicked() {
                            self.generate_metar_from_icao();
                        }
                    });
                    
                    ui.add_space(10.0);  // Reduced from 15.0 to 10.0
                    
                    // Lat/Lon Input with required ICAO
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.set_width(300.0);
                            ui.label("Custom Location ICAO:");
                            ui.add_space(10.0);
                            let icao_edit = egui::TextEdit::singleline(&mut self.input_icao)
                                .desired_width(40.0);
                            ui.add(icao_edit);
                        });
                        ui.horizontal(|ui| {
                            ui.set_width(300.0);
                            ui.label("Lat:");
                            ui.add_space(10.0);
                            let lat_edit = egui::TextEdit::singleline(&mut self.input_lat)
                                .desired_width(80.0);
                            ui.add(lat_edit);
                            ui.add_space(10.0);
                            ui.label("Lon:");
                            ui.add_space(10.0);
                            let lon_edit = egui::TextEdit::singleline(&mut self.input_lon)
                                .desired_width(80.0);
                            ui.add(lon_edit);
                        });
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new(RichText::new("Generate")
                                .color(GENERATE_BUTTON_TEXT))
                                .fill(GENERATE_BUTTON_COLOR))
                                .clicked() {
                                if self.input_icao.is_empty() {
                                    self.error_message = Some("Please enter an ICAO code for the location".to_string());
                                } else {
                                    self.generate_metar_from_coords();
                                }
                            }
                        });
                    });
                    
                    ui.add_space(5.0);  // Reduced from 10.0 to 5.0
                    
                    // Location Search with required ICAO
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.set_width(300.0);
                            ui.label("Custom Location ICAO:");
                            ui.add_space(10.0);
                            let icao_edit = egui::TextEdit::singleline(&mut self.input_icao)
                                .desired_width(40.0);
                            ui.add(icao_edit);
                        });
                        ui.horizontal(|ui| {
                            ui.set_width(300.0);
                            ui.label("Location:");
                            ui.add_space(10.0);
                            let location_edit = egui::TextEdit::singleline(&mut self.input_location)
                                .desired_width(120.0)
                                .min_size(Vec2::new(120.0, 0.0));
                            ui.add(location_edit);
                        });
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new(RichText::new("Generate")
                                .color(GENERATE_BUTTON_TEXT))
                                .fill(GENERATE_BUTTON_COLOR))
                                .clicked() {
                                if self.input_icao.is_empty() {
                                    self.error_message = Some("Please enter an ICAO code for the location".to_string());
                                } else {
                                    self.generate_metar_from_location();
                                }
                            }
                        });
                    });
                });
            });
        });
    }

    fn draw_saved_airports(&mut self, ui: &mut egui::Ui) {
        let airports = get_user_airports();
        let available_height = ui.available_height();

        ui.vertical(|ui| {
            ui.set_min_height(available_height);
            ui.set_max_height(available_height);
            
            // API Selection and Title on same line
            ui.horizontal(|ui| {
                // API Selection on left
                ui.add_space(40.0);
                ui.selectable_value(&mut self.selected_api, ApiType::Standard, "Standard API");
                ui.add_space(20.0);
                ui.selectable_value(&mut self.selected_api, ApiType::OneCall, "One Call API");
                
                // Push title to right edge
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.heading(RichText::new("Saved Airports").color(CYAN_GLOW));
                });
            });
            
            ui.add_space(15.0);

            if airports.is_empty() {
                ui.label("No saved airports found");
            } else {
                egui::ScrollArea::vertical()
                    .max_height(available_height - 100.0)  // Account for header and API selection
                    .show(ui, |ui| {
                        for airport in airports {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&airport.icao).color(TEXT_COLOR));
                                    ui.label(format!("(Lat: {:.4}, Lon: {:.4})", 
                                        airport.latitude, airport.longitude));
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        // Delete button with red color and trashcan icon
                                        let delete_button = egui::Button::new(RichText::new("🗑").color(Color32::RED))
                                            .fill(Color32::from_rgb(40, 0, 0));
                                        if ui.add(delete_button).clicked() {
                                            if let Err(e) = delete_user_airport(&airport.icao) {
                                                self.error_message = Some(format!("Failed to delete airport: {}", e));
                                            } else {
                                                self.success_message = Some(format!("Deleted airport {}", airport.icao));
                                            }
                                        }
                                        if ui.add(egui::Button::new(RichText::new("Generate")
                                            .color(GENERATE_BUTTON_TEXT))
                                            .fill(GENERATE_BUTTON_COLOR))
                                            .clicked() {
                                            self.generate_metar_for_saved_airport(&airport);
                                        }
                                    });
                                });
                            });
                            ui.add_space(5.0);
                        }
                    });
            }
        });
    }

    fn draw_configuration(&mut self, ui: &mut egui::Ui) {
        let available_height = ui.available_height();

        ui.vertical(|ui| {
            ui.set_min_height(available_height);
            ui.set_max_height(available_height);
            
            ui.heading(RichText::new("Configuration").color(CYAN_GLOW));
            ui.add_space(15.0);
            
            // API Keys Configuration
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new("API Keys").color(MAGENTA_GLOW));
                    ui.add_space(10.0);
                    
                    if let Some(config) = &mut self.config {
                        // Standard API Key
                        ui.horizontal(|ui| {
                            ui.add_space(40.0);
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.set_min_width(100.0);  // Reduced from 120.0
                                ui.label(RichText::new("Standard API Key:").size(14.0));
                            });
                            let mut api_key = config["decrypted_api_key"].as_str().unwrap_or("").to_string();
                            let api_edit = egui::TextEdit::singleline(&mut api_key)
                                .desired_width(600.0)
                                .hint_text("32 characters required");
                            if ui.add(api_edit).changed() {
                                // Limit to 32 characters
                                if api_key.len() > 32 {
                                    api_key.truncate(32);
                                }
                                // Show error if less than 32 characters
                                if api_key.len() < 32 {
                                    self.error_message = Some(format!("Standard API Key must be exactly 32 characters (currently {})", api_key.len()));
                                } else {
                                    self.error_message = None;
                                }
                                // Read current config to preserve all data
                                if let Ok(contents) = std::fs::read_to_string("config.json") {
                                    if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&contents) {
                                        json["api_key"] = serde_json::Value::String(crate::config::encrypt_key(&api_key));
                                        if let Ok(config_str) = serde_json::to_string_pretty(&json) {
                                            if let Err(e) = std::fs::write("config.json", config_str) {
                                                self.error_message = Some(format!("Failed to save configuration: {}", e));
                                            }
                                        }
                                        config["decrypted_api_key"] = serde_json::Value::String(api_key);
                                    }
                                }
                            }
                        });
                        
                        // OneCall API Key
                        ui.horizontal(|ui| {
                            ui.add_space(40.0);
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.set_min_width(100.0);  // Reduced from 120.0
                                ui.label(RichText::new("OneCall API Key:").size(14.0));
                            });
                            let mut one_call_key = config["decrypted_one_call_api_key"].as_str().unwrap_or("").to_string();
                            let one_call_edit = egui::TextEdit::singleline(&mut one_call_key)
                                .desired_width(600.0)
                                .hint_text("32 characters required");
                            if ui.add(one_call_edit).changed() {
                                // Limit to 32 characters
                                if one_call_key.len() > 32 {
                                    one_call_key.truncate(32);
                                }
                                // Show error if less than 32 characters
                                if one_call_key.len() < 32 {
                                    self.error_message = Some(format!("OneCall API Key must be exactly 32 characters (currently {})", one_call_key.len()));
                                } else {
                                    self.error_message = None;
                                }
                                // Read current config to preserve all data
                                if let Ok(contents) = std::fs::read_to_string("config.json") {
                                    if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&contents) {
                                        json["one_call_api_key"] = serde_json::Value::String(crate::config::encrypt_key(&one_call_key));
                                        if let Ok(config_str) = serde_json::to_string_pretty(&json) {
                                            if let Err(e) = std::fs::write("config.json", config_str) {
                                                self.error_message = Some(format!("Failed to save configuration: {}", e));
                                            }
                                        }
                                        config["decrypted_one_call_api_key"] = serde_json::Value::String(one_call_key);
                                    }
                                }
                            }
                        });
                    }
                });
            });
            
            ui.add_space(15.0);
            
            // Units Selection
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new("Units").color(MAGENTA_GLOW));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.add_space(40.0);  // Same left margin as other elements
                        let prev_units = self.selected_units;
                        ui.selectable_value(&mut self.selected_units, Units::Metric, "Metric");
                        ui.add_space(20.0);
                        ui.selectable_value(&mut self.selected_units, Units::Imperial, "Imperial");
                        
                        // If units changed, update config.json
                        if prev_units != self.selected_units {
                            if let Ok(contents) = std::fs::read_to_string("config.json") {
                                if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&contents) {
                                    // Update only the units
                                    json["units"] = serde_json::Value::String(match self.selected_units {
                                        Units::Metric => "metric",
                                        Units::Imperial => "imperial",
                                    }.to_string());
                                    // Write back to file
                                    if let Ok(config_str) = serde_json::to_string_pretty(&json) {
                                        if let Err(e) = std::fs::write("config.json", config_str) {
                                            self.error_message = Some(format!("Failed to save configuration: {}", e));
                                        }
                                    }
                                }
                            }
                        }
                    });
                });
            });
        });
    }

    fn generate_metar_from_icao(&mut self) {
        self.error_message = None;
        self.success_message = None;
        self.existing_metar = None;
        
        if self.input_icao.is_empty() {
            self.error_message = Some("Please enter an ICAO code".to_string());
            return;
        }

        // Check for existing METAR
        if let Some(existing_metar) = input_handler::poll_noaa_metar(&self.input_icao) {
            self.existing_metar = Some(existing_metar);
            self.success_message = Some("Found existing METAR. Please choose an option with the buttons.".to_string());
            return;
        }

        // No existing METAR, generate one
        if let Some((lat, lon)) = input_handler::resolve_icao_to_lat_lon(&self.input_icao) {
            self.generate_metar_with_coordinates(lat, lon);
        } else {
            self.error_message = Some(format!("Could not resolve ICAO code: {}", self.input_icao));
        }
    }

    fn generate_metar_from_coords(&mut self) {
        self.error_message = None;
        self.success_message = None;
        
        if self.input_lat.is_empty() || self.input_lon.is_empty() {
            self.error_message = Some("Please enter both latitude and longitude".to_string());
            return;
        }

        let lat = match self.input_lat.parse::<f64>() {
            Ok(lat) => lat,
            Err(_) => {
                self.error_message = Some("Invalid latitude format".to_string());
                return;
            }
        };

        let lon = match self.input_lon.parse::<f64>() {
            Ok(lon) => lon,
            Err(_) => {
                self.error_message = Some("Invalid longitude format".to_string());
                return;
            }
        };

        if let Some((lat, lon)) = input_handler::validate_lat_lon(lat, lon) {
            self.generate_metar_with_coordinates(lat, lon);
        } else {
            self.error_message = Some("Invalid latitude/longitude values".to_string());
        }
    }

    fn generate_metar_from_location(&mut self) {
        self.error_message = None;
        self.success_message = None;
        
        if self.input_location.is_empty() {
            self.error_message = Some("Please enter a location".to_string());
            return;
        }

        if let Some(config) = &self.config {
            if let Some((lat, lon)) = input_handler::resolve_freeform_input(
                &self.input_location,
                config["decrypted_api_key"].as_str().unwrap(),
            ) {
                self.generate_metar_with_coordinates(lat, lon);
            } else {
                self.error_message = Some(format!("Could not resolve location: {}", self.input_location));
            }
        }
    }

    fn generate_metar_for_saved_airport(&mut self, airport: &UserAirport) {
        self.error_message = None;
        self.success_message = None;
        self.input_icao = airport.icao.clone();  // Store ICAO for METAR generation
        self.generate_metar_with_coordinates(airport.latitude, airport.longitude);
    }

    fn generate_metar_with_coordinates(&mut self, lat: f64, lon: f64) {
        if let Some(config) = &self.config {
            let api_key = match self.selected_api {
                ApiType::Standard => config["decrypted_api_key"].as_str(),
                ApiType::OneCall => config["decrypted_one_call_api_key"].as_str(),
            };

            if let Some(key) = api_key {
                let units = match self.selected_units {
                    Units::Metric => "metric",
                    Units::Imperial => "imperial",
                };

                let result = match self.selected_api {
                    ApiType::Standard => {
                        metar_generator::generate_metar(&self.input_icao, lat, lon, key, units)
                    },
                    ApiType::OneCall => {
                        if let Some(weather_data) = one_call_metar::fetch_weather_data(lat, lon, key) {
                            let parsed = one_call_metar::parse_weather_data(&weather_data);
                            Some(one_call_metar::generate_metar(&self.input_icao, &parsed, units))
                        } else {
                            None
                        }
                    },
                };

                match result {
                    Some(metar) => {
                        self.generated_metar = metar;
                        self.success_message = Some("METAR generated successfully".to_string());
                    },
                    None => {
                        self.error_message = Some("Failed to generate METAR".to_string());
                    }
                }
            } else {
                self.error_message = Some("API key not found in configuration".to_string());
            }
        } else {
            self.error_message = Some("Configuration not loaded".to_string());
        }
    }

    fn generate_and_save_from_coords(&mut self) {
        self.error_message = None;
        self.success_message = None;
        
        if self.input_lat.is_empty() || self.input_lon.is_empty() {
            self.error_message = Some("Please enter both latitude and longitude".to_string());
            return;
        }

        let lat = match self.input_lat.parse::<f64>() {
            Ok(lat) => lat,
            Err(_) => {
                self.error_message = Some("Invalid latitude format".to_string());
                return;
            }
        };

        let lon = match self.input_lon.parse::<f64>() {
            Ok(lon) => lon,
            Err(_) => {
                self.error_message = Some("Invalid longitude format".to_string());
                return;
            }
        };

        if let Some((lat, lon)) = input_handler::validate_lat_lon(lat, lon) {
            // Generate METAR first
            self.generate_metar_with_coordinates(lat, lon);
            
            // Then save the airport
            if let Err(e) = save_user_airport(self.input_icao.clone(), lat, lon) {
                self.error_message = Some(format!("Failed to save airport: {}", e));
            } else {
                self.success_message = Some(format!("Generated METAR and saved airport {}", self.input_icao));
            }
        } else {
            self.error_message = Some("Invalid latitude/longitude values".to_string());
        }
    }

    fn generate_and_save_from_location(&mut self) {
        self.error_message = None;
        self.success_message = None;
        
        if self.input_location.is_empty() {
            self.error_message = Some("Please enter a location".to_string());
            return;
        }

        if let Some(config) = &self.config {
            if let Some((lat, lon)) = input_handler::resolve_freeform_input(
                &self.input_location,
                config["decrypted_api_key"].as_str().unwrap(),
            ) {
                // Generate METAR first
                self.generate_metar_with_coordinates(lat, lon);
                
                // Then save the airport
                if let Err(e) = save_user_airport(self.input_icao.clone(), lat, lon) {
                    self.error_message = Some(format!("Failed to save airport: {}", e));
                } else {
                    self.success_message = Some(format!("Generated METAR and saved airport {}", self.input_icao));
                }
            } else {
                self.error_message = Some(format!("Could not resolve location: {}", self.input_location));
            }
        }
    }

    fn draw_output(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Display Results
            if let Some(existing) = &self.existing_metar.clone() {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        egui::Frame::none()
                            .inner_margin(egui::style::Margin::same(8.0))
                            .stroke(Stroke::new(1.0, CYAN_GLOW))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.heading(RichText::new("Existing METAR Found").color(MAGENTA_GLOW));
                                    ui.label(RichText::new(existing).color(TEXT_COLOR).size(16.0));
                                    
                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        let existing = existing.clone();
                                        if ui.button("Use Existing METAR").clicked() {
                                            self.generated_metar = existing;
                                            self.existing_metar = None;
                                            self.success_message = Some("Using existing METAR from NOAA".to_string());
                                        }
                                        ui.add_space(20.0);
                                        if ui.add(egui::Button::new(RichText::new("Generate Synthesized METAR")
                                            .color(GENERATE_BUTTON_TEXT))
                                            .fill(GENERATE_BUTTON_COLOR))
                                            .clicked() {
                                            if let Some((lat, lon)) = input_handler::resolve_icao_to_lat_lon(&self.input_icao) {
                                                self.generate_metar_with_coordinates(lat, lon);
                                                self.existing_metar = None;
                                            }
                                        }
                                    });
                                });
                            });
                    });
                });
            } else if !self.generated_metar.is_empty() {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        egui::Frame::none()
                            .inner_margin(egui::style::Margin::same(8.0))
                            .stroke(Stroke::new(1.0, CYAN_GLOW))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.heading(RichText::new("Generated METAR").color(MAGENTA_GLOW));
                                    ui.label(RichText::new(&self.generated_metar).color(TEXT_COLOR).size(16.0));
                                    
                                    // Add warning statement
                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(RichText::new("Not for aviation purposes").color(MAGENTA_GLOW).size(14.0));
                                            ui.label(RichText::new("For simulator use only.").color(CYAN_GLOW).size(14.0));
                                        });
                                    });
                                    
                                    // Only show save button for custom location METARs
                                    if !self.input_icao.is_empty() && 
                                       (!self.input_lat.is_empty() || !self.input_location.is_empty()) {
                                        ui.add_space(10.0);
                                        ui.horizontal(|ui| {
                                            if ui.button("Save Airport").clicked() {
                                                if !self.input_lat.is_empty() {
                                                    // Save from lat/lon logic...
                                                    if let Ok(lat) = self.input_lat.parse::<f64>() {
                                                        if let Ok(lon) = self.input_lon.parse::<f64>() {
                                                            if let Some((lat, lon)) = input_handler::validate_lat_lon(lat, lon) {
                                                                if let Err(e) = save_user_airport(self.input_icao.to_uppercase(), lat, lon) {
                                                                    self.error_message = Some(format!("Failed to save airport: {}", e));
                                                                } else {
                                                                    self.success_message = Some(format!("Saved airport {}", self.input_icao.to_uppercase()));
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    // Save from location search logic...
                                                    if let Some(config) = &self.config {
                                                        if let Some((lat, lon)) = input_handler::resolve_freeform_input(
                                                            &self.input_location,
                                                            config["decrypted_api_key"].as_str().unwrap(),
                                                        ) {
                                                            if let Err(e) = save_user_airport(self.input_icao.to_uppercase(), lat, lon) {
                                                                self.error_message = Some(format!("Failed to save airport: {}", e));
                                                            } else {
                                                                self.success_message = Some(format!("Saved airport {}", self.input_icao.to_uppercase()));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    }
                                });
                            });
                    });
                });
            }
            
            // Error/Success Messages
            ui.add_space(8.0);
            if let Some(error) = &self.error_message {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    ui.add_space(8.0);
                    ui.colored_label(Color32::RED, RichText::new(error).size(16.0));
                });
            }
            if let Some(success) = &self.success_message {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    ui.add_space(8.0);
                    ui.colored_label(Color32::GREEN, RichText::new(success).size(16.0));
                });
            }
        });
    }
}

// ... existing code ... 