use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{self:?}")]
    FailedSocketIO(#[source] io::Error),

    #[error("{self:?}")]
    EmptyIpcResponse(#[source] io::Error),

    #[error("{self:?}")]
    FailedToBindListenerToSocket(#[source] io::Error),

    #[error("{self:?}")]
    FailedToAcceptConnection(#[source] io::Error),

    #[error("{self:?}")]
    FailedToFindXdgRuntimeDirVar,

    #[error("{self:?}")]
    FailedToConnectToIpcSocket(#[source] io::Error),

    #[error("{self:?}")]
    FailedIpcSerdeOperation(#[source] serde_json::Error),
}
