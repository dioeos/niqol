use std::{ffi::OsString, sync::Arc};

use crate::{
    actions::ActionListener,
    niri::{NiriConnector, NiriListener},
    stores::MarkStore,
};

pub struct Daemon {
    niri_listener: NiriListener,
    action_listener: ActionListener,
}

impl Daemon {
    pub fn new(niri_socket_path: OsString) -> Self {
        let niri_connector = Arc::new(NiriConnector::new(niri_socket_path.into()));

        let mark_store = Arc::new(MarkStore::new());

        let niri_listener = NiriListener::new(Arc::clone(&niri_connector), Arc::clone(&mark_store));
        let action_listener = ActionListener::new(niri_connector, mark_store);

        Self {
            niri_listener,
            action_listener
        }
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        tokio::try_join!(self.niri_listener.run(), self.action_listener.run())?;
        Ok(())
    }
}
