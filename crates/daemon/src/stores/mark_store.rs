use std::collections::HashMap;

use tokio::sync::RwLock;

pub struct MarkStore {
    pub map: RwLock<HashMap<u8, u64>>
}

impl MarkStore {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new())
        }
    }

    pub async fn insert_mark(&self, slot: u8, window_id: u64) {
        self.map.write().await.insert(slot, window_id);
    }

    pub async fn get_mark(&self, slot: u8) -> Option<u64> {
        self.map.read().await.get(&slot).copied()
    }
}
