use niqol_core::ActionRequest;
use serde::{Deserialize};


#[derive(Deserialize)]
pub enum IpcRequest {
    Action(ActionRequest)
}
