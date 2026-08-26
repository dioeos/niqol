use std::{env::var_os, ffi::OsString, fs::{self, OpenOptions}, io::ErrorKind, path::PathBuf, sync::Arc};

use anyhow::{Context, bail};
use tokio::{io::{BufReader, AsyncBufReadExt}, net::{UnixListener, UnixStream}};

use crate::{actions::{ActionHandler, action::ActionRequest}, niri::NiriConnector};



pub struct ActionListener {
    niri_connector: Arc<NiriConnector>
}

impl ActionListener {
    pub fn new(
        connector: Arc<NiriConnector>
    ) -> Self {
        Self {
            niri_connector: connector
        }
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let action_socket = Self::create_action_socket()
            .context("Failed to initialize action socket")?;

        let mut line = String::new();
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open("/tmp/niqol-actions.log")?;

        let stream  = self
            .niri_connector
            .connect()
            .await
            .context("Failed to initialize niri connection")?;

        let handler = ActionHandler::new(stream);

        loop {
            let (action_stream, _) = action_socket.accept().await?;
            let mut stream_reader: BufReader<UnixStream>
                = BufReader::new(action_stream);

            let action_request 
                = Self::read_action_socket_stream(&mut stream_reader, &mut line).await?;
            handler.handle_action_request(action_request, &mut file).await?;
        }
    }

    async fn read_action_socket_stream(
        reader: &mut BufReader<UnixStream>,
        buf: &mut String
    ) -> Result<ActionRequest, anyhow::Error> {
        buf.clear();
        
        let bytes_read = reader
            .read_line(buf)
            .await
            .context("Failed to read cargo-niqol action request")?;

        if bytes_read == 0 {
            bail!("Failed to read cargo-niqol action request");
        }

        let action_request: ActionRequest = serde_json::from_str(buf)
            .context("Failed to parse cargo-niqol action request")?;

        Ok(action_request)
    }

    //socket that cargo-niqol uses in its dispatcher
    fn create_action_socket() -> Result<UnixListener, anyhow::Error>{
        let xdg_os_string: OsString = var_os("XDG_RUNTIME_DIR")
            .context("XDG_RUNTIME_DIR environment variable is not set")?;

        let mut action_socket_path = PathBuf::from(xdg_os_string);
        action_socket_path.push("niqol-actions.sock");


        let listener: UnixListener = Self::create_socket_file(action_socket_path)
            .context("Failed to create Niqol command listener socket")?;

        Ok(listener)
    }

    fn create_socket_file(socket_path: PathBuf) -> Result<UnixListener, anyhow::Error> {
        fs::remove_file(&socket_path)
            .or_else(|err| {
                if err.kind() == ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(err)
                }
            })
        .with_context(|| format!(
                "Failed to remove stale socket at {}",
                socket_path.display()
        ))?;

        let listener = UnixListener::bind(&socket_path)
            .with_context(|| format!(
                    "Failed to bind to socket at {}",
                    socket_path.display()
            ))?;

        Ok(listener)
    }
}
