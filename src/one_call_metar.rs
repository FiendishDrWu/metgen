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
use chrono::offset::TimeZone;
use crate::input_handler;

pub fn fetch_weather_data(lat: f64, lon: f64, api_key: &str) -> Option<Value> {
    input_handler::fetch_one_call_weather_data(lat, lon, api_key)
}

pub fn parse_weather_data(data: &Value) -> HashMap<String, String> {
    let mut weather_data = HashMap::new();
    let current = data.get("current").unwrap_or(&Value::Null);

    if let Some(temp) = current["temp"].as_f64() {
        weather_data.insert("temperature".to_string(), temp.to_string());
    }
    if let Some(dew_point) = current["dew_point"].as_f64() {
        weather_data.insert("dew_point".to_string(), dew_point.to_string());
    }
    if let Some(pressure) = current["pressure"].as_f64() {
        weather_data.insert("pressure".to_string(), pressure.to_string());
    }
    if let Some(humidity) = current["humidity"].as_f64() {
        weather_data.insert("humidity".to_string(), humidity.to_string());
    }
    if let Some(wind_speed) = current["wind_speed"].as_f64() {
        weather_data.insert("wind_speed".to_string(), wind_speed.to_string());
    }
    if let Some(wind_direction) = current["wind_deg"].as_f64() {
        weather_data.insert("wind_direction".to_string(), wind_direction.to_string());
    }
    if let Some(wind_gust) = current["wind_gust"].as_f64() {
        weather_data.insert("wind_gust".to_string(), wind_gust.to_string());
    }
    if let Some(visibility) = current["visibility"].as_f64() {
        weather_data.insert("visibility".to_string(), visibility.to_string());
    }
    if let Some(cloud_coverage) = current["clouds"].as_f64() {
        weather_data.insert("cloud_coverage".to_string(), cloud_coverage.to_string());
    }
    if let Some(weather_conditions) = current["weather"].as_array() {
        let conditions = weather_conditions
            .iter()
            .map(|cond| cond["id"].to_string())
            .collect::<Vec<String>>()
            .join(", ");
        weather_data.insert("weather_conditions".to_string(), conditions);
    }

    // Alerts (if any)
    if let Some(alerts) = data.get("alerts").and_then(|v| v.as_array()) {
        let alert_text = alerts
            .iter()
            .map(|alert| alert["description"].as_str().unwrap_or("Unknown").to_string())
            .collect::<Vec<String>>()
            .join(", ");
        weather_data.insert("alerts".to_string(), alert_text);
    }

    // Hourly forecast (storing first two hours)
    if let Some(hourly) = data.get("hourly").and_then(|v| v.as_array()) {
        let mut forecast_entries = Vec::new();
        
        for hour in hourly.iter().take(2) {
            let entry = vec![
                hour.get("dt").and_then(|v| v.as_i64()).unwrap_or(0).to_string(),
                hour.get("temp").and_then(|v| v.as_f64()).unwrap_or(0.0).to_string(),
                hour.get("dew_point").and_then(|v| v.as_f64()).unwrap_or(0.0).to_string(),
                hour.get("pressure").and_then(|v| v.as_f64()).unwrap_or(0.0).to_string(),
                hour.get("wind_speed").and_then(|v| v.as_f64()).unwrap_or(0.0).to_string(),
                hour.get("wind_deg").and_then(|v| v.as_f64()).unwrap_or(0.0).to_string(),
                hour.get("wind_gust").and_then(|v| v.as_f64()).unwrap_or(0.0).to_string(),
                hour.get("visibility").and_then(|v| v.as_f64()).unwrap_or(0.0).to_string(),
                hour.get("weather")
                    .and_then(|w| w.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|cond| cond["id"].as_i64())
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(","))
                    .unwrap_or_default()
            ];
            
            forecast_entries.push(entry.join("|"));
        }
        
        // Join hours with semicolon separator
        weather_data.insert("forecast".to_string(), forecast_entries.join(";"));
    }

    weather_data
}

