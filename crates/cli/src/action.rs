use niqol_core::ActionRequest;
use clap::Parser;


#[derive(Parser)]
#[command(bin_name = "cargo niqol")]
pub struct NiqolActions {
    #[command(subcommand)]
    pub action_request: ActionRequest
}
