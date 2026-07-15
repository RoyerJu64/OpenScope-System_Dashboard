//! Historique des métriques.
//!
//! - Phase 1 (issue #20) : fenêtre chaude en mémoire (ring buffer ~10 min).
//! - Phase 3 (issues #47–#52) : persistance SQLite avec downsampling par
//!   paliers, exports CSV/JSON, snapshots et diff avant/après.
//!
//! Implémente [`openscope_core::HistoryStore`].
