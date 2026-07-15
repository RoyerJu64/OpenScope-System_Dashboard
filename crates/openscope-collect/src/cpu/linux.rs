use std::collections::BTreeMap;
use std::time::Duration;

use async_trait::async_trait;
use openscope_core::{
    source::CollectorCapability, Collector, ProbeResult, Sample, SourceId, Value,
};

use super::{parse_proc_stat, CpuTimes};

const PROC_STAT: &str = "/proc/stat";

pub struct CpuCollector {
    prev: Option<Vec<CpuTimes>>,
}

impl CpuCollector {
    pub fn new() -> Self {
        Self { prev: None }
    }
}

impl Default for CpuCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Collector for CpuCollector {
    fn id(&self) -> &'static str {
        "cpu"
    }

    async fn probe(&mut self) -> ProbeResult {
        match std::fs::read_to_string(PROC_STAT) {
            Ok(content) => {
                let cores = parse_proc_stat(&content)
                    .iter()
                    .filter(|t| t.core.is_some())
                    .count();
                let mut details = BTreeMap::new();
                details.insert("cores".to_owned(), cores.to_string());
                ProbeResult::Available(CollectorCapability {
                    available: true,
                    reason: None,
                    details,
                })
            }
            Err(e) => ProbeResult::Unavailable(format!("{PROC_STAT} illisible : {e}")),
        }
    }

    async fn collect(&mut self) -> anyhow::Result<Vec<Sample>> {
        let content = std::fs::read_to_string(PROC_STAT)?;
        let current = parse_proc_stat(&content);
        let prev = self.prev.replace(current.clone());

        // Premier tick : pas de delta, donc rien à émettre.
        let Some(prev) = prev else {
            return Ok(Vec::new());
        };

        let source = SourceId::local(); // écrasé par le scheduler
        let mut samples = Vec::with_capacity(current.len());
        for times in &current {
            // Appariement par identité de cœur, pas par index : robuste au
            // hotplug (un cœur qui apparaît attendra le tick suivant).
            let Some(prev_times) = prev.iter().find(|p| p.core == times.core) else {
                continue;
            };
            let Some(usage) = times.usage_since(prev_times) else {
                continue;
            };
            let sample = Sample {
                value: Value::Gauge(usage),
                ..Sample::gauge(&source, "cpu.usage", 0.0)
            };
            samples.push(match times.core {
                Some(core) => sample.with_label("core", core.to_string()),
                None => sample,
            });
        }
        Ok(samples)
    }

    fn default_interval(&self) -> Duration {
        Duration::from_secs(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn probe_reports_core_count() {
        let mut collector = CpuCollector::new();
        match collector.probe().await {
            ProbeResult::Available(cap) => {
                let cores: usize = cap.details["cores"].parse().unwrap();
                assert!(cores >= 1, "au moins un cœur attendu");
            }
            ProbeResult::Unavailable(reason) => panic!("probe a échoué : {reason}"),
        }
    }

    #[tokio::test]
    async fn first_tick_is_empty_then_emits_global_and_cores() {
        let mut collector = CpuCollector::new();
        assert!(collector.collect().await.unwrap().is_empty());

        tokio::time::sleep(Duration::from_millis(120)).await;
        let samples = collector.collect().await.unwrap();
        // Selon l'activité machine, un delta de 50 ms peut être vide sur
        // certains cœurs, mais l'agrégat doit être présent.
        let global = samples
            .iter()
            .find(|s| s.labels.is_empty())
            .expect("échantillon global attendu");
        assert_eq!(global.metric.as_str(), "cpu.usage");
        let v = global.value.as_f64().unwrap();
        assert!((0.0..=100.0).contains(&v), "usage hors bornes : {v}");
        assert!(samples
            .iter()
            .all(|s| s.value.as_f64().is_some_and(|v| (0.0..=100.0).contains(&v))));
    }
}
