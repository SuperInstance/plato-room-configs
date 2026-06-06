//! Filesystem loader for room configurations.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("IO error reading '{path}': {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("JSON parse error in '{path}': {source}")]
    JsonParse {
        path: String,
        source: serde_json::Error,
    },
    #[error("No .room.json files found in '{path}'")]
    NoRoomsFound { path: String },
    #[error("Glob error: {0}")]
    Glob(#[from] glob::GlobError),
    #[error("Glob pattern error: {0}")]
    Pattern(#[from] glob::PatternError),
}

/// A sensor definition within a room config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
    #[serde(rename = "type")]
    pub sensor_type: String,
    pub unit: Option<String>,
    pub source: String,
    pub range: Option<Vec<f64>>,
    pub alarm_range: Option<Vec<f64>>,
}

/// An actuator definition within a room config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actuator {
    #[serde(rename = "type")]
    pub actuator_type: String,
    pub target: String,
    pub range: Option<Vec<f64>>,
    pub safe_default: Value,
}

/// An alarm rule within a room config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    pub id: String,
    pub condition: String,
    pub cooldown_sec: u64,
    pub severity: String,
    pub actions: Vec<String>,
}

/// Room metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMetadata {
    pub location: String,
    pub hardware: Option<String>,
    pub deployment: String,
    #[serde(flatten)]
    pub extra: Value,
}

/// A fully parsed room configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    pub room_id: String,
    pub name: String,
    pub tick_hz: f64,
    pub history_capacity: u64,
    pub sensors: std::collections::HashMap<String, Sensor>,
    pub actuators: std::collections::HashMap<String, Actuator>,
    pub alarms: Vec<Alarm>,
    pub metadata: RoomMetadata,
}

/// A loaded room config with its file path and raw JSON preserved.
#[derive(Debug, Clone)]
pub struct LoadedRoom {
    pub path: PathBuf,
    pub room_id: String,
    pub config: RoomConfig,
    pub raw_json: Value,
}

/// Loads room configurations from the filesystem.
pub struct RoomLoader {
    base_dir: PathBuf,
}

impl RoomLoader {
    /// Create a new loader rooted at the given base directory.
    pub fn new(base_dir: &str) -> Self {
        Self {
            base_dir: PathBuf::from(base_dir),
        }
    }

    /// Load a single room config from a file path (relative to base_dir).
    pub fn load_room(&self, relative_path: &str) -> Result<LoadedRoom, LoaderError> {
        let full_path = self.base_dir.join(relative_path);
        Self::load_room_file(&full_path)
    }

    /// Load a room config from an absolute file path.
    pub fn load_room_file(path: &Path) -> Result<LoadedRoom, LoaderError> {
        let contents = fs::read_to_string(path).map_err(|e| LoaderError::Io {
            path: path.display().to_string(),
            source: e,
        })?;

        let raw_json: Value = serde_json::from_str(&contents).map_err(|e| LoaderError::JsonParse {
            path: path.display().to_string(),
            source: e,
        })?;

        let config: RoomConfig = serde_json::from_str(&contents).map_err(|e| {
            LoaderError::JsonParse {
                path: path.display().to_string(),
                source: e,
            }
        })?;

        let room_id = config.room_id.clone();

        Ok(LoadedRoom {
            path: path.to_path_buf(),
            room_id,
            config,
            raw_json,
        })
    }

    /// Load all .room.json files found under the base directory.
    pub fn load_all_rooms(&self) -> Result<Vec<LoadedRoom>, LoaderError> {
        let pattern = format!("{}/**/*.room.json", self.base_dir.display());
        let paths = glob::glob(&pattern).map_err(LoaderError::Pattern)?;

        let mut rooms = Vec::new();
        for entry in paths {
            let path = entry.map_err(LoaderError::Glob)?;
            rooms.push(Self::load_room_file(&path)?);
        }

        Ok(rooms)
    }

    /// Load all .room.json files from a specific subdirectory.
    pub fn load_rooms_from_dir(&self, subdir: &str) -> Result<Vec<LoadedRoom>, LoaderError> {
        let search_dir = self.base_dir.join(subdir);
        let pattern = format!("{}/**/*.room.json", search_dir.display());
        let paths = glob::glob(&pattern).map_err(LoaderError::Pattern)?;

        let mut rooms = Vec::new();
        for entry in paths {
            let path = entry.map_err(LoaderError::Glob)?;
            rooms.push(Self::load_room_file(&path)?);
        }

        Ok(rooms)
    }

    /// Get the base directory for this loader.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}
