//! J1939 Protocol Implementation
//!
//! SAE J1939 is a CAN-based protocol used in heavy-duty vehicles and industrial equipment.
//! This implementation supports:
//! - Passive listening for broadcast PGNs
//! - Active request for on-demand PGNs (Request PGN 0xEA00)
//! - Complete built-in SPN database (60+ SPNs, 12+ PGNs)
//!
//! ## Features
//!
//! - **Event-Driven**: Passively listens to CAN bus, decodes all known PGNs automatically
//! - **SPN Database**: Pre-defined SPNs covering engine/generator parameters
//! - **Point ID = SPN**: Uses globally unique SPN numbers as point identifiers
//!
//! ## Dependencies
//!
//! This module uses:
//! - [`voltage_j1939`](https://crates.io/crates/voltage_j1939) for J1939 protocol parsing
//! - [`socketcan`](https://crates.io/crates/socketcan) for CAN bus communication (Linux only)
//!
//! ## Example
//!
//! ```rust,ignore
//! use igw::protocols::can::j1939::{J1939Client, J1939Config};
//!
//! let config = J1939Config {
//!     can_interface: "can0".to_string(),
//!     source_address: 0x00,
//!     ..Default::default()
//! };
//!
//! let mut client = J1939Client::new(config);
//! client.connect().await?;
//!
//! // Subscribe to data events
//! let mut rx = client.subscribe();
//! while let Some(event) = rx.recv().await {
//!     match event {
//!         DataEvent::DataUpdate(batch) => {
//!             println!("Received {} data points", batch.len());
//!         }
//!         _ => {}
//!     }
//! }
//! ```

mod client;

// Re-export client
pub use client::{J1939Client, J1939Config};

// Re-export voltage_j1939 types for convenience
pub use voltage_j1939::{
    database_stats, decode_frame, decode_spn, get_spn_def, get_spns_for_pgn, list_supported_pgns,
    parse_can_id, DecodedSpn, J1939Id, SpnDataType, SpnDef,
};
