use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::{Window, WindowId};

pub(crate) struct WindowStore {
    windows: RwLock<HashMap<WindowId, Window>>,
}

impl WindowStore {
    pub(crate) fn new() -> Self {
        Self {
            windows: RwLock::new(HashMap::new()),
        }
    }

    pub(crate) async fn insert_window(&self, window_id: WindowId, window: Window) {
        let mut rw_guard = self.windows.write().await;
        rw_guard.entry(window_id).or_insert(window);
    }

    pub(crate) async fn remove_window(&self, window_id: WindowId) {
        let mut rw_guard = self.windows.write().await;
        rw_guard.remove(&window_id);
    }

    pub(crate) async fn get_window(&self, window_id: WindowId) -> Option<Window> {
        let rw_guard = self.windows.read().await;
        rw_guard.get(&window_id).cloned()
    }
}
