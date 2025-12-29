//! Protocol address parsing.
//!
//! Converts shorthand address strings to `ProtocolAddress` enum variants.

use crate::core::error::{GatewayError, Result};
use crate::core::point::{
    Iec104Address, ModbusAddress, OpcUaAddress, ProtocolAddress, VirtualAddress,
};

#[cfg(feature = "gpio")]
use crate::core::point::GpioAddress;

/// Parse a shorthand address string into a `ProtocolAddress`.
///
/// # Address Formats
///
/// - **Modbus**: `"slave_id:register"` or `"slave_id:register:function_code"`
///   - Example: `"1:100"` → slave_id=1, register=100, function_code=3 (default)
///   - Example: `"1:100:4"` → slave_id=1, register=100, function_code=4
///
/// - **IEC104**: `"ioa"` or `"ioa:type_id"`
///   - Example: `"1001"` → ioa=1001
///   - Example: `"1001:13"` → ioa=1001, type_id=13
///
/// - **OPC UA**: Standard OPC UA node ID format
///   - Example: `"ns=2;i=1234"` → namespace=2, node_id="i=1234"
///   - Example: `"ns=2;s=Temperature"` → namespace=2, node_id="s=Temperature"
///   - Example: `"i=1234"` → namespace=0, node_id="i=1234"
///
/// - **CAN**: `"can_id:byte_offset:bit_pos:bit_len"`
///   - Example: `"0x100:0:0:16"` → can_id=0x100, byte_offset=0, bit_pos=0, bit_len=16
///
/// - **GPIO**: `"pin_number"` or `"pin_number:direction"`
///   - Example: `"17"` → pin=17, direction=input (default)
///   - Example: `"18:output"` → pin=18, direction=output
///
/// - **Virtual**: Any string key
///   - Example: `"temperature"` → key="temperature"
pub fn parse_address(protocol: &str, address: &str) -> Result<ProtocolAddress> {
    match protocol.to_lowercase().as_str() {
        "modbus" => parse_modbus_address(address),
        "iec104" => parse_iec104_address(address),
        "opcua" => parse_opcua_address(address),
        "can" => parse_can_address(address),
        #[cfg(feature = "gpio")]
        "gpio" => parse_gpio_address(address),
        "virtual" => Ok(ProtocolAddress::Virtual(VirtualAddress::new(address))),
        _ => Err(GatewayError::Config(format!(
            "Unknown protocol: {}",
            protocol
        ))),
    }
}

/// Parse Modbus address: "slave_id:register" or "slave_id:register:function_code"
fn parse_modbus_address(address: &str) -> Result<ProtocolAddress> {
    let parts: Vec<&str> = address.split(':').collect();

    match parts.len() {
        2 => {
            let slave_id = parts[0]
                .parse::<u8>()
                .map_err(|_| GatewayError::Config(format!("Invalid slave_id: {}", parts[0])))?;
            let register = parts[1]
                .parse::<u16>()
                .map_err(|_| GatewayError::Config(format!("Invalid register: {}", parts[1])))?;

            Ok(ProtocolAddress::Modbus(ModbusAddress::holding_register(
                slave_id,
                register,
                crate::core::point::DataFormat::default(),
            )))
        }
        3 => {
            let slave_id = parts[0]
                .parse::<u8>()
                .map_err(|_| GatewayError::Config(format!("Invalid slave_id: {}", parts[0])))?;
            let register = parts[1]
                .parse::<u16>()
                .map_err(|_| GatewayError::Config(format!("Invalid register: {}", parts[1])))?;
            let function_code = parts[2]
                .parse::<u8>()
                .map_err(|_| GatewayError::Config(format!("Invalid function_code: {}", parts[2])))?;

            Ok(ProtocolAddress::Modbus(ModbusAddress {
                slave_id,
                register,
                function_code,
                format: crate::core::point::DataFormat::default(),
                byte_order: crate::core::point::ByteOrder::default(),
                bit_position: None,
            }))
        }
        _ => Err(GatewayError::Config(format!(
            "Invalid Modbus address format: {}. Expected 'slave_id:register' or 'slave_id:register:function_code'",
            address
        ))),
    }
}

/// Parse IEC104 address: "ioa" or "ioa:type_id"
fn parse_iec104_address(address: &str) -> Result<ProtocolAddress> {
    let parts: Vec<&str> = address.split(':').collect();

    match parts.len() {
        1 => {
            let ioa = parts[0]
                .parse::<u32>()
                .map_err(|_| GatewayError::Config(format!("Invalid IOA: {}", parts[0])))?;

            Ok(ProtocolAddress::Iec104(Iec104Address {
                ioa,
                type_id: 0, // Will be inferred from data
                common_address: 1,
            }))
        }
        2 => {
            let ioa = parts[0]
                .parse::<u32>()
                .map_err(|_| GatewayError::Config(format!("Invalid IOA: {}", parts[0])))?;
            let type_id = parts[1]
                .parse::<u8>()
                .map_err(|_| GatewayError::Config(format!("Invalid type_id: {}", parts[1])))?;

            Ok(ProtocolAddress::Iec104(Iec104Address {
                ioa,
                type_id,
                common_address: 1,
            }))
        }
        _ => Err(GatewayError::Config(format!(
            "Invalid IEC104 address format: {}. Expected 'ioa' or 'ioa:type_id'",
            address
        ))),
    }
}

