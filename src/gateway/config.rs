//! Gateway configuration types.
//!
//! Defines the TOML-friendly configuration format for the gateway.

use serde::{Deserialize, Serialize};

use crate::core::point::TransformConfig;

/// Gateway configuration (top-level).
///
/// # Example TOML
///
/// ```toml
/// [gateway]
/// name = "My Gateway"
/// default_poll_interval_ms = 1000
///
/// [[channels]]
/// id = 1
/// name = "PLC1"
/// protocol = "modbus"
/// enabled = true
///
/// [channels.parameters]
/// host = "192.168.1.100"
/// port = 502
///
/// [[channels.points]]
/// id = 1001
/// name = "Temperature"
/// address = "1:100"
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayConfig {
    /// Gateway global settings.
    pub gateway: GatewayGlobalConfig,

    /// Channel configurations.
    #[serde(default)]
    pub channels: Vec<ChannelConfig>,
}

/// Gateway global settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayGlobalConfig {
    /// Gateway name for identification.
    pub name: String,

    /// Default polling interval in milliseconds.
    #[serde(default = "default_poll_interval")]
    pub default_poll_interval_ms: u64,

    /// Diagnostics snapshot interval in milliseconds.
    #[serde(default = "default_diagnostics_interval")]
    pub diagnostics_interval_ms: u64,

    /// Enable JSON Lines output for events.
    #[serde(default)]
    pub jsonl_output: bool,
}

fn default_poll_interval() -> u64 {
    1000
}

fn default_diagnostics_interval() -> u64 {
    5000
}

impl Default for GatewayGlobalConfig {
    fn default() -> Self {
        Self {
            name: "IGW".to_string(),
            default_poll_interval_ms: default_poll_interval(),
            diagnostics_interval_ms: default_diagnostics_interval(),
            jsonl_output: false,
        }
    }
}

/// Channel configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChannelConfig {
    /// Channel unique identifier.
    pub id: u32,

    /// Channel display name.
    pub name: String,

    /// Protocol type: "modbus", "iec104", "opcua", "can", "gpio", "virtual".
    pub protocol: String,

    /// Whether this channel is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Channel mode: "polling", "event", or "hybrid".
    #[serde(default)]
    pub mode: ChannelModeConfig,

    /// Polling interval override (uses gateway default if not set).
    pub poll_interval_ms: Option<u64>,

    /// Protocol-specific parameters (JSON object).
    #[serde(default)]
    pub parameters: serde_json::Value,

    /// Point definitions.
    #[serde(default)]
    pub points: Vec<PointDef>,
}

fn default_true() -> bool {
    true
}

/// Channel mode configuration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelModeConfig {
    /// Polling mode (default for most protocols).
    #[default]
    Polling,
    /// Event-driven mode (for IEC104, OPC UA, CAN).
    Event,
    /// Hybrid mode (both polling and events).
    Hybrid,
}

/// Point definition with simplified address format.
///
/// The `address` field uses a protocol-specific shorthand format:
/// - Modbus: "slave_id:register" (e.g., "1:100")
/// - IEC104: "ioa" (e.g., "1001")
/// - OPC UA: "ns=N;i=ID" or "ns=N;s=Name" (e.g., "ns=2;i=1234")
/// - CAN: "can_id:byte_offset:bit_pos:bit_len" (e.g., "0x100:0:0:16")
/// - GPIO: "pin_number" (e.g., "17")
/// - Virtual: "key" (e.g., "temperature")
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PointDef {
    /// Point unique identifier.
    pub id: u32,

    /// Point display name.
    pub name: String,

    /// Protocol-specific address (shorthand format).
    pub address: String,

    /// Data transformation configuration.
    #[serde(default)]
    pub transform: TransformConfig,

    /// Whether this point is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl GatewayConfig {
    /// Load configuration from a TOML file.
    ///
    /// Requires the `cli` feature.
    #[cfg(feature = "cli")]
    pub fn from_file(path: &std::path::Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|e| ConfigError::Io(e.to_string()))?;
        Self::parse(&content)
    }

    /// Parse configuration from a TOML string.
    ///
    /// Requires the `cli` feature.
    #[cfg(feature = "cli")]
    pub fn parse(s: &str) -> Result<Self, ConfigError> {
        toml::from_str(s).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    /// Get enabled channels only.
    pub fn enabled_channels(&self) -> impl Iterator<Item = &ChannelConfig> {
        self.channels.iter().filter(|c| c.enabled)
    }
}

/// Configuration error.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "cli")]
    fn test_parse_gateway_config() {
        let toml_str = r#"
[gateway]
name = "Test Gateway"
default_poll_interval_ms = 500

[[channels]]
id = 1
name = "Modbus Channel"
protocol = "modbus"
enabled = true

[channels.parameters]
host = "127.0.0.1"
port = 502

[[channels.points]]
id = 1001
name = "Temperature"
address = "1:100"

[channels.points.transform]
scale = 0.1
"#;

        let config: GatewayConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.gateway.name, "Test Gateway");
        assert_eq!(config.gateway.default_poll_interval_ms, 500);
        assert_eq!(config.channels.len(), 1);
        assert_eq!(config.channels[0].protocol, "modbus");
        assert_eq!(config.channels[0].points.len(), 1);
        assert_eq!(config.channels[0].points[0].address, "1:100");
    }

    #[test]
    fn test_channel_mode_default() {
        let mode = ChannelModeConfig::default();
        assert_eq!(mode, ChannelModeConfig::Polling);
    }
}
