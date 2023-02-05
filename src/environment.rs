use crate::{timetable::{TimetableGetter, BlockingTimetableGetter}, config::{ConfigStore, TomlConfigStore}};

pub struct Environment {
    pub timetable_getter: Box<dyn TimetableGetter>,
    pub config_store: Box<dyn ConfigStore>
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            config_store: Box::new(TomlConfigStore::default()),
            timetable_getter: Box::new(BlockingTimetableGetter::default())
        }
    }
}