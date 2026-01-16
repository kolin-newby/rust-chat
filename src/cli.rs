use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "rust-chat")]
#[command(about = "Phase 1: TCP chat with a backend abstraction")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Listen for a single incoming TCP connection
    Server {
        /// Port to listen on
        #[arg(short, long, default_value = "9000")]
        port: u16,

        /// Your display name (sent with messages later)
        #[arg(short, long, default_value = "server")]
        username: String,
    },

    /// Connect to a TCP server
    Client {
        /// Server host (IP or hostname)
        #[arg(short = 'H', long)]
        host: String,

        /// Server port
        #[arg(short, long, default_value = "9000")]
        port: u16,

        /// Your display name (sent with messages later)
        #[arg(short, long, default_value = "client")]
        username: String,
    },
}
