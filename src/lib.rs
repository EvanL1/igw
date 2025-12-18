//! # Industrial Gateway (igw)
//!
//! A universal SCADA protocol library for Rust, providing unified abstractions
//! for industrial communication protocols.
//!
//! ## Features
//!
//! - **Protocol Agnostic**: Unified four-remote (T/S/C/A) data model
//! - **Dual Mode Support**: Polling and event-driven communication
//! - **Zero Business Coupling**: Pure protocol layer, no business logic
//! - **Feature Gated**: Compile only what you need
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use igw::prelude::*;
//!
//! // Create a Modbus TCP client
//! let mut client = ModbusTcpClient::new("192.168.1.100:502")?;
//! client.connect().await?;
//!
//! // Read telemetry data
//! let response = client.read(ReadRequest::telemetry(vec![1, 2, 3])).await?;
//! ```
//!
//! ## Supported Protocols
//!
//! | Protocol | Feature Flag | Status |
//! |----------|--------------|--------|
//! | Modbus TCP/RTU | `modbus` | Implemented |
//! | IEC 60870-5-104 | `iec104` | Planned |
//! | DNP3 | `dnp3` | Planned |
//! | OPC UA | `opcua` | Planned |

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod core;
pub mod codec;

#[cfg(feature = "modbus")]
#[cfg_attr(docsrs, doc(cfg(feature = "modbus")))]
pub mod protocols;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::core::{
        traits::*,
        data::*,
        point::*,
        quality::*,
        error::{GatewayError, Result},
    };

    #[cfg(feature = "modbus")]
    pub use crate::protocols::modbus::*;
}

// Re-export core types at crate root for convenience
pub use crate::core::error::{GatewayError, Result};
pub use crate::core::data::{Value, DataType, DataPoint, DataBatch};
pub use crate::core::quality::Quality;
pub use crate::core::traits::{
    Protocol, ProtocolClient, ProtocolCapabilities,
    CommunicationMode, ConnectionState,
};
