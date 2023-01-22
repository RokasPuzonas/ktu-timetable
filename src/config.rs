use std::{path::{Path, PathBuf}, fs};

use directories_next::ProjectDirs;
use eframe::Storage;
use egui::util::cache::CacheStorage;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize)]
pub struct Config {
    pub vidko_code: Option<String>
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vidko_code: Some("E0000".into())
        }
    }
}

pub struct ConfigStorage {
    pub config: Config,

    config_file: Option<PathBuf>
}

impl Default for ConfigStorage {
    fn default() -> Self {
        let project_dirs = ProjectDirs::from("", "",  "KTU Timetable").expect("Failed to determine project directories");
        let config_dir = project_dirs.config_dir();
        Self {
            config: Config::default(),
            config_file: Some(config_dir.join("config.toml"))
        }
    }
}

impl ConfigStorage {
    pub fn memory() -> Self {
        let mut config = Self::default();
        config.config_file = None;
        config
    }

    pub fn attempt_load(&mut self) {
        if self.config_file.is_none() { return; }
        let config_file = self.config_file.as_ref().unwrap();
        let config_str = fs::read_to_string(config_file);
        if let Err(_) = config_str {
            fs::write(config_file, toml::to_string_pretty(&Config::default()).unwrap()).unwrap();
        }
        let config_str = config_str.unwrap();
        self.config = toml::from_str(&config_str).unwrap_or_default();
    }

    pub fn attempt_save(&self) {
        if self.config_file.is_none() { return; }
        let config_file = self.config_file.as_ref().unwrap();
        let config_str = toml::to_string_pretty(&self.config).unwrap();
        fs::write(config_file, config_str).unwrap();
    }
}

