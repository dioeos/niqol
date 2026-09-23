use crate::{Window, WindowId, stores::WindowStore};
use tracing::debug;

pub struct WindowService {
    window_store: WindowStore,
}

impl WindowService {
    pub fn new() -> Self {
        Self {
            window_store: WindowStore::new(),
        }
    }
    pub async fn insert_window(&self, window: Window) {
        debug!(
            window_id = ?window.id,
            window = ?window,
            "inserting window"
        );
        self.window_store.insert_window(window.id, window).await;
    }

    pub async fn remove_window(&self, window_id: WindowId) {
        debug!(
            window_id = ?window_id,
            "removing window"
        );
        self.window_store.remove_window(window_id).await;
    }
}
