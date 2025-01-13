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
use dialoguer::Input;
use serde_json::Value;

mod config;
mod input_handler;
mod metar_generator;
mod one_call_metar;
mod ui;

use config::{load_config, update_config, get_user_airports, save_user_airport, delete_user_airport};
use ui::{clear_screen, draw_banner, draw_menu_box, draw_section_header, draw_input_prompt, draw_output_box, draw_error_box, draw_success_box, read_single_char, select_airport_from_list};

fn main() -> std::io::Result<()> {
    // Clear the screen and reset terminal state at startup
    clear_screen()?;
    
    // Load config, including decrypted keys
    let (config_json, decrypted_api_key, decrypted_one_call_api_key) = load_config();

    if config_json.is_null() {
        draw_error_box("Failed to load configuration.")?;
        return Ok(());
    }

    // Insert decrypted keys back into the config Value so we can pass &config around
    let mut config = config_json;
    config["decrypted_api_key"] = Value::String(decrypted_api_key);
    config["decrypted_one_call_api_key"] = Value::String(decrypted_one_call_api_key);

    loop {
        clear_screen()?;
        draw_banner()?;
        
        draw_menu_box("Main Menu", &[
            "1. Generate METAR",
            "2. Manage Saved Airports",
            "3. Update Configuration",
            "4. Exit",
        ])?;

        draw_input_prompt("Enter your choice (1/2/3/4)")?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        let menu_choice = match read_single_char() {
            Ok(c) => match c {
                '1' => 1,
                '2' => 2,
                '3' => 3,
                '4' => 4,
                _ => {
                    draw_error_box("Invalid input. Please enter a valid number.")?;
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    continue;
                }
            },
            Err(_) => {
                draw_error_box("Invalid input. Please enter a valid number.")?;
                std::thread::sleep(std::time::Duration::from_secs(2));
                continue;
            }
        };

        match menu_choice {
            1 => generate_metar_menu(&config)?,
            2 => manage_saved_airports_menu(&config)?,
            3 => {
                if let Err(e) = update_config() {
                    draw_error_box(&format!("Failed to update configuration: {}", e))?;
                }
                let (new_config_json, new_api_key, new_one_call_api_key) = load_config();
                if !new_config_json.is_null() {
                    config = new_config_json;
                    config["decrypted_api_key"] = Value::String(new_api_key);
                    config["decrypted_one_call_api_key"] = Value::String(new_one_call_api_key);
                }
                continue;
            }
            4 => {
                draw_success_box("Thank you for using METGen! Goodbye.")?;
                return Ok(());
            }
            _ => {
                draw_error_box("Invalid input. Please enter a valid number.")?;
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }
    }
}

fn generate_metar_menu(config: &Value) -> std::io::Result<()> {
    loop {
        clear_screen()?;
        draw_banner()?;
        draw_section_header("METAR Generation Options")?;
        
        draw_menu_box("Select METAR Generation Approach", &[
            "1. Standard (uses metar_generator.rs + /data/2.5/weather)",
            "2. One Call (uses one_call_metar.rs + /data/3.0/onecall)",
        ])?;

        draw_input_prompt("Enter (1) or (2)")?;
        std::thread::sleep(std::time::Duration::from_millis(100)); // Give time for any buffered input to clear
        let approach_choice = match read_single_char() {
            Ok(c) => match c {
                '1' => 1,
                '2' => 2,
                _ => {
                    draw_error_box("Invalid choice. Please enter 1 or 2.")?;
                    continue;
                }
            },
            Err(_) => {
                draw_error_box("Invalid choice. Please enter 1 or 2.")?;
                continue;
            }
        };

        clear_screen()?;
        draw_banner()?;
        draw_menu_box("Input Type", &[
            "1. ICAO Code",
            "2. Latitude/Longitude",
            "3. Freeform Location",
        ])?;

        draw_input_prompt("Enter your choice (1/2/3)")?;
        std::thread::sleep(std::time::Duration::from_millis(100)); // Give time for any buffered input to clear
        let input_type_choice = match read_single_char() {
            Ok(c) => match c {
                '1' => 1,
                '2' => 2,
                '3' => 3,
                _ => {
                    draw_error_box("Invalid choice. Please enter 1, 2, or 3.")?;
                    continue;
                }
            },
            Err(_) => {
                draw_error_box("Invalid choice. Please enter 1, 2, or 3.")?;
                continue;
            }
        };

        match (approach_choice, input_type_choice) {
            // -- Standard approach (metar_generator) --
            (1, 1) => icao_workflow_standard(config)?,
            (1, 2) => latlon_workflow_standard(config)?,
            (1, 3) => freeform_workflow_standard(config)?,

            // -- One Call approach (one_call_metar) --
            (2, 1) => icao_workflow_onecall(config)?,
            (2, 2) => latlon_workflow_onecall(config)?,
            (2, 3) => freeform_workflow_onecall(config)?,

            _ => {
                draw_error_box("Invalid choice. Please enter 1, 2, or 3.")?;
                continue;
            }
        }

        draw_input_prompt("Do you want to try another input? (y/n)")?;
        std::thread::sleep(std::time::Duration::from_millis(100)); // Give time for any buffered input to clear
        let retry = read_single_char()?;
        if retry.to_ascii_lowercase() == 'y' {
            return Ok(());
        } else {
            clear_screen()?;
            draw_banner()?;
            draw_success_box("Thank you for using METGen! Goodbye.")?;
            process::exit(0);
        }
    }
}

fn manage_saved_airports_menu(config: &Value) -> std::io::Result<()> {
    loop {
        clear_screen()?;
        draw_banner()?;
        draw_section_header("Manage Saved Airports")?;
        
        let airports = get_user_airports();
        
        if airports.is_empty() {
            draw_output_box("No saved airports found. Airports will be saved here when you choose to save them after generating a METAR using lat/lon or freeform search.")?;
            draw_input_prompt("Press any key to return to main menu")?;
            read_single_char()?;
            return Ok(());
        }

        draw_menu_box("Options", &[
            "1. Generate METAR for saved airport",
            "2. Delete saved airport",
            "3. Return to main menu",
        ])?;

        draw_input_prompt("Enter your choice (1/2/3)")?;
        let choice = read_single_char()?;

        match choice {
            '1' => saved_airport_workflow(config)?,
            '2' => {
                if let Some(airport) = select_airport_from_list(&airports)? {
                    delete_user_airport(&airport.icao)?;
                    draw_success_box(&format!("Deleted airport {}", airport.icao))?;
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            },
            '3' => return Ok(()),
            _ => {
                draw_error_box("Invalid choice")?;
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}

fn saved_airport_workflow(config: &Value) -> std::io::Result<()> {
    let airports = get_user_airports();
    
    if let Some(airport) = select_airport_from_list(&airports)? {
        clear_screen()?;
        draw_banner()?;
        draw_section_header("Generate METAR for Saved Airport")?;
        
        draw_menu_box("Select API", &[
            "1. Standard (uses metar_generator.rs + /data/2.5/weather)",
            "2. One Call (uses one_call_metar.rs + /data/3.0/onecall)",
        ])?;

        draw_input_prompt("Enter (1) or (2)")?;
        let api_choice = read_single_char()?;

        match api_choice {
            '1' => {
                if let Some(metar) = metar_generator::generate_metar(
                    &airport.icao,
                    airport.latitude,
                    airport.longitude,
                    config["decrypted_api_key"].as_str().unwrap(),
                    config["units"].as_str().unwrap(),
                ) {
                    draw_success_box(&format!("Generated METAR:\n{}", metar))?;
                    // Wait for user acknowledgment before returning to main menu
                    draw_input_prompt("Press any key to return to main menu")?;
                    read_single_char()?;
                    return Ok(());
                } else {
                    draw_error_box("Failed to generate METAR.")?;
                }
            },
            '2' => {
                // Fetch weather data first
                if let Some(weather_data) = one_call_metar::fetch_weather_data(
                    airport.latitude,
                    airport.longitude,
                    config["decrypted_one_call_api_key"].as_str().unwrap(),
                ) {
                    // Parse the weather data
                    let parsed = one_call_metar::parse_weather_data(&weather_data);
                    
                    // Generate METAR
                    let metar = one_call_metar::generate_metar(
                        &airport.icao,
                        &parsed,
                        config["units"].as_str().unwrap(),
                    );
                    draw_success_box(&format!("Generated METAR:\n{}", metar))?;
                    // Wait for user acknowledgment before returning to main menu
                    draw_input_prompt("Press any key to return to main menu")?;
                    read_single_char()?;
                    return Ok(());
                } else {
                    draw_error_box("Failed to generate METAR.")?;
                }
            },
            _ => {
                draw_error_box("Invalid choice")?;
            }
        }
    }
    Ok(())
}

// -------------------------------------------
// 2) Standard Approach Workflows
//    (existing code that calls metar_generator)
// -------------------------------------------

fn icao_workflow_standard(config: &Value) -> std::io::Result<()> {
    clear_screen()?;
    draw_banner()?;
    draw_section_header("ICAO Input")?;
    draw_input_prompt("Enter ICAO code")?;
    let icao: String = Input::new()
        .interact_text()
        .unwrap();

    if let Some(existing_metar) = input_handler::poll_noaa_metar(&icao) {
        draw_output_box(&format!("METAR found for {}: {}", icao, existing_metar))?;
        
        draw_input_prompt("Do you want to use this METAR? (y/n)")?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        let use_existing = read_single_char()?;
        if use_existing.to_ascii_lowercase() == 'y' {
            draw_success_box(&format!("Using existing METAR:\n{}", existing_metar))?;
            return Ok(());
        }
    }

    if let Some((lat, lon)) = input_handler::resolve_icao_to_lat_lon(&icao) {
        if let Some(metar) = metar_generator::generate_metar(
            &icao,
            lat,
            lon,
            config["decrypted_api_key"].as_str().unwrap(),
            config["units"].as_str().unwrap(),
        ) {
            draw_success_box(&format!("Generated METAR:\n{}", metar))?;
        } else {
            draw_error_box("Failed to generate METAR.")?;
        }
    } else {
        draw_error_box(&format!("Could not resolve ICAO code: {}", icao))?;
    }
    Ok(())
}

fn latlon_workflow_standard(config: &Value) -> std::io::Result<()> {
    draw_section_header("Latitude/Longitude Input")?;
    
    draw_input_prompt("Enter latitude (e.g., 37.7749)")?;
    let lat: f64 = Input::new()
        .interact_text()
        .unwrap();
        
    draw_input_prompt("Enter longitude (e.g., -122.4194)")?;
    let lon: f64 = Input::new()
        .interact_text()
        .unwrap();

    if let Some((lat, lon)) = input_handler::validate_lat_lon(lat, lon) {
        draw_input_prompt("Enter ICAO code for the generated METAR")?;
        let icao: String = Input::new()
            .interact_text()
            .unwrap();

        if let Some(metar) = metar_generator::generate_metar(
            &icao,
            lat,
            lon,
            config["decrypted_api_key"].as_str().unwrap(),
            config["units"].as_str().unwrap(),
        ) {
            draw_success_box(&format!("Generated METAR:\n{}", metar))?;
            
            // Ask if user wants to save this airport
            draw_input_prompt("Would you like to save this airport for future use? (y/n)")?;
            let save_choice = read_single_char()?;
            if save_choice.to_ascii_lowercase() == 'y' {
                save_user_airport(icao, lat, lon)?;
                draw_success_box("Airport saved successfully!")?;
            }
        } else {
            draw_error_box("Failed to generate METAR.")?;
        }
    } else {
        draw_error_box("Invalid latitude/longitude entered. Please try again.")?;
    }
    Ok(())
}

fn freeform_workflow_standard(config: &Value) -> std::io::Result<()> {
    draw_section_header("Freeform Location Input")?;
    
    draw_input_prompt("Enter freeform location")?;
    let location: String = Input::new()
        .interact_text()
        .unwrap();

    if let Some((lat, lon)) =
        input_handler::resolve_freeform_input(&location, config["decrypted_api_key"].as_str().unwrap())
    {
        draw_output_box(&format!("Resolved {} to coordinates: ({}, {})", location, lat, lon))?;
        
        draw_input_prompt("Enter ICAO code for the generated METAR")?;
        let icao: String = Input::new()
            .interact_text()
            .unwrap();

        if let Some(metar) = metar_generator::generate_metar(
            &icao,
            lat,
            lon,
            config["decrypted_api_key"].as_str().unwrap(),
            config["units"].as_str().unwrap(),
        ) {
            draw_success_box(&format!("Generated METAR:\n{}", metar))?;
            
            // Ask if user wants to save this airport
            draw_input_prompt("Would you like to save this airport for future use? (y/n)")?;
            let save_choice = read_single_char()?;
            if save_choice.to_ascii_lowercase() == 'y' {
                save_user_airport(icao, lat, lon)?;
                draw_success_box("Airport saved successfully!")?;
            }
        } else {
            draw_error_box("Failed to generate METAR.")?;
        }
    } else {
        draw_error_box(&format!("Failed to resolve location: {}. Please try again.", location))?;
    }
    Ok(())
}

// -------------------------------------------
// 3) One Call Approach Workflows
//    (uses one_call_metar.rs + the One Call API key)
// -------------------------------------------

fn icao_workflow_onecall(config: &Value) -> std::io::Result<()> {
    draw_section_header("ICAO Input (One Call API)")?;
    
    draw_input_prompt("Enter ICAO code")?;
    let icao: String = Input::new()
        .interact_text()
        .unwrap();

    if let Some(existing_metar) = input_handler::poll_noaa_metar(&icao) {
        draw_output_box(&format!("METAR found for {}: {}", icao, existing_metar))?;
        
        draw_input_prompt("Do you want to use this METAR? (y/n)")?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        let use_existing = read_single_char()?;
        if use_existing.to_ascii_lowercase() == 'y' {
            draw_success_box(&format!("Using existing METAR:\n{}", existing_metar))?;
            return Ok(());
        }
    }

    if let Some((lat, lon)) = input_handler::resolve_icao_to_lat_lon(&icao) {
        if let Some(data) = one_call_metar::fetch_weather_data(
            lat,
            lon,
            config["decrypted_one_call_api_key"].as_str().unwrap(),
        ) {
            let parsed = one_call_metar::parse_weather_data(&data);
            let metar = one_call_metar::generate_metar(&icao, &parsed, config["units"].as_str().unwrap());
            draw_success_box(&format!("One Call METAR:\n{}", metar))?;
        } else {
            draw_error_box("\nFailed to fetch data from One Call API.\nNote: The One Call API requires a separate subscription from the standard OpenWeather API.\nPlease check your API key and subscription status.")?;
        }
    } else {
        draw_error_box(&format!("Could not resolve ICAO code: {}", icao))?;
    }
    Ok(())
}

fn latlon_workflow_onecall(config: &Value) -> std::io::Result<()> {
    draw_section_header("Latitude/Longitude Input")?;
    
    draw_input_prompt("Enter latitude (e.g., 37.7749)")?;
    let lat: f64 = Input::new()
        .interact_text()
        .unwrap();
        
    draw_input_prompt("Enter longitude (e.g., -122.4194)")?;
    let lon: f64 = Input::new()
        .interact_text()
        .unwrap();

    if let Some((lat, lon)) = input_handler::validate_lat_lon(lat, lon) {
        draw_input_prompt("Enter ICAO code for the generated METAR")?;
        let icao: String = Input::new()
            .interact_text()
            .unwrap();

        // Fetch weather data first
        if let Some(weather_data) = one_call_metar::fetch_weather_data(
            lat,
            lon,
            config["decrypted_one_call_api_key"].as_str().unwrap(),
        ) {
            // Parse the weather data
            let parsed = one_call_metar::parse_weather_data(&weather_data);
            
            // Generate METAR
            let metar = one_call_metar::generate_metar(
                &icao,
                &parsed,
                config["units"].as_str().unwrap(),
            );
            draw_success_box(&format!("Generated METAR:\n{}", metar))?;
            
            // Ask if user wants to save this airport
            draw_input_prompt("Would you like to save this airport for future use? (y/n)")?;
            let save_choice = read_single_char()?;
            if save_choice.to_ascii_lowercase() == 'y' {
                save_user_airport(icao, lat, lon)?;
                draw_success_box("Airport saved successfully!")?;
            }
        } else {
            draw_error_box("\nFailed to fetch data from One Call API.\nNote: The One Call API requires a separate subscription from the standard OpenWeather API.\nPlease check your API key and subscription status.")?;
        }
    } else {
        draw_error_box("Invalid latitude/longitude values.")?;
    }
    Ok(())
}

fn freeform_workflow_onecall(config: &Value) -> std::io::Result<()> {
    draw_section_header("Freeform Location Input")?;
    
    draw_input_prompt("Enter location (e.g., 'London, UK' or 'Tokyo Airport')")?;
    let location: String = Input::new()
        .interact_text()
        .unwrap();

    if let Some((lat, lon)) = input_handler::resolve_freeform_input(
        &location,
        config["decrypted_api_key"].as_str().unwrap(),
    ) {
        draw_input_prompt("Enter ICAO code for the generated METAR")?;
        let icao: String = Input::new()
            .interact_text()
            .unwrap();

        // Fetch weather data first
        if let Some(weather_data) = one_call_metar::fetch_weather_data(
            lat,
            lon,
            config["decrypted_one_call_api_key"].as_str().unwrap(),
        ) {
            // Parse the weather data
            let parsed = one_call_metar::parse_weather_data(&weather_data);
            
            // Generate METAR
            let metar = one_call_metar::generate_metar(
                &icao,
                &parsed,
                config["units"].as_str().unwrap(),
            );
            draw_success_box(&format!("Generated METAR:\n{}", metar))?;
            
            // Ask if user wants to save this airport
            draw_input_prompt("Would you like to save this airport for future use? (y/n)")?;
            let save_choice = read_single_char()?;
            if save_choice.to_ascii_lowercase() == 'y' {
                save_user_airport(icao, lat, lon)?;
                draw_success_box("Airport saved successfully!")?;
            }
        } else {
            draw_error_box("\nFailed to fetch data from One Call API.\nNote: The One Call API requires a separate subscription from the standard OpenWeather API.\nPlease check your API key and subscription status.")?;
        }
    } else {
        draw_error_box("Could not resolve location to coordinates.")?;
    }
    Ok(())
}
