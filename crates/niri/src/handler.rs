use niri_ipc::Event;
use tracing::{trace, debug};


pub struct NiriEventHandler {
}

impl NiriEventHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn handle_niri_event(
        &self, event: Event
    ) -> anyhow::Result<()> {
        match event {
            Event::WindowFocusChanged { id: Some(id) } => {
                debug!(window_id = id, "window focus changed");
            }
            _ => {
                trace!(?event, "ignoring niri event");
            }
        }
        Ok(())
    }
}
