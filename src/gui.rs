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

#[derive(Default)]
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
        style.visuals.widgets.inactive.bg_fill = PANEL_BACKGROUND;
        style.visuals.widgets.hovered.bg_fill = ACCENT_COLOR;
        style.visuals.widgets.active.bg_fill = MAGENTA_GLOW;
        style.visuals.panel_fill = PANEL_BACKGROUND;
        cc.egui_ctx.set_style(style);
        
        Self {
            config: Some(config),
            ..Default::default()
        }
    }
}

impl eframe::App for MetGenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Draw header
            self.draw_header(ui);
            
            // Draw tabs
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                self.draw_tab_bar(ui);
            });
            
            // Draw content based on selected tab
            ui.add_space(20.0);
            match self.selected_tab {
                Tab::GenerateMetar => self.draw_generate_metar(ui),
                Tab::SavedAirports => self.draw_saved_airports(ui),
                Tab::Configuration => self.draw_configuration(ui),
            }
        });
    }
}

impl MetGenApp {
    fn draw_header(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.heading(RichText::new("METGen").color(CYAN_GLOW).size(32.0));
            ui.label(RichText::new("Synthesized METAR Generation").color(MAGENTA_GLOW).size(16.0));
            ui.add_space(5.0);
            ui.label(
                RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .color(TEXT_COLOR)
                    .size(14.0)
            );
            ui.add_space(10.0);
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
        ui.vertical_centered(|ui| {
            // API Selection
            ui.group(|ui| {
                ui.heading(RichText::new("Select API").color(CYAN_GLOW));
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_api, ApiType::Standard, "Standard API");
                    ui.selectable_value(&mut self.selected_api, ApiType::OneCall, "One Call API");
                });
            });
            
            ui.add_space(20.0);
            
            // Input Methods
            ui.group(|ui| {
                ui.heading(RichText::new("ICAO Code").color(CYAN_GLOW));
                ui.text_edit_singleline(&mut self.input_icao);
                if ui.button("Generate from ICAO").clicked() {
                    self.generate_metar_from_icao();
                }
            });
            
            ui.add_space(10.0);
            
            ui.group(|ui| {
                ui.heading(RichText::new("Latitude/Longitude").color(CYAN_GLOW));
                ui.horizontal(|ui| {
                    ui.label("ICAO:");
                    ui.text_edit_singleline(&mut self.input_icao);
                });
                ui.horizontal(|ui| {
                    ui.label("Lat:");
                    ui.text_edit_singleline(&mut self.input_lat);
                    ui.label("Lon:");
                    ui.text_edit_singleline(&mut self.input_lon);
                });
                if ui.button("Generate from Coordinates").clicked() {
                    self.generate_metar_from_coords();
                }
            });
            
            ui.add_space(10.0);
            
            ui.group(|ui| {
                ui.heading(RichText::new("Location Search").color(CYAN_GLOW));
                ui.horizontal(|ui| {
                    ui.label("ICAO:");
                    ui.text_edit_singleline(&mut self.input_icao);
                });
                ui.text_edit_singleline(&mut self.input_location);
                if ui.button("Generate from Location").clicked() {
                    self.generate_metar_from_location();
                }
            });
            
            // Display Results
            if !self.generated_metar.is_empty() {
                ui.add_space(20.0);
                ui.group(|ui| {
                    ui.heading(RichText::new("Generated METAR").color(MAGENTA_GLOW));
                    ui.label(RichText::new(&self.generated_metar).color(TEXT_COLOR));
                });
            }
            
            // Error/Success Messages
            if let Some(error) = &self.error_message {
                ui.add_space(10.0);
                ui.colored_label(Color32::RED, error);
            }
            if let Some(success) = &self.success_message {
                ui.add_space(10.0);
                ui.colored_label(Color32::GREEN, success);
            }
        });
    }

    fn draw_saved_airports(&mut self, ui: &mut egui::Ui) {
        let airports = get_user_airports();
        
        ui.vertical_centered(|ui| {
            ui.heading(RichText::new("Saved Airports").color(CYAN_GLOW));
            
            if airports.is_empty() {
                ui.label("No saved airports found");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for airport in airports {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&airport.icao).color(TEXT_COLOR));
                                ui.label(format!("(Lat: {:.4}, Lon: {:.4})", 
                                    airport.latitude, airport.longitude));
                                
                                if ui.button("Generate METAR").clicked() {
                                    self.generate_metar_for_saved_airport(&airport);
                                }
                                if ui.button("Delete").clicked() {
                                    if let Err(e) = delete_user_airport(&airport.icao) {
                                        self.error_message = Some(format!("Failed to delete airport: {}", e));
                                    } else {
                                        self.success_message = Some(format!("Deleted airport {}", airport.icao));
                                    }
                                }
                            });
                        });
                    }
                });
            }
        });
    }

    fn draw_configuration(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading(RichText::new("Configuration").color(CYAN_GLOW));
            // Add configuration UI elements here
            ui.label("Configuration options coming soon...");
        });
    }

    fn generate_metar_from_icao(&mut self) {
        self.error_message = None;
        self.success_message = None;
        
        if self.input_icao.is_empty() {
            self.error_message = Some("Please enter an ICAO code".to_string());
            return;
        }

        // Check for existing METAR first
        if let Some(existing_metar) = input_handler::poll_noaa_metar(&self.input_icao) {
            self.generated_metar = existing_metar;
            self.success_message = Some("Using existing METAR from NOAA".to_string());
            return;
        }

        // If no existing METAR, generate one
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
        self.generate_metar_with_coordinates(airport.latitude, airport.longitude);
    }

    fn generate_metar_with_coordinates(&mut self, lat: f64, lon: f64) {
        match self.selected_api {
            ApiType::Standard => {
                if let Some(config) = &self.config {
                    if let Some(metar) = metar_generator::generate_metar(
                        &self.input_icao,
                        lat,
                        lon,
                        config["decrypted_api_key"].as_str().unwrap(),
                        config["units"].as_str().unwrap(),
                    ) {
                        self.generated_metar = metar;
                        self.success_message = Some("METAR generated successfully".to_string());
                    } else {
                        self.error_message = Some("Failed to generate METAR".to_string());
                    }
                }
            }
            ApiType::OneCall => {
                if let Some(config) = &self.config {
                    if let Some(weather_data) = one_call_metar::fetch_weather_data(
                        lat,
                        lon,
                        config["decrypted_one_call_api_key"].as_str().unwrap(),
                    ) {
                        let parsed = one_call_metar::parse_weather_data(&weather_data);
                        self.generated_metar = one_call_metar::generate_metar(
                            &self.input_icao,
                            &parsed,
                            config["units"].as_str().unwrap(),
                        );
                        self.success_message = Some("METAR generated successfully".to_string());
                    } else {
                        self.error_message = Some("Failed to fetch weather data".to_string());
                    }
                }
            }
        }
    }
}

// ... existing code ... 