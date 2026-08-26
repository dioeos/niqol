use std::{fs::File, sync::Arc, io::Write};

use anyhow::Context;
use tokio::net::UnixStream;

use crate::{actions::action::ActionRequest, niri::NiriConnector};



pub struct ActionHandler {
    niri_stream: UnixStream
}

impl ActionHandler {
    pub fn new(
        stream: UnixStream
    ) -> Self {
        Self {
            niri_stream: stream
        }
    }

    pub async fn handle_action_request(
        &self,
        action_request: ActionRequest,
        file: &mut File
    ) -> Result<(), anyhow::Error> {
        match action_request {
            ActionRequest::MarkWindow { slot } => {
                writeln!(file, "Requested to mark window: {}", slot)?;
            }
        }
        Ok(())
    }

    // async fn get_current_focused_window(&self) -> Result<niri_ipc::Window, anyhow::Error> {
    //     // let stream: UnixStream = self
    //     //     .niri_connector
    //     //     .connect()
    //     //     .await
    //     //     .context("Failed to initialize niri connection")?;
    //     //
    //     // let mut reader:
    // }
}
