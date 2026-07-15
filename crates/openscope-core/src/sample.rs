use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::source::SourceId;

/// Identité stable d'une métrique, hiérarchique par convention :
/// `"cpu.usage"`, `"gpu.0.vram.used"`, `"disk.nvme0n1.read_bytes"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MetricId(pub Cow<'static, str>);

impl MetricId {
    pub fn new(id: impl Into<Cow<'static, str>>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MetricId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&'static str> for MetricId {
    fn from(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
}

/// Dimensions d'un échantillon : `{"core": "3"}`, `{"iface": "eth0"}`, `{"pid": "1234"}`.
///
/// `BTreeMap` pour un ordre stable (sérialisation et clés d'agrégation déterministes).
pub type Labels = BTreeMap<String, String>;

/// Valeur d'un échantillon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum Value {
    /// Valeur instantanée : %, °C, octets, watts…
    Gauge(f64),
    /// Compteur cumulatif (octets transférés depuis le boot…) ;
    /// les consommateurs calculent le taux par delta.
    Counter(u64),
    /// État textuel, ex. verdict SMART `"PASSED"`.
    Text(String),
}

impl Value {
    /// Représentation numérique si elle existe (pour graphes et alertes).
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Gauge(v) => Some(*v),
            Value::Counter(v) => Some(*v as f64),
            Value::Text(_) => None,
        }
    }
}

/// Un point de mesure. L'unité de circulation sur le [`crate::MetricBus`]
/// est un batch `Arc<Vec<Sample>>` (un batch = un tick d'un collecteur).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    pub source: SourceId,
    pub metric: MetricId,
    pub ts: SystemTime,
    pub value: Value,
    #[serde(default, skip_serializing_if = "Labels::is_empty")]
    pub labels: Labels,
}

impl Sample {
    /// Constructeur pour le cas courant : gauge sans label, horodaté maintenant.
    pub fn gauge(source: &SourceId, metric: impl Into<MetricId>, v: f64) -> Self {
        Self {
            source: source.clone(),
            metric: metric.into(),
            ts: SystemTime::now(),
            value: Value::Gauge(v),
            labels: Labels::new(),
        }
    }

    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_as_f64() {
        assert_eq!(Value::Gauge(1.5).as_f64(), Some(1.5));
        assert_eq!(Value::Counter(42).as_f64(), Some(42.0));
        assert_eq!(Value::Text("PASSED".into()).as_f64(), None);
    }

    #[test]
    fn sample_builder_sets_labels() {
        let src = SourceId::local();
        let s = Sample::gauge(&src, "cpu.usage", 12.0).with_label("core", "3");
        assert_eq!(s.labels.get("core").map(String::as_str), Some("3"));
        assert_eq!(s.metric.as_str(), "cpu.usage");
    }

    #[test]
    fn metric_id_static_does_not_allocate() {
        let id = MetricId::from("cpu.usage");
        assert!(matches!(id.0, Cow::Borrowed(_)));
    }
}
