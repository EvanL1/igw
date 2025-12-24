//! Data types for the Industrial Gateway.
//!
//! This module defines the core data model based on the "Four Remotes" (四遥) concept:
//! - **Telemetry (T)**: Analog input values (遥测)
//! - **Signal (S)**: Digital input status (遥信)
//! - **Control (C)**: Digital output commands (遥控)
//! - **Adjustment (A)**: Analog output setpoints (遥调)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::core::quality::Quality;

/// The four types of remote data in SCADA systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    /// Telemetry - Analog input measurement (遥测)
    ///
    /// Examples: temperature, pressure, power, current
    Telemetry,

    /// Signal - Digital input status (遥信)
    ///
    /// Examples: switch position, alarm status, door open/closed
    Signal,

    /// Control - Digital output command (遥控)
    ///
    /// Examples: start/stop motor, open/close valve
    Control,

    /// Adjustment - Analog output setpoint (遥调)
    ///
    /// Examples: power setpoint, temperature target, speed reference
    Adjustment,
}

impl DataType {
    /// Get the short code for this data type.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Telemetry => "T",
            Self::Signal => "S",
            Self::Control => "C",
            Self::Adjustment => "A",
        }
    }

    /// Check if this is an input type (device → system).
    #[inline]
    pub fn is_input(&self) -> bool {
        matches!(self, Self::Telemetry | Self::Signal)
    }

    /// Check if this is an output type (system → device).
    #[inline]
    pub fn is_output(&self) -> bool {
        matches!(self, Self::Control | Self::Adjustment)
    }

    /// Check if this is an analog type.
    #[inline]
    pub fn is_analog(&self) -> bool {
        matches!(self, Self::Telemetry | Self::Adjustment)
    }

    /// Check if this is a digital type.
    #[inline]
    pub fn is_digital(&self) -> bool {
        matches!(self, Self::Signal | Self::Control)
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A protocol-agnostic value representation.
///
/// This enum provides a unified way to represent values from different protocols.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// Floating-point number (most common for telemetry/adjustment)
    Float(f64),

    /// Integer value
    Integer(i64),

    /// Boolean value (common for signals/controls)
    Bool(bool),

    /// String value
    String(String),

    /// Raw bytes
    Bytes(Vec<u8>),

    /// Null/missing value
    #[default]
    Null,
}

impl Value {
    /// Try to get the value as f64.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            Self::Integer(v) => Some(*v as f64),
            Self::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Try to get the value as i64.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(v) => Some(*v),
            Self::Float(v) => Some(*v as i64),
            Self::Bool(v) => Some(if *v { 1 } else { 0 }),
            _ => None,
        }
    }

    /// Try to get the value as bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            Self::Integer(v) => Some(*v != 0),
            Self::Float(v) => Some(*v != 0.0),
            _ => None,
        }
    }

    /// Try to get the value as string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Check if this is a null value.
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

// Convenient From implementations
impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Self::Float(v as f64)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Self::Integer(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Self::Integer(v as i64)
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Self::Integer(v as i64)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Self::Integer(v as i64)
    }
}

impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Self::Integer(v as i64)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

/// A single data point with timestamp and quality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// Point identifier (numeric, application-level ID)
    pub id: u32,

    /// Data type (T/S/C/A)
    pub data_type: DataType,

    /// The value
    pub value: Value,

    /// Data quality indicator
    #[serde(default)]
    pub quality: Quality,

    /// Server timestamp (when gateway received the data)
    pub timestamp: DateTime<Utc>,

    /// Source timestamp (when device generated the data, if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_timestamp: Option<DateTime<Utc>>,
}

impl DataPoint {
    /// Create a new data point with current timestamp.
    pub fn new(id: u32, data_type: DataType, value: impl Into<Value>) -> Self {
        Self {
            id,
            data_type,
            value: value.into(),
            quality: Quality::Good,
            timestamp: Utc::now(),
            source_timestamp: None,
        }
    }

    /// Create a telemetry data point.
    pub fn telemetry(id: u32, value: impl Into<Value>) -> Self {
        Self::new(id, DataType::Telemetry, value)
    }

    /// Create a signal data point.
    pub fn signal(id: u32, value: bool) -> Self {
        Self::new(id, DataType::Signal, value)
    }

    /// Create a control data point.
    pub fn control(id: u32, value: bool) -> Self {
        Self::new(id, DataType::Control, value)
    }

    /// Create an adjustment data point.
    pub fn adjustment(id: u32, value: impl Into<Value>) -> Self {
        Self::new(id, DataType::Adjustment, value)
    }

    /// Set the quality.
    #[must_use]
    pub fn with_quality(mut self, quality: Quality) -> Self {
        self.quality = quality;
        self
    }

    /// Set the source timestamp.
    #[must_use]
    pub fn with_source_timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.source_timestamp = Some(ts);
        self
    }
}

/// A batch of data points, organized by type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataBatch {
    /// Telemetry points
    pub telemetry: Vec<DataPoint>,

    /// Signal points
    pub signal: Vec<DataPoint>,

    /// Control points
    pub control: Vec<DataPoint>,

    /// Adjustment points
    pub adjustment: Vec<DataPoint>,
}

impl DataBatch {
    /// Create an empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a data point to the appropriate list.
    pub fn add(&mut self, point: DataPoint) {
        match point.data_type {
            DataType::Telemetry => self.telemetry.push(point),
            DataType::Signal => self.signal.push(point),
            DataType::Control => self.control.push(point),
            DataType::Adjustment => self.adjustment.push(point),
        }
    }

    /// Get total number of points.
    pub fn len(&self) -> usize {
        self.telemetry.len() + self.signal.len() + self.control.len() + self.adjustment.len()
    }

    /// Check if batch is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Merge another batch into this one.
    pub fn merge(&mut self, other: DataBatch) {
        self.telemetry.extend(other.telemetry);
        self.signal.extend(other.signal);
        self.control.extend(other.control);
        self.adjustment.extend(other.adjustment);
    }

    /// Iterate over all points.
    pub fn iter(&self) -> impl Iterator<Item = &DataPoint> {
        self.telemetry
            .iter()
            .chain(self.signal.iter())
            .chain(self.control.iter())
            .chain(self.adjustment.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type() {
        assert!(DataType::Telemetry.is_input());
        assert!(DataType::Signal.is_input());
        assert!(DataType::Control.is_output());
        assert!(DataType::Adjustment.is_output());

        assert!(DataType::Telemetry.is_analog());
        assert!(DataType::Signal.is_digital());
    }

    #[test]
    fn test_value_conversions() {
        let v = Value::from(42.5);
        assert_eq!(v.as_f64(), Some(42.5));
        assert_eq!(v.as_i64(), Some(42));

        let v = Value::from(true);
        assert_eq!(v.as_bool(), Some(true));
        assert_eq!(v.as_f64(), Some(1.0));
    }

    #[test]
    fn test_data_batch() {
        let mut batch = DataBatch::new();
        batch.add(DataPoint::telemetry(1, 25.5));
        batch.add(DataPoint::signal(2, true));

        assert_eq!(batch.len(), 2);
        assert_eq!(batch.telemetry.len(), 1);
        assert_eq!(batch.signal.len(), 1);
    }
}
