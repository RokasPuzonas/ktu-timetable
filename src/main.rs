#![windows_subsystem = "windows"]

mod timetable;
mod app;
mod config;

use app::MainApp;
use config::ConfigStorage;
use eframe::egui;

// TODO: use lazy_static!() to load assets
// TODO: convert events_table to egui widget
// TODO: show errors when loading config
// TODO: Settings menu
// TODO: use "confy" for config loading?
// TODO: refactor persistence

fn main() -> Result<(), ureq::Error> {
    let mut config_storage = ConfigStorage::default();
    config_storage.config.vidko_code = Some("E1810".into());

    let mut native_options = eframe::NativeOptions::default();
    native_options.decorated = true;
    native_options.resizable = true;
    native_options.min_window_size = Some(egui::vec2(480.0, 320.0));
    native_options.initial_window_size = Some(egui::vec2(500.0, 320.0));
    native_options.icon_data = Some(eframe::IconData {
        rgba: image::load_from_memory(include_bytes!("../assets/icon.png"))
            .expect("Failed to load icon")
            .into_rgb8()
            .into_raw(),
        width: 32,
        height: 32,
    });
    let mut app = MainApp::new(config_storage);

    eframe::run_native(
        "KTU timetable",
        native_options,
        Box::new(move |cc| {
            app.on_creation(cc);
            app.refresh_timetable();
            Box::new(app)
        })
    );

    Ok(())
}
