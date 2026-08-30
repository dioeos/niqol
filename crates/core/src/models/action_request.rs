use clap::{Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Subcommand, Serialize, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionRequest {
    MarkWindow { slot: u8 },
    FocusMark { slot: u8 }
}
