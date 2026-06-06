use plato_room_configs::validator::RoomValidator;

fn get_validator() -> RoomValidator {
    RoomValidator::from_schema_dir("schema").expect("Failed to load schemas")
}

#[test]
fn test_valid_room_config() {
    let validator = get_validator();
    let json = serde_json::json!({
        "room_id": "test_room",
        "name": "Test Room",
        "tick_hz": 1.0,
        "history_capacity": 100,
        "sensors": {
            "temp": {
                "type": "float",
                "unit": "°C",
                "source": "mqtt:test/temp",
                "range": [0, 100],
                "alarm_range": [80, 100]
            }
        },
        "actuators": {
            "heater": {
                "type": "boolean",
                "target": "gpio:17",
                "safe_default": false
            }
        },
        "alarms": [
            {
                "id": "overheat",
                "condition": "temp > 80",
                "cooldown_sec": 30,
                "severity": "critical",
                "actions": ["log"]
            }
        ],
        "metadata": {
            "location": "test_lab",
            "deployment": "test"
        }
    });
    assert!(validator.validate_room(&json).is_ok(), "Valid room config should pass");
}

#[test]
fn test_invalid_room_missing_required_field() {
    let validator = get_validator();
    let json = serde_json::json!({
        "room_id": "test_room",
        "name": "Test Room"
        // Missing tick_hz, history_capacity, sensors, actuators, alarms, metadata
    });
    let result = validator.validate_room(&json);
    assert!(result.is_err(), "Missing required fields should fail");
}

#[test]
fn test_invalid_room_bad_room_id() {
    let validator = get_validator();
    let json = serde_json::json!({
        "room_id": "Invalid-ID!",
        "name": "Test Room",
        "tick_hz": 1.0,
        "history_capacity": 100,
        "sensors": {
            "temp": {
                "type": "float",
                "source": "mqtt:test/temp"
            }
        },
        "actuators": {},
        "alarms": [],
        "metadata": {
            "location": "test",
            "deployment": "test"
        }
    });
    let result = validator.validate_room(&json);
    assert!(result.is_err(), "Invalid room_id pattern should fail");
}

#[test]
fn test_invalid_room_zero_tick_hz() {
    let validator = get_validator();
    let json = serde_json::json!({
        "room_id": "test_room",
        "name": "Test Room",
        "tick_hz": 0,
        "history_capacity": 100,
        "sensors": {
            "temp": {
                "type": "float",
                "source": "mqtt:test/temp"
            }
        },
        "actuators": {},
        "alarms": [],
        "metadata": {
            "location": "test",
            "deployment": "test"
        }
    });
    let result = validator.validate_room(&json);
    assert!(result.is_err(), "tick_hz of 0 should fail");
}

#[test]
fn test_invalid_sensor_bad_source() {
    let validator = get_validator();
    let json = serde_json::json!({
        "room_id": "test_room",
        "name": "Test Room",
        "tick_hz": 1.0,
        "history_capacity": 100,
        "sensors": {
            "temp": {
                "type": "float",
                "source": "no-protocol-here"
            }
        },
        "actuators": {},
        "alarms": [],
        "metadata": {
            "location": "test",
            "deployment": "test"
        }
    });
    let result = validator.validate_room(&json);
    assert!(result.is_err(), "Invalid source format should fail");
}

#[test]
fn test_valid_manifest() {
    let validator = get_validator();
    let json = serde_json::json!({
        "manifest_id": "test-deployment",
        "name": "Test Deployment",
        "rooms": [
            {"room_id": "room1", "config": "configs/test/room1.room.json"}
        ],
        "agents": [
            {"agent_id": "watcher", "rooms": ["*"], "rules": "watch everything"}
        ]
    });
    assert!(validator.validate_manifest(&json).is_ok(), "Valid manifest should pass");
}

#[test]
fn test_invalid_manifest_missing_rooms() {
    let validator = get_validator();
    let json = serde_json::json!({
        "manifest_id": "test",
        "name": "Test"
        // Missing rooms and agents
    });
    let result = validator.validate_manifest(&json);
    assert!(result.is_err(), "Missing rooms should fail");
}
