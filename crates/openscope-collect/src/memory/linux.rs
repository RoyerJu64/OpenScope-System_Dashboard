use std::collections::BTreeMap;
use std::time::Duration;

use async_trait::async_trait;
use openscope_core::{source::CollectorCapability, Collector, ProbeResult, Sample, SourceId};

use super::parse_meminfo;

const PROC_MEMINFO: &str = "/proc/meminfo";

pub struct MemoryCollector;

impl MemoryCollector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MemoryCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Collector for MemoryCollector {
    fn id(&self) -> &'static str {
        "memory"
    }

    async fn probe(&mut self) -> ProbeResult {
        match std::fs::read_to_string(PROC_MEMINFO) {
            Ok(content) => {
                let info = parse_meminfo(&content);
                let mut details = BTreeMap::new();
                details.insert("total_bytes".to_owned(), info.total.to_string());
                details.insert("swap".to_owned(), (info.swap_total > 0).to_string());
                ProbeResult::Available(CollectorCapability {
                    available: true,
                    reason: None,
                    details,
                })
            }
            Err(e) => ProbeResult::Unavailable(format!("{PROC_MEMINFO} illisible : {e}")),
        }
    }

    async fn collect(&mut self) -> anyhow::Result<Vec<Sample>> {
        let info = parse_meminfo(&std::fs::read_to_string(PROC_MEMINFO)?);
        let source = SourceId::local(); // écrasé par le scheduler

        let gauge = |metric: &'static str, bytes: u64| Sample::gauge(&source, metric, bytes as f64);
        let mut samples = vec![
            gauge("mem.total_bytes", info.total),
            gauge("mem.used_bytes", info.used()),
            gauge("mem.available_bytes", info.available),
            gauge("mem.free_bytes", info.free),
            gauge("mem.cached_bytes", info.cache()),
        ];
        if info.total > 0 {
            samples.push(Sample::gauge(
                &source,
                "mem.used_pct",
                info.used() as f64 / info.total as f64 * 100.0,
            ));
        }
        if info.swap_total > 0 {
            samples.push(gauge("swap.total_bytes", info.swap_total));
            samples.push(gauge("swap.used_bytes", info.swap_used()));
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
    async fn collects_coherent_values_from_real_meminfo() {
        let mut collector = MemoryCollector::new();
        assert!(collector.probe().await.is_available());

        let samples = collector.collect().await.unwrap();
        let get = |metric: &str| {
            samples
                .iter()
                .find(|s| s.metric.as_str() == metric)
                .and_then(|s| s.value.as_f64())
        };
        let total = get("mem.total_bytes").expect("total attendu");
        let used = get("mem.used_bytes").expect("used attendu");
        let pct = get("mem.used_pct").expect("pct attendu");
        assert!(total > 0.0 && used <= total);
        assert!((0.0..=100.0).contains(&pct));
    }
}
