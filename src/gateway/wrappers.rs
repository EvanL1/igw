//! Protocol channel wrappers.
//!
//! Each wrapper implements `ChannelRuntime` by delegating to the underlying
//! protocol channel implementation.

use async_trait::async_trait;

use crate::core::error::Result;
use crate::core::traits::{
    AdjustmentCommand, ControlCommand, DataEventReceiver, Diagnostics, EventDrivenProtocol,
    PollResult, Protocol, ProtocolClient,
};

use super::runtime::ChannelRuntime;

// ============================================================================
// Virtual Channel Wrapper
// ============================================================================

use crate::protocols::virtual_channel::VirtualChannel;

/// Virtual channel runtime wrapper.
pub struct VirtualRuntime {
    id: u32,
    name: String,
    channel: VirtualChannel,
}

impl VirtualRuntime {
    pub fn new(id: u32, name: String, channel: VirtualChannel) -> Self {
        Self { id, name, channel }
    }
}

#[async_trait]
impl ChannelRuntime for VirtualRuntime {
    fn id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn protocol(&self) -> &str {
        "virtual"
    }

    fn is_event_driven(&self) -> bool {
        true
    }

    async fn connect(&mut self) -> Result<()> {
        self.channel.connect().await
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.channel.disconnect().await
    }

    async fn poll_once(&mut self) -> PollResult {
        self.channel.poll_once().await
    }

    async fn write_control(&mut self, commands: &[(u32, f64)]) -> Result<usize> {
        let cmds: Vec<_> = commands
            .iter()
            .map(|(id, value)| ControlCommand::latching(*id, *value != 0.0))
            .collect();
        let result = self.channel.write_control(&cmds).await?;
        Ok(result.success_count)
    }

    async fn write_adjustment(&mut self, adjustments: &[(u32, f64)]) -> Result<usize> {
        let adjs: Vec<_> = adjustments
            .iter()
            .map(|(id, value)| AdjustmentCommand::new(*id, *value))
            .collect();
        let result = self.channel.write_adjustment(&adjs).await?;
        Ok(result.success_count)
    }

    fn subscribe(&self) -> Option<DataEventReceiver> {
        Some(self.channel.subscribe())
    }

    async fn start_events(&mut self) -> Result<()> {
        self.channel.start().await
    }

    async fn stop_events(&mut self) -> Result<()> {
        self.channel.stop().await
    }

    async fn diagnostics(&self) -> Result<Diagnostics> {
        self.channel.diagnostics().await
    }
}

// ============================================================================
// Modbus Channel Wrapper
// ============================================================================

#[cfg(feature = "modbus")]
pub use modbus_wrapper::ModbusRuntime;

#[cfg(feature = "modbus")]
mod modbus_wrapper {
    use super::*;
    use crate::protocols::modbus::ModbusChannel;

    /// Modbus channel runtime wrapper.
    pub struct ModbusRuntime {
        id: u32,
        name: String,
        channel: ModbusChannel,
    }

    impl ModbusRuntime {
        pub fn new(id: u32, name: String, channel: ModbusChannel) -> Self {
            Self { id, name, channel }
        }
    }

    #[async_trait]
    impl ChannelRuntime for ModbusRuntime {
        fn id(&self) -> u32 {
            self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn protocol(&self) -> &str {
            "modbus"
        }

        fn is_event_driven(&self) -> bool {
            false
        }

        async fn connect(&mut self) -> Result<()> {
            self.channel.connect().await
        }

        async fn disconnect(&mut self) -> Result<()> {
            self.channel.disconnect().await
        }

        async fn poll_once(&mut self) -> PollResult {
            self.channel.poll_once().await
        }

        async fn write_control(&mut self, commands: &[(u32, f64)]) -> Result<usize> {
            let cmds: Vec<_> = commands
                .iter()
                .map(|(id, value)| ControlCommand::latching(*id, *value != 0.0))
                .collect();
            let result = self.channel.write_control(&cmds).await?;
            Ok(result.success_count)
        }

        async fn write_adjustment(&mut self, adjustments: &[(u32, f64)]) -> Result<usize> {
            let adjs: Vec<_> = adjustments
                .iter()
                .map(|(id, value)| AdjustmentCommand::new(*id, *value))
                .collect();
            let result = self.channel.write_adjustment(&adjs).await?;
            Ok(result.success_count)
        }

        fn subscribe(&self) -> Option<DataEventReceiver> {
            None // Modbus is polling-only
        }

        async fn start_events(&mut self) -> Result<()> {
            Ok(()) // No-op for polling channel
        }

        async fn stop_events(&mut self) -> Result<()> {
            Ok(()) // No-op for polling channel
        }

        async fn diagnostics(&self) -> Result<Diagnostics> {
            self.channel.diagnostics().await
        }
    }
}

// ============================================================================
// IEC104 Channel Wrapper
// ============================================================================

#[cfg(feature = "iec104")]
pub use iec104_wrapper::Iec104Runtime;

#[cfg(feature = "iec104")]
mod iec104_wrapper {
    use super::*;
    use crate::protocols::iec104::Iec104Channel;

