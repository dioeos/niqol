use clap::Parser;
use niqol_core::ActionRequest;

#[derive(Parser)]
#[command(bin_name = "cargo niqol")]
pub struct NiqolActions {
    #[command(subcommand)]
    pub action_request: ActionRequest,
}
