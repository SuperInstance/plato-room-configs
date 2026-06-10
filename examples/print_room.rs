//! Load and print a room configuration.
//!
//! Usage: cargo run --example print_room -- <path-to-room.json>

use plato_room_configs::loader::RoomLoader;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let default_path = "configs/fishing-boat/engine-room.room.json".to_string();
    let path = args.get(1).unwrap_or(&default_path);

    let loader = RoomLoader::new(".");
    let room = loader.load_room(path).unwrap_or_else(|e| {
        eprintln!("Failed to load room from '{}': {}", path, e);
        std::process::exit(1);
    });

    println!("╔══════════════════════════════════════════╗");
    println!("║  Room: {} ({})", room.config.name, room.room_id);
    println!("╠══════════════════════════════════════════╣");
    println!("║  Tick Rate:       {} Hz", room.config.tick_hz);
    println!("║  History:         {} readings", room.config.history_capacity);
    println!("║  Sensors:         {}", room.config.sensors.len());
    for (key, sensor) in &room.config.sensors {
        let unit_str = sensor.unit.as_deref().unwrap_or("-");
        println!("║    • {} [{}] ({})", key, sensor.sensor_type, unit_str);
    }
    println!("║  Actuators:       {}", room.config.actuators.len());
    for (key, actuator) in &room.config.actuators {
        println!("║    • {} [{}]", key, actuator.actuator_type);
    }
    println!("║  Alarms:          {}", room.config.alarms.len());
    for alarm in &room.config.alarms {
        println!("║    • {} [{}] ({})", alarm.id, alarm.severity, alarm.condition);
    }
    println!("║  Location:        {}", room.config.metadata.location);
    println!("║  Hardware:        {}", room.config.metadata.hardware.as_deref().unwrap_or("N/A"));
    println!("║  Deployment:      {}", room.config.metadata.deployment);
    println!("╚══════════════════════════════════════════╝");
}
