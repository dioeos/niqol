use std::sync::Arc;
use tracing::{debug};


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
            debug!("Cannot mark window. No current focused window");
            return Ok(())
        };

        let window_id = focused_window.id;
        let debug_slot = slot;

        self.mark_store.insert_mark(slot, focused_window.id).await;
        debug!(window_id = window_id.0, mark = debug_slot, "mark focused window");

        Ok(())
    }

    pub async fn focus_marked_window(
        &self,
        slot: u8
    ) -> anyhow::Result<()> {
        let Some(window_id) = self.mark_store.get_mark(slot).await else {
            debug!(mark = slot, "no window marked");
            return Ok(())
        };

        self.window_manager.focus_window(window_id).await?;
        Ok(())
    }
}
