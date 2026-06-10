# Plato Room Configuration System

> Production-ready room configurations for real-world deployments.

A **Plato Room** is a declarative configuration that defines the sensors, actuators, alarms, and metadata for a physical or virtual space. Each room is a single `.room.json` file that tells the Plato Engine everything it needs to monitor and control that space.

This repository contains:

- **5 complete deployment examples** spanning fishing boats, server racks, smart homes, game worlds, and factories
- **JSON Schemas** for validating all configuration files
- **A Rust crate** (`plato-room-configs`) for loading, parsing, and validating configs programmatically
- **18 room configs** and **5 manifests** covering diverse real-world scenarios

---

## Table of Contents

1. [What Are Plato Room Configs?](#what-are-plato-room-configs)
2. [Quick Start](#quick-start)
3. [Schema Reference](#schema-reference)
4. [How to Write a Room Config](#how-to-write-a-room-config)
5. [Deployment Examples](#deployment-examples)
6. [Configuration Best Practices](#configuration-best-practices)
7. [Integration with PlatoEngine](#integration-with-platoengine)
8. [Rust Crate API](#rust-crate-api)
9. [Running Tests](#running-tests)

---

## What Are Plato Room Configs?

A **Plato Room** models a real space — an engine room on a boat, a server rack in a data center, a kitchen in a smart home, or even a virtual dungeon in a game world. Each room config declares:

- **Sensors**: What data to collect and where it comes from (MQTT, GPIO, Modbus, BACnet, SNMP, Zigbee, etc.)
- **Actuators**: What controls are available and their safe defaults
- **Alarms**: What conditions trigger responses and what actions to take
- **Metadata**: Location, hardware, and deployment context

A **Manifest** links rooms together into a deployment and defines agents that operate across rooms with natural-language rules.

### Why Declarative?

Room configs separate **what** from **how**:

- The config says "monitor coolant temperature, alarm above 95°C"
- The Plato Engine handles polling, buffering, evaluation, and response

This means:
- **Ops teams** can modify configs without touching code
- **Configs are version-controlled** alongside application code
- **Validation is automatic** via JSON Schema
- **New deployments** are copy-paste-modify from examples

---

## Quick Start

### Validate all configs

```bash
cargo run --example validate_all
```

### Print a room config

```bash
cargo run --example print_room -- configs/fishing-boat/engine-room.room.json
```

### Run the test suite

```bash
cargo test
```

### Use in your Rust project

```toml
[dependencies]
plato-room-configs = { git = "https://github.com/SuperInstance/plato-room-configs" }
```

```rust
use plato_room_configs::{RoomLoader, RoomValidator, ManifestLoader};

let validator = RoomValidator::from_schema_dir("schema")?;
let loader = RoomLoader::new(".");

for room in loader.load_all_rooms()? {
    validator.validate_room(&room.raw_json)?;
    println!("✅ {} — {} sensors, {} alarms",
        room.room_id,
        room.config.sensors.len(),
        room.config.alarms.len()
    );
}
```

---

## Schema Reference

### Room Configuration (.room.json)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `room_id` | string | ✅ | Unique snake_case identifier |
| `name` | string | ✅ | Human-readable room name |
| `tick_hz` | number | ✅ | Sensor polling frequency in Hz (0 < x ≤ 100) |
| `history_capacity` | integer | ✅ | Ring buffer size for historical readings |
| `sensors` | object | ✅ | Map of sensor key → sensor definition |
| `actuators` | object | ✅ | Map of actuator key → actuator definition |
| `alarms` | array | ✅ | List of alarm rules |
| `metadata` | object | ✅ | Location, hardware, and deployment info |

#### Sensor Definition

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | One of: `float`, `integer`, `boolean`, `string` |
| `unit` | string\|null | ❌ | Unit of measurement (e.g., `°C`, `psi`, `kts`) |
| `source` | string | ✅ | Data source in `protocol:path` format |
| `range` | [min, max]\|null | ❌ | Valid operating range |
| `alarm_range` | [min, max]\|null | ❌ | Range that indicates alarm condition |

**Supported source protocols:**
`mqtt`, `gpio`, `pwm`, `adc`, `i2c`, `spi`, `modbus`, `bacnet`, `snmp`, `ipmi`, `zigbee`, `game`

#### Actuator Definition

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | One of: `boolean`, `float`, `integer` |
| `target` | string | ✅ | Control target in `protocol:path` format |
| `range` | [min, max]\|null | ❌ | Valid control range for numeric types |
| `safe_default` | boolean\|number | ✅ | Safe state when control is lost |

#### Alarm Rule

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | ✅ | Unique snake_case alarm identifier |
| `condition` | string | ✅ | Boolean expression referencing sensor keys |
| `cooldown_sec` | integer | ✅ | Min seconds between repeated triggers |
| `severity` | string | ✅ | One of: `info`, `warning`, `critical` |
| `actions` | string[] | ✅ | List of actions to execute |

**Condition syntax:** Sensor keys referenced by name with comparison (`>`, `<`, `>=`, `<=`, `==`, `!=`) and logical (`and`, `or`) operators.

Example: `water_level_aft_cm > 70 or water_level_fwd_cm > 70`

### Manifest (.manifest.json)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `manifest_id` | string | ✅ | Unique deployment identifier |
| `name` | string | ✅ | Human-readable deployment name |
| `rooms` | array | ✅ | List of `{room_id, config}` references |
| `agents` | array | ✅ | List of agent definitions |

#### Agent Definition

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agent_id` | string | ✅ | Agent identifier |
| `rooms` | string[] | ✅ | Room IDs or `["*"]` for all |
| `rules` | string | ✅ | Natural-language behavior rules |

---

## How to Write a Room Config

### 1. Start with the basics

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
    "location": "building_a_floor_1",
    "deployment": "my-project"
  }
}
```

### 2. Add sensors

Think about what you need to measure and where the data comes from:

```json
"temperature_c": {
  "type": "float",
  "unit": "°C",
  "source": "mqtt:esp32/room/temp",
  "range": [-10, 60],
  "alarm_range": [40, 60]
}
```

- **`source`** tells the engine where to read from — MQTT topic, GPIO pin, Modbus register, etc.
- **`range`** defines the valid operating envelope — values outside this are logged as anomalous
- **`alarm_range`** is a hint for alarm conditions, but actual alarm logic lives in the `alarms` array

### 3. Add actuators

Define what you can control:

```json
"heater": {
  "type": "boolean",
  "target": "gpio:17",
  "safe_default": false
}
```

**`safe_default`** is critical — it's the state the system reverts to if control is lost. For safety:
- Dangerous things default to **off** (pumps, valves, motors)
- Safety things default to **on** (alarms, nav lights, ventilation)

### 4. Add alarms

Wire sensor conditions to automated responses:

```json
{
  "id": "overheat",
  "condition": "temperature_c > 40",
  "cooldown_sec": 60,
  "severity": "warning",
  "actions": ["notify_ops", "log", "activate_cooling"]
}
```

**Severity guidelines:**
- `info` — Log only, no notification (stock low, routine events)
- `warning` — Notify relevant people, investigate soon (high temp, moderate humidity)
- `critical` — Immediate notification + automated mitigation (fire, flood, overpressure)

**Cooldown** prevents alarm fatigue — set higher for warnings, lower for criticals.

### 5. Validate

Always validate against the schema:

```bash
cargo run --example validate_all
```

---

## Deployment Examples

### 🚢 Fishing Boat — F/V Northern Star

**6 rooms** | `configs/fishing-boat/`

A complete fishing vessel monitoring system with engine room, backdeck, wheelhouse, crow's nest, galley, and bilge. Uses ESP32 and Raspberry Pi hardware with MQTT and GPIO protocols.

Key features:
- **Engine monitoring**: Coolant temp, oil pressure, RPM, exhaust temp with critical alarms
- **Bilge system**: Dual water level sensors with automatic pump activation
- **Navigation**: GPS, depth sounder, autopilot with shallow water alarms
- **Safety**: CO detection in galley, winch overload on backdeck, visibility monitoring

### 🖥️ Server Rack — DC West Rack A14

**3 rooms** | `configs/server-rack/`

Data center rack monitoring covering servers, network switches, and PDUs. Uses IPMI, SNMP protocols.

Key features:
- **Server health**: CPU temp, RAM/disk usage with fan boost and power cycle control
- **Network**: Switch temperature, error rate monitoring
- **Power**: PDU load, voltage monitoring with overload protection and brownout detection

### 🏠 Smart Home

**3 rooms** | `configs/smart-home/`

Residential automation with living room, kitchen, and garage. Uses Zigbee protocol throughout.

Key features:
- **Climate**: Temperature, humidity with thermostat and blind control
- **Safety**: Smoke/CO detection with automatic gas shutoff
- **Security**: Motion detection, garage door monitoring, overnight alerts

### 🎮 Game World — Realm of Eldoria

**3 rooms** | `configs/game-world/`

Virtual game environment monitoring for a tavern, dungeon, and forest. Uses custom `game:` protocol.

Key features:
- **Tavern**: Patron count, ale stock, brawl detection with guard summoning
- **Dungeon**: Boss HP, torch light, trap states with enrage timers
- **Forest**: Day/night cycle, hostile creatures, weather events

### 🏭 Factory — Acme Manufacturing Plant

**3 rooms** | `configs/factory/`

Industrial automation with production line, warehouse, and HVAC plant. Uses Modbus and BACnet protocols.

Key features:
- **Production**: Speed, defect rate, vibration monitoring with emergency stop
- **Warehouse**: Pallet count, climate control with sprinkler system
- **HVAC**: Chiller monitoring, supply temperature, filter differential pressure

---

## Configuration Best Practices

### Sensor Design

1. **Name sensors clearly**: `coolant_temp_c` beats `temp1` — future you will thank you
2. **Always specify units**: Even if "obvious" — `°C`, `psi`, `kts`, `ppm`
3. **Set realistic ranges**: Too narrow = false anomalies; too wide = missed problems
4. **One source per sensor**: If you need the same measurement from two places, use two sensors

### Actuator Safety

1. **Set `safe_default` for everything**: Never leave an actuator without a fallback
2. **Dangerous things default off**: Motors, heaters, valves, weapons
3. **Safety things default on**: Alarms, lights, ventilation, circuit breakers
4. **Boolean is safer than float**: If a simple on/off works, prefer it over analog control

### Alarm Design

1. **Cooldown prevents spam**: Set cooldowns to avoid drowning operators in repeated alerts
   - Critical: 5–30 seconds
   - Warning: 60–300 seconds
   - Info: 300–3600 seconds
2. **Severity honestly**: Over-alerting severity erodes trust. Reserve `critical` for actual emergencies.
3. **Actions should be automated**: If an alarm always triggers the same response, automate it
4. **Document unusual conditions**: Complex conditions should have comments (in the metadata)

### Tick Rate Selection

| Scenario | Recommended `tick_hz` | Rationale |
|----------|-----------------------|-----------|
| Safety-critical (engine, fire) | 1.0–2.0 | Fast detection saves lives |
| Environmental monitoring | 0.1–0.5 | Temperature changes slowly |
| Game world events | 0.5–2.0 | Responsive to player actions |
| Industrial control | 1.0–5.0 | Process control needs speed |
| Warehouse/logging | 0.1–0.2 | Low urgency, save bandwidth |

### Manifest Organization

1. **One manifest per deployment**: Don't mix different physical sites
2. **Use wildcard agents sparingly**: `["*"]` is convenient but consider if an agent really needs all rooms
3. **Rules are enforced by the engine**: Write them clearly — they're your operational policy

### File Organization

```
configs/
  <deployment-name>/
    <room-name>.room.json    — one file per room
    <deployment>.manifest.json — one manifest linking rooms
```

---

## Integration with PlatoEngine

The `plato-room-configs` crate is designed to work seamlessly with the `plato-engine-block` Rust crate:

```rust
use plato_engine_block::PlatoEngine;
use plato_room_configs::{RoomLoader, ManifestLoader};

// Load a deployment manifest
let manifest = ManifestLoader::new(".")
    .resolve("configs/fishing-boat/boat.manifest.json")?;

// Build engine from manifest
let mut engine = PlatoEngine::builder()
    .manifest(manifest.manifest)
    .rooms(manifest.loaded_rooms)
    .build()?;

// Run the engine
engine.run()?;
```

### How it fits together:

1. **`plato-room-configs`** — Defines what to monitor (declarative)
2. **`plato-engine-block`** — Executes the monitoring (runtime)
3. **Your application** — Wires it together and adds domain logic

The config system is intentionally independent — you can use room configs without the Plato Engine if you just need structured IoT configuration.

---

## Rust Crate API

### `RoomLoader`

```rust
let loader = RoomLoader::new("/path/to/repo");

// Load a single room
let room = loader.load_room("configs/fishing-boat/engine-room.room.json")?;

// Load all rooms in the repo
let all_rooms = loader.load_all_rooms()?;

// Load rooms from a subdirectory
let boat_rooms = loader.load_rooms_from_dir("configs/fishing-boat")?;
```

### `RoomValidator`

```rust
let validator = RoomValidator::from_schema_dir("schema")?;

// Validate from parsed JSON
validator.validate_room(&json_value)?;
validator.validate_manifest(&json_value)?;

// Validate from file
validator.validate_room_file(Path::new("configs/.../room.room.json"))?;
```

### `ManifestLoader`

```rust
let loader = ManifestLoader::new("/path/to/repo");

// Load and resolve (loads all referenced rooms)
let resolved = loader.resolve("configs/fishing-boat/boat.manifest.json")?;

// Access loaded rooms
for room in &resolved.loaded_rooms {
    println!("{}: {} sensors", room.room_id, room.config.sensors.len());
}
```

---

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test test_validator
cargo test --test test_loader
cargo test --test test_manifest
cargo test --test test_all_configs

# Run with output
cargo test -- --nocapture

# Validate all configs in the repo
cargo run --example validate_all
```

---

## License

MIT
