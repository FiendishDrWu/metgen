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

use serde_json::Value;
use std::collections::HashMap;
use chrono::Utc;
use crate::input_handler::fetch_weather_data;

pub fn generate_metar(icao: &str, lat: f64, lon: f64, api_key: &str, units: &str) -> Option<String> {
    // Fetch weather data
    let weather_data = fetch_weather_data(lat, lon, api_key)?;
    let parsed_data = parse_weather_data(&weather_data)?;

    // Format METAR components
    let report_time = Utc::now().format("%d%H%MZ").to_string();
    let wind_part = format_wind(
        parsed_data.get("wind_direction"),
        parsed_data.get("wind_speed"),
        parsed_data.get("wind_gust"),
    );
    let visibility_part = format_visibility(
        parsed_data.get("visibility"),
        units,
        parsed_data.get("weather_conditions"),
    );
    let cloud_part = format_clouds(parsed_data.get("cloud_coverage"));
    let temp_dew_part = format_temp_dew(
        parsed_data.get("temperature"),
        parsed_data.get("humidity"),
    );
    let pressure_part = format_pressure(parsed_data.get("pressure"), units);

    let mut metar = format!(
        "{} {} AUTO {} {} {} {} {}",
        icao.to_uppercase(), report_time, wind_part, visibility_part, cloud_part, temp_dew_part, pressure_part
    );

    if let Some(weather_conditions) = parsed_data.get("weather_conditions") {
        let formatted_conditions = format_weather_conditions(weather_conditions);
        if !formatted_conditions.is_empty() {
            metar.push_str(&format!(" {}", formatted_conditions));
        }
    }

    Some(metar)
}

fn parse_weather_data(data: &Value) -> Option<HashMap<String, String>> {
    let mut weather_data = HashMap::new();

    if let Some(temp) = data["main"]["temp"].as_f64() {
        weather_data.insert("temperature".to_string(), temp.to_string());
    }
    if let Some(pressure) = data["main"]["pressure"].as_f64() {
        weather_data.insert("pressure".to_string(), pressure.to_string());
    }
    if let Some(humidity) = data["main"]["humidity"].as_f64() {
        weather_data.insert("humidity".to_string(), humidity.to_string());
    }
    if let Some(wind_speed) = data["wind"]["speed"].as_f64() {
        weather_data.insert("wind_speed".to_string(), wind_speed.to_string());
    }
    if let Some(wind_direction) = data["wind"]["deg"].as_f64() {
        weather_data.insert("wind_direction".to_string(), wind_direction.to_string());
    }
    if let Some(wind_gust) = data["wind"]["gust"].as_f64() {
        weather_data.insert("wind_gust".to_string(), wind_gust.to_string());
    }
    if let Some(visibility) = data["visibility"].as_f64() {
        weather_data.insert("visibility".to_string(), visibility.to_string());
    }
    if let Some(cloud_coverage) = data["clouds"]["all"].as_f64() {
        weather_data.insert("cloud_coverage".to_string(), cloud_coverage.to_string());
    }
    if let Some(weather_conditions) = data["weather"].as_array() {
        let conditions = weather_conditions
            .iter()
            .map(|cond| cond["id"].to_string())
            .collect::<Vec<String>>()
            .join(", ");
        weather_data.insert("weather_conditions".to_string(), conditions);
    }

    Some(weather_data)
}

fn format_wind(direction: Option<&String>, speed: Option<&String>, gust: Option<&String>) -> String {
    let dir = direction.and_then(|d| d.parse::<i32>().ok()).unwrap_or(-1);
    let spd = speed.and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let gst = gust.and_then(|g| g.parse::<f64>().ok()).unwrap_or(0.0);

    if dir < 0 {
        "VRB00KT".to_string()
    } else {
        format!(
            "{:03}{:02}{}KT",
            dir,
            (spd * 1.94384).round() as i32,
            if gst > 0.0 {
                format!("G{:02}", (gst * 1.94384).round() as i32)
            } else {
                "".to_string()
            }
        )
    }
}

