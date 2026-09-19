use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::WindowId;

pub(crate) struct MarkStore {
    marks: RwLock<[Option<WindowId>; 8]>
}

impl MarkStore {
    pub(crate) fn new() -> Self {
        Self { marks: RwLock::new([None; 8]) }
    }

    pub(crate) async fn insert_mark(&self, slot: u8, id: WindowId) {
        let index = usize::from(slot - 1);
        let mut rw_guard = self.marks.write().await;

        for mark in rw_guard.iter_mut() {
            if *mark == Some(id) {
                *mark = None;
            }
        }

        rw_guard[index] = Some(id);
    }

    pub(crate) async fn get_mark(&self, slot: u8) -> Option<WindowId> {
        let index = usize::from(slot - 1);
        let rw_guard = self.marks.read().await;
        rw_guard[index]
    }

    pub(crate) async fn first_mark(&self) -> Option<WindowId> {
        let rw_guard = self.marks.read().await;
        rw_guard[0]
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

    #[tokio::test]
    async fn moves_existing_mark_to_requested() {
        let store = MarkStore::new();
        let id = WindowId(67);
        store.insert_mark(1, id).await;

        assert_eq!(store.get_mark(1).await, Some(id));
        store.insert_mark(2, id).await;
        assert_eq!(store.get_mark(1).await, None);
        assert_eq!(store.get_mark(2).await, Some(id));
    }
}
