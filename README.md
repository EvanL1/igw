# Industrial Gateway (igw)

A universal SCADA protocol library for Rust, providing unified abstractions for industrial communication protocols.

## Features

- **Protocol Agnostic**: Unified four-remote (T/S/C/A) data model
- **Dual Mode Support**: Polling and event-driven communication
- **Zero Business Coupling**: Pure protocol layer, no business logic dependencies
- **Modular Design**: Protocol implementations in separate crates (pluggable)

## Supported Protocols

Protocol implementations are in separate crates:

| Protocol | Crate | Status |
|----------|-------|--------|
| Modbus TCP/RTU | `voltage_modbus` | Available |
| IEC 60870-5-104 | `voltage_iec104` | Planned |
| DNP3 | `voltage_dnp3` | Planned |
| OPC UA | `voltage_opcua` | Planned |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
igw = "0.1"                    # Core traits and data model
voltage_modbus = "0.1"         # Modbus support (optional)
# voltage_iec104 = "0.1"       # IEC 104 support (optional)
```

## Quick Start

```rust
use igw::prelude::*;
use voltage_modbus::ModbusTcpClient;  // Protocol from separate crate

#[tokio::main]
async fn main() -> igw::Result<()> {
    // Create a Modbus TCP client (implements igw::ProtocolClient)
    let mut client = ModbusTcpClient::new("192.168.1.100:502")?;

    // Connect to the device
    client.connect().await?;

    // Read telemetry data
    let response = client.read(ReadRequest::telemetry()).await?;

    for point in response.data.telemetry {
        println!("{}: {:?}", point.id, point.value);
    }

    // Disconnect
    client.disconnect().await?;

    Ok(())
}
```

## Data Model

The library uses the "Four Remotes" concept common in SCADA systems:

| Type | Code | Direction | Description |
|------|------|-----------|-------------|
| Telemetry | T | Input | Analog measurements |
| Signal | S | Input | Digital status |
| Control | C | Output | Digital commands |
| Adjustment | A | Output | Analog setpoints |

## Protocol Traits

### `Protocol` (Base)

```rust
pub trait Protocol: Send + Sync {
    fn connection_state(&self) -> ConnectionState;
    async fn read(&self, request: ReadRequest) -> Result<ReadResponse>;
    async fn diagnostics(&self) -> Result<Diagnostics>;
}
```

### `ProtocolClient` (Active Connection)

```rust
pub trait ProtocolClient: Protocol {
    async fn connect(&mut self) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn write_control(&mut self, commands: &[ControlCommand]) -> Result<WriteResult>;
    async fn write_adjustment(&mut self, adjustments: &[AdjustmentCommand]) -> Result<WriteResult>;
    async fn start_polling(&mut self, config: PollingConfig) -> Result<()>;
    async fn stop_polling(&mut self) -> Result<()>;
}
```

### `EventDrivenProtocol` (For IEC 104, OPC UA)

```rust
pub trait EventDrivenProtocol: Protocol {
    fn subscribe(&self) -> DataEventReceiver;
    fn set_event_handler(&mut self, handler: Arc<dyn DataEventHandler>);
}
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
