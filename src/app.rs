use crate::backend::p2p::P2PBackend;
use crate::backend::ChatBackend;
use crate::cli::{Cli, Command};

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Server { port, username } => {
            println!("Starting server on port: {} as '{}'", port, username);
        }
        Command::Client {
            host,
            port,
            username,
        } => {
            println!(
                "Connecting to host: {} on port: {} as '{}'",
                host, port, username
            );

            let mut backend = P2PBackend::connect(&host, port, username).await?;
            backend.join_room("default").await?;
        }
    }

    Ok(())
}
