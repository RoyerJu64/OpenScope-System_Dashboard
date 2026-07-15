//! Commands IPC. Un module par domaine à partir de la Phase 1
//! (`commands/metrics.rs`, `commands/process.rs`…) ; la Phase 0 n'expose
//! que les capacités.

use std::collections::BTreeMap;
use std::time::Duration;

use openscope_core::{ActionOutcome, Capabilities};
use serde::Serialize;
use tauri::{Manager, State};

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

/// Une série de la fenêtre chaude (points des ~10 dernières minutes).
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct HotSeriesDto {
    pub metric: String,
    pub labels: BTreeMap<String, String>,
    #[cfg_attr(feature = "ts", ts(type = "Array<number>"))]
    pub ts_ms: Vec<u64>,
    pub values: Vec<f64>,
}

/// Fenêtre chaude des métriques demandées (tous labels confondus) :
/// appelée au montage d'une page pour pré-remplir les graphes.
#[tauri::command]
pub fn get_hot_window(state: State<'_, AppState>, metrics: Vec<String>) -> Vec<HotSeriesDto> {
    state
        .query_hot(&openscope_core::SourceId::local(), &metrics)
        .into_iter()
        .map(|s| {
            let (ts_ms, values) = s.points.into_iter().unzip();
            HotSeriesDto {
                metric: s.metric.to_string(),
                labels: s.labels,
                ts_ms,
                values,
            }
        })
        .collect()
}

fn layout_path(app: &tauri::AppHandle, page: &str) -> Result<std::path::PathBuf, String> {
    // Nom contraint : identifiant de page, pas un chemin.
    if page.is_empty() || !page.bytes().all(|b| b.is_ascii_lowercase() || b == b'-') {
        return Err(format!("nom de page invalide : {page}"));
    }
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("layouts");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(format!("{page}.json")))
}

/// Disposition sauvegardée d'une page de widgets (issue #24). Le schéma
/// appartient au frontend : JSON opaque côté backend, versionné par le
/// frontend lui-même.
#[tauri::command]
pub fn get_layout(
    app: tauri::AppHandle,
    page: String,
) -> Result<Option<serde_json::Value>, String> {
    let path = layout_path(&app, &page)?;
    match std::fs::read_to_string(&path) {
        Ok(raw) => Ok(serde_json::from_str(&raw).ok()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn save_layout(
    app: tauri::AppHandle,
    page: String,
    layout: serde_json::Value,
) -> Result<(), String> {
    let path = layout_path(&app, &page)?;
    let raw = serde_json::to_string_pretty(&layout).map_err(|e| e.to_string())?;
    std::fs::write(&path, raw).map_err(|e| e.to_string())
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
