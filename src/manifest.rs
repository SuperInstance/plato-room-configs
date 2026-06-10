//! Manifest loading and room reference resolution.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::loader::{LoadedRoom, RoomLoader};

#[derive(Error, Debug)]
pub enum ManifestError {
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
    #[error("Room reference error: room '{room_id}' config not found at '{path}'")]
    RoomNotFound { room_id: String, path: String },
    #[error("Duplicate room_id '{room_id}' in manifest '{manifest_id}'")]
    DuplicateRoom { room_id: String, manifest_id: String },
}

/// A room reference within a manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestRoomRef {
    pub room_id: String,
    pub config: String,
}

/// An agent definition within a manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestAgent {
    pub agent_id: String,
    pub rooms: Vec<String>,
    pub rules: String,
}

/// A fully parsed deployment manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub manifest_id: String,
    pub name: String,
    pub rooms: Vec<ManifestRoomRef>,
    pub agents: Vec<ManifestAgent>,
}

/// A manifest with its loaded rooms fully resolved.
#[derive(Debug, Clone)]
pub struct ResolvedManifest {
    pub manifest: Manifest,
    pub path: PathBuf,
    pub loaded_rooms: Vec<LoadedRoom>,
}

/// Loads and resolves deployment manifests.
pub struct ManifestLoader {
    base_dir: PathBuf,
}

impl ManifestLoader {
    /// Create a new manifest loader rooted at the given base directory.
    pub fn new(base_dir: &str) -> Self {
        Self {
            base_dir: PathBuf::from(base_dir),
        }
    }

    /// Load a manifest from a relative path.
    pub fn load_manifest(&self, relative_path: &str) -> Result<Manifest, ManifestError> {
        let full_path = self.base_dir.join(relative_path);
        Self::load_manifest_file(&full_path)
    }

    /// Load a manifest from an absolute file path.
    pub fn load_manifest_file(path: &Path) -> Result<Manifest, ManifestError> {
        let contents = fs::read_to_string(path).map_err(|e| ManifestError::Io {
            path: path.display().to_string(),
            source: e,
        })?;

        serde_json::from_str(&contents).map_err(|e| ManifestError::JsonParse {
            path: path.display().to_string(),
            source: e,
        })
    }

    /// Load a manifest and resolve all room references, loading each room config.
    pub fn resolve(&self, relative_path: &str) -> Result<ResolvedManifest, ManifestError> {
        let full_path = self.base_dir.join(relative_path);
        let manifest = Self::load_manifest_file(&full_path)?;

        let room_loader = RoomLoader::new(self.base_dir.to_str().unwrap_or("."));
        let mut loaded_rooms = Vec::new();

        // Check for duplicate room_ids
        let mut seen_ids = std::collections::HashSet::new();
        for room_ref in &manifest.rooms {
            if !seen_ids.insert(room_ref.room_id.clone()) {
                return Err(ManifestError::DuplicateRoom {
                    room_id: room_ref.room_id.clone(),
                    manifest_id: manifest.manifest_id.clone(),
                });
            }
        }

        for room_ref in &manifest.rooms {
            let room = room_loader
                .load_room(&room_ref.config)
                .map_err(|_| ManifestError::RoomNotFound {
                    room_id: room_ref.room_id.clone(),
                    path: room_ref.config.clone(),
                })?;

            if room.room_id != room_ref.room_id {
                return Err(ManifestError::RoomNotFound {
                    room_id: room_ref.room_id.clone(),
                    path: room_ref.config.clone(),
                });
            }

            loaded_rooms.push(room);
        }

        Ok(ResolvedManifest {
            manifest,
            path: full_path,
            loaded_rooms,
        })
    }

    /// Load all manifest files found under the base directory.
    pub fn load_all_manifests(&self) -> Result<Vec<Manifest>, ManifestError> {
        let pattern = format!("{}/**/*.manifest.json", self.base_dir.display());
        let paths = glob::glob(&pattern).map_err(|e| ManifestError::Io {
            path: pattern.clone(),
            source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
        })?;

        let mut manifests = Vec::new();
        for entry in paths {
            let path = entry.map_err(|e| ManifestError::Io {
                path: "glob".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
            })?;
            manifests.push(Self::load_manifest_file(&path)?);
        }

        Ok(manifests)
    }

    /// Get the base directory for this loader.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}

/// Returns which agents are active for a given room_id based on the manifest's agent definitions.
pub fn agents_for_room<'a>(manifest: &'a Manifest, room_id: &str) -> Vec<&'a ManifestAgent> {
    manifest
        .agents
        .iter()
        .filter(|agent| {
            agent.rooms.contains(&"*".to_string()) || agent.rooms.contains(&room_id.to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agents_for_room_wildcard() {
        let manifest = Manifest {
            manifest_id: "test".to_string(),
            name: "Test".to_string(),
            rooms: vec![],
            agents: vec![ManifestAgent {
                agent_id: "watchdog".to_string(),
                rooms: vec!["*".to_string()],
                rules: "monitor all".to_string(),
            }],
        };
        let agents = agents_for_room(&manifest, "engine_room");
        assert_eq!(agents.len(), 1);
    }

    #[test]
    fn test_agents_for_room_specific() {
        let manifest = Manifest {
            manifest_id: "test".to_string(),
            name: "Test".to_string(),
            rooms: vec![],
            agents: vec![ManifestAgent {
                agent_id: "engine_monitor".to_string(),
                rooms: vec!["engine_room".to_string(), "bilge".to_string()],
                rules: "monitor engine".to_string(),
            }],
        };
        let agents = agents_for_room(&manifest, "engine_room");
        assert_eq!(agents.len(), 1);
        let agents = agents_for_room(&manifest, "galley");
        assert_eq!(agents.len(), 0);
    }
}
