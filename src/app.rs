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
        }
    }

    Ok(())
}
