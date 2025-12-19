//! J1939 SPN Database
//!
//! Complete database of common engine/generator SPNs based on SAE J1939 standard.
//! Point ID = SPN (globally unique per SAE J1939 standard).
//!
//! This database covers the most commonly used PGNs for diesel generators and
//! industrial engines. Data is automatically decoded when matching PGNs are received.

use std::collections::HashMap;

/// Data type for SPN values
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpnDataType {
    Uint8,
    Uint16,
    Uint32,
    Int8,
    Int16,
    Int32,
}

/// SPN definition structure
#[derive(Debug, Clone)]
pub struct SpnDef {
    pub spn: u32,
    pub name: &'static str,
    pub pgn: u32,
    pub start_byte: u8,
    pub start_bit: u8,
    pub bit_length: u8,
    pub scale: f64,
    pub offset: f64,
    pub unit: &'static str,
    pub data_type: SpnDataType,
}

// ============================================================================
// SPN Database - Complete definitions for common engine PGNs
// ============================================================================

/// All SPN definitions in the database
pub static SPN_DEFINITIONS: &[SpnDef] = &[
    // ========================================================================
    // EEC1 - Electronic Engine Controller 1 (PGN 61444 / 0xF004)
    // Broadcast rate: 10-100ms (engine dependent)
    // ========================================================================
    SpnDef {
        spn: 899,
        name: "engine_torque_mode",
        pgn: 61444,
        start_byte: 0,
        start_bit: 0,
        bit_length: 4,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 4154,
        name: "actual_engine_retarder_percent",
        pgn: 61444,
        start_byte: 1,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: -125.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 512,
        name: "drivers_demand_engine_percent",
        pgn: 61444,
        start_byte: 1,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: -125.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 513,
        name: "actual_engine_percent_torque",
        pgn: 61444,
        start_byte: 2,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: -125.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 190,
        name: "engine_speed",
        pgn: 61444,
        start_byte: 3,
        start_bit: 0,
        bit_length: 16,
        scale: 0.125,
        offset: 0.0,
        unit: "RPM",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 1483,
        name: "eec1_source_address",
        pgn: 61444,
        start_byte: 5,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 1675,
        name: "engine_starter_mode",
        pgn: 61444,
        start_byte: 6,
        start_bit: 0,
        bit_length: 4,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 2432,
        name: "engine_demand_percent_torque",
        pgn: 61444,
        start_byte: 7,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: -125.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    // ========================================================================
    // EEC2 - Electronic Engine Controller 2 (PGN 61443 / 0xF003)
    // Broadcast rate: 50ms
    // ========================================================================
    SpnDef {
        spn: 558,
        name: "accelerator_pedal_1_low_switch",
        pgn: 61443,
        start_byte: 0,
        start_bit: 0,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 559,
        name: "accelerator_pedal_kickdown",
        pgn: 61443,
        start_byte: 0,
        start_bit: 2,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 1437,
        name: "road_speed_limit_status",
        pgn: 61443,
        start_byte: 0,
        start_bit: 4,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 2970,
        name: "accelerator_pedal_2_low_switch",
        pgn: 61443,
        start_byte: 0,
        start_bit: 6,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 91,
        name: "accelerator_pedal_position_1",
        pgn: 61443,
        start_byte: 1,
        start_bit: 0,
        bit_length: 8,
        scale: 0.4,
        offset: 0.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 92,
        name: "percent_load_current_speed",
        pgn: 61443,
        start_byte: 2,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: 0.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 974,
        name: "remote_accelerator_position",
        pgn: 61443,
        start_byte: 3,
        start_bit: 0,
        bit_length: 8,
        scale: 0.4,
        offset: 0.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 29,
        name: "accelerator_pedal_position_2",
        pgn: 61443,
        start_byte: 4,
        start_bit: 0,
        bit_length: 8,
        scale: 0.4,
        offset: 0.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 2979,
        name: "vehicle_acceleration_rate_limit",
        pgn: 61443,
        start_byte: 5,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 5021,
        name: "momentary_engine_max_power_enable",
        pgn: 61443,
        start_byte: 6,
        start_bit: 0,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    // ========================================================================
    // EEC3 - Electronic Engine Controller 3 (PGN 65247 / 0xFEDF)
    // Broadcast rate: 250ms
    // ========================================================================
    SpnDef {
        spn: 514,
        name: "nominal_friction_percent_torque",
        pgn: 65247,
        start_byte: 0,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: -125.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 515,
        name: "engine_desired_operating_speed",
        pgn: 65247,
        start_byte: 1,
        start_bit: 0,
        bit_length: 16,
        scale: 0.125,
        offset: 0.0,
        unit: "RPM",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 519,
        name: "engine_operating_speed_asymmetry_adjust",
        pgn: 65247,
        start_byte: 3,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 2978,
        name: "estimated_engine_parasitic_losses",
        pgn: 65247,
        start_byte: 4,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: -125.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 6595,
        name: "aftertreatment_1_exhaust_gas_mass_flow",
        pgn: 65247,
        start_byte: 5,
        start_bit: 0,
        bit_length: 16,
        scale: 0.2,
        offset: 0.0,
        unit: "kg/h",
        data_type: SpnDataType::Uint16,
    },
    // ========================================================================
    // ET1 - Engine Temperature 1 (PGN 65262 / 0xFEEE)
    // Broadcast rate: 1000ms
    // ========================================================================
    SpnDef {
        spn: 110,
        name: "engine_coolant_temperature",
        pgn: 65262,
        start_byte: 0,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: -40.0,
        unit: "°C",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 174,
        name: "fuel_temperature",
        pgn: 65262,
        start_byte: 1,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: -40.0,
        unit: "°C",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 175,
        name: "engine_oil_temperature_1",
        pgn: 65262,
        start_byte: 2,
        start_bit: 0,
        bit_length: 16,
        scale: 0.03125,
        offset: -273.0,
        unit: "°C",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 176,
        name: "turbo_oil_temperature",
        pgn: 65262,
        start_byte: 4,
        start_bit: 0,
        bit_length: 16,
        scale: 0.03125,
        offset: -273.0,
        unit: "°C",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 52,
        name: "engine_intercooler_temperature",
        pgn: 65262,
        start_byte: 6,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: -40.0,
        unit: "°C",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 1134,
        name: "engine_intercooler_thermostat_opening",
        pgn: 65262,
        start_byte: 7,
        start_bit: 0,
        bit_length: 8,
        scale: 0.4,
        offset: 0.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    // ========================================================================
    // EFL/P1 - Engine Fluid Level/Pressure 1 (PGN 65263 / 0xFEEF)
    // Broadcast rate: 500ms
    // ========================================================================
    SpnDef {
        spn: 94,
        name: "fuel_delivery_pressure",
        pgn: 65263,
        start_byte: 0,
        start_bit: 0,
        bit_length: 8,
        scale: 4.0,
        offset: 0.0,
        unit: "kPa",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 22,
        name: "extended_crankcase_blowby_pressure",
        pgn: 65263,
        start_byte: 1,
        start_bit: 0,
        bit_length: 8,
        scale: 0.05,
        offset: 0.0,
        unit: "kPa",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 98,
        name: "engine_oil_level",
        pgn: 65263,
        start_byte: 2,
        start_bit: 0,
        bit_length: 8,
        scale: 0.4,
        offset: 0.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 100,
        name: "engine_oil_pressure",
        pgn: 65263,
        start_byte: 3,
        start_bit: 0,
        bit_length: 8,
        scale: 4.0,
        offset: 0.0,
        unit: "kPa",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 101,
        name: "crankcase_pressure",
        pgn: 65263,
        start_byte: 4,
        start_bit: 0,
        bit_length: 16,
        scale: 0.0078125,
        offset: -250.0,
        unit: "kPa",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 109,
        name: "coolant_pressure",
        pgn: 65263,
        start_byte: 6,
        start_bit: 0,
        bit_length: 8,
        scale: 2.0,
        offset: 0.0,
        unit: "kPa",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 111,
        name: "coolant_level",
        pgn: 65263,
        start_byte: 7,
        start_bit: 0,
        bit_length: 8,
        scale: 0.4,
        offset: 0.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    // ========================================================================
    // IC1 - Inlet/Exhaust Conditions 1 (PGN 65270 / 0xFEF6)
    // Broadcast rate: 500ms
    // ========================================================================
    SpnDef {
        spn: 81,
        name: "particulate_trap_inlet_pressure",
        pgn: 65270,
        start_byte: 0,
        start_bit: 0,
        bit_length: 8,
        scale: 0.5,
        offset: 0.0,
        unit: "kPa",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 102,
        name: "boost_pressure",
        pgn: 65270,
        start_byte: 1,
        start_bit: 0,
        bit_length: 8,
        scale: 2.0,
        offset: 0.0,
        unit: "kPa",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 105,
        name: "intake_manifold_temperature",
        pgn: 65270,
        start_byte: 2,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: -40.0,
        unit: "°C",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 106,
        name: "air_inlet_pressure",
        pgn: 65270,
        start_byte: 3,
        start_bit: 0,
        bit_length: 8,
        scale: 2.0,
        offset: 0.0,
        unit: "kPa",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 107,
        name: "air_filter_differential_pressure",
        pgn: 65270,
        start_byte: 4,
        start_bit: 0,
        bit_length: 8,
        scale: 0.05,
        offset: 0.0,
        unit: "kPa",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 173,
        name: "exhaust_gas_temperature",
        pgn: 65270,
        start_byte: 5,
        start_bit: 0,
        bit_length: 16,
        scale: 0.03125,
        offset: -273.0,
        unit: "°C",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 112,
        name: "coolant_filter_differential_pressure",
        pgn: 65270,
        start_byte: 7,
        start_bit: 0,
        bit_length: 8,
        scale: 0.5,
        offset: 0.0,
        unit: "kPa",
        data_type: SpnDataType::Uint8,
    },
    // ========================================================================
    // VEP1 - Vehicle Electrical Power 1 (PGN 65271 / 0xFEF7)
    // Broadcast rate: 1000ms
    // ========================================================================
    SpnDef {
        spn: 114,
        name: "net_battery_current",
        pgn: 65271,
        start_byte: 0,
        start_bit: 0,
        bit_length: 16,
        scale: 1.0,
        offset: -125.0,
        unit: "A",
        data_type: SpnDataType::Int16,
    },
    SpnDef {
        spn: 115,
        name: "alternator_current",
        pgn: 65271,
        start_byte: 2,
        start_bit: 0,
        bit_length: 16,
        scale: 1.0,
        offset: 0.0,
        unit: "A",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 168,
        name: "battery_potential",
        pgn: 65271,
        start_byte: 4,
        start_bit: 0,
        bit_length: 16,
        scale: 0.05,
        offset: 0.0,
        unit: "V",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 158,
        name: "keyswitch_battery_potential",
        pgn: 65271,
        start_byte: 6,
        start_bit: 0,
        bit_length: 16,
        scale: 0.05,
        offset: 0.0,
        unit: "V",
        data_type: SpnDataType::Uint16,
    },
    // ========================================================================
    // AMB - Ambient Conditions (PGN 65269 / 0xFEF5)
    // Broadcast rate: 1000ms
    // ========================================================================
    SpnDef {
        spn: 108,
        name: "barometric_pressure",
        pgn: 65269,
        start_byte: 0,
        start_bit: 0,
        bit_length: 8,
        scale: 0.5,
        offset: 0.0,
        unit: "kPa",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 170,
        name: "cab_interior_temperature",
        pgn: 65269,
        start_byte: 1,
        start_bit: 0,
        bit_length: 16,
        scale: 0.03125,
        offset: -273.0,
        unit: "°C",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 171,
        name: "ambient_air_temperature",
        pgn: 65269,
        start_byte: 3,
        start_bit: 0,
        bit_length: 16,
        scale: 0.03125,
        offset: -273.0,
        unit: "°C",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 172,
        name: "air_inlet_temperature",
        pgn: 65269,
        start_byte: 5,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: -40.0,
        unit: "°C",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 79,
        name: "road_surface_temperature",
        pgn: 65269,
        start_byte: 6,
        start_bit: 0,
        bit_length: 16,
        scale: 0.03125,
        offset: -273.0,
        unit: "°C",
        data_type: SpnDataType::Uint16,
    },
    // ========================================================================
    // LFE - Liquid Fuel Economy (PGN 65266 / 0xFEF2)
    // Broadcast rate: 100ms
    // ========================================================================
    SpnDef {
        spn: 183,
        name: "fuel_rate",
        pgn: 65266,
        start_byte: 0,
        start_bit: 0,
        bit_length: 16,
        scale: 0.05,
        offset: 0.0,
        unit: "L/h",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 184,
        name: "instantaneous_fuel_economy",
        pgn: 65266,
        start_byte: 2,
        start_bit: 0,
        bit_length: 16,
        scale: 0.001953125,
        offset: 0.0,
        unit: "km/L",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 185,
        name: "average_fuel_economy",
        pgn: 65266,
        start_byte: 4,
        start_bit: 0,
        bit_length: 16,
        scale: 0.001953125,
        offset: 0.0,
        unit: "km/L",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 51,
        name: "throttle_position",
        pgn: 65266,
        start_byte: 6,
        start_bit: 0,
        bit_length: 8,
        scale: 0.4,
        offset: 0.0,
        unit: "%",
        data_type: SpnDataType::Uint8,
    },
    // ========================================================================
    // HOURS - Engine Hours, Revolutions (PGN 65253 / 0xFEE5)
    // Broadcast rate: On request or 1000ms
    // ========================================================================
    SpnDef {
        spn: 247,
        name: "engine_total_hours_of_operation",
        pgn: 65253,
        start_byte: 0,
        start_bit: 0,
        bit_length: 32,
        scale: 0.05,
        offset: 0.0,
        unit: "h",
        data_type: SpnDataType::Uint32,
    },
    SpnDef {
        spn: 249,
        name: "engine_total_revolutions",
        pgn: 65253,
        start_byte: 4,
        start_bit: 0,
        bit_length: 32,
        scale: 1000.0,
        offset: 0.0,
        unit: "r",
        data_type: SpnDataType::Uint32,
    },
    // ========================================================================
    // SHUTDN - Engine Shutdown (PGN 65252 / 0xFEE4)
    // Broadcast rate: On shutdown condition
    // ========================================================================
    SpnDef {
        spn: 1081,
        name: "engine_wait_to_start_lamp",
        pgn: 65252,
        start_byte: 0,
        start_bit: 0,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 2432,
        name: "engine_protection_system_timer_override",
        pgn: 65252,
        start_byte: 0,
        start_bit: 2,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 3384,
        name: "engine_protection_system_timer_state",
        pgn: 65252,
        start_byte: 0,
        start_bit: 4,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 3385,
        name: "engine_protection_system_configuration",
        pgn: 65252,
        start_byte: 0,
        start_bit: 6,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    // ========================================================================
    // FC - Fuel Consumption (PGN 65257 / 0xFEE9)
    // Broadcast rate: 1000ms
    // ========================================================================
    SpnDef {
        spn: 182,
        name: "engine_trip_fuel",
        pgn: 65257,
        start_byte: 0,
        start_bit: 0,
        bit_length: 32,
        scale: 0.5,
        offset: 0.0,
        unit: "L",
        data_type: SpnDataType::Uint32,
    },
    SpnDef {
        spn: 250,
        name: "engine_total_fuel_used",
        pgn: 65257,
        start_byte: 4,
        start_bit: 0,
        bit_length: 32,
        scale: 0.5,
        offset: 0.0,
        unit: "L",
        data_type: SpnDataType::Uint32,
    },
    // ========================================================================
    // VH - Vehicle Hours (PGN 65217 / 0xFEC1)
    // Broadcast rate: 1000ms
    // ========================================================================
    SpnDef {
        spn: 246,
        name: "engine_total_idle_hours",
        pgn: 65217,
        start_byte: 0,
        start_bit: 0,
        bit_length: 32,
        scale: 0.05,
        offset: 0.0,
        unit: "h",
        data_type: SpnDataType::Uint32,
    },
    SpnDef {
        spn: 248,
        name: "engine_total_pto_hours",
        pgn: 65217,
        start_byte: 4,
        start_bit: 0,
        bit_length: 32,
        scale: 0.05,
        offset: 0.0,
        unit: "h",
        data_type: SpnDataType::Uint32,
    },
    // ========================================================================
    // TSS - Torque/Speed Control (PGN 64966 / 0xFDC6)
    // Broadcast rate: 100ms
    // ========================================================================
    SpnDef {
        spn: 898,
        name: "engine_override_control_mode",
        pgn: 64966,
        start_byte: 0,
        start_bit: 0,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 897,
        name: "engine_requested_speed_control_conditions",
        pgn: 64966,
        start_byte: 0,
        start_bit: 2,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 518,
        name: "engine_requested_speed_speed_limit",
        pgn: 64966,
        start_byte: 1,
        start_bit: 0,
        bit_length: 16,
        scale: 0.125,
        offset: 0.0,
        unit: "RPM",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 3349,
        name: "tsc1_transmission_rate",
        pgn: 64966,
        start_byte: 4,
        start_bit: 0,
        bit_length: 3,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    // ========================================================================
    // EFL/P2 - Engine Fluid Level/Pressure 2 (PGN 65243 / 0xFEDB)
    // Broadcast rate: 1000ms
    // ========================================================================
    SpnDef {
        spn: 3563,
        name: "engine_fuel_filter_differential_pressure",
        pgn: 65243,
        start_byte: 0,
        start_bit: 0,
        bit_length: 8,
        scale: 2.0,
        offset: 0.0,
        unit: "kPa",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 3564,
        name: "engine_oil_filter_differential_pressure",
        pgn: 65243,
        start_byte: 1,
        start_bit: 0,
        bit_length: 8,
        scale: 0.5,
        offset: 0.0,
        unit: "kPa",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 5578,
        name: "engine_turbocharger_compressor_inlet_pressure",
        pgn: 65243,
        start_byte: 2,
        start_bit: 0,
        bit_length: 16,
        scale: 0.0078125,
        offset: 0.0,
        unit: "kPa",
        data_type: SpnDataType::Uint16,
    },
    // ========================================================================
    // ET2 - Engine Temperature 2 (PGN 65188 / 0xFEA4)
    // Broadcast rate: 1000ms
    // ========================================================================
    SpnDef {
        spn: 441,
        name: "auxiliary_temperature_1",
        pgn: 65188,
        start_byte: 0,
        start_bit: 0,
        bit_length: 16,
        scale: 0.03125,
        offset: -273.0,
        unit: "°C",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 442,
        name: "auxiliary_temperature_2",
        pgn: 65188,
        start_byte: 2,
        start_bit: 0,
        bit_length: 16,
        scale: 0.03125,
        offset: -273.0,
        unit: "°C",
        data_type: SpnDataType::Uint16,
    },
    // ========================================================================
    // ET3 - Engine Temperature 3 (PGN 65189 / 0xFEA5)
    // Broadcast rate: 1000ms
    // ========================================================================
    SpnDef {
        spn: 1638,
        name: "engine_oil_temperature_2",
        pgn: 65189,
        start_byte: 0,
        start_bit: 0,
        bit_length: 16,
        scale: 0.03125,
        offset: -273.0,
        unit: "°C",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 1639,
        name: "engine_exhaust_gas_recirculation_1_temperature",
        pgn: 65189,
        start_byte: 2,
        start_bit: 0,
        bit_length: 16,
        scale: 0.03125,
        offset: -273.0,
        unit: "°C",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 3237,
        name: "engine_charge_air_cooler_2_outlet_temperature",
        pgn: 65189,
        start_byte: 4,
        start_bit: 0,
        bit_length: 16,
        scale: 0.03125,
        offset: -273.0,
        unit: "°C",
        data_type: SpnDataType::Uint16,
    },
    // ========================================================================
    // DD - Distance (PGN 65248 / 0xFEE0)
    // Broadcast rate: 1000ms
    // ========================================================================
    SpnDef {
        spn: 244,
        name: "trip_distance",
        pgn: 65248,
        start_byte: 0,
        start_bit: 0,
        bit_length: 32,
        scale: 0.125,
        offset: 0.0,
        unit: "km",
        data_type: SpnDataType::Uint32,
    },
    SpnDef {
        spn: 245,
        name: "total_vehicle_distance",
        pgn: 65248,
        start_byte: 4,
        start_bit: 0,
        bit_length: 32,
        scale: 0.125,
        offset: 0.0,
        unit: "km",
        data_type: SpnDataType::Uint32,
    },
    // ========================================================================
    // CCVS - Cruise Control/Vehicle Speed (PGN 65265 / 0xFEF1)
    // Broadcast rate: 100ms
    // ========================================================================
    SpnDef {
        spn: 69,
        name: "two_speed_axle_switch",
        pgn: 65265,
        start_byte: 0,
        start_bit: 0,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 70,
        name: "parking_brake_switch",
        pgn: 65265,
        start_byte: 0,
        start_bit: 2,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 84,
        name: "wheel_based_vehicle_speed",
        pgn: 65265,
        start_byte: 1,
        start_bit: 0,
        bit_length: 16,
        scale: 0.00390625,
        offset: 0.0,
        unit: "km/h",
        data_type: SpnDataType::Uint16,
    },
    SpnDef {
        spn: 595,
        name: "cruise_control_active",
        pgn: 65265,
        start_byte: 3,
        start_bit: 0,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 596,
        name: "cruise_control_enable_switch",
        pgn: 65265,
        start_byte: 3,
        start_bit: 2,
        bit_length: 2,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 86,
        name: "cruise_control_set_speed",
        pgn: 65265,
        start_byte: 5,
        start_bit: 0,
        bit_length: 8,
        scale: 1.0,
        offset: 0.0,
        unit: "km/h",
        data_type: SpnDataType::Uint8,
    },
    SpnDef {
        spn: 976,
        name: "pto_state",
        pgn: 65265,
        start_byte: 6,
        start_bit: 0,
        bit_length: 5,
        scale: 1.0,
        offset: 0.0,
        unit: "",
        data_type: SpnDataType::Uint8,
    },
];

// ============================================================================
// Database lookup functions
// ============================================================================

/// Build PGN to SPNs mapping
pub fn build_pgn_database() -> HashMap<u32, Vec<&'static SpnDef>> {
    let mut map: HashMap<u32, Vec<&'static SpnDef>> = HashMap::new();
    for spn in SPN_DEFINITIONS {
        map.entry(spn.pgn).or_default().push(spn);
    }
    map
}

/// Build SPN to definition mapping
pub fn build_spn_database() -> HashMap<u32, &'static SpnDef> {
    let mut map = HashMap::new();
    for spn in SPN_DEFINITIONS {
        map.insert(spn.spn, spn);
    }
    map
}

/// Get all SPNs for a given PGN
pub fn get_spns_for_pgn(pgn: u32) -> Option<&'static [&'static SpnDef]> {
    // Use thread-local cache for performance
    use std::cell::RefCell;
    thread_local! {
        static PGN_CACHE: RefCell<Option<HashMap<u32, Vec<&'static SpnDef>>>> = const { RefCell::new(None) };
    }

    PGN_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.is_none() {
            *cache = Some(build_pgn_database());
        }
        cache.as_ref().and_then(|c| c.get(&pgn)).map(|v| v.as_slice())
    })
}

/// Get a specific SPN definition
pub fn get_spn_def(spn: u32) -> Option<&'static SpnDef> {
    SPN_DEFINITIONS.iter().find(|s| s.spn == spn)
}

