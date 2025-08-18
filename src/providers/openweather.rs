use crate::config::Provider;
use crate::errors::UiError;
use crate::providers::lookup_icao_coords;
use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct WeatherSample {
    pub obs_time: DateTime<Utc>,
    pub lat: f64,
    pub lon: f64,
    pub temp_c: Option<f64>,
    pub dew_c: Option<f64>,
    pub pressure_hpa: Option<f64>,
    pub humidity_pct: Option<f64>,
    pub wind_dir_deg: Option<f64>,
    pub wind_speed_ms: Option<f64>,
    pub wind_gust_ms: Option<f64>,
    pub visibility_m: Option<f64>,
    pub cloud_pct: Option<f64>,
    pub wx_codes: Vec<i32>,
    /// Up to the first 2 hourly forecast points (One Call only) for trend generation
    pub hourly_next: Vec<HourlyPoint>,
}

#[derive(Debug, Clone)]
pub struct HourlyPoint {
    pub dt: DateTime<Utc>,
    pub temp_c: Option<f64>,
    pub dew_c: Option<f64>,
    pub pressure_hpa: Option<f64>,
    pub humidity_pct: Option<f64>,
    pub wind_dir_deg: Option<f64>,
    pub wind_speed_ms: Option<f64>,
    pub wind_gust_ms: Option<f64>,
    pub visibility_m: Option<f64>,
    pub cloud_pct: Option<f64>,
    pub wx_codes: Vec<i32>,
}

#[derive(Deserialize)]
struct OwmCurrent {
    dt: i64,
    coord: Coord,
    main: Main,
    wind: Option<Wind>,
    visibility: Option<f64>,
    weather: Vec<Wx>,
    clouds: Option<Clouds>,
}
#[derive(Deserialize)]
struct Coord {
    lon: f64,
    lat: f64,
}
#[derive(Deserialize)]
struct Main {
    temp: f64,
    pressure: f64,
    humidity: f64,
}
#[derive(Deserialize)]
struct Wind {
    deg: Option<f64>,
    speed: Option<f64>,
    gust: Option<f64>,
}
#[derive(Deserialize)]
struct Wx {
    id: i32,
}
#[derive(Deserialize)]
struct Clouds {
    all: Option<f64>,
}

#[derive(Deserialize)]
struct OwmOneCall {
    lat: f64,
    lon: f64,
    current: OcCurrent,
    hourly: Vec<OcHour>,
}
#[derive(Deserialize)]
struct OcCurrent {
    dt: i64,
    temp: f64,
    pressure: f64,
    humidity: f64,
    dew_point: Option<f64>,
    wind_deg: Option<f64>,
    wind_speed: Option<f64>,
    wind_gust: Option<f64>,
    visibility: Option<f64>,
    weather: Vec<Wx>,
    clouds: Option<f64>,
}
#[derive(Deserialize)]
struct OcHour {
    dt: i64,
    temp: f64,
    pressure: f64,
    humidity: f64,
    dew_point: Option<f64>,
    wind_deg: Option<f64>,
    wind_speed: Option<f64>,
    wind_gust: Option<f64>,
    visibility: Option<f64>,
    weather: Vec<Wx>,
    clouds: Option<f64>,
}

pub fn fetch_sample(
    icao: Option<&str>,
    place: Option<&str>,
    lat: Option<f64>,
    lon: Option<f64>,
    provider: Provider,
    api_key: &str,
) -> Result<WeatherSample, UiError> {
    let (lat, lon) = match (lat, lon, icao, place) {
        (Some(a), Some(b), _, _) => (a, b),
        (_, _, Some(code), _) if !code.is_empty() => lookup_icao_coords(code)
            .ok_or_else(|| UiError::InvalidInput(format!("Unknown ICAO: {}", code)))?,
        (_, _, _, Some(p)) if !p.is_empty() => geocode_place(p, api_key)?,
        _ => {
            return Err(UiError::InvalidInput(
                "Provide ICAO, place name, or lat/lon".into(),
            ))
        }
    };

    match provider {
        Provider::Standard => fetch_current(lat, lon, api_key),
        Provider::OneCall => fetch_onecall(lat, lon, api_key),
    }
}