    /// IEC104 channel runtime wrapper.
    pub struct Iec104Runtime {
        id: u32,
        name: String,
        channel: Iec104Channel,
    }

    impl Iec104Runtime {
        pub fn new(id: u32, name: String, channel: Iec104Channel) -> Self {
            Self { id, name, channel }
        }
    }

    #[async_trait]
    impl ChannelRuntime for Iec104Runtime {
        fn id(&self) -> u32 {
            self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn protocol(&self) -> &str {
            "iec104"
        }

        fn is_event_driven(&self) -> bool {
            true
        }

        async fn connect(&mut self) -> Result<()> {
            self.channel.connect().await
        }

        async fn disconnect(&mut self) -> Result<()> {
            self.channel.disconnect().await
        }

        async fn poll_once(&mut self) -> PollResult {
            self.channel.poll_once().await
        }

        async fn write_control(&mut self, commands: &[(u32, f64)]) -> Result<usize> {
            let cmds: Vec<_> = commands
                .iter()
                .map(|(id, value)| ControlCommand::latching(*id, *value != 0.0))
                .collect();
            let result = self.channel.write_control(&cmds).await?;
            Ok(result.success_count)
        }

        async fn write_adjustment(&mut self, adjustments: &[(u32, f64)]) -> Result<usize> {
            let adjs: Vec<_> = adjustments
                .iter()
                .map(|(id, value)| AdjustmentCommand::new(*id, *value))
                .collect();
            let result = self.channel.write_adjustment(&adjs).await?;
            Ok(result.success_count)
        }

        fn subscribe(&self) -> Option<DataEventReceiver> {
            Some(self.channel.subscribe())
        }

        async fn start_events(&mut self) -> Result<()> {
            self.channel.start().await
        }

        async fn stop_events(&mut self) -> Result<()> {
            self.channel.stop().await
        }

        async fn diagnostics(&self) -> Result<Diagnostics> {
            self.channel.diagnostics().await
        }
    }
}

// ============================================================================
// OPC UA Channel Wrapper
// ============================================================================

#[cfg(feature = "opcua")]
pub use opcua_wrapper::OpcUaRuntime;

#[cfg(feature = "opcua")]
mod opcua_wrapper {
    use super::*;
    use crate::protocols::opcua::OpcUaChannel;

    /// OPC UA channel runtime wrapper.
    pub struct OpcUaRuntime {
        id: u32,
        name: String,
        channel: OpcUaChannel,
    }

    impl OpcUaRuntime {
        pub fn new(id: u32, name: String, channel: OpcUaChannel) -> Self {
            Self { id, name, channel }
        }
    }

    #[async_trait]
    impl ChannelRuntime for OpcUaRuntime {
        fn id(&self) -> u32 {
            self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn protocol(&self) -> &str {
            "opcua"
        }

        fn is_event_driven(&self) -> bool {
            true
        }

        async fn connect(&mut self) -> Result<()> {
            self.channel.connect().await
        }

        async fn disconnect(&mut self) -> Result<()> {
            self.channel.disconnect().await
        }

        async fn poll_once(&mut self) -> PollResult {
            self.channel.poll_once().await
        }

        async fn write_control(&mut self, commands: &[(u32, f64)]) -> Result<usize> {
            let cmds: Vec<_> = commands
                .iter()
                .map(|(id, value)| ControlCommand::latching(*id, *value != 0.0))
                .collect();
            let result = self.channel.write_control(&cmds).await?;
            Ok(result.success_count)
        }

        async fn write_adjustment(&mut self, adjustments: &[(u32, f64)]) -> Result<usize> {
            let adjs: Vec<_> = adjustments
                .iter()
                .map(|(id, value)| AdjustmentCommand::new(*id, *value))
                .collect();
            let result = self.channel.write_adjustment(&adjs).await?;
            Ok(result.success_count)
        }

        fn subscribe(&self) -> Option<DataEventReceiver> {
            Some(self.channel.subscribe())
        }

        async fn start_events(&mut self) -> Result<()> {
            self.channel.start().await
        }

        async fn stop_events(&mut self) -> Result<()> {
            self.channel.stop().await
        }

        async fn diagnostics(&self) -> Result<Diagnostics> {
            self.channel.diagnostics().await
        }
    }
}

// ============================================================================
// CAN Channel Wrapper
// ============================================================================

#[cfg(all(feature = "can", target_os = "linux"))]
pub use can_wrapper::CanRuntime;

#[cfg(all(feature = "can", target_os = "linux"))]
mod can_wrapper {
    use super::*;
    use crate::protocols::can::CanClient;