pub fn generate_metar(icao: &str, weather_data: &HashMap<String, String>, units: &str) -> String {
    let dt = Utc::now().format("%d%H%MZ").to_string();

    // Format each METAR component
    let wind = format_wind(
        weather_data.get("wind_direction"),
        weather_data.get("wind_speed"),
        weather_data.get("wind_gust"),
    );

    let visibility = format_visibility(
        weather_data.get("visibility"),
        units,
        weather_data.get("weather_conditions"),
    );

    let clouds = format_cloud_coverage(weather_data.get("cloud_coverage"));

    // Temperature / Dew
    let temperature = weather_data.get("temperature").and_then(|t| t.parse::<f64>().ok());
    let dew_point = weather_data.get("dew_point").and_then(|d| d.parse::<f64>().ok());
    let temp_dew = if let (Some(temp), Some(dew)) = (temperature, dew_point) {
        let temp_str = if temp < 0.0 {
            format!("M{:02}", temp.abs().round() as i32)
        } else {
            format!("{:02}", temp.round() as i32)
        };
        let dew_str = if dew < 0.0 {
            format!("M{:02}", dew.abs().round() as i32)
        } else {
            format!("{:02}", dew.round() as i32)
        };
        format!("{}/{}", temp_str, dew_str)
    } else {
        "/// ///".to_string()
    };

    let pressure = format_pressure(weather_data.get("pressure"), units);

    // Weather phenomena (excluding 8xx codes: clouds/CLR/etc.)
    let weather = format_weather_conditions(weather_data.get("weather_conditions"));

    // Construct the base METAR string
    let mut metar = format!(
        "{} {} AUTO {} {} {} {} {}",
        icao.to_uppercase(), dt, wind, visibility, clouds, temp_dew, pressure
    );

    // If there’s significant weather, append it
    if !weather.is_empty() {
        metar.push_str(&format!(" {}", weather));
    }

    // Trend section (based on “forecast” data)
    let trend = generate_trend_section(weather_data.get("forecast"), units);
    if !trend.is_empty() {
        metar.push_str(&format!(" {}", trend));
    }

    metar
}

/* ---------------------------------------------------------------------------
   The functions below closely mirror the logic in metar_generator.rs,
   with only minor changes to preserve your existing structure.

   Key Fixes:
   1. `format_wind`: Gust is placed properly before the “KT” (instead of "KTG20").
   2. `format_visibility`: Metric block checks for 10+ km (9999) unless
      there are “reducing conditions” (like RA/TS/etc.).
   3. `format_weather_conditions`: Now excludes all IDs >= 800 (cloud coverage).
 --------------------------------------------------------------------------- */

