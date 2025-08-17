//! ICAO → (lat, lon) lookup using the user's existing `airports.csv`.
//!
//! Expected CSV format (with optional leading `//` comment lines):
//!   ICAO,Latitude,Longitude
//!   KMSO,46.9163,-114.0906
//!
//! Notes:
//! - We ignore any line starting with `//`.
//! - We auto-skip the header if the first field equals "ICAO" (case-insensitive).
//! - File is loaded once into a global map on first use.

use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, sync::OnceLock};

static MAP: OnceLock<HashMap<String, (f64, f64)>> = OnceLock::new();

pub fn lookup_icao_coords(icao: &str) -> Option<(f64, f64)> {
    let map = MAP.get_or_init(|| load_map().unwrap_or_default());
    map.get(&icao.to_uppercase()).copied()
}

fn load_map() -> Result<HashMap<String, (f64, f64)>, String> {
    // Looks for "airports.csv" in the current working directory (next to the exe).
    let path = std::env::current_dir().unwrap_or_default().join("airports.csv");
    let f = File::open(&path).map_err(|e| format!("airports.csv not found: {}", e))?;
    let rdr = BufReader::new(f);

    let mut map = HashMap::new();
    for line in rdr.lines() {
        let line = match line { Ok(s) => s, Err(_) => continue };
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") { continue; }
        let parts: Vec<_> = trimmed.split(',').map(|s| s.trim()).collect();
        if parts.is_empty() { continue; }
        // Skip header row like: ICAO,Latitude,Longitude
        if parts[0].eq_ignore_ascii_case("ICAO") { continue; }
        if parts.len() < 3 { continue; }
        let icao = parts[0].to_uppercase();
        let lat = parts[1].parse::<f64>().ok();
        let lon = parts[2].parse::<f64>().ok();
        if let (Some(a), Some(b)) = (lat, lon) { map.insert(icao, (a, b)); }
    }
    Ok(map)
}
