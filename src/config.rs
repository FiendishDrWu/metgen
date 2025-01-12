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

use std::fs;
use std::path::Path;
use serde_json::{json, Value};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use dialoguer::Input;
use crate::ui::{draw_section_header, draw_input_prompt, draw_error_box, draw_success_box, read_single_char};

const CONFIG_FILE: &str = "config.json";

pub fn create_config() -> std::io::Result<()> {
    draw_section_header("METGen Configuration Setup")?;

    let api_key: String = loop {
        draw_input_prompt("Enter your OpenWeather API key")?;
        let input: String = Input::new()
            .interact_text()
            .unwrap();
        if !input.is_empty() {
            break input;
        }
        draw_error_box("OpenWeather API key is required.")?;
    };

    draw_input_prompt("Enter your One Call API key (or type 'same' to use the same API key, leave blank if not applicable)")?;
    let one_call_api_key: String = Input::new()
        .allow_empty(true)
        .interact_text()
        .unwrap();
    let one_call_api_key = if one_call_api_key.to_lowercase() == "same" {
        api_key.clone()
    } else {
        one_call_api_key
    };

    let units: String = loop {
        draw_input_prompt("Enter preferred units (m=metric, i=imperial)")?;
        std::thread::sleep(std::time::Duration::from_millis(100)); // Give time for any buffered input to clear
        match read_single_char() {
            Ok('m') | Ok('M') => {
                println!("m");  // Echo the input
                break "metric".to_string()
            },
            Ok('i') | Ok('I') => {
                println!("i");  // Echo the input
                break "imperial".to_string()
            },
            Ok(_) => {
                draw_error_box("Invalid units. Please enter 'm' for metric or 'i' for imperial.")?;
                continue;
            },
            Err(_) => continue  // Ignore errors and wait for valid input
        }
    };

    let config = json!({
        "api_key": STANDARD.encode(api_key.as_bytes()),
        "one_call_api_key": STANDARD.encode(one_call_api_key.as_bytes()),
        "units": units,
    });

    fs::write(CONFIG_FILE, config.to_string())?;
    draw_success_box("Configuration created successfully!")?;
    Ok(())
}

pub fn load_config() -> (Value, String, String) {
    if !Path::new(CONFIG_FILE).exists() {
        create_config().expect("Failed to create configuration");
    }

    let config_data = fs::read_to_string(CONFIG_FILE).expect("Failed to read configuration file");
    let config: Value = serde_json::from_str(&config_data).expect("Failed to parse configuration");

    let api_key = String::from_utf8(
        STANDARD
            .decode(config["api_key"].as_str().unwrap())
            .expect("Invalid base64 encoding")
    ).expect("Invalid UTF-8 in API key");

    let one_call_api_key = if let Some(encoded_key) = config["one_call_api_key"].as_str() {
        if !encoded_key.is_empty() {
            String::from_utf8(
                STANDARD
                    .decode(encoded_key)
                    .expect("Invalid base64 encoding")
            ).expect("Invalid UTF-8 in One Call API key")
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    (config, api_key, one_call_api_key)
}

pub fn update_config() -> std::io::Result<()> {
    draw_section_header("Update Configuration")?;
    
    let (mut config, api_key, one_call_api_key) = load_config();

    // OpenWeather API key
    draw_input_prompt(&format!("OpenWeather API key [{}]: ", api_key))?;
    let new_api_key: String = Input::new()
        .allow_empty(true)
        .interact_text()
        .unwrap();
    if !new_api_key.is_empty() {
        config["api_key"] = json!(STANDARD.encode(new_api_key.as_bytes()));
    }

    // One Call API key
    draw_input_prompt(&format!(
        "One Call API key (type 'same' to reuse OpenWeather API key, 'blank' to remove) [{}]: ",
        one_call_api_key
    ))?;
    let new_one_call_api_key: String = Input::new()
        .allow_empty(true)
        .interact_text()
        .unwrap();

    if !new_one_call_api_key.is_empty() {
        config["one_call_api_key"] = json!(match new_one_call_api_key.to_lowercase().as_str() {
            "same" => config["api_key"].as_str().unwrap_or("").to_string(),
            "blank" => STANDARD.encode("".as_bytes()),
            _ => STANDARD.encode(new_one_call_api_key.as_bytes())
        });
    }

    // Units
    draw_input_prompt(&format!("Preferred units (m=metric, i=imperial) [{}]: ", config["units"].as_str().unwrap_or("metric")))?;
    std::thread::sleep(std::time::Duration::from_millis(100)); // Give time for any buffered input to clear
    match read_single_char() {
        Ok('m') | Ok('M') => {
            println!("m");
            config["units"] = json!("metric");
        },
        Ok('i') | Ok('I') => {
            println!("i");
            config["units"] = json!("imperial");
        },
        _ => {}  // Keep existing value
    }

    fs::write(CONFIG_FILE, config.to_string())?;
    draw_success_box("Configuration updated successfully!")?;
    Ok(())
}