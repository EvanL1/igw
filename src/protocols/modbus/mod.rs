//! Modbus protocol implementation.
//!
//! Supports both Modbus TCP and Modbus RTU (with `serial` feature).
//!
//! # Example
//!
//! ```rust,ignore
//! use igw::prelude::*;
//! use igw::protocols::modbus::ModbusTcpClient;
//!
//! let mut client = ModbusTcpClient::builder()
//!     .address("192.168.1.100:502")
//!     .timeout(Duration::from_secs(5))
//!     .build()?;
//!
//! client.connect().await?;
//!
//! // Read holding registers
//! let response = client.read(ReadRequest::telemetry()).await?;
//! ```

mod client;
mod types;

pub use client::*;
pub use types::*;
