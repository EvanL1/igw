//! Modbus-specific types.

use std::time::Duration;

use crate::core::point::PointConfig;

/// Modbus connection type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModbusMode {
    /// Modbus TCP over network.
    Tcp,
    /// Modbus RTU over serial port.
    Rtu,
}

/// Modbus client configuration.
#[derive(Debug, Clone)]
pub struct ModbusConfig {
    /// Connection mode (TCP or RTU).
    pub mode: ModbusMode,

    /// Target address (host:port for TCP, device path for RTU).
    pub address: String,

    /// Connection timeout.
    pub connect_timeout: Duration,

    /// Read/write timeout.
    pub io_timeout: Duration,

    /// Maximum retries on error.
    pub max_retries: u32,

    /// Delay between retries.
    pub retry_delay: Duration,

    /// RTU-specific: baud rate.
    pub baud_rate: Option<u32>,

    /// RTU-specific: data bits.
    pub data_bits: Option<u8>,

    /// RTU-specific: parity (N/E/O).
    pub parity: Option<char>,

    /// RTU-specific: stop bits.
    pub stop_bits: Option<u8>,

    /// Point configurations.
    pub points: Vec<PointConfig>,
}

impl Default for ModbusConfig {
    fn default() -> Self {
        Self {
            mode: ModbusMode::Tcp,
            address: "127.0.0.1:502".to_string(),
            connect_timeout: Duration::from_secs(5),
            io_timeout: Duration::from_secs(3),
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
            baud_rate: None,
            data_bits: None,
            parity: None,
            stop_bits: None,
            points: Vec::new(),
        }
    }
}

/// Builder for ModbusConfig.
#[derive(Debug, Default)]
pub struct ModbusConfigBuilder {
    config: ModbusConfig,
}

impl ModbusConfigBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set TCP mode with address.
    pub fn tcp(mut self, address: impl Into<String>) -> Self {
        self.config.mode = ModbusMode::Tcp;
        self.config.address = address.into();
        self
    }

    /// Set RTU mode with device path.
    pub fn rtu(mut self, device: impl Into<String>) -> Self {
        self.config.mode = ModbusMode::Rtu;
        self.config.address = device.into();
        self
    }

    /// Set connection timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Set IO timeout.
    pub fn io_timeout(mut self, timeout: Duration) -> Self {
        self.config.io_timeout = timeout;
        self
    }

    /// Set maximum retries.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Set RTU baud rate.
    pub fn baud_rate(mut self, rate: u32) -> Self {
        self.config.baud_rate = Some(rate);
        self
    }

    /// Set RTU parity.
    pub fn parity(mut self, parity: char) -> Self {
        self.config.parity = Some(parity);
        self
    }

    /// Add a point configuration.
    pub fn add_point(mut self, point: PointConfig) -> Self {
        self.config.points.push(point);
        self
    }

    /// Set all points.
    pub fn points(mut self, points: Vec<PointConfig>) -> Self {
        self.config.points = points;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> ModbusConfig {
        self.config
    }
}

/// Modbus function codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FunctionCode {
    /// Read Coils (FC01)
    ReadCoils = 0x01,
    /// Read Discrete Inputs (FC02)
    ReadDiscreteInputs = 0x02,
    /// Read Holding Registers (FC03)
    ReadHoldingRegisters = 0x03,
    /// Read Input Registers (FC04)
    ReadInputRegisters = 0x04,
    /// Write Single Coil (FC05)
    WriteSingleCoil = 0x05,
    /// Write Single Register (FC06)
    WriteSingleRegister = 0x06,
    /// Write Multiple Coils (FC15)
    WriteMultipleCoils = 0x0F,
    /// Write Multiple Registers (FC16)
    WriteMultipleRegisters = 0x10,
}

impl FunctionCode {
    /// Check if this is a read function.
    pub fn is_read(&self) -> bool {
        matches!(
            self,
            Self::ReadCoils
                | Self::ReadDiscreteInputs
                | Self::ReadHoldingRegisters
                | Self::ReadInputRegisters
        )
    }

    /// Check if this is a write function.
    pub fn is_write(&self) -> bool {
        matches!(
            self,
            Self::WriteSingleCoil
                | Self::WriteSingleRegister
                | Self::WriteMultipleCoils
                | Self::WriteMultipleRegisters
        )
    }
}

impl From<u8> for FunctionCode {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::ReadCoils,
            0x02 => Self::ReadDiscreteInputs,
            0x03 => Self::ReadHoldingRegisters,
            0x04 => Self::ReadInputRegisters,
            0x05 => Self::WriteSingleCoil,
            0x06 => Self::WriteSingleRegister,
            0x0F => Self::WriteMultipleCoils,
            0x10 => Self::WriteMultipleRegisters,
            _ => Self::ReadHoldingRegisters, // Default fallback
        }
    }
}

/// Modbus exception codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExceptionCode {
    /// Illegal Function (01)
    IllegalFunction = 0x01,
    /// Illegal Data Address (02)
    IllegalDataAddress = 0x02,
    /// Illegal Data Value (03)
    IllegalDataValue = 0x03,
    /// Slave Device Failure (04)
    SlaveDeviceFailure = 0x04,
    /// Acknowledge (05)
    Acknowledge = 0x05,
    /// Slave Device Busy (06)
    SlaveDeviceBusy = 0x06,
    /// Memory Parity Error (08)
    MemoryParityError = 0x08,
    /// Gateway Path Unavailable (0A)
    GatewayPathUnavailable = 0x0A,
    /// Gateway Target Device Failed to Respond (0B)
    GatewayTargetDeviceFailed = 0x0B,
}

impl ExceptionCode {
    /// Get description of the exception.
    pub fn description(&self) -> &'static str {
        match self {
            Self::IllegalFunction => "Illegal Function",
            Self::IllegalDataAddress => "Illegal Data Address",
            Self::IllegalDataValue => "Illegal Data Value",
            Self::SlaveDeviceFailure => "Slave Device Failure",
            Self::Acknowledge => "Acknowledge",
            Self::SlaveDeviceBusy => "Slave Device Busy",
            Self::MemoryParityError => "Memory Parity Error",
            Self::GatewayPathUnavailable => "Gateway Path Unavailable",
            Self::GatewayTargetDeviceFailed => "Gateway Target Device Failed to Respond",
        }
    }
}

impl From<u8> for ExceptionCode {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::IllegalFunction,
            0x02 => Self::IllegalDataAddress,
            0x03 => Self::IllegalDataValue,
            0x04 => Self::SlaveDeviceFailure,
            0x05 => Self::Acknowledge,
            0x06 => Self::SlaveDeviceBusy,
            0x08 => Self::MemoryParityError,
            0x0A => Self::GatewayPathUnavailable,
            0x0B => Self::GatewayTargetDeviceFailed,
            _ => Self::SlaveDeviceFailure, // Default fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modbus_config_builder() {
        let config = ModbusConfigBuilder::new()
            .tcp("192.168.1.1:502")
            .io_timeout(Duration::from_secs(5))
            .max_retries(5)
            .build();

        assert_eq!(config.mode, ModbusMode::Tcp);
        assert_eq!(config.address, "192.168.1.1:502");
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_function_code() {
        assert!(FunctionCode::ReadHoldingRegisters.is_read());
        assert!(!FunctionCode::ReadHoldingRegisters.is_write());
        assert!(FunctionCode::WriteSingleRegister.is_write());
    }
}
