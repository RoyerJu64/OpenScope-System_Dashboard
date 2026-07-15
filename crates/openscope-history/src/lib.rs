//! Historique des métriques.
//!
//! - Phase 1 (issue #20) : [`HotWindow`], fenêtre chaude en mémoire —
//!   les dernières minutes de chaque série, pour pré-remplir les graphes
//!   à l'ouverture et alimenter les pages sans attendre.
//! - Phase 3 (issues #47–#52) : persistance SQLite avec downsampling par
//!   paliers (implémentera [`openscope_core::HistoryStore`]), exports
//!   CSV/JSON, snapshots et diff avant/après.

mod hot;

pub use hot::{HotSeries, HotWindow};
