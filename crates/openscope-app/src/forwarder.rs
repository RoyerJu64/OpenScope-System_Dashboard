//! Pont bus → webview : chaque batch du bus devient un event IPC
//! `metrics-batch`. Un event par tick de collecteur, jamais un par
//! métrique — la frontière webview est le point de contention n°1.

use std::time::UNIX_EPOCH;

use openscope_core::{bus::Batch, Labels, Value};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

/// DTO de l'event `metrics-batch`. Sera généré vers TypeScript via ts-rs
/// (issue #6) ; en attendant, le miroir manuel vit dans
/// `frontend/src/ipc/types.ts`.
#[derive(Serialize, Clone)]
struct BatchDto {
    source: String,
    ts_ms: u64,
    samples: Vec<SampleDto>,
}

#[derive(Serialize, Clone)]
struct SampleDto {
    metric: String,
    value: ValueDto,
    #[serde(skip_serializing_if = "Labels::is_empty")]
    labels: Labels,
}

#[derive(Serialize, Clone)]
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
enum ValueDto {
    Gauge(f64),
    Counter(u64),
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

pub fn spawn(app: AppHandle, mut rx: broadcast::Receiver<Batch>) {
    tauri::async_runtime::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(batch) => {
                    if let Some(dto) = to_dto(&batch) {
                        if let Err(error) = app.emit("metrics-batch", &dto) {
                            tracing::warn!(%error, "émission IPC échouée");
                        }
                    }
                }
                // Webview plus lente que la collecte : on saute les batches
                // perdus, le prochain reçu remet le graphe à jour.
                Err(broadcast::error::RecvError::Lagged(missed)) => {
                    tracing::debug!(missed, "forwarder en retard sur le bus");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}
