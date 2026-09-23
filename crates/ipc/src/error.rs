use std::{io, path::PathBuf};


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{self:?}")]
    FailedSocketIO(#[source] io::Error),

    #[error("{self:?}")]
    FailedToBindListenerToSocket(#[source] io::Error),

    #[error("{self:?}")]
    FailedToAcceptConnection(#[source] io::Error)
}
