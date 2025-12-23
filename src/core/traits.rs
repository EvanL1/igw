//! Core traits for protocol implementations.
//!
//! This module defines the fundamental traits that all protocols must implement.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::core::data::{DataBatch, DataType};
use crate::core::error::Result;

/// Communication mode supported by a protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationMode {
    /// Polling mode - actively request data at intervals.
    ///
    /// Used by: Modbus, BACnet (read), etc.
    Polling,

    /// Event-driven mode - passively receive data updates.
    ///
    /// Used by: IEC 104 (spontaneous), OPC UA (subscriptions), etc.
    EventDriven,

    /// Hybrid mode - supports both polling and events.
    ///
    /// Used by: DNP3, OPC UA, etc.
    Hybrid,
}

/// Connection state of a protocol client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    /// Not connected to the target.
    #[default]
    Disconnected,

    /// Attempting to connect.
    Connecting,

    /// Connected and operational.
    Connected,

    /// Attempting to reconnect after failure.
    Reconnecting,

    /// Connection error state.
    Error,
}

impl ConnectionState {
    /// Check if currently connected.
    #[inline]
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Check if retry is possible.
    #[inline]
    pub fn can_retry(&self) -> bool {
        matches!(self, Self::Disconnected | Self::Error)
    }
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Disconnected => "Disconnected",
            Self::Connecting => "Connecting",
            Self::Connected => "Connected",
            Self::Reconnecting => "Reconnecting",
            Self::Error => "Error",
        };
        write!(f, "{}", s)
    }
}

/// Request for reading data points.
#[derive(Debug, Clone)]
pub struct ReadRequest {
    /// Data type to read (None = all types)
    pub data_type: Option<DataType>,

    /// Point IDs to read (None = all points)
    pub point_ids: Option<Vec<u32>>,
}

impl ReadRequest {
    /// Create a request for all points of a specific type.
    pub fn by_type(data_type: DataType) -> Self {
        Self {
            data_type: Some(data_type),
            point_ids: None,
        }
    }

    /// Create a request for specific points.
    pub fn by_ids(ids: Vec<u32>) -> Self {
        Self {
            data_type: None,
            point_ids: Some(ids),
        }
    }

    /// Create a request for all telemetry points.
    pub fn telemetry() -> Self {
        Self::by_type(DataType::Telemetry)
    }

    /// Create a request for all signal points.
    pub fn signal() -> Self {
        Self::by_type(DataType::Signal)
    }

    /// Create a request for all points.
    pub fn all() -> Self {
        Self {
            data_type: None,
            point_ids: None,
        }
    }
}

/// Response from reading data points.
#[derive(Debug, Clone)]
pub struct ReadResponse {
    /// The data batch containing all read points.
    pub data: DataBatch,

    /// Number of points that failed to read.
    pub failed_count: usize,
}

impl ReadResponse {
    /// Create a successful response.
    pub fn success(data: DataBatch) -> Self {
        Self {
            data,
            failed_count: 0,
        }
    }

    /// Create a response with partial failures.
    pub fn partial(data: DataBatch, failed: usize) -> Self {
        Self {
            data,
            failed_count: failed,
        }
    }
}

/// A control command to write.
#[derive(Debug, Clone)]
pub struct ControlCommand {
    /// Point ID
    pub id: u32,

    /// Command value (true = ON/CLOSE, false = OFF/OPEN)
    pub value: bool,

    /// Pulse duration in milliseconds (None = latching)
    pub pulse_duration_ms: Option<u32>,
}

impl ControlCommand {
    /// Create a latching control command.
    pub fn latching(id: u32, value: bool) -> Self {
        Self {
            id,
            value,
            pulse_duration_ms: None,
        }
    }

    /// Create a pulse control command.
    pub fn pulse(id: u32, value: bool, duration_ms: u32) -> Self {
        Self {
            id,
            value,
            pulse_duration_ms: Some(duration_ms),
        }
    }
}

/// An adjustment command to write.
#[derive(Debug, Clone)]
pub struct AdjustmentCommand {
    /// Point ID
    pub id: u32,

    /// Setpoint value
    pub value: f64,
}

impl AdjustmentCommand {
    /// Create an adjustment command.
    pub fn new(id: u32, value: f64) -> Self {
        Self { id, value }
    }
}

/// Result of write operations.
#[derive(Debug, Clone)]
pub struct WriteResult {
    /// Number of successful writes.
    pub success_count: usize,

    /// IDs of failed writes with error messages.
    pub failures: Vec<(u32, String)>,
}

impl WriteResult {
    /// Create a fully successful result.
    pub fn success(count: usize) -> Self {
        Self {
            success_count: count,
            failures: vec![],
        }
    }

    /// Check if all writes succeeded.
    pub fn is_success(&self) -> bool {
        self.failures.is_empty()
    }
}

/// Polling configuration.
#[derive(Debug, Clone)]
pub struct PollingConfig {
    /// Polling interval in milliseconds.
    pub interval_ms: u64,

    /// Data types to poll (None = all).
    pub data_types: Option<Vec<DataType>>,

    /// Whether to continue on individual point errors.
    pub continue_on_error: bool,
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            interval_ms: 1000,
            data_types: None,
            continue_on_error: true,
        }
    }
}

