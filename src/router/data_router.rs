//! Data router for point mapping and protocol conversion.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::core::data::{DataBatch, DataPoint, Value};
use crate::core::error::Result;
use crate::core::traits::{AdjustmentCommand, ControlCommand, DataEvent, DataEventReceiver};
use crate::store::DataStore;

use super::mapping::{PointMapping, RoutingTable, TriggerCondition};

/// Target channel writer trait.
///
/// This allows the router to write to different channel types uniformly.
#[async_trait]
pub trait TargetWriter: Send + Sync {
    /// Write a data batch to the target.
    async fn write_batch(&self, batch: &DataBatch) -> Result<()>;

    /// Write control commands to the target.
    async fn write_control(&self, commands: &[ControlCommand]) -> Result<()>;

    /// Write adjustment commands to the target.
    async fn write_adjustment(&self, adjustments: &[AdjustmentCommand]) -> Result<()>;
}

/// Data router configuration.
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Routing table.
    pub routing_table: RoutingTable,

    /// Buffer size for event processing.
    pub buffer_size: usize,

    /// Whether to continue on individual mapping errors.
    pub continue_on_error: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            routing_table: RoutingTable::new(),
            buffer_size: 1024,
            continue_on_error: true,
        }
    }
}

impl RouterConfig {
    /// Create a new router configuration.
    pub fn new(routing_table: RoutingTable) -> Self {
        Self {
            routing_table,
            ..Default::default()
        }
    }
}

/// State for trigger condition evaluation.
#[derive(Default)]
struct RouterState {
    /// Last values for change detection: (channel_id, point_id) -> Value
    last_values: HashMap<(u32, String), Value>,
    /// Last forward time for interval triggers: (channel_id, point_id) -> Instant
    last_forward: HashMap<(u32, String), Instant>,
}

