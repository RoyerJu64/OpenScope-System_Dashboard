use std::time::Duration;

use async_trait::async_trait;
use openscope_core::{Collector, MetricId, ProbeResult, Sample, SourceId, Value};

/// Collecteur factice de la Phase 0 : une sinusoïde et un compteur.
/// Il valide la chaîne complète collecteur → bus → IPC → graphe, et sert
/// de modèle d'implémentation pour les collecteurs réels.
pub struct MockCollector {
    tick: u64,
}

impl MockCollector {
    pub fn new() -> Self {
        Self { tick: 0 }
    }
}

impl Default for MockCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Collector for MockCollector {
    fn id(&self) -> &'static str {
        "mock"
    }

    async fn probe(&mut self) -> ProbeResult {
        ProbeResult::available()
    }

    async fn collect(&mut self) -> anyhow::Result<Vec<Sample>> {
        self.tick += 1;
        let t = self.tick as f64;
        // Sinusoïde 0–100 (période ~30 s à 2 Hz) + un peu d'harmonique
        // pour que le graphe ait du relief.
        let sine = 50.0 + 40.0 * (t / 10.0).sin() + 8.0 * (t / 1.7).sin();

        // Le scheduler écrase `source` ; le placeholder évite de coupler
        // les collecteurs à l'identité de la machine.
        let source = SourceId::local();
        Ok(vec![
            Sample::gauge(&source, "mock.sine", sine),
            Sample {
                value: Value::Counter(self.tick * 1500),
                ..Sample::gauge(&source, MetricId::from("mock.counter"), 0.0)
            },
        ])
    }

    fn default_interval(&self) -> Duration {
        Duration::from_millis(500)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_produces_bounded_sine() {
        let mut c = MockCollector::new();
        assert!(c.probe().await.is_available());
        for _ in 0..100 {
            let samples = c.collect().await.unwrap();
            let v = samples[0].value.as_f64().unwrap();
            assert!((0.0..=100.0).contains(&v), "sine hors bornes : {v}");
        }
    }
}
