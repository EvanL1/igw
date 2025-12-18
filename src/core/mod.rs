//! Core abstractions for the Industrial Gateway.
//!
//! This module provides the foundational types and traits that all protocols implement.

pub mod traits;
pub mod data;
pub mod point;
pub mod quality;
pub mod error;

pub use traits::*;
pub use data::*;
pub use point::*;
pub use quality::*;
pub use error::{GatewayError, Result};
