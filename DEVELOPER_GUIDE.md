# Developer Guide — Plato Room Configs

## Architecture Overview

The Room Config system provides declarative JSON configurations for Plato rooms, plus a Rust crate (`plato-room-configs`) for loading, parsing, and validating them programmatically. The crate is the bridge between human-authored config files and the Plato Engine runtime.

### Crate Structure

```
src/
├── lib.rs          — Re-exports: RoomLoader, ManifestLoader, RoomValidator
├── loader.rs       — RoomLoader: filesystem discovery and parsing of .room.json files
├── validator.rs    — RoomValidator: JSON Schema validation for rooms and manifests
├── manifest.rs     — ManifestLoader: manifest parsing and room reference resolution
```

### Data Model

```
LoadedRoom
├── path: PathBuf              — Where the file lives
├── room_id: String            — Unique identifier (e.g., "engine_room")
├── config: RoomConfig         — Fully parsed config
│   ├── room_id, name, tick_hz, history_capacity
│   ├── sensors: HashMap<String, Sensor>
│   │   └── sensor_type, unit, source, range, alarm_range
│   ├── actuators: HashMap<String, Actuator>
│   │   └── actuator_type, target, range, safe_default
│   ├── alarms: Vec<Alarm>
│   │   └── id, condition, cooldown_sec, severity, actions
│   └── metadata: RoomMetadata
│       └── location, hardware, deployment
└── raw_json: Value            — Preserved for schema validation
```

```
ResolvedManifest
├── manifest: Manifest
│   ├── manifest_id, name
│   ├── rooms: Vec<ManifestRoomRef>  (room_id + config path)
│   └── agents: Vec<ManifestAgent>   (agent_id, rooms, rules)
├── path: PathBuf
└── loaded_rooms: Vec<LoadedRoom>    — All referenced rooms loaded and verified
```

### Module Walkthrough

#### `loader.rs` — RoomLoader

Handles filesystem discovery and parsing:

- **`new(base_dir)`** — Root directory for all relative paths.
- **`load_room(relative_path)`** — Parse a single `.room.json` file.
- **`load_all_rooms()`** — Glob for `**/*.room.json` under base_dir.
- **`load_rooms_from_dir(subdir)`** — Glob within a specific subdirectory.
- **`load_room_file(absolute_path)`** — Load from an absolute path (used internally).

Error handling: Every I/O and parse error includes the file path in the error message. The `LoaderError` enum preserves the original error source for debugging.

Key design choice: Both the raw `serde_json::Value` and the typed `RoomConfig` are preserved in `LoadedRoom`. This lets you validate against JSON Schema (needs raw Value) while also working with typed data.

#### `validator.rs` — RoomValidator

JSON Schema validation using the `jsonschema` crate:

- **`from_schema_dir(dir)`** — Expects `room.schema.json` and `manifest.schema.json`.
- **`validate_room(&Value)`** — Validate room config against schema.
- **`validate_manifest(&Value)`** — Validate manifest against schema.
- **`validate(&Value)`** — Auto-detect type by checking for `manifest_id` field.
- **`validate_room_file(&Path)`** — Load + validate in one call.

The validator compiles schemas once at construction, making repeated validation fast.

#### `manifest.rs` — ManifestLoader

Manifest loading with room reference resolution:

- **`resolve(relative_path)`** — The main entry point. Loads the manifest, then loads each referenced room config, verifying that room_ids match.
- **`load_manifest(relative_path)`** — Parse manifest without resolving rooms.
- **`load_all_manifests()`** — Glob for `**/*.manifest.json`.
- **`agents_for_room(manifest, room_id)`** — Given a manifest, find which agents watch a specific room. Supports wildcard `["*"]`.

Duplicate detection: `resolve()` checks for duplicate room_ids within a manifest and returns `ManifestError::DuplicateRoom`.

### Extension Points

#### Custom Validation Rules

After schema validation, add domain-specific checks:

```rust
fn validate_sensor_ranges(room: &RoomConfig) -> Result<(), String> {
    for (name, sensor) in &room.sensors {
        if let Some(range) = &sensor.range {
            if range[0] >= range[1] {
                return Err(format!("Sensor '{}' has invalid range", name));
            }
        }
    }
    Ok(())
}
```

#### Custom Loader

Implement your own loader for databases, S3, or other backends by constructing `LoadedRoom` directly:

```rust
let config: RoomConfig = serde_json::from_str(&json_from_db)?;
let raw_json: Value = serde_json::from_str(&json_from_db)?;
let loaded = LoadedRoom {
    path: PathBuf::from("db://room/engine_room"),
    room_id: config.room_id.clone(),
    config,
    raw_json,
};
```

#### Manifest Generation

Programmatically generate manifests:

```rust
let manifest = Manifest {
    manifest_id: "my_deployment".into(),
    name: "My Deployment".into(),
    rooms: room_ids.iter().map(|id| ManifestRoomRef {
        room_id: id.clone(),
        config: format!("configs/{}/{}.room.json", deployment, id),
    }).collect(),
    agents: vec![ManifestAgent {
        agent_id: "watchdog".into(),
        rooms: vec!["*".into()],
        rules: "Monitor all rooms, alert on critical alarms.".into(),
    }],
};
```

### Testing Strategy

```bash
cargo test                       # Unit tests for all modules
cargo test --test test_validator # Schema validation tests
cargo test --test test_loader    # File loading tests
cargo test --test test_manifest  # Manifest resolution tests
cargo test --test test_all_configs  # Validates all 18 configs in the repo
cargo run --example validate_all   # Example: validate every config file
```

The test suite validates all 18 room configs and 5 manifests against their schemas, ensuring the example configs are always valid.

### Contributing

1. New room configs go in `configs/<deployment>/` with matching schema validation.
2. Schema changes must be backward-compatible (new fields should be optional).
3. Run `cargo test` and `cargo run --example validate_all` before submitting.
4. Keep the crate dependency-light — it's used by the engine runtime at startup.
