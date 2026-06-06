use plato_room_configs::loader::{LoadedRoom, RoomConfig, RoomLoader};

fn get_loader() -> RoomLoader {
    RoomLoader::new(".")
}

#[test]
fn test_load_single_room() {
    let loader = get_loader();
    let room = loader
        .load_room("configs/fishing-boat/engine-room.room.json")
        .expect("Failed to load engine room config");

    assert_eq!(room.room_id, "engine_room");
    assert_eq!(room.config.name, "Engine Room");
    assert_eq!(room.config.tick_hz, 0.2);
    assert_eq!(room.config.history_capacity, 500);
}

#[test]
fn test_room_sensors_parsed() {
    let loader = get_loader();
    let room = loader
        .load_room("configs/fishing-boat/engine-room.room.json")
        .expect("Failed to load");

    assert!(room.config.sensors.contains_key("coolant_temp_c"));
    assert!(room.config.sensors.contains_key("bilge_cm"));
    assert!(room.config.sensors.contains_key("oil_pressure_psi"));

    let coolant = room.config.sensors.get("coolant_temp_c").unwrap();
    assert_eq!(coolant.sensor_type, "float");
    assert_eq!(coolant.source, "mqtt:esp32/engine/coolant");
    assert_eq!(coolant.range.as_ref().unwrap().len(), 2);
}

#[test]
fn test_room_actuators_parsed() {
    let loader = get_loader();
    let room = loader
        .load_room("configs/fishing-boat/engine-room.room.json")
        .expect("Failed to load");

    assert!(room.config.actuators.contains_key("bilge_pump"));
    assert!(room.config.actuators.contains_key("engine_throttle"));

    let pump = room.config.actuators.get("bilge_pump").unwrap();
    assert_eq!(pump.actuator_type, "boolean");
    assert_eq!(pump.safe_default, serde_json::Value::Bool(false));
}

#[test]
fn test_room_alarms_parsed() {
    let loader = get_loader();
    let room = loader
        .load_room("configs/fishing-boat/engine-room.room.json")
        .expect("Failed to load");

    assert!(!room.config.alarms.is_empty());
    let overheat = room.config.alarms.iter().find(|a| a.id == "overheat").unwrap();
    assert_eq!(overheat.severity, "critical");
    assert_eq!(overheat.cooldown_sec, 30);
    assert!(overheat.actions.contains(&"notify_captain".to_string()));
}

#[test]
fn test_room_metadata() {
    let loader = get_loader();
    let room = loader
        .load_room("configs/fishing-boat/engine-room.room.json")
        .expect("Failed to load");

    assert_eq!(room.config.metadata.location, "below_deck_aft");
    assert_eq!(room.config.metadata.deployment, "fishing-boat");
}

#[test]
fn test_load_all_rooms() {
    let loader = get_loader();
    let rooms = loader.load_all_rooms().expect("Failed to load all rooms");
    assert!(rooms.len() >= 15, "Should find at least 15 room configs, found {}", rooms.len());

    // Verify unique room IDs
    let ids: std::collections::HashSet<&str> = rooms.iter().map(|r| r.room_id.as_str()).collect();
    assert_eq!(ids.len(), rooms.len(), "All room IDs should be unique");
}

#[test]
fn test_load_rooms_from_subdir() {
    let loader = get_loader();
    let rooms = loader
        .load_rooms_from_dir("configs/fishing-boat")
        .expect("Failed to load fishing boat rooms");
    assert_eq!(rooms.len(), 6, "Fishing boat should have 6 rooms");
}

#[test]
fn test_raw_json_preserved() {
    let loader = get_loader();
    let room = loader
        .load_room("configs/smart-home/living-room.room.json")
        .expect("Failed to load");

    assert!(room.raw_json.is_object());
    assert_eq!(room.raw_json["room_id"], "living_room");
}
