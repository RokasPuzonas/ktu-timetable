#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod timetable;
mod app;
mod config;
mod events_table;
mod utils;
mod platforms;
mod environment;

#[macro_use]
extern crate lazy_static;

use config::TomlConfigStore;
use environment::Environment;
use timetable::BlockingTimetableGetter;

// TODO: show errors when loading config
// TODO: Settings menu
// TODO: use "confy" for config loading?
// TODO: Setup pipeline

fn main() {
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

    platforms::run_windows_app(Environment {
        timetable_getter: Box::new(timetable_getter),
        config_store: Box::new(config_store)
    })
}

