use tracing_subscriber::{fmt, EnvFilter};

mod action;
mod dispatcher;

use clap::Parser;
use action::NiqolActions;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    let format = fmt::format()
        .with_level(true)
        .with_target(true)
        .compact();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .event_format(format)
        .init();

    let args = NiqolActions::parse_from(get_args());
    dispatcher::dispatch_action(args.action_request).await?;

    Ok(())
}

fn get_args() -> Vec<String> {
    let mut raw_args: Vec<String> = std::env::args().collect();

    if raw_args.get(1).map(String::as_str) == Some("niqol") {
        raw_args.remove(1);
    }
    raw_args
}
