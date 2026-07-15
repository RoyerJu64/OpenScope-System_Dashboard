use std::collections::BTreeMap;
use std::fmt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::bus::MetricBus;
use crate::error::CoreError;

/// Identité d'une source de métriques : `"local"`, `"ssh:serveur-prod"`…
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceId(pub String);

impl SourceId {
    pub fn local() -> Self {
        Self("local".to_owned())
    }

    pub fn ssh(host_alias: &str) -> Self {
        Self(format!("ssh:{host_alias}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Ce qu'une source sait fournir, par collecteur, établi au `probe()`.
/// L'UI s'en sert pour n'afficher que les widgets pertinents.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Capabilities {
    /// collecteur → détail (ex. "cpu" → {available: true, details: {"cores": "16", "rapl": "true"}})
    pub collectors: BTreeMap<String, CollectorCapability>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectorCapability {
    pub available: bool,
    /// Raison si indisponible (affichable en tooltip), détails si disponible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub details: BTreeMap<String, String>,
}

/// État de connexion d'une source (surtout pertinent pour le distant).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", content = "detail", rename_all = "snake_case")]
pub enum SourceStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

/// Action exécutable sur une source, routée en local ou via SSH.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Kill { pid: u32, signal: i32 },
    SetPriority { pid: u32, nice: i32 },
    ContainerStart { id: String },
    ContainerStop { id: String },
    ContainerRestart { id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionOutcome {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ActionOutcome {
    pub fn success() -> Self {
        Self {
            ok: true,
            message: None,
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: Some(message.into()),
        }
    }
}

/// Une source de métriques : la machine locale (scheduler de collecte)
/// ou une machine distante (`openscope-remote`). Une fois démarrée, elle
/// publie ses batches sur le bus jusqu'à `stop()`.
#[async_trait]
pub trait MetricSource: Send + Sync {
    fn id(&self) -> SourceId;

    /// Démarre la publication sur le bus. Non bloquant : la source
    /// spawne ses propres tâches.
    async fn start(&mut self, bus: MetricBus) -> Result<(), CoreError>;

    async fn stop(&mut self) -> Result<(), CoreError>;

    fn status(&self) -> SourceStatus;

    fn capabilities(&self) -> Capabilities;

    /// Exécute une action (kill, renice, docker…) sur cette source.
    async fn execute(&self, action: Action) -> Result<ActionOutcome, CoreError>;
}
