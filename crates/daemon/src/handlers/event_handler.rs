use std::sync::Arc;

use niqol_niri::{NiriEvent};
use niqol_core::MarkService;
use tracing::{trace, debug};


pub(crate) struct EventHandler {
    mark_service: Arc<MarkService>
}

impl EventHandler {
    pub(crate) fn new(
        mark_service: Arc<MarkService>
    ) -> Self {
        Self { mark_service }
    }

    pub(crate) async fn handle_event(
        &self,
        event: NiriEvent
    ) -> anyhow::Result<()> {
        match event {
            NiriEvent::WindowFocusChanged { id: Some(id) } => {
                debug!(window_id = id, "window focus changed");
            }
            _ => {
                trace!(?event, "ignoring niri event");
            }
        }
        Ok(())
    }
}
