use tokio::sync::RwLock;

use crate::WindowId;

pub(crate) struct MarkStore {
    marks: RwLock<[Option<WindowId>; 9]>,
}

//@NOTE: `next_mark` and `prev_mark` are internal functions and therefore operate with 0-based
//        indices rather than 1-based slots. `insert_mark` and `get_mark` are user facing functions
//        as they operate based on what the user wants to do, which is why there are 1-based slots.
impl MarkStore {
    pub(crate) fn new() -> Self {
        Self {
            marks: RwLock::new([None; 9]),
        }
    }

    pub(crate) async fn all_marks(&self) -> Vec<(usize, WindowId)> {
        self.marks
            .read()
            .await
            .iter()
            .enumerate()
            .filter_map(|(slot, id)| id.as_ref().map(|id| (slot, *id)))
            .collect()
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

    pub(crate) async fn next_mark(&self, current_slot: usize) -> (Option<usize>, Option<WindowId>) {
        let rw_guard = self.marks.read().await;
        for step in 1..=rw_guard.len() {
            let index = (current_slot + step) % rw_guard.len();
            if let Some(window_id) = rw_guard[index] {
                return (Some(index), Some(window_id));
            }
        }

        (None, None)
    }

    pub(crate) async fn prev_mark(&self, current_slot: usize) -> (Option<usize>, Option<WindowId>) {
        let rw_guard = self.marks.read().await;
        for step in 1..=rw_guard.len() {
            let index = (current_slot + rw_guard.len() - step) % rw_guard.len();
            if let Some(window_id) = rw_guard[index] {
                return (Some(index), Some(window_id));
            }
        }

        (None, None)
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

    #[tokio::test]
    async fn next_mark_wraps_when_index_overflows() {
        let store = MarkStore::new();
        let id = WindowId(67);
        store.insert_mark(1, id).await;

        let result = store.next_mark(8).await;
        assert_eq!(result, (Some(0), Some(id)));
    }

    #[tokio::test]
    async fn prev_mark_wraps_when_index_underflows() {
        let store = MarkStore::new();
        let id = WindowId(67);
        store.insert_mark(9, id).await;

        let result = store.prev_mark(0).await;
        assert_eq!(result, (Some(8), Some(id)));
    }
}
