use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionRequest {
    MarkWindow { slot: u8 }
}
