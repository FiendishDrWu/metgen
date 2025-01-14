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

#[derive(Default)]
pub struct MetGenApp {
    current_view: View,
    input_icao: String,
    input_lat: String,
    input_lon: String,
    input_location: String,
    generated_metar: String,
    error_message: Option<String>,
    success_message: Option<String>,
    config: Option<Value>,
    selected_api: ApiType,
}

#[derive(Default, PartialEq)]
enum View {
    #[default]
    Main,
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
    
    fn draw_header(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.heading(RichText::new("METGen").color(CYAN_GLOW).size(32.0));
            ui.label(RichText::new("Synthesized METAR Generation").color(MAGENTA_GLOW).size(16.0));
            ui.add_space(10.0);
            
            // Version info with glow effect
            ui.label(
                RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .color(TEXT_COLOR)
                    .size(14.0)
            );
            ui.add_space(20.0);
        });
    }
    
    fn draw_main_menu(&mut self, ui: &mut egui::Ui) {
        let button_size = Vec2::new(200.0, 40.0);
        
        ui.vertical_centered(|ui| {
            if ui.add_sized(button_size, egui::Button::new("Generate METAR")).clicked() {
                self.current_view = View::GenerateMetar;
            }
            
            ui.add_space(10.0);
            if ui.add_sized(button_size, egui::Button::new("Manage Saved Airports")).clicked() {
                self.current_view = View::SavedAirports;
            }
            
            ui.add_space(10.0);
            if ui.add_sized(button_size, egui::Button::new("Update Configuration")).clicked() {
                self.current_view = View::Configuration;
            }
            
            ui.add_space(10.0);
            if ui.add_sized(button_size, egui::Button::new("Exit")).clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
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
            
            ui.add_space(20.0);
            if ui.button("Back to Main Menu").clicked() {
                self.current_view = View::Main;
                self.clear_inputs();
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
            
            ui.add_space(20.0);
            if ui.button("Back to Main Menu").clicked() {
                self.current_view = View::Main;
            }
        });
    }
    
    fn clear_inputs(&mut self) {
        self.input_icao.clear();
        self.input_lat.clear();
        self.input_lon.clear();
        self.input_location.clear();
        self.generated_metar.clear();
        self.error_message = None;
        self.success_message = None;
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
                            
                            // Save the airport if it's not already saved
                            if !self.input_icao.is_empty() {
                                if let Err(e) = save_user_airport(self.input_icao.clone(), lat, lon) {
                                    self.error_message = Some(format!("Failed to save airport: {}", e));
                                }
                            }
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
                            
                            // Save the airport if it's not already saved
                            if !self.input_icao.is_empty() {
                                if let Err(e) = save_user_airport(self.input_icao.clone(), lat, lon) {
                                    self.error_message = Some(format!("Failed to save airport: {}", e));
                                }
                            }
                        } else {
                            self.error_message = Some("Failed to fetch weather data".to_string());
                        }
                    }
                }
            }
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
                match self.selected_api {
                    ApiType::Standard => {
                        if let Some(metar) = metar_generator::generate_metar(
                            &self.input_icao,
                            lat,
                            lon,
                            config["decrypted_api_key"].as_str().unwrap(),
                            config["units"].as_str().unwrap(),
                        ) {
                            self.generated_metar = metar;
                            self.success_message = Some("METAR generated successfully".to_string());
                            
                            // Save the airport if it's not already saved
                            if !self.input_icao.is_empty() {
                                if let Err(e) = save_user_airport(self.input_icao.clone(), lat, lon) {
                                    self.error_message = Some(format!("Failed to save airport: {}", e));
                                }
                            }
                        } else {
                            self.error_message = Some("Failed to generate METAR".to_string());
                        }
                    }
                    ApiType::OneCall => {
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
                            
                            // Save the airport if it's not already saved
                            if !self.input_icao.is_empty() {
                                if let Err(e) = save_user_airport(self.input_icao.clone(), lat, lon) {
                                    self.error_message = Some(format!("Failed to save airport: {}", e));
                                }
                            }
                        } else {
                            self.error_message = Some("Failed to fetch weather data".to_string());
                        }
                    }
                }
            } else {
                self.error_message = Some(format!("Could not resolve location: {}", self.input_location));
            }
        }
    }
    
    fn generate_metar_for_saved_airport(&mut self, airport: &UserAirport) {
        self.error_message = None;
        self.success_message = None;

        match self.selected_api {
            ApiType::Standard => {
                if let Some(config) = &self.config {
                    if let Some(metar) = metar_generator::generate_metar(
                        &airport.icao,
                        airport.latitude,
                        airport.longitude,
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
                        airport.latitude,
                        airport.longitude,
                        config["decrypted_one_call_api_key"].as_str().unwrap(),
                    ) {
                        let parsed = one_call_metar::parse_weather_data(&weather_data);
                        self.generated_metar = one_call_metar::generate_metar(
                            &airport.icao,
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

    fn draw_configuration(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading(RichText::new("Configuration").color(CYAN_GLOW));
            ui.add_space(20.0);

            let mut api_key = String::new();
            let mut one_call_api_key = String::new();
            let mut units = String::new();

            if let Some(config) = &self.config {
                api_key = config["decrypted_api_key"].as_str().unwrap_or("").to_string();
                one_call_api_key = config["decrypted_one_call_api_key"].as_str().unwrap_or("").to_string();
                units = config["units"].as_str().unwrap_or("metric").to_string();
            }

            ui.group(|ui| {
                ui.heading(RichText::new("API Keys").color(MAGENTA_GLOW));
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("OpenWeather API Key:");
                    if ui.text_edit_singleline(&mut api_key).changed() {
                        if let Some(config) = &mut self.config {
                            config["decrypted_api_key"] = serde_json::Value::String(api_key.clone());
                        }
                    }
                });

                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label("One Call API Key:");
                    if ui.text_edit_singleline(&mut one_call_api_key).changed() {
                        if let Some(config) = &mut self.config {
                            config["decrypted_one_call_api_key"] = serde_json::Value::String(one_call_api_key.clone());
                        }
                    }
                });
            });

            ui.add_space(20.0);
            ui.group(|ui| {
                ui.heading(RichText::new("Units").color(MAGENTA_GLOW));
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.radio_value(&mut units, "metric".to_string(), "Metric").clicked() {
                        if let Some(config) = &mut self.config {
                            config["units"] = serde_json::Value::String("metric".to_string());
                        }
                    }
                    if ui.radio_value(&mut units, "imperial".to_string(), "Imperial").clicked() {
                        if let Some(config) = &mut self.config {
                            config["units"] = serde_json::Value::String("imperial".to_string());
                        }
                    }
                });
            });

            ui.add_space(20.0);
            if ui.button("Save Configuration").clicked() {
                if let Err(e) = crate::config::save_config(&api_key, &one_call_api_key, &units) {
                    self.error_message = Some(format!("Failed to save configuration: {}", e));
                } else {
                    self.success_message = Some("Configuration saved successfully".to_string());
                }
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

            ui.add_space(20.0);
            if ui.button("Back to Main Menu").clicked() {
                self.current_view = View::Main;
                self.error_message = None;
                self.success_message = None;
            }
        });
    }
}

impl eframe::App for MetGenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_header(ui);
            
            match self.current_view {
                View::Main => self.draw_main_menu(ui),
                View::GenerateMetar => self.draw_generate_metar(ui),
                View::SavedAirports => self.draw_saved_airports(ui),
                View::Configuration => self.draw_configuration(ui),
            }
        });
    }
} 