/// Data router for forwarding data between channels.
pub struct DataRouter<S: DataStore> {
    config: RouterConfig,
    store: Arc<S>,
    targets: Arc<RwLock<HashMap<u32, Arc<dyn TargetWriter>>>>,
    state: Arc<RwLock<RouterState>>,
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl<S: DataStore + 'static> DataRouter<S> {
    /// Create a new data router.
    pub fn new(config: RouterConfig, store: Arc<S>) -> Self {
        Self {
            config,
            store,
            targets: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(RouterState::default())),
            task_handle: None,
        }
    }

    /// Register a target channel writer.
    pub async fn register_target(&self, channel_id: u32, writer: Arc<dyn TargetWriter>) {
        let mut targets = self.targets.write().await;
        targets.insert(channel_id, writer);
    }

    /// Unregister a target channel.
    pub async fn unregister_target(&self, channel_id: u32) {
        let mut targets = self.targets.write().await;
        targets.remove(&channel_id);
    }

    /// Update the routing table.
    pub fn set_routing_table(&mut self, table: RoutingTable) {
        self.config.routing_table = table;
    }

    /// Get the routing table.
    pub fn routing_table(&self) -> &RoutingTable {
        &self.config.routing_table
    }

    /// Start the routing task.
    pub async fn start(&mut self) -> Result<()> {
        let rx = self.store.subscribe();
        let routing_table = self.config.routing_table.clone();
        let continue_on_error = self.config.continue_on_error;
        let targets = self.targets.clone();
        let state = self.state.clone();

        let handle = tokio::spawn(async move {
            Self::routing_loop(rx, routing_table, targets, state, continue_on_error).await;
        });

        self.task_handle = Some(handle);
        Ok(())
    }

    /// Stop the routing task.
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
        Ok(())
    }

    /// Check if the router is running.
    pub fn is_running(&self) -> bool {
        self.task_handle.as_ref().is_some_and(|h| !h.is_finished())
    }

    /// Main routing loop.
    async fn routing_loop(
        mut rx: DataEventReceiver,
        routing_table: RoutingTable,
        targets: Arc<RwLock<HashMap<u32, Arc<dyn TargetWriter>>>>,
        state: Arc<RwLock<RouterState>>,
        continue_on_error: bool,
    ) {
        while let Some(event) = rx.recv().await {
            if let DataEvent::DataUpdate(batch) = event {
                // Note: DataEvent doesn't include channel_id, so we need to match by point_id
                // In a real implementation, you might want to extend DataEvent
                Self::process_batch(&batch, &routing_table, &targets, &state, continue_on_error)
                    .await;
            }
        }
    }

    /// Process a data batch and forward according to mappings.
    async fn process_batch(
        batch: &DataBatch,
        routing_table: &RoutingTable,
        targets: &Arc<RwLock<HashMap<u32, Arc<dyn TargetWriter>>>>,
        state: &Arc<RwLock<RouterState>>,
        _continue_on_error: bool,
    ) {
        // Group points by target channel
        let mut target_batches: HashMap<u32, DataBatch> = HashMap::new();

        for point in batch.iter() {
            // Find all mappings that match this point
            // Note: Without channel_id in DataEvent, we check all channels
            for mapping in routing_table.enabled_mappings() {
                if mapping.source_point != point.id {
                    continue;
                }

                // Check trigger condition
                if !Self::should_forward(mapping, point, state).await {
                    continue;
                }

                // Apply transformation
                let transformed = Self::transform_point(point, mapping);

                // Add to target batch
                target_batches
                    .entry(mapping.target_channel)
                    .or_default()
                    .add(transformed);
            }
        }

        // Write to targets
        let targets_guard = targets.read().await;
        for (channel_id, batch) in target_batches {
            if let Some(writer) = targets_guard.get(&channel_id) {
                let _ = writer.write_batch(&batch).await;
            }
        }
    }

    /// Check if a point should be forwarded based on trigger condition.
    async fn should_forward(
        mapping: &PointMapping,
        point: &DataPoint,
        state: &Arc<RwLock<RouterState>>,
    ) -> bool {
        let trigger = mapping
            .trigger
            .as_ref()
            .unwrap_or(&TriggerCondition::Always);

        match trigger {
            TriggerCondition::Always => true,

            TriggerCondition::OnChange => {
                let key = (mapping.source_channel, mapping.source_point.clone());
                let mut state = state.write().await;
                let changed = state.last_values.get(&key) != Some(&point.value);
                if changed {
                    state.last_values.insert(key, point.value.clone());
                }
                changed
            }

            TriggerCondition::Threshold { min, max } => {
                if let Some(v) = point.value.as_f64() {
                    let above_min = min.map_or(true, |m| v >= m);
                    let below_max = max.map_or(true, |m| v <= m);
                    above_min && below_max
                } else {
                    true
                }
            }

            TriggerCondition::Interval { min_interval_ms } => {
                let key = (mapping.source_channel, mapping.source_point.clone());
                let mut state = state.write().await;
                let now = Instant::now();

                if let Some(last) = state.last_forward.get(&key) {
                    if now.duration_since(*last).as_millis() < *min_interval_ms as u128 {
                        return false;
                    }
                }
                state.last_forward.insert(key, now);
                true
            }

            TriggerCondition::Deadband { deadband } => {
                let key = (mapping.source_channel, mapping.source_point.clone());
                let mut state = state.write().await;

                if let Some(v) = point.value.as_f64() {
                    if let Some(last_value) = state.last_values.get(&key).and_then(|v| v.as_f64()) {
                        if (v - last_value).abs() < *deadband {
                            return false;
                        }
                    }
                    state.last_values.insert(key, point.value.clone());
                }
                true
            }
        }
    }

    /// Transform a data point according to mapping configuration.
    fn transform_point(point: &DataPoint, mapping: &PointMapping) -> DataPoint {
        let new_value = match &point.value {
            Value::Float(v) => Value::Float(mapping.transform.apply(*v)),
            Value::Integer(v) => Value::Float(mapping.transform.apply(*v as f64)),
            Value::Bool(v) => Value::Bool(mapping.transform.apply_bool(*v)),
            other => other.clone(),
        };

        DataPoint {
            id: mapping.effective_target_point().to_string(),
            data_type: point.data_type,
            value: new_value,
            quality: point.quality,
            timestamp: point.timestamp,
            source_timestamp: point.source_timestamp,
        }
    }

    /// Manually route a single batch (for testing or direct forwarding).
    pub async fn route_batch(&self, source_channel: u32, batch: &DataBatch) -> Result<()> {
        let mut target_batches: HashMap<u32, DataBatch> = HashMap::new();

        for point in batch.iter() {
            let mappings = self
                .config
                .routing_table
                .find_by_source(source_channel, &point.id);

            for mapping in mappings {
                if !Self::should_forward(mapping, point, &self.state).await {
                    continue;
                }

                let transformed = Self::transform_point(point, mapping);

                target_batches
                    .entry(mapping.target_channel)
                    .or_default()
                    .add(transformed);
            }
        }

        let targets_guard = self.targets.read().await;
        for (channel_id, batch) in target_batches {
            if let Some(writer) = targets_guard.get(&channel_id) {
                writer.write_batch(&batch).await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryStore;

    struct MockWriter {
        received: Arc<RwLock<Vec<DataBatch>>>,
    }

    impl MockWriter {
        fn new() -> Self {
            Self {
                received: Arc::new(RwLock::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl TargetWriter for MockWriter {
        async fn write_batch(&self, batch: &DataBatch) -> Result<()> {
            self.received.write().await.push(batch.clone());
            Ok(())
        }

        async fn write_control(&self, _commands: &[ControlCommand]) -> Result<()> {
            Ok(())
        }

        async fn write_adjustment(&self, _adjustments: &[AdjustmentCommand]) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_router_route_batch() {
        let store = Arc::new(MemoryStore::new());
        let mut table = RoutingTable::new();
        table.add(PointMapping::direct(1, "temp", 2, "temp_out"));

        let config = RouterConfig::new(table);
        let router = DataRouter::new(config, store);

        let writer = Arc::new(MockWriter::new());
        router.register_target(2, writer.clone()).await;

        let mut batch = DataBatch::new();
        batch.add(DataPoint::telemetry("temp", 25.5));

        router.route_batch(1, &batch).await.unwrap();

        let received = writer.received.read().await;
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].len(), 1);

        let point = received[0].iter().next().unwrap();
        assert_eq!(point.id, "temp_out");
    }
}
