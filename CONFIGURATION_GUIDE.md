# Configuration Guide — Writing Plato Room Configs

This guide walks through everything you need to know to write, test, and deploy Plato room configurations.

---

## File Structure

Every room is a single `.room.json` file. Every deployment has a `.manifest.json` that links rooms together.

```
configs/
  my-deployment/
    bridge.room.json        — a room
    engine.room.json        — another room
    my-deployment.manifest.json — links rooms + defines agents
```

---

## Minimal Room Config

The absolute minimum valid room config:

```json
{
  "room_id": "minimal_room",
  "name": "Minimal Room",
  "tick_hz": 1.0,
  "history_capacity": 100,
  "sensors": {
    "status": {
      "type": "boolean",
      "source": "gpio:1"
    }
  },
  "actuators": {},
  "alarms": [],
  "metadata": {
    "location": "test",
    "deployment": "test"
  }
}
```

Every room needs at least one sensor. Actuators and alarms are optional.

---

## Sensor Sources

The `source` field uses a `protocol:path` format. Choose the right protocol for your hardware:

| Protocol | Example | Use When |
|----------|---------|----------|
| `mqtt` | `mqtt:esp32/engine/coolant` | Wireless/remote sensors via MQTT broker |
| `gpio` | `gpio:17` | Direct GPIO pin on Raspberry Pi, ESP32 |
| `pwm` | `pwm:0` | PWM-controlled devices |
| `adc` | `adc:0` | Analog-to-digital converter channels |
| `i2c` | `i2c:0x48/temp` | I2C bus devices |
| `spi` | `spi:0/pressure` | SPI bus devices |
| `modbus` | `modbus:register/40001` | Industrial Modbus TCP/RTU |
| `bacnet` | `bacnet:analog-input/1` | Building automation (HVAC) |
| `snmp` | `snmp:ifSpeed.1` | Network equipment monitoring |
| `ipmi` | `ipmi:sensor/cpu-temp` | Server hardware monitoring |
| `zigbee` | `zigbee:sensor/room-temp` | Zigbee mesh networks (smart home) |
| `game` | `game:entity-count/tag:hostile` | Virtual/game world entities |

### Choosing Range vs Alarm Range

- **`range`**: The physically possible range of the sensor. Values outside this are anomalous.
- **`alarm_range`**: The subset of the range that's concerning. Use this for documentation; actual alarm logic goes in the `alarms` array.

Example — coolant temperature:
```json
"range": [0, 120],       // Sensor can read 0–120°C
"alarm_range": [95, 120] // Anything above 95°C is concerning
```

For boolean sensors, both fields are `null`.

---

## Actuator Safe Defaults

The `safe_default` is the fallback state when communication is lost. This is your last line of defense.

### Rules of Thumb

| Actuator | Safe Default | Reasoning |
|----------|-------------|-----------|
| Heater | `false` (off) | Uncontrolled heating is dangerous |
| Pump | `false` (off) | Uncontrolled pumping wastes energy, can overflow |
| Alarm/siren | `true` (on) | If in doubt, alert humans |
| Ventilation | `true` (on) | Stale air is worse than fresh air |
| Brake | `true` (engaged) | Stopped is safer than moving |
| Throttle | `0.5` (mid) | Neutral position for analog controls |
| Gas valve | `false` (closed) | Gas leaks are catastrophic |
| Emergency stop | `true` (engaged) | Fail-safe means stop everything |

### Numeric Actuators

For `float` or `integer` actuators, provide both a `range` and a `safe_default`:

```json
"throttle": {
  "type": "float",
  "target": "pwm:0",
  "range": [0, 1],
  "safe_default": 0.0
}
```

---

## Alarm Conditions

Conditions are simple boolean expressions that reference sensor keys:

### Simple comparisons
```
coolant_temp_c > 95
oil_pressure_psi < 15
smoke_ppm > 100
```

