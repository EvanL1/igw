//! IGW CLI Entry Point
//!
//! Industrial Gateway command-line interface.

use clap::Parser;
use std::path::PathBuf;

/// Industrial Gateway - Universal SCADA Protocol Gateway
#[derive(Parser, Debug)]
#[command(name = "igw", version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        println!("IGW v{}", env!("CARGO_PKG_VERSION"));
        println!("Config: {:?}", args.config);
    }

    run_headless(&args);
}

fn run_headless(args: &Args) {
    println!("IGW Headless Mode");
    println!("Config: {:?}", args.config);

    // TODO: Load config and start gateway
    if !args.config.exists() {
        eprintln!("Warning: Config file not found: {:?}", args.config);
        eprintln!("Creating example config...");
        create_example_config(&args.config);
    }

    // Start runtime
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        println!("Gateway started. Press Ctrl+C to stop.");
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c");
        println!("Shutting down...");
    });
}

fn create_example_config(path: &PathBuf) {
    let example = r#"# IGW Configuration Example

[gateway]
name = "Industrial Gateway"

# Modbus TCP Channel Example
# [[channels]]
# id = 1
# name = "Modbus PLC"
# protocol = "modbus-tcp"
# address = "192.168.1.100:502"
#
# [[channels.points]]
# id = "temperature"
# data_type = "telemetry"
# address = { slave_id = 1, function_code = 3, register = 100 }

# IEC 104 Channel Example
# [[channels]]
# id = 2
# name = "IEC 104 RTU"
# protocol = "iec104"
# address = "192.168.1.200:2404"
"#;

    if let Err(e) = std::fs::write(path, example) {
        eprintln!("Failed to create config: {}", e);
    } else {
        println!("Example config created: {:?}", path);
    }
}