fn format_wind(direction: Option<&String>, speed: Option<&String>, gust: Option<&String>) -> String {
    let dir = direction.and_then(|d| d.parse::<i32>().ok()).unwrap_or(-1);
    let spd = speed.and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let gst = gust.and_then(|g| g.parse::<f64>().ok()).unwrap_or(0.0);

    // Convert m/s to knots
    let spd_knots = (spd * 1.94384).round() as i32;
    let gst_knots = (gst * 1.94384).round() as i32;

    // If direction is unknown, default VRB
    if dir < 0 {
        "VRB00KT".to_string()
    } else if gst_knots > 0 {
        format!("{:03}{:02}G{:02}KT", dir, spd_knots, gst_knots)
    } else {
        format!("{:03}{:02}KT", dir, spd_knots)
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
                let gcd_val = gcd(numerator, denominator);
                let reduced_num = numerator / gcd_val;
                let reduced_den = denominator / gcd_val;

                if reduced_den == 1 {
                    format!("{}SM", reduced_num)
                } else {
                    format!("{}/{}SM", reduced_num, reduced_den)
                }
            } else {
                // 1 mile or more
                let whole = visibility_sm.floor() as i32;
                let fraction = ((visibility_sm - whole as f64) * 4.0).round() / 4.0;
                if fraction == 0.0 {
                    format!("{}SM", whole)
                } else {
                    let numerator = (fraction * 4.0).round() as i32;
                    let denominator = 4;
                    let gcd_val = gcd(numerator, denominator);
                    let num = numerator / gcd_val;
                    let den = denominator / gcd_val;

                    if den == 1 {
                        format!("{}SM", whole + num)
                    } else {
                        format!("{} {}/{}SM", whole, num, den)
                    }
                }
            }
        } else {
            // Metric
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

fn format_pressure(pressure: Option<&String>, units: &str) -> String {
    if let Some(p) = pressure.and_then(|p| p.parse::<f64>().ok()) {
        if units == "imperial" {
            // Convert hPa to inHg (approx. p * 0.02953), then format "A2992"
            format!("A{:04}", (p * 0.02953 * 100.0).round() as i32)
        } else {
            // QNH in hPa, e.g. "Q1013"
            format!("Q{:04}", p.round() as i32)
        }
    } else {
        "Q////".to_string()
    }
}

fn format_weather_conditions(weather_conditions: Option<&String>) -> String {
    // This weather_map is unchanged, but we’ll filter out any codes >= 800
    // so that we don’t include cloud coverage in the METAR phenomena line.
    let weather_map = vec![
        (200, "TSRA"), (201, "TSRA"), (202, "+TSRA"),
        (210, "TS"),   (211, "TS"),   (212, "+TS"),
        (221, "TS"),   (230, "TSRA"), (231, "TSRA"), (232, "+TSRA"),
        (300, "-DZ"),  (301, "DZ"),   (302, "+DZ"),  (310, "-DZRA"),
        (311, "DZRA"), (312, "+DZRA"),(313, "SHRA"), (314, "+SHRA"),
        (321, "SHRA"), (500, "-RA"),  (501, "RA"),   (502, "+RA"),
        (503, "+RA"),  (504, "+RA"),  (511, "FZRA"), (520, "-SHRA"),
        (521, "SHRA"), (522, "+SHRA"),(531, "SHRA"), (600, "-SN"),
        (601, "SN"),   (602, "+SN"),  (611, "SLT"),  (612, "-SHSL"),
        (613, "SHSL"), (615, "-RASN"),(616, "RASN"), (620, "-SHSN"),
        (621, "SHSN"), (622, "+SHSN"),(701, "BR"),   (711, "FU"),
        (721, "HZ"),   (731, "DU"),   (741, "FG"),   (751, "SA"),
        (761, "DU"),   (762, "VA"),   (771, "SQ"),   (781, "+FC"),

        // We still define 8xx codes here if needed for reference,
        // but we won't display them in the final METAR phenomena line.
        (800, "CLR"),  (801, "FEW"),  (802, "SCT"),  (803, "BKN"), (804, "OVC"),
    ];

    if let Some(cond_str) = weather_conditions {
        cond_str
            .split(", ")
            .filter_map(|id_str| id_str.parse::<i32>().ok())
            // Filter out codes >= 800 so we don’t duplicate cloud coverage
            .filter(|&id| id < 800)
            .filter_map(|id| weather_map.iter().find(|&&(code, _)| code == id))
            .map(|&(_, abbreviation)| abbreviation)
            .collect::<Vec<&str>>()
            .join(" ")
    } else {
        "".to_string()
    }
}

fn format_cloud_coverage(cloud_coverage: Option<&String>) -> String {
    match cloud_coverage.and_then(|c| c.parse::<i32>().ok()) {
        Some(0) => "CLR".to_string(),
        Some(c) if c <= 25 => "FEW".to_string(),
        Some(c) if c <= 50 => "SCT".to_string(),
        Some(c) if c <= 87 => "BKN".to_string(),
        Some(c) if c <= 100 => "OVC".to_string(),
        _ => "CLR".to_string(),
    }
}

fn generate_trend_section(forecast_data: Option<&String>, units: &str) -> String {
    let mut trends = String::new();

    if let Some(forecast) = forecast_data {
        // Split into hours
        for hour_data in forecast.split(';') {
            let fields: Vec<&str> = hour_data.split('|').collect();
            if fields.len() != 9 { continue; } // Skip if format doesn't match

            // Parse fields (dt|temp|dew|pressure|wind_speed|wind_deg|wind_gust|visibility|weather)
            let dt = fields[0].parse::<i64>().unwrap_or(0);
            let trend_time = match Utc.timestamp_opt(dt, 0) {
                chrono::LocalResult::Single(datetime) => datetime.format("%H%MZ").to_string(),
                _ => continue,
            };

            // Format wind
            let wind = format_wind(
                Some(&fields[5].to_string()), // wind_deg
                Some(&fields[4].to_string()), // wind_speed
                Some(&fields[6].to_string()), // wind_gust
            );

            // Format visibility
            let visibility = format_visibility(
                Some(&fields[7].to_string()),
                units,
                Some(&fields[8].to_string()), // weather conditions
            );

            // Weather string
            let weather_str = format_weather_conditions(Some(&fields[8].to_string()));

            // Pressure
            let pressure = format_pressure(Some(&fields[3].to_string()), units);

            // Temperature / Dew
            let temp = fields[1].parse::<f64>().ok();
            let dew = fields[2].parse::<f64>().ok();
            let temp_dew = if let (Some(temp), Some(dew)) = (temp, dew) {
                let temp_str = if temp < 0.0 {
                    format!("M{:02}", temp.abs().round() as i32)
                } else {
                    format!("{:02}", temp.round() as i32)
                };
                let dew_str = if dew < 0.0 {
                    format!("M{:02}", dew.abs().round() as i32)
                } else {
                    format!("{:02}", dew.round() as i32)
                };
                format!("{}/{}", temp_str, dew_str)
            } else {
                "/// ///".to_string()
            };

            // Only show a forecast line if there are significant changes
            if !weather_str.is_empty() || visibility != "9999" || wind.contains("G") {
                trends.push_str(&format!(
                    " FCST {} {} {} {} {} {}",
                    trend_time, wind, visibility, weather_str, temp_dew, pressure
                ));
            }
        }
    }

    trends.trim().to_string()
}

pub fn gcd(a: i32, b: i32) -> i32 {
    if b == 0 {
        a.abs()
    } else {
        gcd(b, a % b)
    }
}
