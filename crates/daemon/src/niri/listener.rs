use std::fs::{File, OpenOptions};
use std::io::Write;

use anyhow::{Context, bail};
use niri_ipc::Event;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixStream,
};

use crate::niri::NiriConnector;

pub struct NiriListener {
    niri_connector: NiriConnector,
}

impl NiriListener {
    pub fn new(connector: NiriConnector) -> Self {
        Self {
            niri_connector: connector,
        }
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let stream = self
            .niri_connector
            .connect()
            .await
            .context("Failed to initialize niri connection")?;

        let mut stream_reader: BufReader<UnixStream> = BufReader::new(stream);

        let mut line = String::new();
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open("/tmp/niqol.log")?;

        self.niri_connector
            .send_event_stream_handshake(&mut stream_reader, &mut line, &mut file)
            .await?;

        loop {
            let event: Event = Self::read_niri_stream(&mut stream_reader, &mut line).await?;

            Self::handle_niri_event(event, &mut file).await?;
        }
    }

    async fn read_niri_stream(
        reader: &mut BufReader<UnixStream>,
        buf: &mut String,
    ) -> Result<niri_ipc::Event, anyhow::Error> {
        buf.clear();

        let bytes_read = reader
            .read_line(buf)
            .await
            .context("Failed to read niri IPC event")?;

        if bytes_read == 0 {
            bail!("Failed to read niri IPC event");
        };

        let event: Event = serde_json::from_str(buf).context("Failed to parse niri IPC event")?;

        Ok(event)
    }

    async fn handle_niri_event(event: Event, file: &mut File) -> Result<(), anyhow::Error> {
        match event {
            Event::WindowFocusChanged { id: Some(id) } => {
                writeln!(file, "Window focus changed: {}", id)?;
            }
            _ => {
                writeln!(file, "Handling some event")?;
            }
        }
        Ok(())
    }
}
