use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Units {
    Imperial,
    Metric,
}

impl Default for Units {
    fn default() -> Self {
        Units::Imperial
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Provider {
    Standard,
    OneCall,
}

impl Default for Provider {
    fn default() -> Self {
        Provider::Standard
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub owm_api_key: String,
    pub units: Units,
    pub provider: Provider,
}

/// Portable config: keep `config.json` next to the executable (same folder as the .exe and airports.csv)
fn config_path() -> PathBuf {
    // Prefer the executable's directory; fall back to current working dir if needed.
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("config.json")
}

pub fn load_or_default() -> Config {
    let path = config_path();
    if let Ok(s) = fs::read_to_string(&path) {
        if let Ok(mut cfg) = serde_json::from_str::<Config>(&s) {
            // Decode if it looks like base64 (very light heuristic); not real encryption.
            if maybe_base64(&cfg.owm_api_key) {
                if let Ok(decoded) = general_purpose::STANDARD.decode(cfg.owm_api_key.as_bytes()) {
                    if let Ok(txt) = String::from_utf8(decoded) {
                        cfg.owm_api_key = txt;
                    }
                }
            }
            return cfg;
        }
    }
    Config {
        owm_api_key: String::new(),
        units: Units::Imperial,
        provider: Provider::Standard,
    }
}

pub fn save(cfg: &Config) -> Result<(), String> {
    let path = config_path();
    let mut cfg_encoded = cfg.clone();
    cfg_encoded.owm_api_key = general_purpose::STANDARD.encode(cfg.owm_api_key.as_bytes());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let s = serde_json::to_string_pretty(&cfg_encoded).map_err(|e| e.to_string())?;
    fs::write(path, s).map_err(|e| e.to_string())
}

fn maybe_base64(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '='))
        && s.len() % 4 == 0
}
