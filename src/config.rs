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
use std::io::{self, Write};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use base64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAirport {
    pub icao: String,
    pub latitude: f64,
    pub longitude: f64,
}

const CONFIG_FILE: &str = "config.json";
const AIRPORTS_FILE: &str = "airports.json";

pub fn load_config() -> (Value, String, String) {
    match fs::read_to_string(CONFIG_FILE) {
        Ok(contents) => {
            match serde_json::from_str(&contents) {
                Ok(json) => {
                    let config: Value = json;
                    let api_key = config["api_key"].as_str().unwrap_or("").to_string();
                    let one_call_api_key = config["one_call_api_key"].as_str().unwrap_or("").to_string();
                    
                    // Decrypt API keys
                    let decrypted_api_key = decrypt_key(&api_key);
                    let decrypted_one_call_api_key = decrypt_key(&one_call_api_key);
                    
                    (config, decrypted_api_key, decrypted_one_call_api_key)
                }
                Err(_) => (Value::Null, String::new(), String::new())
            }
        }
        Err(_) => (Value::Null, String::new(), String::new())
    }
}

pub fn save_config(api_key: &str, one_call_api_key: &str, units: &str) -> io::Result<()> {
    let encrypted_api_key = encrypt_key(api_key);
    let encrypted_one_call_api_key = encrypt_key(one_call_api_key);
    
    let config = serde_json::json!({
        "api_key": encrypted_api_key,
        "one_call_api_key": encrypted_one_call_api_key,
        "units": units
    });
    
    let config_str = serde_json::to_string_pretty(&config)?;
    fs::write(CONFIG_FILE, config_str)?;
    Ok(())
}

pub fn get_user_airports() -> Vec<UserAirport> {
    match fs::read_to_string(AIRPORTS_FILE) {
        Ok(contents) => {
            serde_json::from_str(&contents).unwrap_or_else(|_| Vec::new())
        }
        Err(_) => Vec::new()
    }
}

pub fn save_user_airport(icao: String, lat: f64, lon: f64) -> io::Result<()> {
    let mut airports = get_user_airports();
    
    // Check if airport already exists
    if !airports.iter().any(|a| a.icao == icao) {
        airports.push(UserAirport {
            icao,
            latitude: lat,
            longitude: lon,
        });
        
        let airports_json = serde_json::to_string_pretty(&airports)?;
        fs::write(AIRPORTS_FILE, airports_json)?;
    }
    
    Ok(())
}

pub fn delete_user_airport(icao: &str) -> io::Result<()> {
    let mut airports = get_user_airports();
    airports.retain(|a| a.icao != icao);
    
    let airports_json = serde_json::to_string_pretty(&airports)?;
    fs::write(AIRPORTS_FILE, airports_json)?;
    Ok(())
}

fn encrypt_key(key: &str) -> String {
    base64::encode(key)
}

fn decrypt_key(encrypted: &str) -> String {
    base64::decode(encrypted)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default()
}