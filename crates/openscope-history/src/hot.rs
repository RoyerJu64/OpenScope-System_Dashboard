//! Fenêtre chaude : les N dernières minutes de chaque série, en mémoire.
//!
//! Abonnée au bus via `append_batch` ; interrogée par l'UI au montage
//! d'une page pour pré-remplir les graphes au lieu de partir d'un écran
//! vide. La persistance longue durée (SQLite) est un autre consommateur
//! du bus, pas cette structure.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, UNIX_EPOCH};

use openscope_core::{Labels, MetricId, Sample, SourceId};

/// Une série de la fenêtre chaude, points en (ms epoch, valeur).
#[derive(Debug, Clone, PartialEq)]
pub struct HotSeries {
    pub metric: MetricId,
    pub labels: Labels,
    pub points: Vec<(u64, f64)>,
}

type SeriesKey = (SourceId, MetricId, Labels);

pub struct HotWindow {
    retention: Duration,
    series: Mutex<HashMap<SeriesKey, VecDeque<(u64, f64)>>>,
}

impl HotWindow {
    /// `retention` : profondeur de la fenêtre (10 min par défaut dans l'app).
    pub fn new(retention: Duration) -> Self {
        Self {
            retention,
            series: Mutex::new(HashMap::new()),
        }
    }

    /// Ingère un batch du bus. Les valeurs non numériques (Text) sont
    /// ignorées ; les points plus vieux que la rétention sont purgés au
    /// fil de l'eau.
    pub fn append_batch(&self, batch: &[Sample]) {
        let mut series = self.series.lock().unwrap();
        for sample in batch {
            let Some(value) = sample.value.as_f64() else {
                continue;
            };
            let Ok(ts) = sample.ts.duration_since(UNIX_EPOCH) else {
                continue;
            };
            let ts_ms = ts.as_millis() as u64;
            let points = series
                .entry((
                    sample.source.clone(),
                    sample.metric.clone(),
                    sample.labels.clone(),
                ))
                .or_default();
            points.push_back((ts_ms, value));

            let horizon = ts_ms.saturating_sub(self.retention.as_millis() as u64);
            while points.front().is_some_and(|(t, _)| *t < horizon) {
                points.pop_front();
            }
        }
    }

    /// Les séries d'une source dont la métrique figure dans `metrics`
    /// (tous labels confondus), points dans l'ordre chronologique.
    pub fn query(&self, source: &SourceId, metrics: &[String]) -> Vec<HotSeries> {
        let series = self.series.lock().unwrap();
        let mut out: Vec<HotSeries> = series
            .iter()
            .filter(|((src, metric, _), _)| {
                src == source && metrics.iter().any(|m| m == metric.as_str())
            })
            .map(|((_, metric, labels), points)| HotSeries {
                metric: metric.clone(),
                labels: labels.clone(),
                points: points.iter().copied().collect(),
            })
            .collect();
        out.sort_by(|a, b| (&a.metric, &a.labels).cmp(&(&b.metric, &b.labels)));
        out
    }
}

/// Fenêtre par défaut de l'application.
impl Default for HotWindow {
    fn default() -> Self {
        Self::new(Duration::from_secs(600))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn sample_at(source: &SourceId, metric: &'static str, ts: SystemTime, v: f64) -> Sample {
        Sample {
            ts,
            ..Sample::gauge(source, metric, v)
        }
    }

    #[test]
    fn appends_queries_and_sorts() {
        let hot = HotWindow::new(Duration::from_secs(600));
        let source = SourceId::local();
        let now = SystemTime::now();

        hot.append_batch(&[
            sample_at(&source, "cpu.usage", now, 10.0),
            sample_at(&source, "cpu.usage", now, 20.0).with_label("core", "0"),
            sample_at(&source, "mem.used_pct", now, 50.0),
        ]);

        let got = hot.query(&source, &["cpu.usage".to_owned()]);
        assert_eq!(got.len(), 2, "global + core=0");
        assert!(got[0].labels.is_empty(), "série sans label d'abord");
        assert_eq!(got[0].points[0].1, 10.0);

        let none = hot.query(&SourceId::ssh("ailleurs"), &["cpu.usage".to_owned()]);
        assert!(none.is_empty(), "les sources sont isolées");
    }

    #[test]
    fn old_points_are_pruned() {
        let hot = HotWindow::new(Duration::from_secs(60));
        let source = SourceId::local();
        let now = SystemTime::now();

        hot.append_batch(&[sample_at(
            &source,
            "cpu.usage",
            now - Duration::from_secs(120),
            1.0,
        )]);
        hot.append_batch(&[sample_at(&source, "cpu.usage", now, 2.0)]);

        let got = hot.query(&source, &["cpu.usage".to_owned()]);
        assert_eq!(got[0].points.len(), 1, "le point hors fenêtre est purgé");
        assert_eq!(got[0].points[0].1, 2.0);
    }

    #[test]
    fn text_values_are_ignored() {
        let hot = HotWindow::default();
        let source = SourceId::local();
        let mut sample = Sample::gauge(&source, "disk.smart", 0.0);
        sample.value = openscope_core::Value::Text("PASSED".into());
        hot.append_batch(&[sample]);
        assert!(hot.query(&source, &["disk.smart".to_owned()]).is_empty());
    }
}
