pub mod client;
pub mod error;
pub mod protocol;
pub mod request;

use std::{fs, io, path::PathBuf};

use tokio::net::{UnixListener, UnixStream};

use crate::error::Error::{self, FailedToAcceptConnection, FailedToBindListenerToSocket};

pub struct IpcSocket {
    listener: UnixListener,
}

impl IpcSocket {
    pub fn bind(socket_path: PathBuf) -> Result<Self, Error> {
        fs::remove_file(&socket_path).or_else(|err| match err.kind() {
            io::ErrorKind::NotFound => Ok(()),
            _ => Err(Error::FailedSocketIO(err)),
        })?;

        let listener: UnixListener =
            UnixListener::bind(&socket_path).map_err(FailedToBindListenerToSocket)?;

        Ok(Self { listener })
    }

    pub async fn accept(&self) -> Result<UnixStream, Error> {
        let (stream, _) = self
            .listener
            .accept()
            .await
            .map_err(FailedToAcceptConnection)?;

        Ok(stream)
    }
}
