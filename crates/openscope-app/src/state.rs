use std::sync::{Arc, Mutex};

use openscope_collect::scheduler::LocalScheduler;
use openscope_core::Capabilities;

/// État partagé de l'application, injecté dans les commands Tauri.
#[derive(Clone, Default)]
pub struct AppState {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Default)]
struct Inner {
    /// Gardé vivant ici ; son Drop arrête les tâches de collecte.
    scheduler: Option<LocalScheduler>,
}

impl AppState {
    pub fn install_scheduler(&self, scheduler: LocalScheduler) {
        self.inner.lock().unwrap().scheduler = Some(scheduler);
    }

    pub fn capabilities(&self) -> Capabilities {
        self.inner
            .lock()
            .unwrap()
            .scheduler
            .as_ref()
            .map(|s| s.capabilities().clone())
            .unwrap_or_default()
    }
}