    /// CAN channel runtime wrapper.
    pub struct CanRuntime {
        id: u32,
        name: String,
        channel: CanClient,
    }

    impl CanRuntime {
        pub fn new(id: u32, name: String, channel: CanClient) -> Self {
            Self { id, name, channel }
        }
    }

    #[async_trait]
    impl ChannelRuntime for CanRuntime {
        fn id(&self) -> u32 {
            self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn protocol(&self) -> &str {
            "can"
        }

        fn is_event_driven(&self) -> bool {
            true
        }

        async fn connect(&mut self) -> Result<()> {
            self.channel.connect().await
        }

        async fn disconnect(&mut self) -> Result<()> {
            self.channel.disconnect().await
        }

        async fn poll_once(&mut self) -> PollResult {
            self.channel.poll_once().await
        }

        async fn write_control(&mut self, _commands: &[(u32, f64)]) -> Result<usize> {
            // CAN write not supported
            Ok(0)
        }

        async fn write_adjustment(&mut self, _adjustments: &[(u32, f64)]) -> Result<usize> {
            // CAN write not supported
            Ok(0)
        }

        fn subscribe(&self) -> Option<DataEventReceiver> {
            Some(self.channel.subscribe())
        }

        async fn start_events(&mut self) -> Result<()> {
            self.channel.start().await
        }

        async fn stop_events(&mut self) -> Result<()> {
            self.channel.stop().await
        }

        async fn diagnostics(&self) -> Result<Diagnostics> {
            self.channel.diagnostics().await
        }
    }
}

// ============================================================================
// GPIO Channel Wrapper
// ============================================================================

#[cfg(all(feature = "gpio", target_os = "linux"))]
pub use gpio_wrapper::GpioRuntime;

#[cfg(all(feature = "gpio", target_os = "linux"))]
mod gpio_wrapper {
    use super::*;
    use crate::protocols::gpio::GpioChannel;

    /// GPIO channel runtime wrapper.
    pub struct GpioRuntime {
        id: u32,
        name: String,
        channel: GpioChannel,
    }

    impl GpioRuntime {
        pub fn new(id: u32, name: String, channel: GpioChannel) -> Self {
            Self { id, name, channel }
        }
    }

    #[async_trait]
    impl ChannelRuntime for GpioRuntime {
        fn id(&self) -> u32 {
            self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn protocol(&self) -> &str {
            "gpio"
        }

        fn is_event_driven(&self) -> bool {
            false
        }

        async fn connect(&mut self) -> Result<()> {
            self.channel.connect().await
        }

        async fn disconnect(&mut self) -> Result<()> {
            self.channel.disconnect().await
        }

        async fn poll_once(&mut self) -> PollResult {
            self.channel.poll_once().await
        }

        async fn write_control(&mut self, commands: &[(u32, f64)]) -> Result<usize> {
            let cmds: Vec<_> = commands
                .iter()
                .map(|(id, value)| ControlCommand::latching(*id, *value != 0.0))
                .collect();
            let result = self.channel.write_control(&cmds).await?;
            Ok(result.success_count)
        }

        async fn write_adjustment(&mut self, adjustments: &[(u32, f64)]) -> Result<usize> {
            let adjs: Vec<_> = adjustments
                .iter()
                .map(|(id, value)| AdjustmentCommand::new(*id, *value))
                .collect();
            let result = self.channel.write_adjustment(&adjs).await?;
            Ok(result.success_count)
        }

        fn subscribe(&self) -> Option<DataEventReceiver> {
            None // GPIO is polling-only
        }

        async fn start_events(&mut self) -> Result<()> {
            Ok(()) // No-op for polling channel
        }

        async fn stop_events(&mut self) -> Result<()> {
            Ok(()) // No-op for polling channel
        }

        async fn diagnostics(&self) -> Result<Diagnostics> {
            self.channel.diagnostics().await
        }
    }
}
