use std::{ffi::OsString, sync::Arc};

use crate::{
    niri::{NiriConnector, NiriListener},
    actions::{ActionListener}
};

pub struct Daemon {
    niri_listener: NiriListener,
    action_listener: ActionListener
}

impl Daemon {
    pub fn new(niri_socket_path: OsString) -> Self {
        let niri_connector = Arc::new(NiriConnector::new(niri_socket_path.into()));

        Self {
            niri_listener: NiriListener::new(niri_connector.clone()),
            action_listener: ActionListener::new(niri_connector)
        }
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        tokio::try_join!(
            self.niri_listener.run(),
            self.action_listener.run()
        )?;
        Ok(())
    }
}