fn format_visibility(
    visibility: Option<&String>,
    units: &str,
    weather_conditions: Option<&String>,
) -> String {
    if let Some(vis) = visibility.and_then(|v| v.parse::<f64>().ok()) {
        if units == "imperial" {
            let visibility_sm = vis / 1609.344;
            let reducing_conditions = weather_conditions.map_or(false, |conditions| {
                conditions.split(", ").any(|condition| {
                    condition.parse::<i32>().ok().map_or(false, |id| {
                        (200..800).contains(&id)
                    })
                })
            });

            if (vis - 10000.0).abs() < f64::EPSILON && !reducing_conditions {
                return "10SM".to_string();
            }

            // Below 1 mile, show fraction
            if visibility_sm < 1.0 {
                let fraction = (visibility_sm * 4.0).round() / 4.0;
                let numerator = (fraction * 4.0).round() as i32;
                let denominator = 4;
                let gcd = crate::one_call_metar::gcd(numerator, denominator);
                let reduced_num = numerator / gcd;
                let reduced_den = denominator / gcd;

                if reduced_den == 1 {
                    format!("{}SM", reduced_num)
                } else {
                    format!("{}/{}SM", reduced_num, reduced_den)
                }
            } else {
                // Handle visibility of 1 mile or more, including fractional miles
                let whole = visibility_sm.floor() as i32;
                let fraction = ((visibility_sm - whole as f64) * 4.0).round() / 4.0;

                if fraction == 0.0 {
                    format!("{}SM", whole)
                } else {
                    let numerator = (fraction * 4.0).round() as i32;
                    let denominator = 4;
                    let gcd = crate::one_call_metar::gcd(numerator, denominator);
                    let reduced_num = numerator / gcd;
                    let reduced_den = denominator / gcd;

                    if reduced_den == 1 {
                        format!("{}SM", whole + reduced_num)
                    } else {
                        format!("{} {}/{}SM", whole, reduced_num, reduced_den)
                    }
                }
            }
        } else {
            // Metric units
            let rounded_vis = ((vis / 100.0).round() * 100.0) as i32;
            if rounded_vis == 10000 {
                "9999".to_string()
            } else {
                format!("{:04}", rounded_vis)
            }
        }
    } else {
        "////".to_string()
    }
}

fn format_clouds(cloud_coverage: Option<&String>) -> String {
    match cloud_coverage.and_then(|c| c.parse::<i32>().ok()) {
        Some(0) => "CLR".to_string(),
        Some(c) if c <= 25 => "FEW".to_string(),
        Some(c) if c <= 50 => "SCT".to_string(),
        Some(c) if c <= 87 => "BKN".to_string(),
        Some(c) if c <= 100 => "OVC".to_string(),
        _ => "CLR".to_string(),
    }
}

fn format_temp_dew(temp: Option<&String>, humidity: Option<&String>) -> String {
    let temp = temp.and_then(|t| t.parse::<f64>().ok());
    let humidity = humidity.and_then(|h| h.parse::<f64>().ok());

    if let (Some(temp), Some(humidity)) = (temp, humidity) {
        let dew_point = temp - ((100.0 - humidity) / 5.0);
        let temp_str = if temp < 0.0 {
            format!("M{:02}", temp.abs().round() as i32)
        } else {
            format!("{:02}", temp.round() as i32)
        };
        let dew_str = if dew_point < 0.0 {
            format!("M{:02}", dew_point.abs().round() as i32)
        } else {
            format!("{:02}", dew_point.round() as i32)
        };
        format!("{}/{}", temp_str, dew_str)
    } else {
        "/// ///".to_string()
    }
}

fn format_pressure(pressure: Option<&String>, units: &str) -> String {
    if let Some(p) = pressure.and_then(|p| p.parse::<f64>().ok()) {
        if units == "imperial" {
            format!("A{:04}", (p * 0.02953 * 100.0).round() as i32)
        } else {
            format!("Q{:04}", p.round() as i32)
        }
    } else {
        "Q////".to_string()
    }
}

fn format_weather_conditions(weather_conditions: &str) -> String {
    let weather_map = vec![
        (200, "TSRA"), (201, "TSRA"), (202, "+TSRA"),
        (210, "TS"), (211, "TS"), (212, "+TS"),
        (221, "TS"), (230, "TSRA"), (231, "TSRA"), (232, "+TSRA"),
        (300, "-DZ"), (301, "DZ"), (302, "+DZ"), (310, "-DZRA"),
        (311, "DZRA"), (312, "+DZRA"), (313, "SHRA"), (314, "+SHRA"),
        (321, "SHRA"), (500, "-RA"), (501, "RA"), (502, "+RA"),
        (503, "+RA"), (504, "+RA"), (511, "FZRA"), (520, "-SHRA"),
        (521, "SHRA"), (522, "+SHRA"), (531, "SHRA"), (600, "-SN"),
        (601, "SN"), (602, "+SN"), (611, "SLT"), (612, "-SHSL"),
        (613, "SHSL"), (615, "-RASN"), (616, "RASN"), (620, "-SHSN"),
        (621, "SHSN"), (622, "+SHSN"), (701, "BR"), (711, "FU"),
        (721, "HZ"), (731, "DU"), (741, "FG"), (751, "SA"),
        (761, "DU"), (762, "VA"), (771, "SQ"), (781, "+FC"),
        (800, ""), (801, "FEW"), (802, "SCT"), (803, "BKN"), (804, "OVC"),
    ];

    weather_conditions
        .split(", ")
        .filter_map(|id| id.parse::<i32>().ok())
        .filter(|&id| id < 800)
        .filter_map(|id| weather_map.iter().find(|&&(code, _)| code == id))
        .map(|&(_, abbreviation)| abbreviation)
        .collect::<Vec<&str>>()
        .join(" ")
}
