//! GPIO (General Purpose Input/Output) protocol adapter.
//!
//! Provides direct hardware GPIO control on Linux systems.
//!
//! # Platform Support
//!
//! - Linux: Uses the gpiod (libgpiod) library
//! - Other platforms: Not supported
//!
//! # Example
//!
//! ```rust,ignore
//! use igw::protocols::gpio::{GpioChannel, GpioChannelConfig, GpioPinConfig};
//! use igw::store::MemoryStore;
//! use std::sync::Arc;
//!
//! let store = Arc::new(MemoryStore::new());
//! let config = GpioChannelConfig::new()
//!     .add_pin(GpioPinConfig::digital_input("gpiochip0", 17, "door_sensor"))
//!     .add_pin(GpioPinConfig::digital_output("gpiochip0", 18, "alarm_led"));
//!
//! let mut gpio = GpioChannel::new(config, store, 1);
//! gpio.connect().await?;
//!
//! // Read DI
//! let response = gpio.read(ReadRequest::all()).await?;
//!
//! // Control DO
//! gpio.write_control(&[ControlCommand::latching("alarm_led", true)]).await?;
//! ```

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::core::data::{DataBatch, DataPoint};
use crate::core::error::{GatewayError, Result};
use crate::core::point::PointConfig;
use crate::core::traits::{
    AdjustmentCommand, CommunicationMode, ConnectionState, ControlCommand, Diagnostics,
    PollingConfig, Protocol, ProtocolCapabilities, ProtocolClient, ReadRequest, ReadResponse,
    WriteResult,
};
use crate::store::DataStore;

/// GPIO pin direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioDirection {
    /// Input pin (DI - Digital Input).
    Input,
    /// Output pin (DO - Digital Output).
    Output,
}

/// GPIO pin configuration.
#[derive(Debug, Clone)]
pub struct GpioPinConfig {
    /// GPIO chip name (e.g., "gpiochip0").
    pub chip: String,

    /// Pin number/offset.
    pub pin: u32,

    /// Pin direction.
    pub direction: GpioDirection,

    /// Point ID mapping.
    pub point_id: String,

    /// Active low (invert logic).
    pub active_low: bool,

    /// Debounce time for inputs (microseconds).
    pub debounce_us: Option<u64>,
}

impl GpioPinConfig {
    /// Create a digital input configuration.
    pub fn digital_input(
        chip: impl Into<String>,
        pin: u32,
        point_id: impl Into<String>,
    ) -> Self {
        Self {
            chip: chip.into(),
            pin,
            direction: GpioDirection::Input,
            point_id: point_id.into(),
            active_low: false,
            debounce_us: Some(1000), // 1ms default debounce
        }
    }

    /// Create a digital output configuration.
    pub fn digital_output(
        chip: impl Into<String>,
        pin: u32,
        point_id: impl Into<String>,
    ) -> Self {
        Self {
            chip: chip.into(),
            pin,
            direction: GpioDirection::Output,
            point_id: point_id.into(),
            active_low: false,
            debounce_us: None,
        }
    }

    /// Set active low mode.
    pub fn with_active_low(mut self, active_low: bool) -> Self {
        self.active_low = active_low;
        self
    }

    /// Set debounce time.
    pub fn with_debounce(mut self, debounce_us: u64) -> Self {
        self.debounce_us = Some(debounce_us);
        self
    }
}

/// GPIO channel configuration.
#[derive(Debug, Clone, Default)]
pub struct GpioChannelConfig {
    /// Pin configurations.
    pub pins: Vec<GpioPinConfig>,

    /// Polling interval for inputs.
    pub poll_interval: Duration,
}

impl GpioChannelConfig {
    /// Create a new configuration.
    pub fn new() -> Self {
        Self {
            pins: Vec::new(),
            poll_interval: Duration::from_millis(100),
        }
    }

    /// Add a pin configuration.
    pub fn add_pin(mut self, pin: GpioPinConfig) -> Self {
        self.pins.push(pin);
        self
    }

    /// Set poll interval.
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Get input pins.
    pub fn input_pins(&self) -> impl Iterator<Item = &GpioPinConfig> {
        self.pins
            .iter()
            .filter(|p| p.direction == GpioDirection::Input)
    }

    /// Get output pins.
    pub fn output_pins(&self) -> impl Iterator<Item = &GpioPinConfig> {
        self.pins
            .iter()
            .filter(|p| p.direction == GpioDirection::Output)
    }
}

/// GPIO channel diagnostics.
#[derive(Debug, Default)]
struct GpioDiagnostics {
    read_count: u64,
    write_count: u64,
    error_count: u64,
    last_error: Option<String>,
}

/// GPIO channel adapter.
///
/// This is a stub implementation. The actual GPIO operations require
/// platform-specific libraries (gpiod on Linux).
pub struct GpioChannel<S: DataStore> {
    config: GpioChannelConfig,
    store: Arc<S>,
    channel_id: u32,
    state: Arc<std::sync::RwLock<ConnectionState>>,
    diagnostics: Arc<RwLock<GpioDiagnostics>>,
    poll_task: Option<tokio::task::JoinHandle<()>>,
    // Output states (simulated)
    output_states: Arc<RwLock<std::collections::HashMap<String, bool>>>,
}

