use std::collections::BTreeMap;
use std::time::Duration;

use async_trait::async_trait;
use openscope_core::{
    source::CollectorCapability, Collector, ProbeResult, Sample, SourceId, Value,
};

use super::freq::FreqReader;
use super::rapl::RaplReader;
use super::temp::TempReader;
use super::{parse_proc_stat, CpuTimes};

const PROC_STAT: &str = "/proc/stat";

pub struct CpuCollector {
    prev: Option<Vec<CpuTimes>>,
    freq: Option<FreqReader>,
    temp: Option<TempReader>,
    rapl: Option<RaplReader>,
}

impl CpuCollector {
    pub fn new() -> Self {
        Self {
            prev: None,
            freq: None,
            temp: None,
            rapl: None,
        }
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
        let content = match std::fs::read_to_string(PROC_STAT) {
            Ok(content) => content,
            Err(e) => return ProbeResult::Unavailable(format!("{PROC_STAT} illisible : {e}")),
        };
        let cores = parse_proc_stat(&content)
            .iter()
            .filter(|t| t.core.is_some())
            .count();

        // Sources optionnelles : leur absence n'est jamais une erreur,
        // seulement une capacité en moins.
        self.freq = Some(FreqReader::new()).filter(FreqReader::probe);
        self.temp = TempReader::detect();
        self.rapl = RaplReader::detect();

        let mut details = BTreeMap::new();
        details.insert("cores".to_owned(), cores.to_string());
        details.insert("freq".to_owned(), self.freq.is_some().to_string());
        details.insert("rapl".to_owned(), self.rapl.is_some().to_string());
        if let Some(temp) = &self.temp {
            details.insert("temp_sensor".to_owned(), temp.sensor().to_owned());
        }

        ProbeResult::Available(CollectorCapability {
            available: true,
            reason: None,
            details,
        })
    }

    async fn collect(&mut self) -> anyhow::Result<Vec<Sample>> {
        let source = SourceId::local(); // écrasé par le scheduler
        let mut samples = Vec::new();

        // Usage : deltas de /proc/stat (silencieux au premier tick).
        let content = std::fs::read_to_string(PROC_STAT)?;
        let current = parse_proc_stat(&content);
        let prev = self.prev.replace(current.clone());
        if let Some(prev) = prev {
            for times in &current {
                // Appariement par identité de cœur, pas par index : robuste
                // au hotplug (un cœur qui apparaît attendra le tick suivant).
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
        }

        // Fréquences par cœur.
        if let Some(freq) = &self.freq {
            for (core, mhz) in freq.read_all() {
                samples.push(
                    Sample::gauge(&source, "cpu.freq_mhz", mhz)
                        .with_label("core", core.to_string()),
                );
            }
        }

        // Températures : package sans label + une série par sonde.
        if let Some(temp) = &self.temp {
            let readings = temp.read();
            if let Some(package) = TempReader::package_of(&readings) {
                samples.push(Sample::gauge(&source, "cpu.temp_c", package));
            }
            for (label, celsius) in readings {
                samples.push(
                    Sample::gauge(&source, "cpu.temp_c", celsius).with_label("sensor", label),
                );
            }
        }

        // Consommation RAPL (None au premier tick ou sur wrap).
        if let Some(rapl) = &mut self.rapl {
            if let Some(watts) = rapl.read_watts() {
                samples.push(Sample::gauge(&source, "cpu.power_w", watts));
            }
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
    async fn probe_reports_core_count_and_optional_sources() {
        let mut collector = CpuCollector::new();
        match collector.probe().await {
            ProbeResult::Available(cap) => {
                let cores: usize = cap.details["cores"].parse().unwrap();
                assert!(cores >= 1, "au moins un cœur attendu");
                // Présents quel que soit le matériel : la valeur documente
                // la capacité détectée.
                assert!(cap.details.contains_key("freq"));
                assert!(cap.details.contains_key("rapl"));
            }
            ProbeResult::Unavailable(reason) => panic!("probe a échoué : {reason}"),
        }
    }

    #[tokio::test]
    async fn first_tick_emits_only_instant_metrics_then_usage() {
        let mut collector = CpuCollector::new();
        collector.probe().await;

        // Premier tick : pas d'usage (pas de delta), mais freq/temp
        // peuvent déjà être présents.
        let first = collector.collect().await.unwrap();
        assert!(
            first.iter().all(|s| s.metric.as_str() != "cpu.usage"),
            "l'usage exige un delta"
        );

        tokio::time::sleep(Duration::from_millis(120)).await;
        let samples = collector.collect().await.unwrap();
        let global = samples
            .iter()
            .find(|s| s.metric.as_str() == "cpu.usage" && s.labels.is_empty())
            .expect("échantillon global attendu");
        let v = global.value.as_f64().unwrap();
        assert!((0.0..=100.0).contains(&v), "usage hors bornes : {v}");

        // Toutes les métriques émises sont numériques et finies.
        assert!(samples
            .iter()
            .all(|s| s.value.as_f64().is_some_and(f64::is_finite)));
    }
}
