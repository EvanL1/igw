//! J1939 Protocol Implementation
//!
//! SAE J1939 is a CAN-based protocol used in heavy-duty vehicles and industrial equipment.
//! This implementation supports:
//! - Passive listening for broadcast PGNs
//! - Active request for on-demand PGNs (Request PGN 0xEA00)
//! - Complete built-in SPN database (88 SPNs, 16 PGNs)
//!
//! ## Features
//!
//! - **Event-Driven**: Passively listens to CAN bus, decodes all known PGNs automatically
//! - **SPN Database**: 88 pre-defined SPNs covering engine/generator parameters
//! - **Point ID = SPN**: Uses globally unique SPN numbers as point identifiers
//!
//! ## Example
//!
//! ```rust,ignore
//! use igw::protocols::j1939::{J1939Client, J1939Config};
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

mod database;
mod client;

pub use database::{SpnDataType, SpnDef, SPN_DEFINITIONS, get_spn_def, get_spns_for_pgn, database_stats};
pub use client::{J1939Client, J1939Config};
