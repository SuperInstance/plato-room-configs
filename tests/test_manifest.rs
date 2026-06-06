use plato_room_configs::manifest::{agents_for_room, ManifestLoader};

fn get_loader() -> ManifestLoader {
    ManifestLoader::new(".")
}

#[test]
fn test_load_fishing_boat_manifest() {
    let loader = get_loader();
    let manifest = loader
        .load_manifest("configs/fishing-boat/boat.manifest.json")
        .expect("Failed to load manifest");

    assert_eq!(manifest.manifest_id, "fishing-boat");
    assert_eq!(manifest.name, "F/V Northern Star");
    assert_eq!(manifest.rooms.len(), 6);
    assert_eq!(manifest.agents.len(), 2);
}

#[test]
fn test_resolve_fishing_boat_manifest() {
    let loader = get_loader();
    let resolved = loader
        .resolve("configs/fishing-boat/boat.manifest.json")
        .expect("Failed to resolve manifest");

    assert_eq!(resolved.loaded_rooms.len(), 6);

    // Verify all room IDs match
    let room_ids: Vec<&str> = resolved.loaded_rooms.iter().map(|r| r.room_id.as_str()).collect();
    assert!(room_ids.contains(&"engine_room"));
    assert!(room_ids.contains(&"backdeck"));
    assert!(room_ids.contains(&"wheelhouse"));
    assert!(room_ids.contains(&"crows_nest"));
    assert!(room_ids.contains(&"galley"));
    assert!(room_ids.contains(&"bilge"));
}

#[test]
fn test_agents_for_room_wildcard() {
    let loader = get_loader();
    let manifest = loader
        .load_manifest("configs/fishing-boat/boat.manifest.json")
        .expect("Failed to load");

    let agents = agents_for_room(&manifest, "galley");
    assert!(agents.iter().any(|a| a.agent_id == "watchdog"), "Watchdog should cover galley via wildcard");
}

#[test]
fn test_agents_for_room_specific() {
    let loader = get_loader();
    let manifest = loader
        .load_manifest("configs/fishing-boat/boat.manifest.json")
        .expect("Failed to load");

    let agents = agents_for_room(&manifest, "engine_room");
    assert!(agents.iter().any(|a| a.agent_id == "engine_monitor"), "Engine monitor should cover engine_room");
}

#[test]
fn test_load_all_manifests() {
    let loader = get_loader();
    let manifests = loader.load_all_manifests().expect("Failed to load all manifests");
    assert!(
        manifests.len() >= 5,
        "Should find at least 5 manifests, found {}",
        manifests.len()
    );

    let ids: Vec<&str> = manifests.iter().map(|m| m.manifest_id.as_str()).collect();
    assert!(ids.contains(&"fishing-boat"));
    assert!(ids.contains(&"server-rack"));
    assert!(ids.contains(&"smart-home"));
    assert!(ids.contains(&"game-world"));
    assert!(ids.contains(&"factory"));
}

#[test]
fn test_manifest_agents_rules() {
    let loader = get_loader();
    let manifest = loader
        .load_manifest("configs/factory/factory.manifest.json")
        .expect("Failed to load");

    assert_eq!(manifest.agents.len(), 2);
    assert_eq!(manifest.agents[0].agent_id, "plant_controller");
    assert_eq!(manifest.agents[1].agent_id, "energy_manager");
}
