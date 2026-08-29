use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::WindowId;

pub(crate) struct MarkStore {
    map: RwLock<HashMap<u8, WindowId>>
}

impl MarkStore {
    pub(crate) fn new() -> Self {
        Self { map: RwLock::new(HashMap::new()) }
    }

    pub(crate) async fn insert_mark(&self, slot: u8, id: WindowId) {
        self.map.write().await.insert(slot, id);
    }

    pub(crate) async fn get_mark(&self, slot: u8) -> Option<WindowId> {
        self.map.read().await.get(&slot).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn inserts_and_gets_mark() {
        let store = MarkStore::new();
        let id = WindowId(42);

        store.insert_mark(1, id).await;

        assert_eq!(store.get_mark(1).await, Some(id));
    }

    #[tokio::test]
    async fn returns_none_for_empty_requested_slot() {
        let store = MarkStore::new();

        assert_eq!(store.get_mark(1).await, None);
    }
}
