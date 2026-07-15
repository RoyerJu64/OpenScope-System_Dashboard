use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use openscope_core::{
    source::CollectorCapability, Capabilities, Collector, MetricBus, ProbeResult, SourceId,
};
use tokio::task::JoinHandle;

/// Bornes des intervalles configurables (issue #11) : en dessous de
/// 100 ms la collecte devient elle-même une charge ; au-delà d'une heure
/// c'est un collecteur désactivé qui s'ignore.
pub const MIN_INTERVAL: Duration = Duration::from_millis(100);
pub const MAX_INTERVAL: Duration = Duration::from_secs(3600);

/// Granularité de sommeil : chaque boucle relit son intervalle au moins
/// toutes les 250 ms, donc un changement s'applique presque immédiatement
/// même si l'ancien intervalle était long.
const SLEEP_CHUNK_MS: u64 = 250;

/// Scheduler de collecte locale : chaque collecteur disponible tourne dans
/// sa propre tâche à son propre intervalle — un collecteur lent (SMART,
/// Docker) ne bloque jamais les autres. Les intervalles sont modifiables
/// à chaud via [`LocalScheduler::set_interval`].
pub struct LocalScheduler {
    capabilities: Capabilities,
    intervals: BTreeMap<String, Arc<AtomicU64>>,
    handles: Vec<JoinHandle<()>>,
}

impl LocalScheduler {
    /// Démarre avec les intervalles par défaut de chaque collecteur.
    pub async fn start(
        source: SourceId,
        bus: MetricBus,
        collectors: Vec<Box<dyn Collector>>,
    ) -> Self {
        Self::start_with_intervals(source, bus, collectors, &BTreeMap::new()).await
    }

    /// Probe chaque collecteur puis démarre une tâche par collecteur
    /// disponible. `overrides` (venant de la config utilisateur) prime sur
    /// `default_interval()`. Les indisponibles sont consignés dans les
    /// capacités avec leur raison (dégradation gracieuse, jamais d'erreur).
    pub async fn start_with_intervals(
        source: SourceId,
        bus: MetricBus,
        collectors: Vec<Box<dyn Collector>>,
        overrides: &BTreeMap<String, Duration>,
    ) -> Self {
        let mut capabilities = Capabilities::default();
        let mut intervals = BTreeMap::new();
        let mut handles = Vec::new();

        for mut collector in collectors {
            let id = collector.id();
            match collector.probe().await {
                ProbeResult::Available(cap) => {
                    tracing::info!(collector = id, "collecteur disponible");
                    capabilities.collectors.insert(id.to_owned(), cap);

                    let interval = overrides
                        .get(id)
                        .copied()
                        .unwrap_or_else(|| collector.default_interval())
                        .clamp(MIN_INTERVAL, MAX_INTERVAL);
                    let shared = Arc::new(AtomicU64::new(interval.as_millis() as u64));
                    intervals.insert(id.to_owned(), Arc::clone(&shared));

                    handles.push(spawn_collect_loop(
                        collector,
                        source.clone(),
                        bus.clone(),
                        shared,
                    ));
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
            intervals,
            handles,
        }
    }

    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    /// Intervalles courants des collecteurs actifs.
    pub fn intervals(&self) -> BTreeMap<String, Duration> {
        self.intervals
            .iter()
            .map(|(id, ms)| {
                (
                    id.clone(),
                    Duration::from_millis(ms.load(Ordering::Relaxed)),
                )
            })
            .collect()
    }

    /// Change l'intervalle d'un collecteur actif, borné à
    /// [`MIN_INTERVAL`, `MAX_INTERVAL`]. Pris en compte en ≤ 250 ms.
    /// Retourne `false` si le collecteur est inconnu ou indisponible.
    pub fn set_interval(&self, collector: &str, interval: Duration) -> bool {
        let Some(slot) = self.intervals.get(collector) else {
            return false;
        };
        let clamped = interval.clamp(MIN_INTERVAL, MAX_INTERVAL);
        slot.store(clamped.as_millis() as u64, Ordering::Relaxed);
        true
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
    interval_ms: Arc<AtomicU64>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut slept_ms: u64 = 0;
        loop {
            // Sommeil par tranches : l'intervalle est relu à chaque tranche,
            // un raccourcissement prend donc effet sans attendre la fin de
            // l'ancien (long) intervalle.
            let target = interval_ms.load(Ordering::Relaxed);
            if slept_ms < target {
                let chunk = (target - slept_ms).min(SLEEP_CHUNK_MS);
                tokio::time::sleep(Duration::from_millis(chunk)).await;
                slept_ms += chunk;
                continue;
            }
            slept_ms = 0;

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
        assert!(scheduler.intervals().contains_key("mock"));

        let batch = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("aucun batch reçu en 2 s")
            .unwrap();
        assert_eq!(batch[0].source, source);
        assert_eq!(batch[0].metric.as_str(), "mock.sine");
    }

    #[tokio::test]
    async fn set_interval_takes_effect_and_rejects_unknown() {
        let bus = MetricBus::default();
        let mut rx = bus.subscribe();

        let scheduler = LocalScheduler::start_with_intervals(
            SourceId::local(),
            bus.clone(),
            vec![Box::new(MockCollector::new())],
            // Intervalle long au départ : sans set_interval, aucun batch
            // n'arriverait pendant le test.
            &BTreeMap::from([("mock".to_owned(), Duration::from_secs(3600))]),
        )
        .await;

        assert!(!scheduler.set_interval("inconnu", Duration::from_millis(500)));
        assert!(scheduler.set_interval("mock", Duration::from_millis(100)));

        // Le raccourcissement doit produire un batch bien avant l'heure.
        let batch = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("le nouvel intervalle n'a pas pris effet")
            .unwrap();
        assert_eq!(batch[0].metric.as_str(), "mock.sine");
        assert_eq!(
            scheduler.intervals()["mock"],
            Duration::from_millis(100),
            "l'intervalle courant doit refléter le changement"
        );
    }

    #[tokio::test]
    async fn intervals_are_clamped() {
        let bus = MetricBus::default();
        let scheduler =
            LocalScheduler::start(SourceId::local(), bus, vec![Box::new(MockCollector::new())])
                .await;

        assert!(scheduler.set_interval("mock", Duration::from_millis(1)));
        assert_eq!(scheduler.intervals()["mock"], MIN_INTERVAL);
        assert!(scheduler.set_interval("mock", Duration::from_secs(864_000)));
        assert_eq!(scheduler.intervals()["mock"], MAX_INTERVAL);
    }
}
