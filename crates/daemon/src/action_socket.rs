use std::{fs, io::ErrorKind, path::PathBuf};

use anyhow::Context;
use tokio::net::{UnixListener, UnixStream};


//represents an actual socket (bi-directional stream of actions
pub(crate) struct ActionSocket {
    listener: UnixListener
}

impl ActionSocket {
    pub(crate) fn bind(socket_path: PathBuf) -> anyhow::Result<Self> {
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

        let listener: UnixListener = UnixListener::bind(&socket_path)
            .with_context(|| format!(
                    "Failed to bind to socket at {}",
                    socket_path.display()
            ))?;

        Ok(Self { listener })
    }

    pub(crate) async fn accept(&self) -> anyhow::Result<UnixStream> {
        let (stream, _) = self.listener.accept().await?;
        Ok(stream)
    }
}
