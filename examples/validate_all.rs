//! Validate all room configs and manifests in a directory.
//!
//! Usage: cargo run --example validate_all -- [directory]

use plato_room_configs::loader::RoomLoader;
use plato_room_configs::manifest::ManifestLoader;
use plato_room_configs::validator::RoomValidator;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let base = args.get(1).map(|s| s.as_str()).unwrap_or(".");

    println!("🔍 Validating Plato room configs in: {}", base);
    println!();

    // Load validator
    let schema_dir = format!("{}/schema", base);
    let validator = RoomValidator::from_schema_dir(&schema_dir).unwrap_or_else(|e| {
        eprintln!("❌ Failed to load schemas from '{}': {}", schema_dir, e);
        std::process::exit(1);
    });

    let mut total = 0usize;
    let mut passed = 0usize;
    let mut failed = 0usize;

    // Validate room configs
    let loader = RoomLoader::new(base);
    let rooms = loader.load_all_rooms().unwrap_or_else(|e| {
        eprintln!("❌ Failed to scan for rooms: {}", e);
        std::process::exit(1);
    });

    println!("📋 Room Configurations ({}):", rooms.len());
    for room in &rooms {
        total += 1;
        match validator.validate_room(&room.raw_json) {
            Ok(()) => {
                passed += 1;
                println!("  ✅ {} ({})", room.room_id, room.path.display());
            }
            Err(e) => {
                failed += 1;
                println!("  ❌ {} ({}): {}", room.room_id, room.path.display(), e);
            }
        }
    }

    // Validate manifests
    let manifest_loader = ManifestLoader::new(base);
    let manifests = manifest_loader.load_all_manifests().unwrap_or_else(|e| {
        eprintln!("❌ Failed to scan for manifests: {}", e);
        std::process::exit(1);
    });

    println!();
    println!("📋 Manifests ({}):", manifests.len());
    for manifest in &manifests {
        total += 1;
        let raw = serde_json::to_value(manifest).expect("Serialize manifest");
        match validator.validate_manifest(&raw) {
            Ok(()) => {
                passed += 1;
                println!("  ✅ {} — {} rooms", manifest.manifest_id, manifest.rooms.len());
            }
            Err(e) => {
                failed += 1;
                println!("  ❌ {}: {}", manifest.manifest_id, e);
            }
        }
    }

    println!();
    println!("═══════════════════════════════════════");
    println!("Total: {} | Passed: {} | Failed: {}", total, passed, failed);
    if failed > 0 {
        std::process::exit(1);
    }
    println!("All configurations valid! 🎉");
}
