use std::time::Duration;

use async_trait::async_trait;

use crate::sample::Sample;
use crate::source::CollectorCapability;

/// Résultat de la détection au démarrage.
#[derive(Debug, Clone, PartialEq)]
pub enum ProbeResult {
    /// Le collecteur peut fonctionner ; détail des capacités détectées.
    Available(CollectorCapability),
    /// Matériel/service absent ou inaccessible. La raison est montrée à
    /// l'utilisateur (tooltip), jamais comme une erreur bloquante.
    Unavailable(String),
}

impl ProbeResult {
    pub fn available() -> Self {
        Self::Available(CollectorCapability {
            available: true,
            reason: None,
            details: Default::default(),
        })
    }

    pub fn is_available(&self) -> bool {
        matches!(self, ProbeResult::Available(_))
    }
}

/// Un domaine de collecte (cpu, memory, gpu, disk, network, process,
/// docker, vm…) ou un plugin. Instancié par le registry de
/// `openscope-collect`, tické par le scheduler à son intervalle configuré.
#[async_trait]
pub trait Collector: Send + Sync {
    /// Identifiant stable, utilisé dans la config et les capacités : "cpu", "gpu"…
    fn id(&self) -> &'static str;

    /// Détection : appelé une fois au démarrage (et sur demande de rescan).
    /// Ne doit jamais paniquer ; toute absence devient `Unavailable`.
    async fn probe(&mut self) -> ProbeResult;

    /// Un tick de collecte. Doit rester rapide ; le travail bloquant
    /// (smartctl, libvirt) passe par `spawn_blocking`.
    async fn collect(&mut self) -> anyhow::Result<Vec<Sample>>;

    /// Intervalle par défaut, surchargeable par l'utilisateur.
    fn default_interval(&self) -> Duration;
}
