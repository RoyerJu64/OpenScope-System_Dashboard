//! OpenScope — binaire desktop. Uniquement de l'orchestration : le
//! bootstrap câble bus, scheduler de collecte, forwarder IPC et commands ;
//! la logique métier vit dans les crates `openscope-*`.

// Pas de console sur Windows en release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod forwarder;
mod state;

use openscope_collect::scheduler::LocalScheduler;
use openscope_core::{MetricBus, SourceId};
use tauri::Manager;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_capabilities,
            commands::get_collector_intervals,
            commands::set_collector_interval
        ])
        .setup(|app| {
            let bus = MetricBus::default();
            let state = state::AppState::default();
            app.manage(state.clone());

            forwarder::spawn(app.handle().clone(), bus.subscribe());

            // Démarrage de la collecte locale. Le scheduler vit aussi
            // longtemps que l'app : on le range dans l'état partagé.
            tauri::async_runtime::spawn(async move {
                let scheduler = LocalScheduler::start(
                    SourceId::local(),
                    bus,
                    openscope_collect::default_collectors(),
                )
                .await;
                state.install_scheduler(scheduler);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("échec du démarrage de l'application Tauri");
}
