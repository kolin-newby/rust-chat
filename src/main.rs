mod app;
mod backend;
mod cli;

use crate::cli::Cli;
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    app::run(cli).await
}
