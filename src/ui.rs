use crate::config::{self, Config, Units, Provider};
use crate::errors::UiError;
use crate::metar::render_metar;
use crate::providers::{fetch_noaa_metar, fetch_sample, lookup_icao_coords, WeatherSample};
use crossbeam_channel::{unbounded, Sender, Receiver};
use egui::{Align, Button, CentralPanel, Context, Grid, Layout, TextEdit, TopBottomPanel, Visuals};
use std::thread;

pub struct AppState {
    cfg: Config,
    icao: String,
    place: String,
    lat: String,
    lon: String,
    output: String,
    status: String,
    is_busy: bool,
    tx: Sender<UiMsg>,
    rx: Receiver<UiMsg>,
}

enum UiMsg { Done(Result<String, String>) }

impl AppState {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(Visuals::dark());
        let cfg = config::load_or_default();
        let (tx, rx) = unbounded();
        Self {
            icao: String::new(),
            place: String::new(),
            lat: String::new(),
            lon: String::new(),
            output: String::new(),
            status: String::from("Ready"),
            is_busy: false,
            cfg,
            tx, rx,
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Drain background results first
        while let Ok(UiMsg::Done(res)) = self.rx.try_recv() {
            self.is_busy = false;
            match res {
                Ok(metar) => { self.output = metar; self.status = "Success".into(); }
                Err(e) => { self.output.clear(); self.status = e; }
            }
        }

        TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("METAR Maker");
            ui.horizontal(|ui| {
                ui.label("Units:");
                egui::ComboBox::from_id_salt("units")
                    .selected_text(format!("{:?}", self.cfg.units))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.cfg.units, Units::Imperial, "Imperial (US)");
                        ui.selectable_value(&mut self.cfg.units, Units::Metric, "Metric");
                    });
                ui.separator();
                ui.label("Provider:");
                egui::ComboBox::from_id_salt("provider")
                    .selected_text(format!("{:?}", self.cfg.provider))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.cfg.provider, Provider::Standard, "Standard");
                        ui.selectable_value(&mut self.cfg.provider, Provider::OneCall, "OneCall");
                    });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.group(|ui| {
                    ui.label("API Keys (stored base64-obfuscated in config)");
                    ui.horizontal(|ui| { ui.label("OpenWeather:"); ui.add(TextEdit::singleline(&mut self.cfg.owm_api_key).hint_text("32-char key")); });
                });

                ui.separator();

                Grid::new("inputs").striped(true).show(ui, |ui| {
                    ui.label("ICAO:");
                    let resp = ui.add(TextEdit::singleline(&mut self.icao).hint_text("KMSO"));
                    if resp.changed() {
                        let code = self.icao.trim().to_uppercase();
                        if code.len() == 4 && code.chars().all(|c| c.is_ascii_alphanumeric()) {
                            if let Some((la, lo)) = lookup_icao_coords(&code) {
                                self.lat = format!("{:.4}", la);
                                self.lon = format!("{:.4}", lo);
                            }
                        }
                    }
                    ui.end_row();

                    ui.label("Place name:"); ui.add(TextEdit::singleline(&mut self.place).hint_text("Missoula, MT")); ui.end_row();
                    ui.label("Latitude:"); ui.add(TextEdit::singleline(&mut self.lat).hint_text("46.9163")); ui.end_row();
                    ui.label("Longitude:"); ui.add(TextEdit::singleline(&mut self.lon).hint_text("-114.0906")); ui.end_row();
                });

                ui.horizontal(|ui| {
                    let gen_btn = ui.add_enabled(!self.is_busy, Button::new("Generate METAR"));
                    if gen_btn.clicked() {
                        let icao = self.icao.trim().to_uppercase();
                        let place = self.place.trim().to_string();
                        let lat = self.lat.trim().parse::<f64>().ok();
                        let lon = self.lon.trim().parse::<f64>().ok();
                        let units = self.cfg.units;
                        let provider = self.cfg.provider;
                        let key = self.cfg.owm_api_key.clone();
                        let tx = self.tx.clone();
                        self.is_busy = true; self.status = "Working...".into();
                        thread::spawn(move || {
                            if !icao.is_empty() {
                                match fetch_noaa_metar(&icao) {
                                    Ok(Some(raw)) => { let _ = tx.send(UiMsg::Done(Ok(raw))); return; }
                                    Ok(None) => { /* fallthrough */ }
                                    Err(e) => { let _ = tx.send(UiMsg::Done(Err(e))); return; }
                                }
                            }
                            let icao_opt = if icao.is_empty() { None } else { Some(icao.as_str()) };
                            let place_opt = if place.is_empty() { None } else { Some(place.as_str()) };
                            let res: Result<WeatherSample, UiError> = fetch_sample(icao_opt, place_opt, lat, lon, provider, &key);
                            match res { Ok(s) => { let line = render_metar(&s, units, icao_opt); let _ = tx.send(UiMsg::Done(Ok(line))); }
                                       Err(e) => { let _ = tx.send(UiMsg::Done(Err(e.to_string()))); } }
                        });
                    }
                    if ui.button("Save Config").clicked() {
                        if let Err(e) = config::save(&self.cfg) { self.status = format!("Failed to save: {e}"); } else { self.status = "Saved".into(); }
                    }
                    ui.label(format!("Status: {}", self.status));
                });

                ui.separator();
                ui.label("Output METAR:");
                ui.add(TextEdit::multiline(&mut self.output).desired_rows(4).lock_focus(true));
            });
        });
    }
}