fn geocode_place(q: &str, api_key: &str) -> Result<(f64, f64), UiError> {
    #[derive(Deserialize)]
    struct Hit {
        lat: f64,
        lon: f64,
    }
    let url = format!(
        "https://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={}",
        urlencoding::encode(q),
        api_key
    );
    let client = Client::new();
    let res = client.get(url).send()?;
    if res.status().as_u16() == 401 {
        return Err(UiError::Unauthorized);
    }
    let arr: Vec<Hit> = res.json().map_err(|e| UiError::Parse(e.to_string()))?;
    arr.into_iter()
        .next()
        .map(|h| (h.lat, h.lon))
        .ok_or_else(|| UiError::NotFound)
}

fn fetch_current(lat: f64, lon: f64, api_key: &str) -> Result<WeatherSample, UiError> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&units=metric&appid={}",
        lat, lon, api_key
    );
    let client = Client::new();
    let res = client.get(url).send()?;
    if res.status().as_u16() == 401 {
        return Err(UiError::Unauthorized);
    }
    let m: OwmCurrent = res.json().map_err(|e| UiError::Parse(e.to_string()))?;
    Ok(WeatherSample {
        obs_time: chrono::DateTime::from_timestamp(m.dt, 0)
            .unwrap()
            .with_timezone(&Utc),
        lat: m.coord.lat,
        lon: m.coord.lon,
        temp_c: Some(m.main.temp),
        dew_c: None,
        pressure_hpa: Some(m.main.pressure),
        humidity_pct: Some(m.main.humidity),
        wind_dir_deg: m.wind.as_ref().and_then(|w| w.deg),
        wind_speed_ms: m.wind.as_ref().and_then(|w| w.speed),
        wind_gust_ms: m.wind.as_ref().and_then(|w| w.gust),
        visibility_m: m.visibility,
        cloud_pct: m.clouds.and_then(|c| c.all),
        wx_codes: m.weather.into_iter().map(|w| w.id).collect(),
        hourly_next: Vec::new(),
    })
}

fn fetch_onecall(lat: f64, lon: f64, api_key: &str) -> Result<WeatherSample, UiError> {
    let url = format!(
        "https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&units=metric&appid={}",
        lat, lon, api_key
    );
    let client = Client::new();
    let res = client.get(url).send()?;
    if res.status().as_u16() == 401 {
        return Err(UiError::Unauthorized);
    }
    let m: OwmOneCall = res.json().map_err(|e| UiError::Parse(e.to_string()))?;

    let mut hourly_next: Vec<HourlyPoint> = Vec::new();
    for h in m.hourly.into_iter().take(2) {
        hourly_next.push(HourlyPoint {
            dt: chrono::DateTime::from_timestamp(h.dt, 0)
                .unwrap()
                .with_timezone(&Utc),
            temp_c: Some(h.temp),
            dew_c: h.dew_point,
            pressure_hpa: Some(h.pressure),
            humidity_pct: Some(h.humidity),
            wind_dir_deg: h.wind_deg,
            wind_speed_ms: h.wind_speed,
            wind_gust_ms: h.wind_gust,
            visibility_m: h.visibility,
            cloud_pct: h.clouds,
            wx_codes: h.weather.into_iter().map(|w| w.id).collect(),
        });
    }

    Ok(WeatherSample {
        obs_time: chrono::DateTime::from_timestamp(m.current.dt, 0)
            .unwrap()
            .with_timezone(&Utc),
        lat: m.lat,
        lon: m.lon,
        temp_c: Some(m.current.temp),
        dew_c: m.current.dew_point,
        pressure_hpa: Some(m.current.pressure),
        humidity_pct: Some(m.current.humidity),
        wind_dir_deg: m.current.wind_deg,
        wind_speed_ms: m.current.wind_speed,
        wind_gust_ms: m.current.wind_gust,
        visibility_m: m.current.visibility,
        cloud_pct: m.current.clouds,
        wx_codes: m.current.weather.into_iter().map(|w| w.id).collect(),
        hourly_next,
    })
}
