//! Modbus TCP/RTU client implementation.

use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::{
    traits::*,
    data::*,
    error::{GatewayError, Result},
};

use super::types::{ModbusConfig, ModbusConfigBuilder, ModbusMode};

/// Modbus TCP client.
///
/// # Example
///
/// ```rust,ignore
/// use igw::protocols::modbus::ModbusTcpClient;
///
/// let mut client = ModbusTcpClient::new("192.168.1.100:502")?;
/// client.connect().await?;
/// ```
pub struct ModbusTcpClient {
    config: ModbusConfig,
    state: Arc<RwLock<ConnectionState>>,
    is_connected: Arc<AtomicBool>,
    diagnostics: Arc<RwLock<DiagnosticsData>>,
}

struct DiagnosticsData {
    read_count: u64,
    write_count: u64,
    error_count: u64,
    last_error: Option<String>,
}

impl ModbusTcpClient {
    /// Create a new Modbus TCP client with the given address.
    pub fn new(address: impl Into<String>) -> Result<Self> {
        let config = ModbusConfigBuilder::new()
            .tcp(address)
            .build();
        Self::with_config(config)
    }

    /// Create a client with custom configuration.
    pub fn with_config(config: ModbusConfig) -> Result<Self> {
        if config.mode != ModbusMode::Tcp {
            return Err(GatewayError::config("ModbusTcpClient requires TCP mode"));
        }

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            is_connected: Arc::new(AtomicBool::new(false)),
            diagnostics: Arc::new(RwLock::new(DiagnosticsData {
                read_count: 0,
                write_count: 0,
                error_count: 0,
                last_error: None,
            })),
        })
    }

    /// Create a builder for custom configuration.
    pub fn builder() -> ModbusConfigBuilder {
        ModbusConfigBuilder::new()
    }

    /// Get the target address.
    pub fn address(&self) -> &str {
        &self.config.address
    }

    async fn set_state(&self, state: ConnectionState) {
        *self.state.write().await = state;
        self.is_connected.store(state.is_connected(), Ordering::SeqCst);
    }

    #[allow(dead_code)]
    async fn record_error(&self, error: &str) {
        let mut diag = self.diagnostics.write().await;
        diag.error_count += 1;
        diag.last_error = Some(error.to_string());
    }
}

impl ProtocolCapabilities for ModbusTcpClient {
    fn name(&self) -> &'static str {
        "modbus_tcp"
    }

    fn supported_modes(&self) -> &[CommunicationMode] {
        &[CommunicationMode::Polling]
    }

    fn supports_client(&self) -> bool {
        true
    }

    fn supports_server(&self) -> bool {
        false
    }
}

#[async_trait]
impl Protocol for ModbusTcpClient {
    fn connection_state(&self) -> ConnectionState {
        if self.is_connected.load(Ordering::SeqCst) {
            ConnectionState::Connected
        } else {
            ConnectionState::Disconnected
        }
    }

    async fn read(&self, _request: ReadRequest) -> Result<ReadResponse> {
        if !self.is_connected.load(Ordering::SeqCst) {
            return Err(GatewayError::NotConnected);
        }

        // TODO: Implement actual Modbus read
        // This is a placeholder that returns empty data
        let batch = DataBatch::new();

        let mut diag = self.diagnostics.write().await;
        diag.read_count += 1;

        Ok(ReadResponse::success(batch))
    }

    async fn diagnostics(&self) -> Result<Diagnostics> {
        let diag = self.diagnostics.read().await;
        Ok(Diagnostics {
            protocol: self.name().to_string(),
            connection_state: self.connection_state(),
            read_count: diag.read_count,
            write_count: diag.write_count,
            error_count: diag.error_count,
            last_error: diag.last_error.clone(),
            extra: serde_json::json!({
                "address": self.config.address,
                "mode": "tcp",
            }),
        })
    }
}

#[async_trait]
impl ProtocolClient for ModbusTcpClient {
    async fn connect(&mut self) -> Result<()> {
        self.set_state(ConnectionState::Connecting).await;

        // TODO: Implement actual TCP connection
        // For now, just simulate success

        self.set_state(ConnectionState::Connected).await;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.set_state(ConnectionState::Disconnected).await;
        Ok(())
    }

    async fn write_control(&mut self, commands: &[ControlCommand]) -> Result<WriteResult> {
        if !self.is_connected.load(Ordering::SeqCst) {
            return Err(GatewayError::NotConnected);
        }

        // TODO: Implement actual Modbus write
        let mut diag = self.diagnostics.write().await;
        diag.write_count += commands.len() as u64;

        Ok(WriteResult::success(commands.len()))
    }

    async fn write_adjustment(&mut self, adjustments: &[AdjustmentCommand]) -> Result<WriteResult> {
        if !self.is_connected.load(Ordering::SeqCst) {
            return Err(GatewayError::NotConnected);
        }

        // TODO: Implement actual Modbus write
        let mut diag = self.diagnostics.write().await;
        diag.write_count += adjustments.len() as u64;

        Ok(WriteResult::success(adjustments.len()))
    }

    async fn start_polling(&mut self, _config: PollingConfig) -> Result<()> {
        if !self.is_connected.load(Ordering::SeqCst) {
            return Err(GatewayError::NotConnected);
        }

        // TODO: Implement polling task
        Ok(())
    }

    async fn stop_polling(&mut self) -> Result<()> {
        // TODO: Stop polling task
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_modbus_client_creation() {
        let client = ModbusTcpClient::new("127.0.0.1:502").unwrap();
        assert_eq!(client.name(), "modbus_tcp");
        assert_eq!(client.connection_state(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_modbus_connect_disconnect() {
        let mut client = ModbusTcpClient::new("127.0.0.1:502").unwrap();

        client.connect().await.unwrap();
        assert_eq!(client.connection_state(), ConnectionState::Connected);

        client.disconnect().await.unwrap();
        assert_eq!(client.connection_state(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_read_requires_connection() {
        let client = ModbusTcpClient::new("127.0.0.1:502").unwrap();
        let result = client.read(ReadRequest::all()).await;
        assert!(matches!(result, Err(GatewayError::NotConnected)));
    }
}
