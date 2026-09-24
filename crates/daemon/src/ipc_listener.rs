use anyhow::{Context, bail};
use niqol_core::{ActionRequest, ActionResponse, QueryResponse};
use niqol_ipc::{IpcSocket, protocol, request::IpcRequest};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixStream,
    sync::mpsc::Sender,
};
use tracing::{debug, info};

use crate::handlers::QueryHandler;

//@NOTE: The `ipc_tx` in the listener is responsible for receiving
//       some ipc event and then broadcasting the request to the
//       proper channel depending on the type of the IpcRequest enum,
//       such as action_tx || query_tx
pub(super) struct IpcListener {
    ipc_socket: IpcSocket,
    action_tx: Sender<ActionRequest>,
    query_handler: QueryHandler,
}

impl IpcListener {
    pub(super) fn new(
        ipc_socket: IpcSocket,
        action_tx: Sender<ActionRequest>,
        query_handler: QueryHandler,
    ) -> Self {
        Self {
            ipc_socket,
            action_tx,
            query_handler,
        }
    }

    pub(super) async fn run(self) -> anyhow::Result<()> {
        info!("listener started");

        loop {
            let ipc_stream = self.ipc_socket.accept().await?;
            let mut buf = String::new();
            let mut reader = BufReader::new(ipc_stream);

            let ipc_request = Self::read_ipc_stream(&mut reader, &mut buf).await?;

            match ipc_request {
                IpcRequest::Action(action_req) => {
                    self.action_tx
                        .send(action_req)
                        .await
                        .context("Failed to emit action request to daemon")?;
                    let json = serde_json::to_string(&ActionResponse::Success)?;
                    protocol::write_to_ipc_socket(reader.get_mut(), &json).await?;
                    debug!("action ack");
                }
                IpcRequest::Query(query_req) => {
                    let query_response: QueryResponse =
                        self.query_handler.handle_query_request(query_req).await?;

                    let json = serde_json::to_string(&query_response)?;
                    protocol::write_to_ipc_socket(reader.get_mut(), &json).await?;
                    debug!("query handled");
                }
            }
        }
    }

    async fn read_ipc_stream(
        reader: &mut BufReader<UnixStream>,
        buf: &mut String,
    ) -> anyhow::Result<IpcRequest> {
        buf.clear();

        let bytes_read = reader
            .read_line(buf)
            .await
            .context("Failed to read ipc stream request")?;

        if bytes_read == 0 {
            bail!("ipc stream closed unexpectedly");
        }

        let ipc_request: IpcRequest =
            serde_json::from_str(buf).context("Failed to deserialize ipc stream request")?;

        Ok(ipc_request)
    }
}
