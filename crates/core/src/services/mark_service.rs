use std::sync::Arc;

use crate::{WindowManager, stores::MarkStore};


pub struct MarkService {
    mark_store: Arc<MarkStore>,
    window_manager: Arc<dyn WindowManager>
}

impl MarkService {
    pub fn new(
        window_manager: Arc<dyn WindowManager>
    ) -> Self {
        Self {
            mark_store: Arc::new(MarkStore::new()),
            window_manager
        }
    }

    pub async fn mark_focused_window(
        &self,
        slot: u8
    ) -> anyhow::Result<()> {

        let Some(focused_window) = self.window_manager.get_focused_window().await? else {
            return Ok(())
        };

        self.mark_store.insert_mark(slot, focused_window.id).await;

        Ok(())
    }
}
