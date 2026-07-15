use std::sync::Arc;

use openscope_core::{
    source::CollectorCapability, Capabilities, Collector, MetricBus, ProbeResult, SourceId,
};
use tokio::task::JoinHandle;

/// Scheduler de collecte locale : chaque collecteur disponible tourne dans
/// sa propre tâche à son propre intervalle — un collecteur lent (SMART,
/// Docker) ne bloque jamais les autres.
pub struct LocalScheduler {
    capabilities: Capabilities,
    handles: Vec<JoinHandle<()>>,
}

impl LocalScheduler {
    /// Probe chaque collecteur puis démarre une tâche par collecteur
    /// disponible. Les indisponibles sont consignés dans les capacités
    /// avec leur raison (dégradation gracieuse, jamais d'erreur).
    pub async fn start(
        source: SourceId,
        bus: MetricBus,
        collectors: Vec<Box<dyn Collector>>,
    ) -> Self {
        let mut capabilities = Capabilities::default();
        let mut handles = Vec::new();

        for mut collector in collectors {
            let id = collector.id();
            match collector.probe().await {
                ProbeResult::Available(cap) => {
                    tracing::info!(collector = id, "collecteur disponible");
                    capabilities.collectors.insert(id.to_owned(), cap);
                    handles.push(spawn_collect_loop(collector, source.clone(), bus.clone()));
                }
                ProbeResult::Unavailable(reason) => {
                    tracing::info!(collector = id, %reason, "collecteur indisponible");
                    capabilities.collectors.insert(
                        id.to_owned(),
                        CollectorCapability {
                            available: false,
                            reason: Some(reason),
                            details: Default::default(),
                        },
                    );
                }
            }
        }

        Self {
            capabilities,
            handles,
        }
    }

    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    pub fn stop(&mut self) {
        for handle in self.handles.drain(..) {
            handle.abort();
        }
    }
}

impl Drop for LocalScheduler {
    fn drop(&mut self) {
        self.stop();
    }
}

fn spawn_collect_loop(
    mut collector: Box<dyn Collector>,
    source: SourceId,
    bus: MetricBus,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(collector.default_interval());
        // Si un tick est raté (machine en veille, collecte lente), on saute
        // plutôt que de rattraper en rafale.
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;
            match collector.collect().await {
                Ok(samples) if samples.is_empty() => {}
                Ok(mut samples) => {
                    for sample in &mut samples {
                        sample.source = source.clone();
                    }
                    bus.publish(Arc::new(samples));
                }
                Err(error) => {
                    tracing::warn!(collector = collector.id(), %error, "échec de collecte");
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockCollector;

    #[tokio::test]
    async fn scheduler_publishes_mock_batches() {
        let bus = MetricBus::default();
        let mut rx = bus.subscribe();
        let source = SourceId::local();

        let scheduler = LocalScheduler::start(
            source.clone(),
            bus.clone(),
            vec![Box::new(MockCollector::new())],
        )
        .await;

        assert!(scheduler.capabilities().collectors["mock"].available);

        let batch = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
            .await
            .expect("aucun batch reçu en 2 s")
            .unwrap();
        assert_eq!(batch[0].source, source);
        assert_eq!(batch[0].metric.as_str(), "mock.sine");
    }
}
