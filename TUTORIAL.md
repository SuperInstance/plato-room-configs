# Tutorial — Writing and Validating Room Configurations

This tutorial walks you through creating a room configuration from scratch, validating it, and loading it from Rust.

**Prerequisites:** Rust toolchain, this repo cloned.

## Step 1: Understand the Room Config Structure

Every room is a single `.room.json` file. Here's the minimal skeleton:

```json
{
  "room_id": "my_room",
  "name": "My Room",
  "tick_hz": 1.0,
  "history_capacity": 500,
  "sensors": {},
  "actuators": {},
  "alarms": [],
  "metadata": {
    "location": "building_a",
    "deployment": "my-project"
  }
}
```

Create `configs/workshop/laser_cutter.room.json`:

```json
{
  "room_id": "laser_cutter",
  "name": "Laser Cutter Station",
  "tick_hz": 2.0,
  "history_capacity": 1000,
  "sensors": {
    "exhaust_temp_c": {
      "type": "float",
      "unit": "°C",
      "source": "mqtt:workshop/laser/exhaust_temp",
      "range": [15, 120],
      "alarm_range": [80, 120]
    },
    "coolant_flow_lpm": {
      "type": "float",
      "unit": "L/min",
      "source": "gpio:17",
      "range": [0, 20],
      "alarm_range": [0, 2]
    },
    "air_quality_ppm": {
      "type": "float",
      "unit": "ppm",
      "source": "i2c:/dev/i2c-1/0x5a",
      "range": [0, 1000],
      "alarm_range": [400, 1000]
    },
    "laser_active": {
      "type": "boolean",
      "source": "gpio:22",
      "range": null
    }
  },
  "actuators": {
    "laser_power": {
      "type": "boolean",
      "target": "gpio:23",
      "safe_default": false
    },
    "exhaust_fan": {
      "type": "boolean",
      "target": "gpio:24",
      "safe_default": true
    },
    "emergency_stop": {
      "type": "boolean",
      "target": "gpio:25",
      "safe_default": true
    }
  },
  "alarms": [
    {
      "id": "exhaust_overtemp",
      "condition": "exhaust_temp_c > 80",
      "cooldown_sec": 30,
      "severity": "critical",
      "actions": ["laser_power=false", "emergency_stop=true", "notify_ops"]
    },
    {
      "id": "coolant_low",
      "condition": "coolant_flow_lpm < 2",
      "cooldown_sec": 60,
      "severity": "critical",
      "actions": ["laser_power=false", "notify_ops"]
    },
    {
      "id": "poor_air_quality",
      "condition": "air_quality_ppm > 400",
      "cooldown_sec": 30,
      "severity": "warning",
      "actions": ["exhaust_fan=true", "log"]
    }
  ],
  "metadata": {
    "location": "workshop_bay_3",
    "hardware": "rpi4+laser_cutter_v2",
    "deployment": "makerspace",
    "safety_class": "class_4_laser"
  }
}
```

## Step 2: Validate the Config

```bash
cargo run --example validate_all
```

You should see:

```
✅ laser_cutter — 4 sensors, 3 alarms
```

If there's an error, the validator tells you exactly what's wrong:

```
❌ laser_cutter: Validation failed: "exhaust_temp_c" is a required property
```

## Step 3: Load the Config from Rust

Create a small Rust program to inspect the room:

```rust
use plato_room_configs::{RoomLoader, RoomValidator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load
    let loader = RoomLoader::new(".");
    let room = loader.load_room("configs/workshop/laser_cutter.room.json")?;

    println!("Room: {} ({})", room.config.name, room.room_id);
    println!("  Tick rate: {} Hz", room.config.tick_hz);
    println!("  History capacity: {}", room.config.history_capacity);

    println!("\n  Sensors:");
    for (name, sensor) in &room.config.sensors {
        println!("    {} ({}, {})", name, sensor.sensor_type,
                 sensor.unit.as_deref().unwrap_or("—"));
    }

    println!("\n  Actuators:");
    for (name, act) in &room.config.actuators {
        println!("    {} → {} (safe: {})", name, act.target, act.safe_default);
    }

    println!("\n  Alarms:");
    for alarm in &room.config.alarms {
        println!("    [{}] {} ({})", alarm.severity, alarm.id, alarm.condition);
    }

    // Validate
    let validator = RoomValidator::from_schema_dir("schema")?;
    validator.validate_room(&room.raw_json)?;
    println!("\n  ✅ Validation passed");

    Ok(())
}
```

## Step 4: Create a Manifest

Now link rooms together into a deployment. Create `configs/workshop/workshop.manifest.json`:

```json
{
  "manifest_id": "makerspace_workshop",
  "name": "Makerspace Workshop",
  "rooms": [
    {
      "room_id": "laser_cutter",
      "config": "configs/workshop/laser_cutter.room.json"
    }
  ],
  "agents": [
    {
      "agent_id": "safety_watchdog",
      "rooms": ["*"],
      "rules": "Monitor all rooms for critical alarms. Shut down equipment and notify the workshop manager. For warnings, log and activate ventilation."
    },
    {
      "agent_id": "maintenance_tracker",
      "rooms": ["laser_cutter"],
      "rules": "Track exhaust temperature trends over time. Schedule filter replacements when temperature patterns indicate degradation."
    }
  ]
}
```

## Step 5: Load and Resolve the Manifest

```rust
use plato_room_configs::{ManifestLoader, RoomValidator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = ManifestLoader::new(".");
    let resolved = loader.resolve("configs/workshop/workshop.manifest.json")?;

    println!("Manifest: {} ({})", resolved.manifest.name, resolved.manifest.manifest_id);
    println!("  {} rooms, {} agents",
             resolved.loaded_rooms.len(),
             resolved.manifest.agents.len());

    // Show which agents watch each room
    for room in &resolved.loaded_rooms {
        let agents = plato_room_configs::manifest::agents_for_room(
            &resolved.manifest, &room.room_id
        );
        let agent_names: Vec<_> = agents.iter().map(|a| &a.agent_id).collect();
        println!("  {} → agents: {:?}", room.config.name, agent_names);
    }

    // Validate everything
    let validator = RoomValidator::from_schema_dir("schema")?;
    for room in &resolved.loaded_rooms {
        validator.validate_room(&room.raw_json)?;
    }
    println!("  ✅ All rooms validated");

    Ok(())
}
```

## What You Built

- ✅ A complete room configuration with 4 sensors, 3 actuators, and 3 alarms
- ✅ Validated against JSON Schema
- ✅ A deployment manifest linking rooms to agents
- ✅ Programmatic loading and inspection from Rust

## Next Steps

- Browse `configs/fishing-boat/` for a 6-room real-world deployment
- Read the schema files in `schema/` for the full field reference
- Create configs for your own deployment scenarios
