//! Modbus TCP client demonstration.
//!
//! This example shows basic usage of the Modbus TCP client.
//!
//! Run with:
//! ```bash
//! cargo run --example modbus_demo
//! ```

use igw::prelude::*;
use igw::protocols::modbus::ModbusTcpClient;

#[tokio::main]
async fn main() -> igw::Result<()> {
    println!("=== Industrial Gateway (igw) Modbus Demo ===\n");

    // Create a Modbus TCP client
    // Note: This demo uses a placeholder address - replace with your actual device
    let address = std::env::var("MODBUS_HOST").unwrap_or_else(|_| "127.0.0.1:502".to_string());

    println!("Creating Modbus TCP client for: {}", address);
    let mut client = ModbusTcpClient::new(&address)?;

    // Show protocol capabilities
    println!("\nProtocol Capabilities:");
    println!("  Name: {}", client.name());
    println!("  Modes: {:?}", client.supported_modes());
    println!("  Supports Client: {}", client.supports_client());
    println!("  Supports Server: {}", client.supports_server());

    // Connect to the device
    println!("\nConnecting...");
    match client.connect().await {
        Ok(_) => println!("Connected successfully!"),
        Err(e) => {
            println!("Connection failed: {}", e);
            println!("(This is expected if no Modbus device is running)");
        }
    }

    // Check connection state
    println!("\nConnection State: {:?}", client.connection_state());

    // Get diagnostics
    let diag = client.diagnostics().await?;
    println!("\nDiagnostics:");
    println!("  Read Count: {}", diag.read_count);
    println!("  Write Count: {}", diag.write_count);
    println!("  Error Count: {}", diag.error_count);

    // Try to read data (will fail if not connected to a real device)
    if client.connection_state().is_connected() {
        println!("\nReading telemetry data...");
        match client.read(ReadRequest::telemetry()).await {
            Ok(response) => {
                println!("Read {} telemetry points", response.data.telemetry.len());
                for point in &response.data.telemetry {
                    println!("  {}: {:?} (Quality: {})", point.id, point.value, point.quality);
                }
            }
            Err(e) => println!("Read failed: {}", e),
        }

        // Try to write a control command
        println!("\nWriting control command...");
        let cmd = ControlCommand::latching("valve1", true);
        match client.write_control(&[cmd]).await {
            Ok(result) => println!("Write result: {} succeeded", result.success_count),
            Err(e) => println!("Write failed: {}", e),
        }
    } else {
        println!("\nSkipping read/write (not connected)");
    }

    // Disconnect
    println!("\nDisconnecting...");
    client.disconnect().await?;
    println!("Disconnected.");

    // Demonstrate data types
    println!("\n=== Data Model Demo ===\n");

    // Create some data points
    let mut batch = DataBatch::new();
    batch.add(DataPoint::telemetry("temperature", 25.5));
    batch.add(DataPoint::telemetry("pressure", 101.3));
    batch.add(DataPoint::signal("door_open", true));
    batch.add(DataPoint::signal("alarm_active", false));

    println!("Created DataBatch with {} points:", batch.len());
    println!("  Telemetry: {} points", batch.telemetry.len());
    println!("  Signal: {} points", batch.signal.len());

    for point in batch.iter() {
        println!(
            "  [{:?}] {}: {:?} ({})",
            point.data_type,
            point.id,
            point.value,
            point.quality
        );
    }

    // Demonstrate value conversions
    println!("\n=== Value Conversion Demo ===\n");

    let v = Value::Float(42.5);
    println!("Float value: {:?}", v);
    println!("  as_f64: {:?}", v.as_f64());
    println!("  as_i64: {:?}", v.as_i64());
    println!("  as_bool: {:?}", v.as_bool());

    let v = Value::Bool(true);
    println!("\nBool value: {:?}", v);
    println!("  as_f64: {:?}", v.as_f64());
    println!("  as_i64: {:?}", v.as_i64());
    println!("  as_bool: {:?}", v.as_bool());

    println!("\n=== Demo Complete ===");

    Ok(())
}
