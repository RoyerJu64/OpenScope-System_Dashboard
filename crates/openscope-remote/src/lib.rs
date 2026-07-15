//! Monitoring distant via SSH (Phase 4, issues #56–#63).
//!
//! Déploie le binaire `openscoped` sur la machine distante via SFTP puis
//! dialogue en JSON-Lines sur stdio — aucun port à ouvrir, rien à
//! installer. Chaque machine devient un [`openscope_core::MetricSource`]
//! qui publie sur le même bus que la collecte locale.
