//! Système de plugins (Phase 5, issues #64–#69).
//!
//! Runtime WASM sandboxé (Extism), manifeste `plugin.toml` avec
//! permissions explicites, circuit breaker : un plugin défaillant ne
//! dégrade jamais l'application.
