//! Data routing module for point mapping and protocol conversion.
//!
//! This module provides data routing capabilities:
//! - Point-to-point mapping between channels
//! - Data transformation during routing
//! - Conditional forwarding (on-change, threshold, interval)
//!
//! # Example
//!
//! ```rust,ignore
//! use igw::router::{DataRouter, RouterConfig, PointMapping, RoutingTable};
//!
//! // Create routing table
//! let mut table = RoutingTable::new();
//! table.add(PointMapping::direct(1, "temp", 2, "temp_104"));
//!
//! // Create router
//! let config = RouterConfig { routing_table: table, ..Default::default() };
//! let mut router = DataRouter::new(config, store);
//! router.start().await?;
//! ```

mod data_router;
mod mapping;

pub use data_router::{DataRouter, RouterConfig, TargetWriter};
pub use mapping::{PointMapping, RoutingTable, TriggerCondition};
