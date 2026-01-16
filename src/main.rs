mod cli;
mod app;

use clap::Parser;
use crate::cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    app::run(cli).await
}