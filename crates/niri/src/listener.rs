use std::sync::Arc;

use anyhow::{Context, bail};
use niri_ipc::Event;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixStream, sync::mpsc::Sender,
};
use tracing::info;

use crate::{NiriConnector};

pub struct NiriListener {
    niri_connector: Arc<NiriConnector>,
    niri_tx: Sender<niri_ipc::Event>
}

impl NiriListener {
    pub fn new(
        niri_connector: Arc<NiriConnector>,
        niri_tx: Sender<niri_ipc::Event>
    ) -> Self {
        Self {
            niri_connector,
            niri_tx
        }
    }

    #[tracing::instrument(name = "niri_listener", level = "info", skip(self))]
    pub async fn run(&self) -> anyhow::Result<()> {
        info!("listener started");
        let stream = self
            .niri_connector
            .connect()
            .await
            .context("Failed to initialize niri connection")?;

        let mut stream_reader = BufReader::new(stream);
        let mut buf = String::new();

        self.niri_connector
            .send_event_stream_handshake(&mut stream_reader, &mut buf)
            .await
            .context("Failed to initialize niri event stream")?;

        loop {
            let event: Event = Self::read_niri_stream(&mut stream_reader, &mut buf).await?;

            //@TODO: this should emit domain model instead of niri_ipc in the future
            self.niri_tx
                .send(event)
                .await
                .context("Failed to emit niri event to daemon")?;
        }
    }

    async fn read_niri_stream(
        reader: &mut BufReader<UnixStream>,
        buf: &mut String,
    ) -> anyhow::Result<Event> {
        buf.clear();

        let bytes_read = reader
            .read_line(buf)
            .await
            .context("Failed to read niri IPC event")?;

        if bytes_read == 0 {
            bail!("niri IPC event stream closed unexpectedly")
        }

        let event: Event =
            serde_json::from_str(buf).context("Failed to deserialize niri IPC event")?;

        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use niri_ipc::{Reply, Request, Response};
    use tempfile::tempdir;
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt},
        net::UnixListener, sync::mpsc,
    };

    fn socket_path() -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("niri-test.sock");
        (dir, path)
    }

    fn create_listener(connector: Arc<NiriConnector>) -> NiriListener {
        let (niri_tx, _niri_rx) = mpsc::channel(32);
        NiriListener::new(connector, niri_tx)
    }

    #[tokio::test]
    async fn run_sends_niri_event() {
        let (_dir, path) = socket_path();
        let listener = UnixListener::bind(&path).unwrap();

        //server stream (niri) needs to be writing events
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);
            let mut buf = String::new();
            //read that the run function send event stream
            reader.read_line(&mut buf).await.unwrap();

            let request: Request = serde_json::from_str(&buf).unwrap();

            assert!(matches!(request, Request::EventStream));

            let reply = Reply::Ok(Response::Handled);
            let mut reply_payload = serde_json::to_string(&reply).unwrap();
            reply_payload.push('\n');

            reader.get_mut().write_all(reply_payload.as_bytes()).await.unwrap();
            reader.get_mut().flush().await.unwrap();

            let event = Event::WindowFocusChanged { id: Some(42) };
            let mut event_payload = serde_json::to_string(&event).unwrap();
            event_payload.push('\n');

            reader.get_mut().write_all(event_payload.as_bytes()).await.unwrap();
            reader.get_mut().flush().await.unwrap();
        });

        let (niri_tx, mut niri_rx) = mpsc::channel(32);

        let connector = Arc::new(NiriConnector::new(path));
        let listener = NiriListener::new(connector, niri_tx);

        let run_task = tokio::spawn(async move {
            listener.run().await.unwrap();
        });

        let received_event = niri_rx.recv().await.unwrap();

        assert!(matches!(
            received_event,
            niri_ipc::Event::WindowFocusChanged { id: Some(42) }
        ));
        
        drop(run_task);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn run_fails_when_niri_connection_cannot_be_initialized() {
        let (_dir, path) = socket_path();
        let connector = Arc::new(NiriConnector::new(path));

        let listener = create_listener(connector);

        let err = listener.run().await.unwrap_err();

        assert!(
            err.to_string()
                .contains("Failed to initialize niri connection")
        );
    }

    #[tokio::test]
    async fn run_fails_when_event_stream_cannot_be_initialized() {
        let (_dir, path) = socket_path();
        let listener = UnixListener::bind(&path).unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);

            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();

            //connection drops no reply
            drop(reader);
        });

        let connector = Arc::new(NiriConnector::new(path));
        let listener = create_listener(connector);

        let err = listener.run().await.unwrap_err();
        let chain = format!("{err:#}");

        assert!(chain.contains("Failed to initialize niri event stream"));
        assert!(chain.contains("niri closed the connection before acknowledging EventStream"));

        server.await.unwrap();
    }

    #[tokio::test]
    async fn read_niri_stream_returns_event() {
        let (client_stream, mut server_stream) = UnixStream::pair().unwrap();

        let event = Event::WindowFocusChanged { id: Some(42) };

        let mut payload = serde_json::to_string(&event).unwrap();
        payload.push('\n');

        server_stream.write_all(payload.as_bytes()).await.unwrap();

        let mut reader = BufReader::new(client_stream);
        let mut buf = String::new();

        let received = NiriListener::read_niri_stream(&mut reader, &mut buf)
            .await
            .unwrap();

        assert!(matches!(
                received,
                Event::WindowFocusChanged { id: Some(42) }
        ));
    }

    #[tokio::test]
    async fn read_niri_stream_fails_when_stream_closes() {
        let (client_stream, server_stream) = UnixStream::pair().unwrap();

        drop(server_stream);

        let mut reader = BufReader::new(client_stream);
        let mut buf = String::new();

        let err = NiriListener::read_niri_stream(&mut reader, &mut buf)
            .await
            .unwrap_err();

        assert!(
            err.to_string()
                .contains("niri IPC event stream closed unexpectedly")
        );
    }

    #[tokio::test]
    async fn read_niri_stream_can_read_consecutive_events() {
        let (client_stream, mut server_stream) = UnixStream::pair().unwrap();

        let first = Event::WindowFocusChanged { id: Some(42) };
        let second = Event::WindowFocusChanged { id: Some(43) };

        let payload = format!(
            "{}\n{}\n",
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        );

        server_stream
            .write_all(payload.as_bytes())
            .await
            .unwrap();

        let mut reader = BufReader::new(client_stream);
        let mut buf = String::new();

        let first_received = NiriListener::read_niri_stream(&mut reader, &mut buf)
            .await
            .unwrap();

        let second_received = NiriListener::read_niri_stream(&mut reader, &mut buf)
            .await
            .unwrap();

        assert!(matches!(
                first_received,
                Event::WindowFocusChanged { id: Some(42) }
        ));
        assert!(matches!(
                second_received,
                Event::WindowFocusChanged { id: Some(43) }
        ));
    }
}
