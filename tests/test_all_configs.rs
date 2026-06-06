//! Validates every .room.json and .manifest.json in the repo against the schemas.

use plato_room_configs::validator::RoomValidator;

fn get_validator() -> RoomValidator {
    RoomValidator::from_schema_dir("schema").expect("Failed to load schemas")
}

fn find_configs(pattern: &str) -> Vec<std::path::PathBuf> {
    glob::glob(pattern)
        .expect("Invalid glob pattern")
        .filter_map(|e| e.ok())
        .collect()
}

#[test]
fn validate_all_room_configs() {
    let validator = get_validator();
    let rooms = find_configs("configs/**/*.room.json");
    assert!(!rooms.is_empty(), "No .room.json files found");

    let mut failures = Vec::new();
    for path in &rooms {
        if let Err(e) = validator.validate_room_file(path) {
            failures.push(format!("{}: {}", path.display(), e));
        }
    }

    assert!(
        failures.is_empty(),
        "Room config validation failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn validate_all_manifests() {
    let validator = get_validator();
    let manifests = find_configs("configs/**/*.manifest.json");
    assert!(!manifests.is_empty(), "No .manifest.json files found");

    let mut failures = Vec::new();
    for path in &manifests {
        if let Err(e) = validator.validate_manifest_file(path) {
            failures.push(format!("{}: {}", path.display(), e));
        }
    }

    assert!(
        failures.is_empty(),
        "Manifest validation failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn count_total_configs() {
    let rooms = find_configs("configs/**/*.room.json");
    let manifests = find_configs("configs/**/*.manifest.json");
    println!(
        "Found {} room configs and {} manifests",
        rooms.len(),
        manifests.len()
    );
    assert!(rooms.len() >= 15, "Expected at least 15 room configs");
    assert!(manifests.len() >= 5, "Expected at least 5 manifests");
}