impl<S: DataStore> GpioChannel<S> {
    /// Create a new GPIO channel.
    pub fn new(config: GpioChannelConfig, store: Arc<S>, channel_id: u32) -> Self {
        // Build point configs from pin configs
        let point_configs: Vec<PointConfig> = config
            .pins
            .iter()
            .map(|pin| {
                let data_type = match pin.direction {
                    GpioDirection::Input => crate::core::data::DataType::Signal,
                    GpioDirection::Output => crate::core::data::DataType::Control,
                };
                PointConfig::new(
                    &pin.point_id,
                    data_type,
                    crate::core::point::ProtocolAddress::Generic(format!(
                        "gpio:{}:{}",
                        pin.chip, pin.pin
                    )),
                )
            })
            .collect();

        store.set_point_configs(channel_id, point_configs);

        Self {
            config,
            store,
            channel_id,
            state: Arc::new(std::sync::RwLock::new(ConnectionState::Disconnected)),
            diagnostics: Arc::new(RwLock::new(GpioDiagnostics::default())),
            poll_task: None,
            output_states: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    fn set_state(&self, state: ConnectionState) {
        if let Ok(mut s) = self.state.write() {
            *s = state;
        }
    }

    fn get_state(&self) -> ConnectionState {
        self.state
            .read()
            .map(|s| *s)
            .unwrap_or(ConnectionState::Error)
    }

    /// Read a single GPIO pin (stub implementation).
    async fn read_pin(&self, pin_config: &GpioPinConfig) -> Result<DataPoint> {
        // In a real implementation, this would use gpiod:
        // let chip = gpiod::Chip::new(&pin_config.chip)?;
        // let line = chip.get_line(pin_config.pin)?;
        // let value = line.get_value()?;

        // Stub: return false for inputs
        let value = false;
        let adjusted = if pin_config.active_low { !value } else { value };

        Ok(DataPoint::signal(&pin_config.point_id, adjusted))
    }

    /// Write to a single GPIO pin (stub implementation).
    async fn write_pin(&self, pin_config: &GpioPinConfig, value: bool) -> Result<()> {
        // In a real implementation, this would use gpiod:
        // let chip = gpiod::Chip::new(&pin_config.chip)?;
        // let line = chip.get_line(pin_config.pin)?;
        // let adjusted = if pin_config.active_low { !value } else { value };
        // line.request(gpiod::LineRequestFlags::OUTPUT, adjusted as u8, "igw")?;
        // line.set_value(adjusted as u8)?;

        // Store the output state
        let adjusted = if pin_config.active_low { !value } else { value };
        self.output_states
            .write()
            .await
            .insert(pin_config.point_id.clone(), adjusted);

        Ok(())
    }

    /// Read output state (for feedback).
    async fn read_output_state(&self, point_id: &str) -> Option<bool> {
        self.output_states.read().await.get(point_id).copied()
    }
}

impl<S: DataStore> ProtocolCapabilities for GpioChannel<S> {
    fn name(&self) -> &'static str {
        "GPIO"
    }

    fn supported_modes(&self) -> &[CommunicationMode] {
        &[CommunicationMode::Polling]
    }
}

#[async_trait]
impl<S: DataStore + 'static> Protocol for GpioChannel<S> {
    fn connection_state(&self) -> ConnectionState {
        self.get_state()
    }

    async fn read(&self, request: ReadRequest) -> Result<ReadResponse> {
        if !self.get_state().is_connected() {
            return Err(GatewayError::NotConnected);
        }

        let mut batch = DataBatch::new();

        // Read all input pins
        for pin in self.config.input_pins() {
            // Filter by request
            if let Some(ref ids) = request.point_ids {
                if !ids.contains(&pin.point_id) {
                    continue;
                }
            }

            match self.read_pin(pin).await {
                Ok(point) => batch.add(point),
                Err(e) => {
                    let mut diag = self.diagnostics.write().await;
                    diag.error_count += 1;
                    diag.last_error = Some(e.to_string());
                }
            }
        }

        // Also include output states as feedback
        for pin in self.config.output_pins() {
            if let Some(ref ids) = request.point_ids {
                if !ids.contains(&pin.point_id) {
                    continue;
                }
            }

            if let Some(state) = self.read_output_state(&pin.point_id).await {
                batch.add(DataPoint::control(&pin.point_id, state));
            }
        }

        {
            let mut diag = self.diagnostics.write().await;
            diag.read_count += 1;
        }

        // Store to DataStore
        self.store.write_batch(self.channel_id, &batch).await?;

        Ok(ReadResponse::success(batch))
    }

