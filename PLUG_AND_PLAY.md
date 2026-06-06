# Plug & Play — Plato Room Configs

Copy-paste templates for the three most common patterns.

## Pattern 1: Minimal Room Config

```json
{
  "room_id": "my_room",
  "name": "My Room",
  "tick_hz": 1.0,
  "history_capacity": 500,
  "sensors": {
    "temperature_c": {
      "type": "float",
      "unit": "°C",
      "source": "mqtt:device/temp",
      "range": [-10, 60],
      "alarm_range": [40, 60]
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
      "condition": "temperature_c > 40",
      "cooldown_sec": 60,
      "severity": "warning",
      "actions": ["heater=false", "notify_ops"]
    }
  ],
  "metadata": {
    "location": "building_a",
    "deployment": "my-project"
  }
}
```

Save as `<name>.room.json`. Validate: `cargo run --example validate_all`

## Pattern 2: Multi-Room Manifest

```json
{
  "manifest_id": "my_deployment",
  "name": "My Deployment",
  "rooms": [
    { "room_id": "room_a", "config": "configs/deploy/room_a.room.json" },
    { "room_id": "room_b", "config": "configs/deploy/room_b.room.json" }
  ],
  "agents": [
    {
      "agent_id": "watchdog",
      "rooms": ["*"],
      "rules": "Monitor all rooms. Alert on critical alarms. Log warnings."
    }
  ]
}
```

Save as `<name>.manifest.json`.

## Pattern 3: Rust Loader

```rust
use plato_room_configs::{RoomLoader, RoomValidator, ManifestLoader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Validate all configs
    let validator = RoomValidator::from_schema_dir("schema")?;
    let loader = RoomLoader::new(".");
    for room in loader.load_all_rooms()? {
        validator.validate_room(&room.raw_json)?;
        println!("✅ {} — {} sensors", room.room_id, room.config.sensors.len());
    }

    // Load a manifest with resolved rooms
    let manifest = ManifestLoader::new(".")
        .resolve("configs/my-deploy/deploy.manifest.json")?;
    println!("{}: {} rooms", manifest.manifest.name, manifest.loaded_rooms.len());

    Ok(())
}
```

Add to `Cargo.toml`:
```toml
[dependencies]
plato-room-configs = { git = "https://github.com/SuperInstance/plato-room-configs" }
```

## Quick Reference

| What | Code / Field |
|------|-------------|
| Sensor types | `float`, `integer`, `boolean`, `string` |
| Source protocols | `mqtt:`, `gpio:`, `i2c:`, `spi:`, `modbus:`, `snmp:`, `ipmi:`, `bacnet:`, `zigbee:`, `game:` |
| Alarm severities | `info`, `warning`, `critical` |
| Alarm condition ops | `>`, `<`, `>=`, `<=`, `==`, `!=`, `and`, `or` |
| Tick rate guide | Safety: 1-2 Hz · Environment: 0.1-0.5 Hz · Industrial: 1-5 Hz |
| Validate all | `cargo run --example validate_all` |
| Load one room | `loader.load_room("path/to/room.room.json")` |
| Load all rooms | `loader.load_all_rooms()` |
| Resolve manifest | `ManifestLoader::new(".").resolve("path/manifest.json")` |
| Agents for room | `agents_for_room(&manifest, "room_id")` |
