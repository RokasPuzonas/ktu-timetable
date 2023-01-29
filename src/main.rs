#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod timetable;
mod app;
mod config;
mod events_table;
mod utils;

#[macro_use]
extern crate lazy_static;

use app::MainApp;
use chrono::{Local, NaiveDate, NaiveTime};
use config::{MemoryConfigStore, Config, TomlConfigStore};
use eframe::egui;
use timetable::{DummyTimetableGetter, Timetable, Event, BlockingTimetableGetter};

// TODO: use lazy_static!() to load assets
// TODO: convert events_table to egui widget
// TODO: show errors when loading config
// TODO: Settings menu
// TODO: use "confy" for config loading?
// TODO: refactor persistence
// TODO: Setup pipeline

fn main() -> Result<(), ureq::Error> {
    let config_store = TomlConfigStore::default();
    // let config_store = MemoryConfigStore::new(Config {
    //     vidko: None//Some("E1810".into())
    // });

    let timetable_getter = BlockingTimetableGetter::default();
    // let timetable_getter = DummyTimetableGetter::new(Timetable {
    //     events: vec![
    //         Event {
    //             category: timetable::EventCategory::Default,
    //             date: NaiveDate::from_ymd_opt(2023, 1, 30).unwrap(),
    //             start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
    //             end_time: NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
    //             description: "Foobarbaz".into(),
    //             summary: "P123B123 Dummy module".into(),
    //             location: "XI r.-521".into(),
    //             module_name: Some("Dummy module".into())
    //         }
    //     ]
    // });

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
    let mut app = MainApp::new(Box::new(config_store),  Box::new(timetable_getter));

    eframe::run_native(
        "KTU timetable",
        native_options,
        Box::new(move |cc| {
            app.init(cc);
            Box::new(app)
        })
    );

    Ok(())
}
