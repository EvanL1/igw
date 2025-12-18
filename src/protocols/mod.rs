//! Protocol implementations.
//!
//! Each protocol is behind a feature flag to minimize dependencies.

#[cfg(feature = "modbus")]
#[cfg_attr(docsrs, doc(cfg(feature = "modbus")))]
pub mod modbus;

#[cfg(feature = "modbus")]
pub use modbus::*;
