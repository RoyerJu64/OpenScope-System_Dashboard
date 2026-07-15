//! Collecte système locale : un module par domaine, chacun implémentant
//! [`openscope_core::Collector`], orchestrés par le [`scheduler`].
//!
//! Phase 0 : seul le collecteur factice [`mock::MockCollector`] existe,
//! pour valider la chaîne collecteur → bus → IPC → widget.
//! Les collecteurs réels (cpu, memory, disk, network, process…) arrivent
//! en Phase 1 (issues #12 à #19).

pub mod mock;
pub mod scheduler;

use openscope_core::Collector;

/// Le registry : la liste des collecteurs connus pour cette plateforme.
/// Ajouter un domaine = ajouter une ligne ici, rien d'autre ne change.
pub fn default_collectors() -> Vec<Box<dyn Collector>> {
    vec![Box::new(mock::MockCollector::new())]
}