/// Parse OPC UA address: "ns=N;i=ID" or "ns=N;s=Name" or "i=ID"
fn parse_opcua_address(address: &str) -> Result<ProtocolAddress> {
    let mut namespace_index = 0u16;
    let mut node_id = address.to_string();

    // Check for namespace prefix
    if address.starts_with("ns=") {
        if let Some(semi_pos) = address.find(';') {
            let ns_str = &address[3..semi_pos];
            namespace_index = ns_str
                .parse()
                .map_err(|_| GatewayError::Config(format!("Invalid namespace: {}", ns_str)))?;
            node_id = address[semi_pos + 1..].to_string();
        } else {
            return Err(GatewayError::Config(format!(
                "Invalid OPC UA address format: {}. Expected 'ns=N;i=ID' or 'ns=N;s=Name'",
                address
            )));
        }
    }

    // Validate node ID format
    if !node_id.starts_with("i=")
        && !node_id.starts_with("s=")
        && !node_id.starts_with("g=")
        && !node_id.starts_with("b=")
    {
        return Err(GatewayError::Config(format!(
            "Invalid OPC UA node ID: {}. Expected 'i=N', 's=Name', 'g=GUID', or 'b=Base64'",
            node_id
        )));
    }

    Ok(ProtocolAddress::OpcUa(OpcUaAddress {
        node_id,
        namespace_index,
    }))
}

/// Parse CAN address: "can_id:byte_offset:bit_pos:bit_len"
fn parse_can_address(address: &str) -> Result<ProtocolAddress> {
    // For now, store as Generic since CAN address is complex
    // TODO: Add CanAddress to ProtocolAddress enum
    Ok(ProtocolAddress::Generic(address.to_string()))
}

/// Parse GPIO address: "pin_number" or "chip:pin" or "chip:pin:direction"
#[cfg(feature = "gpio")]
fn parse_gpio_address(address: &str) -> Result<ProtocolAddress> {
    let parts: Vec<&str> = address.split(':').collect();

    match parts.len() {
        1 => {
            // Just pin number, default chip
            let pin = parts[0]
                .parse::<u32>()
                .map_err(|_| GatewayError::Config(format!("Invalid GPIO pin: {}", parts[0])))?;
            Ok(ProtocolAddress::Gpio(GpioAddress::digital_input(
                "gpiochip0",
                pin,
            )))
        }
        2 => {
            // chip:pin
            let chip = parts[0].to_string();
            let pin = parts[1]
                .parse::<u32>()
                .map_err(|_| GatewayError::Config(format!("Invalid GPIO pin: {}", parts[1])))?;
            Ok(ProtocolAddress::Gpio(GpioAddress::digital_input(chip, pin)))
        }
        3 => {
            // chip:pin:direction
            let chip = parts[0].to_string();
            let pin = parts[1]
                .parse::<u32>()
                .map_err(|_| GatewayError::Config(format!("Invalid GPIO pin: {}", parts[1])))?;
            let addr = match parts[2].to_lowercase().as_str() {
                "input" | "in" | "di" => GpioAddress::digital_input(chip, pin),
                "output" | "out" | "do" => GpioAddress::digital_output(chip, pin),
                _ => {
                    return Err(GatewayError::Config(format!(
                        "Invalid GPIO direction: {}. Expected 'input' or 'output'",
                        parts[2]
                    )))
                }
            };
            Ok(ProtocolAddress::Gpio(addr))
        }
        _ => Err(GatewayError::Config(format!(
            "Invalid GPIO address format: {}. Expected 'pin', 'chip:pin', or 'chip:pin:direction'",
            address
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modbus_address() {
        let addr = parse_modbus_address("1:100").unwrap();
        if let ProtocolAddress::Modbus(m) = addr {
            assert_eq!(m.slave_id, 1);
            assert_eq!(m.register, 100);
            assert_eq!(m.function_code, 3);
        } else {
            panic!("Expected Modbus address");
        }
    }

    #[test]
    fn test_parse_modbus_address_with_function() {
        let addr = parse_modbus_address("2:200:4").unwrap();
        if let ProtocolAddress::Modbus(m) = addr {
            assert_eq!(m.slave_id, 2);
            assert_eq!(m.register, 200);
            assert_eq!(m.function_code, 4);
        } else {
            panic!("Expected Modbus address");
        }
    }

    #[test]
    fn test_parse_iec104_address() {
        let addr = parse_iec104_address("1001").unwrap();
        if let ProtocolAddress::Iec104(i) = addr {
            assert_eq!(i.ioa, 1001);
        } else {
            panic!("Expected IEC104 address");
        }
    }

    #[test]
    fn test_parse_opcua_address() {
        let addr = parse_opcua_address("ns=2;i=1234").unwrap();
        if let ProtocolAddress::OpcUa(o) = addr {
            assert_eq!(o.namespace_index, 2);
            assert_eq!(o.node_id, "i=1234");
        } else {
            panic!("Expected OPC UA address");
        }
    }

    #[test]
    fn test_parse_opcua_address_no_namespace() {
        let addr = parse_opcua_address("i=1234").unwrap();
        if let ProtocolAddress::OpcUa(o) = addr {
            assert_eq!(o.namespace_index, 0);
            assert_eq!(o.node_id, "i=1234");
        } else {
            panic!("Expected OPC UA address");
        }
    }

    #[test]
    fn test_parse_virtual_address() {
        let addr = parse_address("virtual", "temperature").unwrap();
        if let ProtocolAddress::Virtual(v) = addr {
            assert_eq!(v.tag, "temperature");
        } else {
            panic!("Expected Virtual address");
        }
    }
}
