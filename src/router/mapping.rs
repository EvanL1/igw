//! Point mapping configuration for data routing.

use serde::{Deserialize, Serialize};

use crate::core::point::TransformConfig;

/// A single point mapping rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointMapping {
    /// Source channel ID.
    pub source_channel: u32,

    /// Source point ID.
    pub source_point: String,

    /// Target channel ID.
    pub target_channel: u32,

    /// Target point ID (defaults to same as source if None).
    pub target_point: Option<String>,

    /// Data transformation to apply.
    #[serde(default)]
    pub transform: TransformConfig,

    /// Whether this mapping is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Trigger condition (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<TriggerCondition>,
}

fn default_true() -> bool {
    true
}

impl PointMapping {
    /// Create a simple 1:1 mapping.
    pub fn direct(
        source_channel: u32,
        source_point: impl Into<String>,
        target_channel: u32,
        target_point: impl Into<String>,
    ) -> Self {
        Self {
            source_channel,
            source_point: source_point.into(),
            target_channel,
            target_point: Some(target_point.into()),
            transform: TransformConfig::default(),
            enabled: true,
            trigger: None,
        }
    }

    /// Create a mapping with same point ID on both ends.
    pub fn same_id(source_channel: u32, point_id: impl Into<String>, target_channel: u32) -> Self {
        let id = point_id.into();
        Self {
            source_channel,
            source_point: id.clone(),
            target_channel,
            target_point: Some(id),
            transform: TransformConfig::default(),
            enabled: true,
            trigger: None,
        }
    }

    /// Add transformation.
    pub fn with_transform(mut self, transform: TransformConfig) -> Self {
        self.transform = transform;
        self
    }

    /// Add trigger condition.
    pub fn with_trigger(mut self, trigger: TriggerCondition) -> Self {
        self.trigger = Some(trigger);
        self
    }

    /// Set enabled state.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Get effective target point ID.
    pub fn effective_target_point(&self) -> &str {
        self.target_point.as_deref().unwrap_or(&self.source_point)
    }
}

/// Trigger condition for conditional forwarding.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerCondition {
    /// Always forward (default behavior).
    Always,

    /// Forward only when value changes.
    OnChange,

    /// Forward when value is within threshold.
    Threshold {
        /// Minimum value (inclusive).
        min: Option<f64>,
        /// Maximum value (inclusive).
        max: Option<f64>,
    },

    /// Forward at fixed interval (deduplicate).
    Interval {
        /// Minimum interval between forwards in milliseconds.
        min_interval_ms: u64,
    },

    /// Forward when value changes by more than deadband.
    Deadband {
        /// Deadband value.
        deadband: f64,
    },
}

impl Default for TriggerCondition {
    fn default() -> Self {
        Self::Always
    }
}

/// Routing table for a gateway.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoutingTable {
    /// List of point mappings.
    pub mappings: Vec<PointMapping>,
}

impl RoutingTable {
    /// Create empty routing table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a mapping.
    pub fn add(&mut self, mapping: PointMapping) {
        self.mappings.push(mapping);
    }

    /// Add multiple mappings.
    pub fn add_all(&mut self, mappings: impl IntoIterator<Item = PointMapping>) {
        self.mappings.extend(mappings);
    }

    /// Find mappings for a source point.
    pub fn find_by_source(&self, channel_id: u32, point_id: &str) -> Vec<&PointMapping> {
        self.mappings
            .iter()
            .filter(|m| m.enabled && m.source_channel == channel_id && m.source_point == point_id)
            .collect()
    }

    /// Get all mappings targeting a specific channel.
    pub fn targets_for_channel(&self, channel_id: u32) -> Vec<&PointMapping> {
        self.mappings
            .iter()
            .filter(|m| m.enabled && m.target_channel == channel_id)
            .collect()
    }

    /// Get all enabled mappings.
    pub fn enabled_mappings(&self) -> Vec<&PointMapping> {
        self.mappings.iter().filter(|m| m.enabled).collect()
    }

    /// Get number of mappings.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Remove all mappings.
    pub fn clear(&mut self) {
        self.mappings.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_mapping_direct() {
        let mapping = PointMapping::direct(1, "temp", 2, "temp_104");
        assert_eq!(mapping.source_channel, 1);
        assert_eq!(mapping.source_point, "temp");
        assert_eq!(mapping.target_channel, 2);
        assert_eq!(mapping.effective_target_point(), "temp_104");
    }

    #[test]
    fn test_point_mapping_same_id() {
        let mapping = PointMapping::same_id(1, "pressure", 2);
        assert_eq!(mapping.effective_target_point(), "pressure");
    }

    #[test]
    fn test_routing_table() {
        let mut table = RoutingTable::new();
        table.add(PointMapping::direct(1, "a", 2, "a"));
        table.add(PointMapping::direct(1, "b", 2, "b"));
        table.add(PointMapping::direct(1, "a", 3, "a"));

        let mappings = table.find_by_source(1, "a");
        assert_eq!(mappings.len(), 2);

        let targets = table.targets_for_channel(2);
        assert_eq!(targets.len(), 2);
    }
}
