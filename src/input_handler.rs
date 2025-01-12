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

use reqwest::{blocking::Client, StatusCode};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

const NOAA_METAR_URL: &str = "https://aviationweather.gov/api/data/metar";
const NOAA_AIRPORT_URL: &str = "https://aviationweather.gov/api/data/airport";
const GEOCODING_URL: &str = "http://api.openweathermap.org/geo/1.0/direct";
const ONE_CALL_URL: &str = "https://api.openweathermap.org/data/3.0/onecall";

// Bundle the airports.csv file into the binary
const BUNDLED_AIRPORTS_CSV: &str = include_str!("../airports.csv");

fn get_airports_data() -> Result<String, String> {
    // Try to read from the external file first
    let airports_csv_path = get_resource_path("airports.csv");
    match fs::read_to_string(&airports_csv_path) {
        Ok(data) => Ok(data),
        Err(_) => Ok(BUNDLED_AIRPORTS_CSV.to_string()) // Fall back to bundled data
    }
}

fn get_resource_path(filename: &str) -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push(filename);
    path
}

pub fn poll_noaa_metar(icao: &str) -> Option<String> {
    let params = [
        ("ids", icao),
        ("format", "json"),
        ("taf", "false"),
    ];

    let client = Client::new();
    match client.get(NOAA_METAR_URL).query(&params).send() {
        Ok(response) if response.status() == StatusCode::OK => {
            match response.json::<Value>() {
                Ok(metar_data) => {
                    // NOAA returns an array, so index into [0]
                    if let Some(array) = metar_data.as_array() {
                        if let Some(first_record) = array.first() {
                            if let Some(raw_metar) = first_record["rawOb"].as_str() {
                                return Some(raw_metar.to_string());
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Failed to parse METAR data for {}: {}", icao, e),
            }
        }
        Err(e) => eprintln!("Error querying NOAA METAR API for {}: {}", icao, e),
        _ => eprintln!("Unexpected response when querying NOAA METAR API."),
    }
    None
}

pub fn resolve_icao_to_lat_lon(icao: &str) -> Option<(f64, f64)> {
    let params = [("ids", icao), ("format", "json")];

    let client = Client::new();
    match client.get(NOAA_AIRPORT_URL).query(&params).send() {
        Ok(response) => {
            match response.status() {
                StatusCode::NOT_FOUND => {
                    eprintln!("Airport not found in NOAA database.");
                    // Fall through to local database
                }
                StatusCode::BAD_REQUEST => {
                    eprintln!("Invalid ICAO code format.");
                    // Fall through to local database
                }
                _ if !response.status().is_success() => {
                    eprintln!("Unexpected NOAA API error. Falling back to local database.");
                    // Fall through to local database
                }
                StatusCode::OK => {
                    if let Ok(airport_data) = response.json::<Value>() {
                        if let Some(arr) = airport_data.as_array() {
                            if let Some(first_record) = arr.first() {
                                if let (Some(lat), Some(lon)) = (
                                    first_record["lat"].as_f64(),
                                    first_record["lon"].as_f64(),
                                ) {
                                    return Some((lat, lon));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Err(e) => {
            eprintln!("Error querying NOAA Airport API: {}", e);
            // Fall through to local database
        }
    }

    // Fallback to local database
    let csv_data = match get_airports_data() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading airports database: {}", e);
            return None;
        }
    };

    // Skip header row and license text, parse subsequent rows
    let mut found_header = false;
    for line in csv_data.lines() {
        if line.starts_with("//") || line.trim().is_empty() {
            continue;  // Skip license text and empty lines
        }
        if !found_header {
            found_header = true;
            continue;  // Skip the header row (ICAO,Latitude,Longitude)
        }

        let fields: Vec<&str> = line.split(',').collect();
        // Ensure we have enough columns and the ICAO matches
        if fields.len() >= 3 && fields[0].eq_ignore_ascii_case(icao) {
            // fields[1] = latitude, fields[2] = longitude
            if let (Ok(lat), Ok(lon)) = (fields[1].parse::<f64>(), fields[2].parse::<f64>()) {
                return Some((lat, lon));
            }
        }
    }
    None
}

pub fn resolve_freeform_input(location: &str, api_key: &str) -> Option<(f64, f64)> {
    let params = [
        ("q", location.to_string()),
        ("appid", api_key.to_string()),
        ("limit", "1".to_string()),
    ];

    let client = Client::new();
    match client.get(GEOCODING_URL).query(&params).send() {
        Ok(response) => {
            match response.status() {
                StatusCode::UNAUTHORIZED => {
                    // Don't output anything here - let the main workflow handle the error message
                    return None;
                }
                StatusCode::NOT_FOUND => {
                    eprintln!("Location not found. Please check your input.");
                    return None;
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    eprintln!("API rate limit exceeded. Please try again later.");
                    return None;
                }
                StatusCode::BAD_REQUEST => {
                    eprintln!("Invalid location format. Please check your input.");
                    return None;
                }
                _ if !response.status().is_success() => {
                    eprintln!("Unexpected API error. Please try again later.");
                    return None;
                }
                _ => {}
            }
            
            if let Ok(geocode_data) = response.json::<Vec<Value>>() {
                if !geocode_data.is_empty() {
                    let lat = geocode_data[0]["lat"].as_f64();
                    let lon = geocode_data[0]["lon"].as_f64();

                    if let (Some(lat), Some(lon)) = (lat, lon) {
                        return Some((lat, lon));
                    }
                }
            }
            None
        }
        Err(e) => {
            eprintln!("Error resolving location: {}", e);
            None
        }
    }
}

pub fn fetch_weather_data(lat: f64, lon: f64, api_key: &str) -> Option<Value> {
    if api_key.is_empty() {
        eprintln!("API key is missing or invalid.");
        return None;
    }

    let params = [
        ("lat", lat.to_string()),
        ("lon", lon.to_string()),
        ("appid", api_key.to_string()),
        ("units", "metric".to_string()),
    ];

    let client = Client::new();
    match client.get("https://api.openweathermap.org/data/2.5/weather").query(&params).send() {
        Ok(response) => {
            match response.status() {
                StatusCode::UNAUTHORIZED => {
                    return None;
                }
                StatusCode::NOT_FOUND => {
                    eprintln!("Location not found or invalid coordinates.");
                    return None;
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    eprintln!("API rate limit exceeded. Please try again later.");
                    return None;
                }
                StatusCode::BAD_REQUEST => {
                    eprintln!("Invalid request parameters. Please check your input.");
                    return None;
                }
                _ if !response.status().is_success() => {
                    eprintln!("Unexpected API error. Please try again later.");
                    return None;
                }
                _ => {}
            }
            
            match response.json::<Value>() {
                Ok(data) => {
                    // Commented out: Optional feature to save weather data for testing/verification
                    // Allows comparing the synthesized METAR against raw weather data
                    // if let Ok(json_string) = serde_json::to_string_pretty(&data) {
                    //     let _ = fs::write("weather.json", json_string);
                    // }
                    Some(data)
                }
                Err(e) => {
                    eprintln!("Error parsing weather data: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching weather data: {}", e);
            None
        }
    }
}

pub fn validate_lat_lon(lat: f64, lon: f64) -> Option<(f64, f64)> {
    if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon) {
        Some((lat, lon))
    } else {
        eprintln!("Latitude must be between -90 and 90, and longitude must be between -180 and 180.");
        None
    }
}

pub fn fetch_one_call_weather_data(lat: f64, lon: f64, api_key: &str) -> Option<Value> {
    if api_key.is_empty() {
        eprintln!("One Call API key is missing or invalid.");
        return None;
    }

    let params = [
        ("lat", lat.to_string()),
        ("lon", lon.to_string()),
        ("appid", api_key.to_string()),
        ("exclude", "minutely".to_string()),
        ("units", "metric".to_string()),
    ];

    let client = Client::new();
    match client.get(ONE_CALL_URL).query(&params).send() {
        Ok(response) => {
            match response.status() {
                StatusCode::UNAUTHORIZED => {
                    return None;
                }
                StatusCode::NOT_FOUND => {
                    eprintln!("Location not found or invalid coordinates.");
                    return None;
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    eprintln!("API rate limit exceeded. Please try again later.");
                    return None;
                }
                StatusCode::BAD_REQUEST => {
                    eprintln!("Invalid request parameters. Please check your input.");
                    return None;
                }
                _ if !response.status().is_success() => {
                    eprintln!("Unexpected API error. Please try again later.");
                    return None;
                }
                _ => {}
            }
            
            match response.json::<Value>() {
                Ok(data) => {
                    // Commented out: Optional feature to save weather data for testing/verification
                    // Allows comparing the synthesized METAR against raw weather data
                    // if let Ok(json_string) = serde_json::to_string_pretty(&data) {
                    //     let _ = fs::write("weather.json", json_string);
                    // }
                    Some(data)
                }
                Err(e) => {
                    eprintln!("Error parsing weather data: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching weather data: {}", e);
            None
        }
    }
}
