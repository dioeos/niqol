use std::collections::HashMap;

use tokio::sync::Mutex;

pub struct MarkStore {
    pub map: Mutex<HashMap<u8, u64>>,
}

impl MarkStore {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    pub async fn insert_mark(&self, slot: u8, window_id: u64) {
        self.map.lock().await.insert(slot, window_id);
    }
}
