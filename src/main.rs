use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "METAR Maker",
        native_options,
        Box::new(|cc| Ok(Box::new(metar_maker_gui::ui::AppState::new(cc))))
    )
}
