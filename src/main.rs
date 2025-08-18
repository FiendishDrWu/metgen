use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "METGen",
        native_options,
        Box::new(|cc| Ok(Box::new(metgen::ui::AppState::new(cc)))),
    )
}
