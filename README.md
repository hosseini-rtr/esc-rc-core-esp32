# RC Car Control System

A Rust-based RC car control system using WebSocket communication between a server and an ESP32 microcontroller.

## System Architecture

```
rc-core/
├── server/           # WebSocket server implementation
└── esp32-firmware/   # ESP32 firmware code
```

## Prerequisites

- Rust toolchain (stable)
- ESP32 development tools:
  ```bash
  cargo install espup
  espup install
  cargo install cargo-espflash
  ```
- ESP32 DevKit board
- L298N or similar motor driver
- DC motors (2x)
- Power supply (battery pack)

## Hardware Setup

### Pin Connections

Connect your ESP32 to the motor driver:

```
ESP32 Pin | Connection
----------|------------
GPIO32    | Left Motor Forward
GPIO33    | Left Motor Backward
GPIO25    | Right Motor Forward
GPIO26    | Right Motor Backward
GND       | Common Ground
```

## Software Setup

### 1. Server Setup

```bash
# Navigate to server directory
cd server

# Build the server
cargo build --release

# Run the server
cargo run --release
```

The server will start on `0.0.0.0:8080` by default.

### 2. ESP32 Firmware Setup

1. Edit WiFi credentials in `esp32-firmware/src/main.rs`:

```rust
const SSID: &str = "YourWiFiSSID";        // Replace with your WiFi name
const PASSWORD: &str = "YourWiFiPassword"; // Replace with your WiFi password
const SERVER_URL: &str = "ws://192.168.1.100:8080"; // Replace with your server IP
```

2. Build and flash:

```bash
# Navigate to ESP32 firmware directory
cd esp32-firmware

# Build the firmware
cargo build

# Flash to ESP32 (using default USB port)
cargo espflash flash

# If you need to specify a port
cargo espflash --port /dev/ttyUSB0 flash
```

## WebSocket Commands

The system uses JSON messages for communication. Here are the available commands:

### Movement Commands

1. Move Forward:

```json
{
  "Move": {
    "direction": "Forward",
    "speed": 100
  }
}
```

2. Move Backward:

```json
{
  "Move": {
    "direction": "Backward",
    "speed": 100
  }
}
```

3. Turn Left:

```json
{
  "Move": {
    "direction": "Left",
    "speed": 100
  }
}
```

4. Turn Right:

```json
{
  "Move": {
    "direction": "Right",
    "speed": 100
  }
}
```

5. Stop:

```json
"Stop"
```

6. Ping (check connection):

```json
"Ping"
```

### Response Format

The ESP32 will respond with status updates:

```json
{
  "Status": {
    "battery_level": 100,
    "connected": true,
    "current_speed": 100,
    "current_direction": "Forward"
  }
}
```

## Testing Connection

You can test the WebSocket connection using websocat:

```bash
# Install websocat
cargo install websocat

# Connect to the server
websocat ws://localhost:8080

# Send a command (example: move forward)
{"Move":{"direction":"Forward","speed":100}}
```

## Troubleshooting

1. If ESP32 fails to connect to WiFi:

   - Verify SSID and password
   - Ensure WiFi network is 2.4GHz (ESP32 doesn't support 5GHz)

2. If WebSocket connection fails:

   - Check if server IP is correct
   - Verify server is running and accessible
   - Check firewall settings

3. If motors don't respond:
   - Verify pin connections
   - Check motor driver power supply
   - Verify motor driver logic connections

## Development

### Adding New Commands

1. Add new command variants to `protocol.rs`:

```rust
pub enum RcCommand {
    Move(MotorCommand),
    Stop,
    Ping,
    // Add your new command here
}
```

2. Update the handler in both server and ESP32 firmware

## License

TBD

## Contributing

Feel free to open issues or submit pull requests for improvements.
