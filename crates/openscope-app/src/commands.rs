//! Commands IPC. Un module par domaine à partir de la Phase 1
//! (`commands/metrics.rs`, `commands/process.rs`…) ; la Phase 0 n'expose
//! que les capacités.

use openscope_core::Capabilities;
use tauri::State;

use crate::state::AppState;

/// Ce que la machine sait fournir, par collecteur. Le frontend s'en sert
/// pour n'afficher que les widgets pertinents (dégradation gracieuse).
#[tauri::command]
pub fn get_capabilities(state: State<'_, AppState>) -> Capabilities {
    state.capabilities()
}
