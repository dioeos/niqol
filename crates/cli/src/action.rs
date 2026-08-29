use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(bin_name = "cargo niqol")]
pub struct NiqolActions {
    #[command(subcommand)]
    pub action_request: ActionRequest
}
#[derive(Deserialize, Subcommand, Serialize, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionRequest {
    MarkWindow { slot: u8 },
    FocusMark { slot: u8 }
}
