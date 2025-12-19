//! J1939 Protocol Client Implementation
//!
//! Implements the igw Protocol traits for J1939/CAN communication.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, ExtendedId, Frame, Socket};
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;

use crate::core::data::{DataBatch, DataPoint, DataType, Value};
use crate::core::error::{GatewayError, Result};
use crate::core::quality::Quality;
use crate::core::traits::{
    CommunicationMode, ConnectionState, ControlCommand, AdjustmentCommand,
    DataEvent, DataEventHandler, DataEventReceiver, DataEventSender,
    Diagnostics, EventDrivenProtocol, PollingConfig, Protocol, ProtocolCapabilities,
    ProtocolClient, ReadRequest, ReadResponse, WriteResult,
};

use super::database::{SpnDataType, SpnDef, SPN_DEFINITIONS, database_stats};

// ============================================================================
// Configuration
// ============================================================================

/// J1939 client configuration.
#[derive(Debug, Clone)]
pub struct J1939Config {
    /// CAN interface name (e.g., "can0").
    pub can_interface: String,

    /// Source address of the target device (ECU address).
    pub source_address: u8,

    /// Our address for sending request PGNs.
    pub our_address: u8,

    /// Request interval for on-demand PGNs in milliseconds.
    pub request_interval_ms: u64,
}

impl Default for J1939Config {
    fn default() -> Self {
        Self {
            can_interface: "can0".to_string(),
            source_address: 0x00,
            our_address: 0xFE,
            request_interval_ms: 1000,
        }
    }
}

// ============================================================================
// Internal types
// ============================================================================

/// Runtime point configuration (point_id = SPN)
#[derive(Debug, Clone)]
struct J1939Point {
    point_id: u32,
    name: String,
    spn: u32,
    pgn: u32,
    start_byte: u8,
    start_bit: u8,
    bit_length: u8,
    scale: f64,
    offset: f64,
    data_type: SpnDataType,
}

impl From<&SpnDef> for J1939Point {
    fn from(spn: &SpnDef) -> Self {
        Self {
            point_id: spn.spn,
            name: spn.name.to_string(),
            spn: spn.spn,
            pgn: spn.pgn,
            start_byte: spn.start_byte,
            start_bit: spn.start_bit,
            bit_length: spn.bit_length,
            scale: spn.scale,
            offset: spn.offset,
            data_type: spn.data_type,
        }
    }
}

// ============================================================================
// J1939Client
// ============================================================================

/// J1939 protocol client.
///
/// Implements event-driven communication over CAN bus using the SAE J1939 protocol.
pub struct J1939Client {
    config: J1939Config,

    // Point mappings by PGN
    pgn_points: HashMap<u32, Vec<J1939Point>>,

    // Connection state
    connection_state: Arc<RwLock<ConnectionState>>,
    is_connected: Arc<AtomicBool>,

    // Statistics
    read_count: AtomicU64,
    error_count: AtomicU64,
    last_error: Arc<RwLock<Option<String>>>,

    // Tasks
    receive_handle: Option<JoinHandle<()>>,

    // Event channel
    event_sender: Option<DataEventSender>,
    event_handler: Option<Arc<dyn DataEventHandler>>,

    // Cached data (latest values)
    cached_data: Arc<RwLock<HashMap<String, DataPoint>>>,
}

