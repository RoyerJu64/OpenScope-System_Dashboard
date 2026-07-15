//! Moteur d'alertes (Phase 3, issues #53–#55).
//!
//! Règles déclaratives (métrique + condition + durée de maintien +
//! sévérité), machine à états `Ok → Pending → Firing → Resolved` pour
//! éviter le flapping, sorties vers notifications desktop et UI.
