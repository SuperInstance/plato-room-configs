//! JSON Schema validation for room configurations and manifests.

use jsonschema::{JSONSchema, ValidationError};
use serde_json::Value;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidatorError {
    #[error("Failed to read schema file '{path}': {source}")]
    SchemaFileRead {
        path: String,
        source: std::io::Error,
    },
    #[error("Failed to parse schema JSON: {0}")]
    SchemaParse(#[from] serde_json::Error),
    #[error("Schema compilation failed: {0}")]
    SchemaCompile(String),
    #[error("Validation failed: {errors}")]
    ValidationFailed { errors: String },
}

impl From<ValidationError<'_>> for ValidatorError {
    fn from(err: ValidationError<'_>) -> Self {
        ValidatorError::SchemaCompile(err.to_string())
    }
}

/// Validates room configurations and manifests against JSON Schemas.
pub struct RoomValidator {
    room_schema: JSONSchema,
    manifest_schema: JSONSchema,
}

impl RoomValidator {
    /// Create a validator by loading schemas from the given directory.
    /// Expects `room.schema.json` and `manifest.schema.json` to exist in the directory.
    pub fn from_schema_dir(dir: &str) -> Result<Self, ValidatorError> {
        let room_path = format!("{}/room.schema.json", dir);
        let manifest_path = format!("{}/manifest.schema.json", dir);
        Self::from_schema_files(&room_path, &manifest_path)
    }

    /// Create a validator by loading a room schema from a specific file.
    pub fn from_schema_file(path: &str) -> Result<Self, ValidatorError> {
        let manifest_path = path.replace("room.schema.json", "manifest.schema.json");
        Self::from_schema_files(path, &manifest_path)
    }

    /// Create a validator from explicit schema file paths.
    pub fn from_schema_files(room_schema_path: &str, manifest_schema_path: &str) -> Result<Self, ValidatorError> {
        let room_schema_str = fs::read_to_string(room_schema_path).map_err(|e| {
            ValidatorError::SchemaFileRead {
                path: room_schema_path.to_string(),
                source: e,
            }
        })?;
        let manifest_schema_str = fs::read_to_string(manifest_schema_path).map_err(|e| {
            ValidatorError::SchemaFileRead {
                path: manifest_schema_path.to_string(),
                source: e,
            }
        })?;

        let room_schema_json: Value = serde_json::from_str(&room_schema_str)?;
        let manifest_schema_json: Value = serde_json::from_str(&manifest_schema_str)?;

        let room_schema = JSONSchema::compile(&room_schema_json)
            .map_err(|e| ValidatorError::SchemaCompile(e.to_string()))?;
        let manifest_schema = JSONSchema::compile(&manifest_schema_json)
            .map_err(|e| ValidatorError::SchemaCompile(e.to_string()))?;

        Ok(Self {
            room_schema,
            manifest_schema,
        })
    }

    /// Validate a room configuration JSON value against the room schema.
    pub fn validate_room(&self, value: &Value) -> Result<(), ValidatorError> {
        let result = self.room_schema.validate(value);
        match result {
            Ok(()) => Ok(()),
            Err(errors) => {
                let msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
                Err(ValidatorError::ValidationFailed {
                    errors: msgs.join("; "),
                })
            }
        }
    }

    /// Validate a manifest JSON value against the manifest schema.
    pub fn validate_manifest(&self, value: &Value) -> Result<(), ValidatorError> {
        let result = self.manifest_schema.validate(value);
        match result {
            Ok(()) => Ok(()),
            Err(errors) => {
                let msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
                Err(ValidatorError::ValidationFailed {
                    errors: msgs.join("; "),
                })
            }
        }
    }

    /// Validate a JSON string as a room configuration.
    pub fn validate_room_str(&self, json_str: &str) -> Result<(), ValidatorError> {
        let value: Value = serde_json::from_str(json_str)?;
        self.validate_room(&value)
    }

    /// Validate a JSON string as a manifest.
    pub fn validate_manifest_str(&self, json_str: &str) -> Result<(), ValidatorError> {
        let value: Value = serde_json::from_str(json_str)?;
        self.validate_manifest(&value)
    }

    /// Validate a room config file.
    pub fn validate_room_file(&self, path: &Path) -> Result<(), ValidatorError> {
        let contents = fs::read_to_string(path).map_err(|e| ValidatorError::SchemaFileRead {
            path: path.display().to_string(),
            source: e,
        })?;
        self.validate_room_str(&contents)
    }

    /// Validate a manifest file.
    pub fn validate_manifest_file(&self, path: &Path) -> Result<(), ValidatorError> {
        let contents = fs::read_to_string(path).map_err(|e| ValidatorError::SchemaFileRead {
            path: path.display().to_string(),
            source: e,
        })?;
        self.validate_manifest_str(&contents)
    }

    /// Validate a raw JSON value, auto-detecting whether it's a room or manifest.
    pub fn validate(&self, value: &Value) -> Result<(), ValidatorError> {
        // If it has "manifest_id", treat as manifest; otherwise treat as room.
        if value.get("manifest_id").is_some() {
            self.validate_manifest(value)
        } else {
            self.validate_room(value)
        }
    }
}
