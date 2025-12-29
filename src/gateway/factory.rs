//! Channel factory.
//!
//! Creates `ChannelRuntime` instances from configuration.

use crate::core::error::{GatewayError, Result};
use crate::core::point::PointConfig;

use super::config::ChannelConfig;
use super::parse_address;
use super::runtime::ChannelRuntime;
use super::wrappers::VirtualRuntime;

/// Create a channel from configuration.
pub fn create_channel(config: &ChannelConfig) -> Result<Box<dyn ChannelRuntime>> {
    match config.protocol.to_lowercase().as_str() {
        #[cfg(feature = "modbus")]
        "modbus" => create_modbus_channel(config),

        #[cfg(feature = "iec104")]
        "iec104" => create_iec104_channel(config),

        #[cfg(feature = "opcua")]
        "opcua" => create_opcua_channel(config),

        #[cfg(all(feature = "can", target_os = "linux"))]
        "can" => create_can_channel(config),

        #[cfg(all(feature = "gpio", target_os = "linux"))]
        "gpio" => create_gpio_channel(config),

        "virtual" => create_virtual_channel(config),

        protocol => Err(GatewayError::Config(format!(
            "Unsupported protocol: {}. Check if the required feature is enabled.",
            protocol
        ))),
    }
}

/// Convert PointDef list to PointConfig list.
fn build_point_configs(config: &ChannelConfig) -> Result<Vec<PointConfig>> {
    let mut points = Vec::new();

    for point_def in &config.points {
        if !point_def.enabled {
            continue;
        }

        let address = parse_address(&config.protocol, &point_def.address)?;

        points.push(PointConfig {
            id: point_def.id,
            name: Some(point_def.name.clone()),
            address,
            transform: point_def.transform.clone(),
            poll_group: None,
            enabled: true,
        });
    }

    Ok(points)
}

// ============================================================================
// Protocol-specific channel creators
// ============================================================================

#[cfg(feature = "modbus")]
fn create_modbus_channel(config: &ChannelConfig) -> Result<Box<dyn ChannelRuntime>> {
    use super::wrappers::ModbusRuntime;
    use crate::protocols::modbus::ModbusChannelParamsConfig;

    // Parse parameters
    let params: ModbusChannelParamsConfig = serde_json::from_value(config.parameters.clone())
        .map_err(|e| GatewayError::Config(format!("Invalid Modbus parameters: {}", e)))?;

    // Build channel config
    let channel_config = params.to_channel_config();

    // Build point configs
    let points = build_point_configs(config)?;
    let channel_config = channel_config.with_points(points);

    // Create channel
    let channel = crate::protocols::modbus::ModbusChannel::new(channel_config, config.id);

    Ok(Box::new(ModbusRuntime::new(
        config.id,
        config.name.clone(),
        channel,
    )))
}

#[cfg(feature = "iec104")]
fn create_iec104_channel(config: &ChannelConfig) -> Result<Box<dyn ChannelRuntime>> {
    use super::wrappers::Iec104Runtime;
    use crate::protocols::iec104::Iec104ParamsConfig;

    // Parse parameters
    let params: Iec104ParamsConfig = serde_json::from_value(config.parameters.clone())
        .map_err(|e| GatewayError::Config(format!("Invalid IEC104 parameters: {}", e)))?;

    // Build point configs
    let points = build_point_configs(config)?;

    // Build channel config
    let channel_config = params.to_config().with_points(points);

    // Create channel
    let channel = crate::protocols::iec104::Iec104Channel::new(channel_config);

    Ok(Box::new(Iec104Runtime::new(
        config.id,
        config.name.clone(),
        channel,
    )))
}

#[cfg(feature = "opcua")]
fn create_opcua_channel(config: &ChannelConfig) -> Result<Box<dyn ChannelRuntime>> {
    use super::wrappers::OpcUaRuntime;
    use crate::protocols::opcua::OpcUaParamsConfig;

    // Parse parameters
    let params: OpcUaParamsConfig = serde_json::from_value(config.parameters.clone())
        .map_err(|e| GatewayError::Config(format!("Invalid OPC UA parameters: {}", e)))?;

    // Build point configs
    let points = build_point_configs(config)?;

    // Build channel config
    let channel_config = params.to_config().with_points(points);

    // Create channel
    let channel = crate::protocols::opcua::OpcUaChannel::new(channel_config);

    Ok(Box::new(OpcUaRuntime::new(
        config.id,
        config.name.clone(),
        channel,
    )))
}

#[cfg(all(feature = "can", target_os = "linux"))]
fn create_can_channel(config: &ChannelConfig) -> Result<Box<dyn ChannelRuntime>> {
    use super::wrappers::CanRuntime;
    use crate::protocols::can::CanChannelParamsConfig;

    // Parse parameters
    let params: CanChannelParamsConfig = serde_json::from_value(config.parameters.clone())
        .map_err(|e| GatewayError::Config(format!("Invalid CAN parameters: {}", e)))?;

    // Build channel config
    let channel_config = params.to_config();

    // Create channel
    let channel = crate::protocols::can::CanClient::new(channel_config);

    Ok(Box::new(CanRuntime::new(
        config.id,
        config.name.clone(),
        channel,
    )))
}

#[cfg(all(feature = "gpio", target_os = "linux"))]
fn create_gpio_channel(config: &ChannelConfig) -> Result<Box<dyn ChannelRuntime>> {
    use super::wrappers::GpioRuntime;
    use crate::protocols::gpio::GpioChannelParamsConfig;

    // Parse parameters
    let params: GpioChannelParamsConfig = serde_json::from_value(config.parameters.clone())
        .map_err(|e| GatewayError::Config(format!("Invalid GPIO parameters: {}", e)))?;

    // Build channel config
    let channel_config = params.to_config();

    // Create channel
    let channel = crate::protocols::gpio::GpioChannel::new(channel_config);

    Ok(Box::new(GpioRuntime::new(
        config.id,
        config.name.clone(),
        channel,
    )))
}

fn create_virtual_channel(config: &ChannelConfig) -> Result<Box<dyn ChannelRuntime>> {
    use crate::protocols::virtual_channel::{VirtualChannel, VirtualChannelParamsConfig};

    // Parse parameters (optional for virtual)
    let params: VirtualChannelParamsConfig =
        serde_json::from_value(config.parameters.clone()).unwrap_or_default();

    // Build point configs
    let points = build_point_configs(config)?;

    // Build channel config
    let channel_config = params.to_config().with_points(points);

    // Create channel
    let channel = VirtualChannel::new(channel_config);

    Ok(Box::new(VirtualRuntime::new(
        config.id,
        config.name.clone(),
        channel,
    )))
}
