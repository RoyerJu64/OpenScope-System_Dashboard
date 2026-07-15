use std::path::Path;
use std::time::SystemTime;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::sample::{Labels, MetricId, Sample};
use crate::source::SourceId;

/// Résolution demandée pour une requête d'historique. Correspond aux
/// paliers de downsampling de `openscope-history`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    Raw,
    TenSeconds,
    OneMinute,
    FifteenMinutes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RangeQuery {
    pub source: SourceId,
    pub metrics: Vec<MetricId>,
    pub from: SystemTime,
    pub to: SystemTime,
    pub resolution: Resolution,
}

/// Un point agrégé : aux résolutions non brutes, min/max encadrent avg.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub ts: SystemTime,
    pub avg: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Series {
    pub metric: MetricId,
    pub labels: Labels,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Csv,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SnapshotId(pub i64);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotMeta {
    pub id: SnapshotId,
    pub name: String,
    pub ts: SystemTime,
}

/// Diff de deux snapshots (base de la comparaison avant/après).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotDiff {
    pub a: SnapshotMeta,
    pub b: SnapshotMeta,
    /// métrique+labels → (valeur dans a, valeur dans b)
    pub changed: Vec<MetricDelta>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricDelta {
    pub metric: MetricId,
    pub labels: Labels,
    pub before: Option<f64>,
    pub after: Option<f64>,
}

/// Stockage d'historique. Implémenté par `openscope-history` (SQLite) ;
/// une implémentation en mémoire sert aux tests.
#[async_trait]
pub trait HistoryStore: Send + Sync {
    async fn append(&self, batch: &[Sample]) -> Result<(), CoreError>;

    async fn query(&self, q: RangeQuery) -> Result<Vec<Series>, CoreError>;

    async fn create_snapshot(&self, name: &str) -> Result<SnapshotId, CoreError>;

    async fn list_snapshots(&self) -> Result<Vec<SnapshotMeta>, CoreError>;

    async fn diff_snapshots(&self, a: SnapshotId, b: SnapshotId)
        -> Result<SnapshotDiff, CoreError>;

    async fn export(
        &self,
        q: RangeQuery,
        format: ExportFormat,
        path: &Path,
    ) -> Result<(), CoreError>;
}
