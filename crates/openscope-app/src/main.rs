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
            commands::set_collector_interval,
            commands::list_processes,
            commands::kill_process,
            commands::set_priority,
            commands::get_hot_window
        ])
        .setup(|app| {
            let bus = MetricBus::default();
            let state = state::AppState::default();
            app.manage(state.clone());

            forwarder::spawn(app.handle().clone(), bus.subscribe());

            // Fenêtre chaude : consommateur du bus comme un autre.
            let hot = std::sync::Arc::new(openscope_history::HotWindow::default());
            state.install_hot(hot.clone());
            let mut hot_rx = bus.subscribe();
            tauri::async_runtime::spawn(async move {
                loop {
                    match hot_rx.recv().await {
                        Ok(batch) => hot.append_batch(&batch),
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            });

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
