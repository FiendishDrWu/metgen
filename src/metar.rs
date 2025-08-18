use crate::config::Units;
use crate::providers::WeatherSample;
use crate::wx_format::*;
use chrono::{Datelike, Timelike};

pub fn render_metar(s: &WeatherSample, units: Units, icao_opt: Option<&str>) -> String {
    let icao = icao_opt.unwrap_or("XXXX").to_uppercase();
    let t = s.obs_time;
    let time = format!("{:02}{:02}{:02}Z", t.day(), t.hour(), t.minute());

    let wind = format_wind(s.wind_dir_deg, s.wind_speed_ms, s.wind_gust_ms);
    let vis = format_visibility(s.visibility_m, units, &s.wx_codes);
    let wx = format_weather_conditions(&s.wx_codes);
    let clouds = format_clouds(s.cloud_pct);
    let temp_dew = format_temp_dew(s.temp_c, s.dew_c, s.humidity_pct);
    let alt = format_pressure(s.pressure_hpa, units);

    let mut parts = vec![icao, time, "AUTO".into()];
    for p in [wind, vis, wx, clouds, temp_dew, alt] {
        if !p.is_empty() {
            parts.push(p);
        }
    }

    // One Call trend (optional): if we have 1–2 hourly points, generate a compact trend suffix.
    if !s.hourly_next.is_empty() {
        if let Some(tr) = generate_trend(s, units) {
            parts.push(tr);
        }
    }

    parts.join(" ")
}

/// Generate a compact trend using the first 1–2 One Call hourly points.
/// Examples: "BECMG 5SM", "TEMPO -RA", "BECMG 18015KT".
fn generate_trend(s: &WeatherSample, units: Units) -> Option<String> {
    let next = s.hourly_next.get(0)?; // at least one

    // Visibility change: report if crossing 10 km threshold or delta ≥ 2 km
    let mut tokens: Vec<String> = Vec::new();
    if let (Some(v0), Some(v1)) = (s.visibility_m, next.visibility_m) {
        let cross_10k = (v0 < 10_000.0 && v1 >= 10_000.0) || (v0 >= 10_000.0 && v1 < 10_000.0);
        let delta = (v1 - v0).abs();
        if cross_10k || delta >= 2000.0 {
            let vis = format_visibility(Some(v1), units, &next.wx_codes);
            if !vis.is_empty() {
                tokens.push(format!("BECMG {}", vis));
            }
        }
    }

    // Wind change: report if speed change ≥ 5 kt or direction ≥ 30°
    let kt = |ms: Option<f64>| ms.map(|v| (v * 1.94384).round() as i32).unwrap_or(0);
    if let (Some(d0), Some(d1)) = (s.wind_dir_deg, next.wind_dir_deg) {
        if let (Some(s0), Some(s1)) = (s.wind_speed_ms, next.wind_speed_ms) {
            let dd = ((d1 - d0 + 540.0) % 360.0) - 180.0; // shortest angle
            if dd.abs() >= 30.0 || (kt(Some(s1)) - kt(Some(s0))).abs() >= 5 {
                let w = format_wind(next.wind_dir_deg, next.wind_speed_ms, next.wind_gust_ms);
                if !w.is_empty() {
                    tokens.push(format!("BECMG {}", w));
                }
            }
        }
    }

    // Phenomena onset/cessation — if any difference in codes, mark TEMPO with next codes
    if s.wx_codes != next.wx_codes {
        let w = format_weather_conditions(&next.wx_codes);
        if !w.is_empty() {
            tokens.push(format!("TEMPO {}", w));
        }
    }

    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" "))
    }
}
