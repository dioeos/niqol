use std::collections::HashMap;

use niri_ipc::Window;
use tokio::sync::RwLock;
use tracing::trace;

pub struct WindowStore {
    pub map: RwLock<HashMap<u64, Window>>,
}

impl WindowStore {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
        }
    }

    pub async fn replace_store(&self, windows: Vec<Window>) {
        let new_window_map: HashMap<u64, Window> = windows
            .into_iter()
            .map(|window| (window.id, window))
            .collect();

        let mut current_map = self.map.write().await;

        trace!(window_store = ?*current_map, "window store before replacement");

        *current_map = new_window_map;

        trace!(window_store = ?*current_map, "window store replaced");
    }
}