### Compound with `and`/`or`
```
temperature_c > 35 and humidity_pct > 80
water_level_aft_cm > 70 or water_level_fwd_cm > 70
```

### Boolean sensors
```
brawl_active == true
stove_active == true
```

### Setting Cooldowns

Cooldowns prevent alarm storms. A value of `0` means it fires every tick (use sparingly).

| Severity | Typical Cooldown | Example |
|----------|-----------------|---------|
| `critical` | 5–30 seconds | Engine overheat, CO detected, flooding |
| `warning` | 60–300 seconds | High temperature, high humidity, low stock |
| `info` | 300–3600 seconds | Log entries, routine notifications |

---

## Manifests

Manifests tie rooms into a deployment and define cross-room agents:

```json
{
  "manifest_id": "my-boat",
  "name": "My Vessel",
  "rooms": [
    {"room_id": "engine_room", "config": "configs/my-boat/engine.room.json"},
    {"room_id": "bridge", "config": "configs/my-boat/bridge.room.json"}
  ],
  "agents": [
    {
      "agent_id": "watchdog",
      "rooms": ["*"],
      "rules": "escalate any unacknowledged critical alarm within 60 seconds"
    }
  ]
}
```

### Agent Room Assignment

- `["*"]` — Agent operates on all rooms in the deployment
- `["engine_room", "bilge"]` — Agent only sees those specific rooms

### Agent Rules

Rules are natural-language descriptions of agent behavior. The Plato Engine interprets these:

- `"escalate any alarm unacknowledged for 60s"` — Time-based escalation
- `"cross-reference coolant temp with bilge levels for leak detection"` — Cross-sensor correlation
- `"optimize production throughput while maintaining safety constraints"` — Optimization with guardrails

---

## Validation

Always validate configs before deploying:

```bash
# Validate everything
cargo run --example validate_all

# Programmatic validation
cargo test --test test_all_configs
```

The JSON Schema catches:
- Missing required fields
- Invalid room_id formats (must be snake_case)
- Out-of-range tick_hz values
- Invalid source protocol formats
- Missing safe defaults on actuators

---

## Common Patterns

### Safety Shutdown Chain

For critical systems, chain sensors → alarms → actuator shutdown:

```json
{
  "sensors": {
    "temp_c": {"type": "float", "source": "mqtt:dev/temp", "range": [0, 120]}
  },
  "actuators": {
    "main_power": {"type": "boolean", "target": "gpio:17", "safe_default": false}
  },
  "alarms": [
    {
      "id": "overheat",
      "condition": "temp_c > 100",
      "cooldown_sec": 5,
      "severity": "critical",
      "actions": ["emergency_shutdown", "notify_ops", "log"]
    }
  ]
}
```

### Multi-Condition Alarm

Use `or` for redundant sensors measuring the same thing:

```json
{
  "id": "flooding",
  "condition": "water_aft_cm > 70 or water_fwd_cm > 70",
  "cooldown_sec": 10,
  "severity": "critical",
  "actions": ["activate_emergency_pump", "notify_captain", "sound_alarm"]
}
```

### Graduated Alarms

Stack alarms at different thresholds for the same sensor:

```json
{
  "id": "temp_warning",
  "condition": "temp_c > 80",
  "cooldown_sec": 300,
  "severity": "warning",
  "actions": ["notify_ops", "log"]
},
{
  "id": "temp_critical",
  "condition": "temp_c > 100",
  "cooldown_sec": 10,
  "severity": "critical",
  "actions": ["emergency_shutdown", "notify_ops", "sound_alarm"]
}
```

---

## Checklist Before Deploying

- [ ] All sensors have realistic ranges
- [ ] All actuators have safe defaults
- [ ] Alarm cooldowns prevent spam
- [ ] Critical alarms have automated actions
- [ ] `room_id` matches the filename convention
- [ ] Manifest references correct file paths
- [ ] `cargo run --example validate_all` passes with zero failures
- [ ] `cargo test` passes
