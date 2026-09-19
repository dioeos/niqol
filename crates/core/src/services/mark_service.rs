use std::sync::{Mutex, Arc};
use tracing::{debug};


use crate::{WindowManager, stores::MarkStore};

const MARK_STORE_LAST_POS: u8 = 8;


pub struct MarkService {
    mark_store: Arc<MarkStore>,
    window_manager: Arc<dyn WindowManager>,
    last_focused_slot: Mutex<Option<u8>>
}

impl MarkService {
    pub fn new(
        window_manager: Arc<dyn WindowManager>
    ) -> Self {
        Self {
            mark_store: Arc::new(MarkStore::new()),
            window_manager,
            last_focused_slot: Mutex::new(None)
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
        self.set_last_focused_slot(slot);
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
        self.set_last_focused_slot(slot);
        Ok(())
    }

    pub async fn focus_next_marked_window(
        &self
    ) -> anyhow::Result<()> {
        let last_slot = *self.last_focused_slot.lock().unwrap();

        let next_slot = match last_slot {
            Some(slot) if slot < MARK_STORE_LAST_POS => slot + 1,
            _ => 1
        };

        let Some(window_id) = self.mark_store.get_mark(next_slot).await else {
            debug!(mark = next_slot, "no window marked");
            return Ok(())
        };

        self.window_manager.focus_window(window_id).await?;
        self.set_last_focused_slot(next_slot);

        Ok(())
    }

    fn set_last_focused_slot(&self, slot: u8) {
        *self.last_focused_slot.lock().unwrap() = Some(slot);
    }
}
