use crate::config::Units;

pub fn format_wind(dir_deg: Option<f64>, spd_ms: Option<f64>, gst_ms: Option<f64>) -> String {
    let kt = |ms: f64| (ms * 1.94384).round() as i32;
    match (dir_deg, spd_ms) {
        (_, Some(s)) if kt(s) == 0 => "00000KT".into(),
        (None, Some(s)) => format!(
            "VRB{:02}KT{}",
            kt(s),
            gst_ms.map(|g| format!("G{:02}", kt(g))).unwrap_or_default()
        ),
        (Some(d), Some(s)) => format!(
            "{:03}{:02}KT{}",
            (d.round() as i32).rem_euclid(360),
            kt(s),
            gst_ms.map(|g| format!("G{:02}", kt(g))).unwrap_or_default()
        ),
        _ => String::new(),
    }
}

pub fn format_visibility(vis_m: Option<f64>, _units: Units, wx_codes: &[i32]) -> String {
    if vis_m.is_none() {
        return String::new();
    }
    let m = vis_m.unwrap();
    let has_reducing = wx_codes
        .iter()
        .any(|id| matches!(id, 200..=599 | 701..=799));
    if m >= 10_000.0 && !has_reducing {
        return "10SM".into();
    }

    // meters → statute miles, nearest 1/4
    let sm = m / 1609.344;
    let q = (sm * 4.0).round() / 4.0; // quantize to quarters
    if (q - q.trunc()).abs() < 1e-6 {
        format!("{}SM", q as i32)
    } else {
        let whole = q.trunc() as i32;
        let frac = ((q - q.trunc()) * 4.0).round() as i32; // 1..3 quarters
        let frac_str = match frac {
            1 => "1/4",
            2 => "1/2",
            3 => "3/4",
            _ => "",
        };
        if whole == 0 {
            format!("{}SM", frac_str)
        } else {
            format!("{} {}SM", whole, frac_str)
        }
    }
}

pub fn format_weather_conditions(ids: &[i32]) -> String {
    // Broader mapping of OpenWeather weather IDs → METAR tokens
    // Reference: https://openweathermap.org/weather-conditions
    use std::fmt::Write as _;
    let mut out = String::new();
    for id in ids {
        let code = match *id {
            // Thunderstorm 2xx
            200 => "-TSRA",
            201 => "TSRA",
            202 => "+TSRA",
            210 => "-TS",
            211 => "TS",
            212 => "+TS",
            221 => "TS", // ragged thunderstorm
            230 => "-TSDZ",
            231 => "TSDZ",
            232 => "+TSDZ",

            // Drizzle 3xx
            300 => "-DZ",
            301 => "-DZ",
            302 => "+DZ",
            310 => "-DZ",
            311 => "DZ",
            312 => "+DZ",
            313 => "SHDZ",
            314 => "+SHDZ",
            321 => "SHDZ",

            // Rain 5xx
            500 => "-RA",
            501 => "RA",
            502 => "+RA",
            503 => "+RA",
            504 => "+RA",
            511 => "FZRA",
            520 => "-SHRA",
            521 => "SHRA",
            522 => "+SHRA",
            531 => "SHRA",

            // Snow 6xx
            600 => "-SN",
            601 => "SN",
            602 => "+SN",
            611 => "PL",
            612 => "SHPL",
            613 => "PL",
            615 => "RASN",
            616 => "SNRA",
            620 => "-SHSN",
            621 => "SHSN",
            622 => "+SHSN",

            // Atmosphere 7xx
            701 => "BR",
            711 => "HZ",
            721 => "HZ",
            731 => "DU",
            741 => "FG",
            751 => "DS",
            761 => "DU",
            762 => "VA",
            771 => "SQ",
            781 => "FC",

            // Clouds 800–804 handled elsewhere as sky condition; ignore here
            _ => "",
        };
        if !code.is_empty() {
            let _ = write!(&mut out, "{} ", code);
        }
    }
    out.trim().to_string()
}

pub fn format_clouds(pct: Option<f64>) -> String {
    match pct.map(|v| v.round() as i32) {
        Some(0) => "SKC".into(),
        Some(p) if (1..=25).contains(&p) => "FEW".into(),
        Some(p) if (26..=50).contains(&p) => "SCT".into(),
        Some(p) if (51..=87).contains(&p) => "BKN".into(),
        Some(_) => "OVC".into(),
        None => String::new(),
    }
}

pub fn format_temp_dew(t_c: Option<f64>, dew_c: Option<f64>, rh: Option<f64>) -> String {
    let dew = match (dew_c, t_c, rh) {
        (Some(d), _, _) => Some(d),
        (None, Some(t), Some(h)) => Some(dewpoint_from_rh(t, h)),
        _ => None,
    };
    match (t_c, dew) {
        (Some(t), Some(d)) => format!("{}/{}", c_to_metar(t), c_to_metar(d)),
        (Some(t), None) => format!("{}/", c_to_metar(t)),
        _ => String::new(),
    }
}

pub fn format_pressure(hpa: Option<f64>, units: Units) -> String {
    match (hpa, units) {
        (Some(p), Units::Imperial) => {
            // Convert hPa → inHg and round to hundredths for altimeter setting.
            let inhg = p * 0.02953998_f64;
            format!("A{:04}", (inhg * 100.0).round() as i32)
        }
        (Some(p), Units::Metric) => format!("Q{:04}", p.round() as i32),
        _ => String::new(),
    }
}

fn dewpoint_from_rh(t_c: f64, rh_pct: f64) -> f64 {
    let a = 17.625;
    let b = 243.04;
    let gamma = (a * t_c) / (b + t_c) + (rh_pct / 100.0).ln();
    (b * gamma) / (a - gamma)
}

fn c_to_metar(v: f64) -> String {
    let r = v.round() as i32;
    if r < 0 {
        format!("M{:02}", -r)
    } else {
        format!("{:02}", r)
    }
}
