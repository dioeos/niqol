use anyhow::Context;
use anyhow::bail;
use niqol_core::ActionRequest;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixStream,
    sync::mpsc::Sender,
};
use tracing::{debug, info};

use crate::action_socket::ActionSocket;

pub(crate) struct ActionListener {
    action_socket: ActionSocket,
    action_tx: Sender<ActionRequest>,
}

impl ActionListener {
    pub(crate) fn new(action_socket: ActionSocket, action_tx: Sender<ActionRequest>) -> Self {
        Self {
            action_socket,
            action_tx,
        }
    }

    #[tracing::instrument(name = "action_listener", level = "info", skip(self))]
    pub(crate) async fn run(self) -> anyhow::Result<()> {
        info!("listener started");
        //receives "focus-window" / "mark-window" action
        //this should use mark service which uses window manager
        loop {
            let action_stream = self.action_socket.accept().await?;
            let mut buf = String::new();
            let mut reader = BufReader::new(action_stream);

            let action_request: ActionRequest =
                Self::read_action_stream(&mut reader, &mut buf).await?;

            self.action_tx
                .send(action_request)
                .await
                .context("Failed to emit action request to daemon")?;
        }
    }

    pub(crate) async fn read_action_stream(
        reader: &mut BufReader<UnixStream>,
        buf: &mut String,
    ) -> anyhow::Result<ActionRequest> {
        buf.clear();

        let bytes_read = reader
            .read_line(buf)
            .await
            .context("Failed to read cargo niqol action request")?;

        if bytes_read == 0 {
            bail!("action stream closed unexpectedly");
        }

        let action_request: ActionRequest = serde_json::from_str(buf)
            .context("Failed to deserialize cargo niqol action request")?;

        Ok(action_request)
    }
}
