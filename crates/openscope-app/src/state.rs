use std::sync::{Arc, Mutex};

use openscope_collect::scheduler::LocalScheduler;
use openscope_core::{Capabilities, SourceId};
use openscope_history::{HotSeries, HotWindow};

/// État partagé de l'application, injecté dans les commands Tauri.
#[derive(Clone, Default)]
pub struct AppState {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Default)]
struct Inner {
    /// Gardé vivant ici ; son Drop arrête les tâches de collecte.
    scheduler: Option<LocalScheduler>,
    /// Table des processus (état des deltas CPU entre deux pulls).
    #[cfg(target_os = "linux")]
    process: openscope_collect::process::ProcessTable,
    /// Fenêtre chaude des métriques (issue #20).
    hot: Option<Arc<HotWindow>>,
}

impl AppState {
    pub fn install_scheduler(&self, scheduler: LocalScheduler) {
        self.inner.lock().unwrap().scheduler = Some(scheduler);
    }

    pub fn install_hot(&self, hot: Arc<HotWindow>) {
        self.inner.lock().unwrap().hot = Some(hot);
    }

    pub fn query_hot(&self, source: &SourceId, metrics: &[String]) -> Vec<HotSeries> {
        self.inner
            .lock()
            .unwrap()
            .hot
            .as_ref()
            .map(|hot| hot.query(source, metrics))
            .unwrap_or_default()
    }

    pub fn capabilities(&self) -> Capabilities {
        let mut caps = self
            .inner
            .lock()
            .unwrap()
            .scheduler
            .as_ref()
            .map(|s| s.capabilities().clone())
            .unwrap_or_default();
        // La table des processus n'est pas un collecteur du scheduler :
        // sa capacité est déclarée ici (issue #19).
        caps.collectors.insert(
            "process".to_owned(),
            openscope_core::source::CollectorCapability {
                available: cfg!(target_os = "linux"),
                reason: (!cfg!(target_os = "linux"))
                    .then(|| "Windows/macOS : issues #43/#44".to_owned()),
                details: Default::default(),
            },
        );
        caps
    }

    pub fn collector_intervals(&self) -> std::collections::BTreeMap<String, u64> {
        self.inner
            .lock()
            .unwrap()
            .scheduler
            .as_ref()
            .map(|s| {
                s.intervals()
                    .into_iter()
                    .map(|(id, d)| (id, d.as_millis() as u64))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn process_rows(&self) -> Vec<crate::commands::ProcessRowDto> {
        #[cfg(target_os = "linux")]
        {
            self.inner
                .lock()
                .unwrap()
                .process
                .snapshot()
                .into_iter()
                .map(Into::into)
                .collect()
        }
        #[cfg(not(target_os = "linux"))]
        {
            Vec::new() // issues #43/#44 (M2)
        }
    }

    pub fn set_collector_interval(&self, collector: &str, interval: std::time::Duration) -> bool {
        self.inner
            .lock()
            .unwrap()
            .scheduler
            .as_ref()
            .is_some_and(|s| s.set_interval(collector, interval))
    }
}
