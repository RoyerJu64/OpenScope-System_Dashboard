//! Commands IPC. Un module par domaine à partir de la Phase 1
//! (`commands/metrics.rs`, `commands/process.rs`…) ; la Phase 0 n'expose
//! que les capacités.

use std::collections::BTreeMap;
use std::time::Duration;

use openscope_core::Capabilities;
use tauri::State;

use crate::state::AppState;

/// Ce que la machine sait fournir, par collecteur. Le frontend s'en sert
/// pour n'afficher que les widgets pertinents (dégradation gracieuse).
#[tauri::command]
pub fn get_capabilities(state: State<'_, AppState>) -> Capabilities {
    state.capabilities()
}

/// Intervalles de collecte courants, en millisecondes par collecteur.
#[tauri::command]
pub fn get_collector_intervals(state: State<'_, AppState>) -> BTreeMap<String, u64> {
    state.collector_intervals()
}

/// Change l'intervalle d'un collecteur à chaud (borné à [100 ms, 1 h]
/// par le scheduler). La persistance arrive avec la config (issue #30).
#[tauri::command]
pub fn set_collector_interval(
    state: State<'_, AppState>,
    collector: String,
    ms: u64,
) -> Result<(), String> {
    if state.set_collector_interval(&collector, Duration::from_millis(ms)) {
        Ok(())
    } else {
        Err(format!("collecteur inconnu ou indisponible : {collector}"))
    }
}