/// Protocol diagnostics information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostics {
    /// Protocol name.
    pub protocol: String,

    /// Connection state.
    pub connection_state: ConnectionState,

    /// Number of successful reads.
    pub read_count: u64,

    /// Number of successful writes.
    pub write_count: u64,

    /// Number of errors.
    pub error_count: u64,

    /// Last error message.
    pub last_error: Option<String>,

    /// Protocol-specific information.
    #[serde(default)]
    pub extra: serde_json::Value,
}

impl Diagnostics {
    /// Create new diagnostics.
    pub fn new(protocol: impl Into<String>) -> Self {
        Self {
            protocol: protocol.into(),
            connection_state: ConnectionState::Disconnected,
            read_count: 0,
            write_count: 0,
            error_count: 0,
            last_error: None,
            extra: serde_json::Value::Null,
        }
    }
}

/// Protocol capabilities description.
pub trait ProtocolCapabilities {
    /// Get the protocol name.
    fn name(&self) -> &'static str;

    /// Get supported communication modes.
    fn supported_modes(&self) -> &[CommunicationMode];

    /// Check if client role is supported.
    fn supports_client(&self) -> bool {
        true
    }

    /// Check if server role is supported.
    fn supports_server(&self) -> bool {
        false
    }

    /// Get protocol version.
    fn version(&self) -> &'static str {
        "1.0"
    }
}

/// Base protocol trait - read-only operations.
#[async_trait]
pub trait Protocol: ProtocolCapabilities + Send + Sync {
    /// Get current connection state.
    fn connection_state(&self) -> ConnectionState;

    /// Read data points.
    async fn read(&self, request: ReadRequest) -> Result<ReadResponse>;

    /// Get diagnostics information.
    async fn diagnostics(&self) -> Result<Diagnostics>;
}

/// Client protocol trait - active connection + write operations.
#[async_trait]
pub trait ProtocolClient: Protocol {
    /// Connect to the target device/server.
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the target.
    async fn disconnect(&mut self) -> Result<()>;

    /// Execute a single poll cycle and return collected data.
    ///
    /// This is the primary method for data acquisition. The caller (service layer)
    /// is responsible for storing the returned data. The protocol layer only
    /// handles device communication.
    ///
    /// # Returns
    ///
    /// A `DataBatch` containing all successfully read points from configured sources.
    async fn poll_once(&mut self) -> Result<DataBatch>;

    /// Write control commands.
    async fn write_control(&mut self, commands: &[ControlCommand]) -> Result<WriteResult>;

    /// Write adjustment commands.
    async fn write_adjustment(&mut self, adjustments: &[AdjustmentCommand]) -> Result<WriteResult>;

    /// Start polling task (legacy, prefer using poll_once() with external loop).
    ///
    /// This method is kept for backward compatibility. New implementations
    /// should use `poll_once()` with an external polling loop managed by the service layer.
    async fn start_polling(&mut self, config: PollingConfig) -> Result<()>;

    /// Stop polling task.
    async fn stop_polling(&mut self) -> Result<()>;

    /// Attempt to reconnect after a failure.
    async fn try_reconnect(&mut self) -> Result<()> {
        self.disconnect().await.ok();
        self.connect().await
    }
}

/// Server protocol trait - passive connection acceptance.
#[async_trait]
pub trait ProtocolServer: Protocol {
    /// Start listening on the specified address.
    async fn listen(&mut self, addr: &str) -> Result<()>;

    /// Stop listening and close all connections.
    async fn stop(&mut self) -> Result<()>;

    /// Get number of connected clients.
    fn connected_clients(&self) -> usize;
}

/// Data event for event-driven protocols.
#[derive(Debug, Clone)]
pub enum DataEvent {
    /// Data update received.
    DataUpdate(DataBatch),

    /// Connection state changed.
    ConnectionChanged(ConnectionState),

    /// Error occurred.
    Error(String),

    /// Heartbeat/keep-alive.
    Heartbeat,
}

/// Event receiver type.
pub type DataEventReceiver = mpsc::Receiver<DataEvent>;

/// Event sender type.
pub type DataEventSender = mpsc::Sender<DataEvent>;

/// Event handler trait.
#[async_trait]
pub trait DataEventHandler: Send + Sync {
    /// Handle data update event.
    async fn on_data_update(&self, batch: DataBatch);

    /// Handle connection state change.
    async fn on_connection_changed(&self, state: ConnectionState);

    /// Handle error event.
    async fn on_error(&self, error: &str);
}

/// Event-driven protocol extension trait.
#[async_trait]
pub trait EventDrivenProtocol: Protocol {
    /// Subscribe to data events.
    fn subscribe(&self) -> DataEventReceiver;

    /// Set event handler.
    fn set_event_handler(&mut self, handler: Arc<dyn DataEventHandler>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state() {
        assert!(!ConnectionState::Disconnected.is_connected());
        assert!(ConnectionState::Connected.is_connected());
        assert!(ConnectionState::Disconnected.can_retry());
        assert!(!ConnectionState::Connecting.can_retry());
    }

    #[test]
    fn test_read_request() {
        let req = ReadRequest::telemetry();
        assert_eq!(req.data_type, Some(DataType::Telemetry));
        assert!(req.point_ids.is_none());
    }

    #[test]
    fn test_control_command() {
        let cmd = ControlCommand::latching(1, true);
        assert!(cmd.pulse_duration_ms.is_none());

        let cmd = ControlCommand::pulse(1, true, 500);
        assert_eq!(cmd.pulse_duration_ms, Some(500));
    }
}
