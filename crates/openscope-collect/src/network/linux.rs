use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use openscope_core::{source::CollectorCapability, Collector, ProbeResult, Sample, SourceId};

use super::{parse_net_dev, IfaceStats};

const PROC_NET_DEV: &str = "/proc/net/dev";

pub struct NetworkCollector {
    prev: Option<(Vec<IfaceStats>, Instant)>,
}

impl NetworkCollector {
    pub fn new() -> Self {
        Self { prev: None }
    }
}

impl Default for NetworkCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Collector for NetworkCollector {
    fn id(&self) -> &'static str {
        "network"
    }

    async fn probe(&mut self) -> ProbeResult {
        match std::fs::read_to_string(PROC_NET_DEV) {
            Ok(content) => {
                let ifaces: Vec<String> = parse_net_dev(&content)
                    .into_iter()
                    .map(|i| i.name)
                    .filter(|n| n != "lo")
                    .collect();
                let mut details = BTreeMap::new();
                details.insert("interfaces".to_owned(), ifaces.join(","));
                ProbeResult::Available(CollectorCapability {
                    available: true,
                    reason: None,
                    details,
                })
            }
            Err(e) => ProbeResult::Unavailable(format!("{PROC_NET_DEV} illisible : {e}")),
        }
    }

    async fn collect(&mut self) -> anyhow::Result<Vec<Sample>> {
        let source = SourceId::local(); // écrasé par le scheduler
        let mut samples = Vec::new();

        let content = std::fs::read_to_string(PROC_NET_DEV)?;
        let current = parse_net_dev(&content);
        let now = Instant::now();
        let prev = self.prev.replace((current.clone(), now));

        // Premier tick : pas de delta.
        let Some((prev, prev_at)) = prev else {
            return Ok(samples);
        };
        let dt = now.duration_since(prev_at).as_secs_f64();
        if dt <= 0.0 {
            return Ok(samples);
        }

        let (mut sum_rx, mut sum_tx) = (0.0, 0.0);
        for cur in &current {
            let Some(p) = prev.iter().find(|p| p.name == cur.name) else {
                continue;
            };
            // Compteur qui recule (interface recréée) → interface sautée.
            let (Some(drx), Some(dtx)) = (
                cur.rx_bytes.checked_sub(p.rx_bytes),
                cur.tx_bytes.checked_sub(p.tx_bytes),
            ) else {
                continue;
            };
            let rx_bps = drx as f64 / dt;
            let tx_bps = dtx as f64 / dt;
            if cur.name != "lo" {
                sum_rx += rx_bps;
                sum_tx += tx_bps;
            }
            samples.push(
                Sample::gauge(&source, "net.rx_bps", rx_bps).with_label("iface", cur.name.clone()),
            );
            samples.push(
                Sample::gauge(&source, "net.tx_bps", tx_bps).with_label("iface", cur.name.clone()),
            );
        }
        samples.push(Sample::gauge(&source, "net.rx_bps", sum_rx));
        samples.push(Sample::gauge(&source, "net.tx_bps", sum_tx));

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
    async fn second_tick_emits_global_and_per_iface_rates() {
        let mut collector = NetworkCollector::new();
        assert!(collector.probe().await.is_available());
        assert!(collector.collect().await.unwrap().is_empty());

        tokio::time::sleep(Duration::from_millis(80)).await;
        let samples = collector.collect().await.unwrap();
        let global_rx = samples
            .iter()
            .find(|s| s.metric.as_str() == "net.rx_bps" && s.labels.is_empty())
            .expect("débit global attendu");
        assert!(global_rx.value.as_f64().unwrap() >= 0.0);
        // lo est mesurée par interface mais exclue de la somme globale.
        assert!(samples.iter().all(|s| s.value.as_f64().unwrap() >= 0.0));
    }
}
