use reqwest::blocking::Client;

pub fn fetch_noaa_metar(icao: &str) -> Result<Option<String>, String> {
    if icao.len() != 4 || !icao.chars().all(|c| c.is_ascii_alphanumeric()) { return Ok(None); }
    let url = format!("https://aviationweather.gov/api/data/metar?ids={}&format=raw", icao);
    let client = Client::new();
    let res = client.get(url).send().map_err(|e| e.to_string())?;
    let sc = res.status();
    if sc.as_u16() == 404 { return Ok(None); }
    if sc.as_u16() == 401 { return Err("NOAA unauthorized".into()); }
    if !sc.is_success() { return Err(format!("NOAA error: {}", sc)); }
    let text = res.text().map_err(|e| e.to_string())?;
    if text.trim().is_empty() { return Ok(None); }
    Ok(Some(text.trim().to_string()))
}
