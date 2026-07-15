//! Commands IPC. Un module par domaine à partir de la Phase 1
//! (`commands/metrics.rs`, `commands/process.rs`…) ; la Phase 0 n'expose
//! que les capacités.

use std::collections::BTreeMap;
use std::time::Duration;

use openscope_core::{ActionOutcome, Capabilities};
use serde::Serialize;
use tauri::State;

use crate::state::AppState;

/// Une ligne de la table des processus (miroir TS généré par ts-rs).
/// `cpu_pct` est en % d'un cœur, convention htop.
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct ProcessRowDto {
    pub pid: i32,
    pub ppid: i32,
    pub name: String,
    pub user: String,
    pub state: String,
    pub cpu_pct: f64,
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub rss_bytes: u64,
    pub cmdline: String,
}

#[cfg(target_os = "linux")]
impl From<openscope_collect::process::ProcessRow> for ProcessRowDto {
    fn from(row: openscope_collect::process::ProcessRow) -> Self {
        Self {
            pid: row.pid,
            ppid: row.ppid,
            name: row.name,
            user: row.user,
            state: row.state.to_string(),
            cpu_pct: row.cpu_pct,
            rss_bytes: row.rss_bytes,
            cmdline: row.cmdline,
        }
    }
}

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

/// Snapshot complet de la table des processus (pull : appelé par la
/// page Processus quand elle est visible). Tri et recherche côté front.
#[tauri::command]
pub fn list_processes(state: State<'_, AppState>) -> Vec<ProcessRowDto> {
    state.process_rows()
}

/// Envoie un signal à un processus (défaut SIGTERM). La confirmation
/// est côté UI ; ici on exécute.
#[tauri::command]
pub fn kill_process(pid: i32, signal: Option<i32>) -> ActionOutcome {
    #[cfg(target_os = "linux")]
    {
        openscope_collect::process::actions::kill(pid, signal.unwrap_or(libc::SIGTERM))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (pid, signal);
        ActionOutcome::failure("non supporté sur cet OS (issue #43/#44)")
    }
}

/// Change la priorité (nice -20..19) d'un processus.
#[tauri::command]
pub fn set_priority(pid: i32, nice: i32) -> ActionOutcome {
    #[cfg(target_os = "linux")]
    {
        openscope_collect::process::actions::set_priority(pid, nice)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (pid, nice);
        ActionOutcome::failure("non supporté sur cet OS (issue #43/#44)")
    }
}
