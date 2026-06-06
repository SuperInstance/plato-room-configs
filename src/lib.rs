//! # Plato Room Configuration System
//!
//! Production-ready room configurations for real-world deployments.
//! Each config defines sensors, actuators, alarms, and metadata for a physical space.
//!
//! ## Quick Start
//!
//! ```no_run
//! use plato_room_configs::loader::RoomLoader;
//! use plato_room_configs::validator::RoomValidator;
//!
//! let validator = RoomValidator::from_schema_file("schema/room.schema.json").unwrap();
//! let loader = RoomLoader::new(".");
//! let rooms = loader.load_all_rooms().unwrap();
//!
//! for room in &rooms {
//!     if let Err(e) = validator.validate(&room.raw_json) {
//!         eprintln!("Validation failed for {}: {}", room.room_id, e);
//!     }
//! }
//! ```

pub mod loader;
pub mod manifest;
pub mod validator;

pub use loader::RoomLoader;
pub use manifest::ManifestLoader;
pub use validator::RoomValidator;
