//! Contrat commun d'OpenScope : types de données, traits et bus de métriques.
//!
//! Ce crate ne dépend d'aucune API système. Tous les autres crates du
//! workspace dépendent de lui, et uniquement de lui (sauf `openscope-app`,
//! seul point d'assemblage).

pub mod bus;
pub mod collector;
pub mod error;
pub mod history;
pub mod sample;
pub mod source;

pub use bus::MetricBus;
pub use collector::{Collector, ProbeResult};
pub use error::CoreError;
pub use history::{ExportFormat, HistoryStore, RangeQuery, Resolution, Series};
pub use sample::{Labels, MetricId, Sample, Value};
pub use source::{Action, ActionOutcome, Capabilities, MetricSource, SourceId, SourceStatus};
