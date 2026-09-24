use niqol_core::{ActionRequest, QueryRequest};
use serde::Deserialize;

#[derive(Deserialize)]
pub enum IpcRequest {
    Action(ActionRequest),
    Query(QueryRequest),
}
