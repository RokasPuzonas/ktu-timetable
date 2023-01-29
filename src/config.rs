use std::{path::{Path, PathBuf}, fs, io, error::Error, fmt};

use directories_next::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub vidko: Option<String>
}
impl Default for Config {
    fn default() -> Self {
        Self { vidko: None }
    }
}

#[derive(Debug)]
pub enum LoadConfigError {
    NotFound,
    FileError(io::Error),
    TomlError(toml::de::Error)
}
impl Error for LoadConfigError {}
impl fmt::Display for LoadConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoadConfigError::FileError(e) => write!(f, "File error: {}", e),
            LoadConfigError::TomlError(e) => write!(f, "Toml error: {}", e),
            LoadConfigError::NotFound     => write!(f, "Not found"),
        }
    }
}

#[derive(Debug)]
pub enum SaveConfigError {
    FileError(io::Error),
    TomlError(toml::ser::Error)
}
impl Error for SaveConfigError {}
impl fmt::Display for SaveConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SaveConfigError::FileError(e) => write!(f, "File error: {}", e),
            SaveConfigError::TomlError(e) => write!(f, "Toml error: {}", e),
        }
    }
}

pub trait ConfigStore {
    fn load(&self) -> Result<Config, LoadConfigError>;
    fn save(&self, config: &Config) -> Result<(), SaveConfigError>;
}



pub struct TomlConfigStore {
    filename: PathBuf
}
impl TomlConfigStore {
    fn new(filename: &Path) -> Self {
        Self {
            filename: filename.into()
        }
    }
}
impl Default for TomlConfigStore {
    fn default() -> Self {
        let project_dirs = ProjectDirs::from("", "",  "KTU Timetable").expect("Failed to determine home directory");
        let config_dir = project_dirs.config_dir();
        Self::new(&config_dir.join("config.toml"))
    }
}
impl ConfigStore for TomlConfigStore {
    fn load(&self) -> Result<Config, LoadConfigError> {
        let config_str = fs::read_to_string(&self.filename)
            .map_err(|e| LoadConfigError::FileError(e))?;

        toml::from_str(&config_str)
            .map_err(|e| LoadConfigError::TomlError(e))
    }

    fn save(&self, config: &Config) -> Result<(), SaveConfigError> {
        let directory = Path::parent(&self.filename).unwrap();
        if !Path::is_dir(directory) {
            fs::create_dir_all(directory)
                .map_err(|e| SaveConfigError::FileError(e))?;
        }

        let config_str = toml::to_string_pretty(config)
            .map_err(|e| SaveConfigError::TomlError(e))?;

        fs::write(&self.filename, config_str)
            .map_err(|e| SaveConfigError::FileError(e))?;

        Ok(())
    }
}


pub struct MemoryConfigStore {
    config: Option<Config>
}
impl MemoryConfigStore {
    pub fn new(config: Config) -> Self {
        Self { config: Some(config) }
    }
}
impl ConfigStore for MemoryConfigStore {
    fn load(&self) -> Result<Config, LoadConfigError> {
        self.config.clone().ok_or(LoadConfigError::NotFound)
    }

    fn save(&self, _config: &Config) -> Result<(), SaveConfigError> {
        Ok(())
    }
}