# Microbit Project

This is a Rust-based project for BBC micro:bit development, focusing on low-level hardware communication implementations.

## Project Structure

```
├── application/
│   └── uwb/            # UART communication implementation
├── microbit-lib/       # Core library for micro:bit functionality
├── microbit-cutebot/   # Cutebot robot control library
└── memory.x           # Linker script for memory layout
```

## Features

### UART Communication (UWB Application)

- Implements basic UART communication with 115200 baud rate
- Echo functionality: receives and transmits back characters
- Uses pins P0.02 (RX) and P0.03 (TX) for UART communication

### Cutebot Robot Control

- Motor control functionality with variable speed settings
- RGB LED control (left and right) with customizable colors
- Uses internal I2C bus (pins P0.08 (SCL) and P0.16 (SDA)) for communication
- Integrated with UART communication for status reporting

## Setup

1. Install Rust and add the target and add tools:
   ```bash
   rustup target add thumbv7em-none-eabihf
   cargo install probe-rs-tools flip-link
   ```

2. Build the uwb application of project:
   ```bash
   cargo build --release --manifest-path ./application/uwb/Cargo.toml --target thumbv7em-none-eabihf
   ```

3. Run the uwb application of project:
   ```bash
   cargo run --release --manifest-path ./application/uwb/Cargo.toml --target thumbv7em-none-eabihf
   ```

This will start the UART communication application with:
- 115200 baud rate
- Echo functionality enabled
- Using pins P0.02 (RX) and P0.03 (TX)

## Development

The project uses:
- `no_std` for bare metal development
- Cortex-M RT for microcontroller support
- Custom HAL implementation for micro:bit v2

## License

This project is open source and available under the MIT License.