impl J1939Client {
    /// Create a new J1939 client with the given configuration.
    pub fn new(config: J1939Config) -> Self {
        // Build point mappings from SPN database
        let mut pgn_points: HashMap<u32, Vec<J1939Point>> = HashMap::new();
        for spn_def in SPN_DEFINITIONS {
            pgn_points
                .entry(spn_def.pgn)
                .or_default()
                .push(J1939Point::from(spn_def));
        }

        Self {
            config,
            pgn_points,
            connection_state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            is_connected: Arc::new(AtomicBool::new(false)),
            read_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            last_error: Arc::new(RwLock::new(None)),
            receive_handle: None,
            event_sender: None,
            event_handler: None,
            cached_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Parse J1939 CAN ID (29-bit extended frame).
    fn parse_can_id(can_id: u32) -> (u8, u32, u8, u8) {
        let sa = (can_id & 0xFF) as u8;
        let ps = ((can_id >> 8) & 0xFF) as u8;
        let pf = ((can_id >> 16) & 0xFF) as u8;
        let dp = ((can_id >> 24) & 0x01) as u8;
        let priority = ((can_id >> 26) & 0x07) as u8;

        let pgn = if pf >= 240 {
            ((dp as u32) << 16) | ((pf as u32) << 8) | (ps as u32)
        } else {
            ((dp as u32) << 16) | ((pf as u32) << 8)
        };

        (priority, pgn, ps, sa)
    }

    /// Decode SPN value from CAN data.
    fn decode_spn(data: &[u8], point: &J1939Point) -> Option<f64> {
        if data.len() <= point.start_byte as usize {
            return None;
        }

        let raw_value = match point.data_type {
            SpnDataType::Uint8 => {
                if point.bit_length == 8 && point.start_bit == 0 {
                    data[point.start_byte as usize] as u64
                } else {
                    let byte = data[point.start_byte as usize];
                    let mask = (1u8 << point.bit_length) - 1;
                    ((byte >> point.start_bit) & mask) as u64
                }
            }
            SpnDataType::Uint16 => {
                let idx = point.start_byte as usize;
                if idx + 1 >= data.len() {
                    return None;
                }
                u16::from_le_bytes([data[idx], data[idx + 1]]) as u64
            }
            SpnDataType::Uint32 => {
                let idx = point.start_byte as usize;
                if idx + 3 >= data.len() {
                    return None;
                }
                u32::from_le_bytes([data[idx], data[idx + 1], data[idx + 2], data[idx + 3]]) as u64
            }
            SpnDataType::Int8 => {
                let byte = data[point.start_byte as usize] as i8;
                byte as i64 as u64
            }
            SpnDataType::Int16 => {
                let idx = point.start_byte as usize;
                if idx + 1 >= data.len() {
                    return None;
                }
                let val = i16::from_le_bytes([data[idx], data[idx + 1]]);
                val as i64 as u64
            }
            SpnDataType::Int32 => {
                let idx = point.start_byte as usize;
                if idx + 3 >= data.len() {
                    return None;
                }
                let val = i32::from_le_bytes([data[idx], data[idx + 1], data[idx + 2], data[idx + 3]]);
                val as i64 as u64
            }
        };

        // Check for "not available" values
        let max_value = (1u64 << point.bit_length) - 1;
        if raw_value >= max_value - 1 {
            return None;
        }

        let value = (raw_value as f64) * point.scale + point.offset;
        Some(value)
    }

    /// Start the receive task.
    fn start_receive_task(&mut self) -> Result<()> {
        let can_interface = self.config.can_interface.clone();
        let source_address = self.config.source_address;
        let is_connected = Arc::clone(&self.is_connected);
        let pgn_points = self.pgn_points.clone();
        let cached_data = Arc::clone(&self.cached_data);
        let read_count = self.read_count.clone();
        let error_count = self.error_count.clone();
        let last_error = Arc::clone(&self.last_error);
        let event_sender = self.event_sender.clone();
        let event_handler = self.event_handler.clone();

        let handle = tokio::spawn(async move {
            let socket = match CanSocket::open(&can_interface) {
                Ok(s) => s,
                Err(e) => {
                    *last_error.write().await = Some(format!("Failed to open CAN socket: {}", e));
                    error_count.fetch_add(1, Ordering::Relaxed);
                    return;
                }
            };

            loop {
                if !is_connected.load(Ordering::SeqCst) {
                    break;
                }

                match socket.read_frame() {
                    Ok(frame) => {
                        if let Some(id) = frame.id().as_extended() {
                            let can_id = id.as_raw();
                            let (_, pgn, _, sa) = Self::parse_can_id(can_id);

                            if sa != source_address {
                                continue;
                            }

                            if let Some(points) = pgn_points.get(&pgn) {
                                let timestamp = chrono::Utc::now();
                                let mut batch = DataBatch::new();

                                for point in points {
                                    if let Some(value) = Self::decode_spn(frame.data(), point) {
                                        let data_point = DataPoint {
                                            id: point.spn.to_string(),
                                            data_type: DataType::Telemetry,
                                            value: Value::Float(value),
                                            quality: Quality::Good,
                                            timestamp,
                                        };

                                        batch.push(data_point.clone());

                                        // Update cache
                                        cached_data.write().await.insert(
                                            point.spn.to_string(),
                                            data_point,
                                        );
                                    }
                                }

                                if !batch.is_empty() {
                                    read_count.fetch_add(1, Ordering::Relaxed);

                                    // Send event
                                    if let Some(ref sender) = event_sender {
                                        let _ = sender.send(DataEvent::DataUpdate(batch.clone())).await;
                                    }

                                    // Call handler
                                    if let Some(ref handler) = event_handler {
                                        handler.on_data_update(batch).await;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        *last_error.write().await = Some(format!("CAN read error: {}", e));
                        error_count.fetch_add(1, Ordering::Relaxed);
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });

        self.receive_handle = Some(handle);
        Ok(())
    }
}

// ============================================================================
// Trait Implementations
// ============================================================================

impl ProtocolCapabilities for J1939Client {
    fn name(&self) -> &'static str {
        "J1939"
    }

    fn supported_modes(&self) -> &[CommunicationMode] {
        &[CommunicationMode::EventDriven]
    }

    fn supports_client(&self) -> bool {
        true
    }

    fn supports_server(&self) -> bool {
        false
    }

    fn version(&self) -> &'static str {
        "SAE J1939-21"
    }
}

#[async_trait]
impl Protocol for J1939Client {
    fn connection_state(&self) -> ConnectionState {
        *futures::executor::block_on(self.connection_state.read())
    }

    async fn read(&self, request: ReadRequest) -> Result<ReadResponse> {
        let cached = self.cached_data.read().await;

        let mut batch = DataBatch::new();

        match (&request.data_type, &request.point_ids) {
            (None, None) => {
                // Return all cached data
                for point in cached.values() {
                    batch.push(point.clone());
                }
            }
            (Some(DataType::Telemetry), None) => {
                for point in cached.values() {
                    if point.data_type == DataType::Telemetry {
                        batch.push(point.clone());
                    }
                }
            }
            (None, Some(ids)) => {
                for id in ids {
                    if let Some(point) = cached.get(id) {
                        batch.push(point.clone());
                    }
                }
            }
            (Some(dtype), Some(ids)) => {
                for id in ids {
                    if let Some(point) = cached.get(id) {
                        if point.data_type == *dtype {
                            batch.push(point.clone());
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(ReadResponse::success(batch))
    }

    async fn diagnostics(&self) -> Result<Diagnostics> {
        let (pgn_count, spn_count) = database_stats();

        Ok(Diagnostics {
            protocol: "J1939".to_string(),
            connection_state: *self.connection_state.read().await,
            read_count: self.read_count.load(Ordering::Relaxed),
            write_count: 0,
            error_count: self.error_count.load(Ordering::Relaxed),
            last_error: self.last_error.read().await.clone(),
            extra: serde_json::json!({
                "can_interface": self.config.can_interface,
                "source_address": format!("0x{:02X}", self.config.source_address),
                "pgn_count": pgn_count,
                "spn_count": spn_count,
            }),
        })
    }
}

#[async_trait]
impl ProtocolClient for J1939Client {
    async fn connect(&mut self) -> Result<()> {
        *self.connection_state.write().await = ConnectionState::Connecting;

        // Verify CAN interface exists
        let _socket = CanSocket::open(&self.config.can_interface).map_err(|e| {
            GatewayError::Connection(format!(
                "Failed to open CAN interface {}: {}",
                self.config.can_interface, e
            ))
        })?;

        self.is_connected.store(true, Ordering::SeqCst);
        *self.connection_state.write().await = ConnectionState::Connected;

        // Start receive task
        self.start_receive_task()?;

        // Notify connection change
        if let Some(ref sender) = self.event_sender {
            let _ = sender.send(DataEvent::ConnectionChanged(ConnectionState::Connected)).await;
        }
        if let Some(ref handler) = self.event_handler {
            handler.on_connection_changed(ConnectionState::Connected).await;
        }

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.is_connected.store(false, Ordering::SeqCst);

        if let Some(handle) = self.receive_handle.take() {
            handle.abort();
        }

        *self.connection_state.write().await = ConnectionState::Disconnected;

        // Notify connection change
        if let Some(ref sender) = self.event_sender {
            let _ = sender.send(DataEvent::ConnectionChanged(ConnectionState::Disconnected)).await;
        }
        if let Some(ref handler) = self.event_handler {
            handler.on_connection_changed(ConnectionState::Disconnected).await;
        }

        Ok(())
    }

    async fn write_control(&mut self, _commands: &[ControlCommand]) -> Result<WriteResult> {
        // J1939 control requires proprietary PGN support
        Err(GatewayError::NotSupported(
            "J1939 control commands require proprietary PGN implementation".to_string(),
        ))
    }

    async fn write_adjustment(&mut self, _adjustments: &[AdjustmentCommand]) -> Result<WriteResult> {
        // J1939 adjustment requires proprietary PGN support
        Err(GatewayError::NotSupported(
            "J1939 adjustment commands require proprietary PGN implementation".to_string(),
        ))
    }

    async fn start_polling(&mut self, _config: PollingConfig) -> Result<()> {
        // J1939 is event-driven, no polling needed
        Ok(())
    }

    async fn stop_polling(&mut self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl EventDrivenProtocol for J1939Client {
    fn subscribe(&self) -> DataEventReceiver {
        // This is a simplified implementation
        // In a real implementation, you'd want to support multiple subscribers
        let (tx, rx) = mpsc::channel(100);
        // Note: This overwrites any existing sender, which is not ideal
        // A proper implementation would use a broadcast channel
        rx
    }

    fn set_event_handler(&mut self, handler: Arc<dyn DataEventHandler>) {
        self.event_handler = Some(handler);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_can_id() {
        // EEC1 from SA=0x00: CAN ID = 0x0CF00400
        let (priority, pgn, _ps, sa) = J1939Client::parse_can_id(0x0CF00400);
        assert_eq!(priority, 3);
        assert_eq!(pgn, 61444);
        assert_eq!(sa, 0x00);

        // ET1 from SA=0x00: CAN ID = 0x18FEEE00
        let (priority, pgn, _ps, sa) = J1939Client::parse_can_id(0x18FEEE00);
        assert_eq!(priority, 6);
        assert_eq!(pgn, 65262);
        assert_eq!(sa, 0x00);
    }

    #[test]
    fn test_decode_spn() {
        let point = J1939Point {
            point_id: 110,
            name: "coolant_temp".to_string(),
            spn: 110,
            pgn: 65262,
            start_byte: 0,
            start_bit: 0,
            bit_length: 8,
            scale: 1.0,
            offset: -40.0,
            data_type: SpnDataType::Uint8,
        };

        // Coolant temp = 90°C, raw value = 130 (90 + 40)
        let data = [130u8, 0, 0, 0, 0, 0, 0, 0];
        let value = J1939Client::decode_spn(&data, &point);
        assert_eq!(value, Some(90.0));
    }

    #[test]
    fn test_config_default() {
        let config = J1939Config::default();
        assert_eq!(config.can_interface, "can0");
        assert_eq!(config.source_address, 0x00);
        assert_eq!(config.our_address, 0xFE);
    }

    #[test]
    fn test_client_creation() {
        let config = J1939Config::default();
        let client = J1939Client::new(config);
        assert_eq!(client.name(), "J1939");
        assert_eq!(client.supported_modes(), &[CommunicationMode::EventDriven]);
    }
}