/// Get statistics about the database
pub fn database_stats() -> (usize, usize) {
    let pgn_count = build_pgn_database().len();
    let spn_count = SPN_DEFINITIONS.len();
    (pgn_count, spn_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_stats() {
        let (pgn_count, spn_count) = database_stats();
        assert!(pgn_count > 10, "Should have at least 10 PGNs");
        assert!(spn_count > 60, "Should have at least 60 SPNs");
    }

    #[test]
    fn test_get_spns_for_pgn() {
        // EEC1 should have multiple SPNs
        let spns = get_spns_for_pgn(61444);
        assert!(spns.is_some());
        let spns = spns.unwrap();
        assert!(spns.len() >= 5, "EEC1 should have at least 5 SPNs");

        // Check engine speed is present
        assert!(spns.iter().any(|s| s.spn == 190), "Should have SPN 190 (engine_speed)");
    }

    #[test]
    fn test_get_spn_def() {
        // Engine speed
        let spn = get_spn_def(190);
        assert!(spn.is_some());
        let spn = spn.unwrap();
        assert_eq!(spn.name, "engine_speed");
        assert_eq!(spn.pgn, 61444);
        assert_eq!(spn.scale, 0.125);

        // Coolant temperature
        let spn = get_spn_def(110);
        assert!(spn.is_some());
        let spn = spn.unwrap();
        assert_eq!(spn.name, "engine_coolant_temperature");
        assert_eq!(spn.offset, -40.0);
    }

    #[test]
    fn test_spn_uniqueness() {
        let mut seen = std::collections::HashSet::new();
        for spn in SPN_DEFINITIONS {
            assert!(
                seen.insert(spn.spn),
                "Duplicate SPN found: {} ({})",
                spn.spn,
                spn.name
            );
        }
    }

    #[test]
    fn test_all_pgns_present() {
        let pgn_map = build_pgn_database();

        // Common engine PGNs that should be present
        let expected_pgns = [
            61444, 61443, 65247, // EEC1, EEC2, EEC3
            65262, 65263,       // ET1, EFL/P1
            65270, 65271,       // IC1, VEP1
            65266, 65253,       // LFE, HOURS
        ];

        for pgn in expected_pgns {
            assert!(
                pgn_map.contains_key(&pgn),
                "Missing PGN: {}",
                pgn
            );
        }
    }
}