    async fn diagnostics(&self) -> Result<Diagnostics> {
        let diag = self.diagnostics.read().await;
        let input_count = self.config.input_pins().count();
        let output_count = self.config.output_pins().count();

        Ok(Diagnostics {
            protocol: self.name().to_string(),
            connection_state: self.get_state(),
            read_count: diag.read_count,
            write_count: diag.write_count,
            error_count: diag.error_count,
            last_error: diag.last_error.clone(),
            extra: serde_json::json!({
                "input_pins": input_count,
                "output_pins": output_count,
                "channel_id": self.channel_id,
            }),
        })
    }
}

#[async_trait]
impl<S: DataStore + 'static> ProtocolClient for GpioChannel<S> {
    async fn connect(&mut self) -> Result<()> {
        // In a real implementation, validate GPIO chips and pins exist
        self.set_state(ConnectionState::Connected);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(task) = self.poll_task.take() {
            task.abort();
        }
        self.set_state(ConnectionState::Disconnected);
        Ok(())
    }

    async fn write_control(&mut self, commands: &[ControlCommand]) -> Result<WriteResult> {
        if !self.get_state().is_connected() {
            return Err(GatewayError::NotConnected);
        }

        let mut success_count = 0;
        let mut failures = Vec::new();

        for cmd in commands {
            // Find corresponding output pin
            let pin = self
                .config
                .pins
                .iter()
                .find(|p| p.point_id == cmd.id && p.direction == GpioDirection::Output);

            match pin {
                Some(p) => match self.write_pin(p, cmd.value).await {
                    Ok(()) => success_count += 1,
                    Err(e) => failures.push((cmd.id.clone(), e.to_string())),
                },
                None => {
                    failures.push((cmd.id.clone(), "Output pin not found".into()));
                }
            }
        }

        {
            let mut diag = self.diagnostics.write().await;
            diag.write_count += success_count as u64;
        }

        Ok(WriteResult {
            success_count,
            failures,
        })
    }

    async fn write_adjustment(
        &mut self,
        _adjustments: &[AdjustmentCommand],
    ) -> Result<WriteResult> {
        // GPIO doesn't support analog output
        Err(GatewayError::Unsupported(
            "GPIO does not support analog adjustment".into(),
        ))
    }

    async fn start_polling(&mut self, config: PollingConfig) -> Result<()> {
        let interval = Duration::from_millis(config.interval_ms);
        let state = self.state.clone();
        let store = self.store.clone();
        let channel_id = self.channel_id;
        let pins: Vec<GpioPinConfig> = self.config.input_pins().cloned().collect();

        let task = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);

            loop {
                ticker.tick().await;

                // Check if still connected
                if !state
                    .read()
                    .map(|s| s.is_connected())
                    .unwrap_or(false)
                {
                    break;
                }

                // Read all input pins
                let mut batch = DataBatch::new();
                for pin in &pins {
                    // Stub: return false
                    let value = if pin.active_low { true } else { false };
                    batch.add(DataPoint::signal(&pin.point_id, value));
                }

                // Store
                let _ = store.write_batch(channel_id, &batch).await;
            }
        });

        self.poll_task = Some(task);
        Ok(())
    }

    async fn stop_polling(&mut self) -> Result<()> {
        if let Some(task) = self.poll_task.take() {
            task.abort();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryStore;

    #[tokio::test]
    async fn test_gpio_channel_connect() {
        let store = Arc::new(MemoryStore::new());
        let config = GpioChannelConfig::new()
            .add_pin(GpioPinConfig::digital_input("gpiochip0", 17, "input1"))
            .add_pin(GpioPinConfig::digital_output("gpiochip0", 18, "output1"));

        let mut gpio = GpioChannel::new(config, store, 1);

        assert_eq!(gpio.connection_state(), ConnectionState::Disconnected);

        gpio.connect().await.unwrap();
        assert_eq!(gpio.connection_state(), ConnectionState::Connected);

        gpio.disconnect().await.unwrap();
        assert_eq!(gpio.connection_state(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_gpio_write_control() {
        let store = Arc::new(MemoryStore::new());
        let config = GpioChannelConfig::new()
            .add_pin(GpioPinConfig::digital_output("gpiochip0", 18, "led"));

        let mut gpio = GpioChannel::new(config, store, 1);
        gpio.connect().await.unwrap();

        let result = gpio
            .write_control(&[ControlCommand::latching("led", true)])
            .await
            .unwrap();

        assert_eq!(result.success_count, 1);
        assert!(result.failures.is_empty());

        // Check output state
        let state = gpio.read_output_state("led").await;
        assert_eq!(state, Some(true));
    }

    #[tokio::test]
    async fn test_gpio_diagnostics() {
        let store = Arc::new(MemoryStore::new());
        let config = GpioChannelConfig::new()
            .add_pin(GpioPinConfig::digital_input("gpiochip0", 17, "di1"))
            .add_pin(GpioPinConfig::digital_output("gpiochip0", 18, "do1"));

        let gpio = GpioChannel::new(config, store, 1);
        let diag = gpio.diagnostics().await.unwrap();

        assert_eq!(diag.protocol, "GPIO");
        assert_eq!(diag.extra["input_pins"], 1);
        assert_eq!(diag.extra["output_pins"], 1);
    }
}
