//! Pont bus → webview. La frontière webview est le point de contention
//! n°1 : les batches du bus sont coalescés par fenêtres de 100 ms et
//! émis en un seul event `metrics-batch` (un tableau de batches) —
//! quatre collecteurs qui tickent à la même seconde = un event, pas
//! quatre (issue #21).

use std::time::UNIX_EPOCH;

use openscope_core::{bus::Batch, Labels, Value};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

/// DTO de l'event `metrics-batch`. La définition TypeScript est générée
/// par `cargo xtask gen-types` vers `frontend/src/ipc/bindings/`.
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct BatchDto {
    source: String,
    /// Millisecondes epoch (u64 côté Rust, number côté JSON/TS).
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    ts_ms: u64,
    samples: Vec<SampleDto>,
}

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SampleDto {
    metric: String,
    value: ValueDto,
    labels: Labels,
}

#[derive(Serialize, Clone)]
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ValueDto {
    Gauge(f64),
    Counter(#[cfg_attr(feature = "ts", ts(type = "number"))] u64),
    Text(String),
}

impl From<&Value> for ValueDto {
    fn from(value: &Value) -> Self {
        match value {
            Value::Gauge(v) => ValueDto::Gauge(*v),
            Value::Counter(v) => ValueDto::Counter(*v),
            Value::Text(v) => ValueDto::Text(v.clone()),
        }
    }
}

fn to_dto(batch: &Batch) -> Option<BatchDto> {
    let first = batch.first()?;
    let ts_ms = first.ts.duration_since(UNIX_EPOCH).ok()?.as_millis() as u64;
    Some(BatchDto {
        source: first.source.to_string(),
        ts_ms,
        samples: batch
            .iter()
            .map(|s| SampleDto {
                metric: s.metric.to_string(),
                value: (&s.value).into(),
                labels: s.labels.clone(),
            })
            .collect(),
    })
}

/// Fenêtre de coalescence : assez courte pour rester imperceptible à
/// l'œil, assez longue pour regrouper les collecteurs d'un même tick.
const FLUSH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

pub fn spawn(app: AppHandle, mut rx: broadcast::Receiver<Batch>) {
    tauri::async_runtime::spawn(async move {
        let mut pending: Vec<BatchDto> = Vec::new();
        let mut flush = tokio::time::interval(FLUSH_INTERVAL);
        flush.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                received = rx.recv() => match received {
                    Ok(batch) => {
                        if let Some(dto) = to_dto(&batch) {
                            pending.push(dto);
                        }
                    }
                    // Webview plus lente que la collecte : on saute les
                    // batches perdus, les suivants remettent à jour.
                    Err(broadcast::error::RecvError::Lagged(missed)) => {
                        tracing::debug!(missed, "forwarder en retard sur le bus");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                },
                _ = flush.tick() => {
                    if pending.is_empty() {
                        continue;
                    }
                    if let Err(error) = app.emit("metrics-batch", &pending) {
                        tracing::warn!(%error, "émission IPC échouée");
                    }
                    pending.clear();
                }
            }
        }
    });
}
