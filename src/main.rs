#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

// METGen - The Synthesized METAR Generator
// Copyright (C) 2025 FiendishDrWu
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::process;
use eframe::egui::ViewportBuilder;

mod config;
mod input_handler;
mod metar_generator;
mod one_call_metar;
mod gui;

use config::{load_config, ensure_config_exists};
use gui::{MetGenApp, Tab};

fn main() -> eframe::Result<()> {
    // Create default config if it doesn't exist
    let is_first_run = ensure_config_exists().unwrap_or(false);

    // Load config, including decrypted keys
    let (config_json, decrypted_api_key, decrypted_one_call_api_key) = load_config();

    if config_json.is_null() {
        eprintln!("Failed to load configuration.");
        process::exit(1);
    }

    // Insert decrypted keys back into the config Value
    let mut config = config_json;
    config["decrypted_api_key"] = serde_json::Value::String(decrypted_api_key);
    config["decrypted_one_call_api_key"] = serde_json::Value::String(decrypted_one_call_api_key);
    config["is_first_run"] = serde_json::Value::Bool(is_first_run);

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title("METGen - Synthesized METAR Generator")
            .with_app_id("metgen"),
        follow_system_theme: true,
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };

    eframe::run_native(
        "METGen - Synthesized METAR Generator",
        options,
        Box::new(|cc| Box::new(MetGenApp::new(cc, config)))
    )
}
