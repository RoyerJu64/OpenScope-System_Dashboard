//! Collecte système locale : un module par domaine, chacun implémentant
//! [`openscope_core::Collector`], orchestrés par le [`scheduler`].
//!
//! Collecteurs disponibles : `cpu` (Linux). Les autres domaines
//! (memory, disk, network, process…) arrivent avec les issues #13 à #19.
//! [`mock::MockCollector`] reste disponible pour les tests et le bench.

#[cfg(target_os = "linux")]
pub mod cpu;
#[cfg(target_os = "linux")]
pub mod disk;
#[cfg(target_os = "linux")]
pub mod memory;
pub mod mock;
#[cfg(target_os = "linux")]
pub mod network;
#[cfg(target_os = "linux")]
pub mod process;
pub mod scheduler;

use openscope_core::Collector;

/// Le registry : la liste des collecteurs connus pour cette plateforme.
/// Ajouter un domaine = ajouter une ligne ici, rien d'autre ne change.
pub fn default_collectors() -> Vec<Box<dyn Collector>> {
    #[cfg(target_os = "linux")]
    {
        vec![
            Box::new(cpu::CpuCollector::new()),
            Box::new(memory::MemoryCollector::new()),
            Box::new(disk::DiskCollector::new()),
            Box::new(network::NetworkCollector::new()),
        ]
    }
    #[cfg(not(target_os = "linux"))]
    {
        // Windows/macOS : issues #43 et #44 (M2).
        Vec::new()
    }
}
