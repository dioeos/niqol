use std::sync::Arc;

use niqol_core::{ActionRequest, MarkService};
use tracing::{debug};


pub(crate) struct ActionHandler {
    mark_service: Arc<MarkService>
}

impl ActionHandler {
    pub(crate) fn new(
        mark_service: Arc<MarkService>
    ) -> Self {
        Self { mark_service }
    }

    pub(crate) async fn handle_action_request(
        &self,
        request: ActionRequest
    ) -> anyhow::Result<()> {
        match request {
            ActionRequest::MarkWindow { slot } => {
                self.mark_service.mark_focused_window(slot).await?;
            }
            ActionRequest::FocusMark { slot } => {
                self.mark_service.focus_marked_window(slot).await?;
            }
        }
        debug!(action_request = ?request, "successfully handled");
        Ok(())
    }
}
