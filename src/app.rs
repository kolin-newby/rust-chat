use crate::backend::p2p::P2PBackend;
use crate::backend::ChatBackend;
use crate::cli::{Cli, Command};
use crate::protocol::ChatEvent;

use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Server { port, username } => {
            println!("Starting server on port: {} as '{}'", port, username);

            let mut backend = P2PBackend::listen(port, username).await?;
            backend.join_room("default").await?;

            return run_interactive(backend).await;
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

            return run_interactive(backend).await;
        }
    }
}

async fn run_interactive(mut backend: P2PBackend) -> anyhow::Result<()> {
    let (input_tx, mut input_rx) = mpsc::channel::<String>(64);

    tokio::spawn(async move {
        let mut stdin = BufReader::new(io::stdin());
        let mut line = String::new();

        loop {
            line.clear();

            let bytes = match stdin.read_line(&mut line).await {
                Ok(n) => n,
                Err(_) => {
                    break;
                }
            };

            if bytes == 0 {
                break;
            }

            let msg = line.trim_end_matches(&['\n', '\r'][..]);

            if msg.is_empty() {
                continue;
            }

            if input_tx.send(msg.to_string()).await.is_err() {
                break;
            }
        }
    });

    loop {
        // this while loop empties the 'input_rx' channel
        while let Ok(msg) = input_rx.try_recv() {
            if msg.is_empty() {
                continue;
            }
            if msg == "/quit" {
                println!("exiting interactive loop, goodbye...");
                return Ok(());
            }

            if backend.send_message("default", &msg).await.is_err() {
                println!("disconnected, quitting loop");
                return Ok(());
            };
        }

        // this grabs all the events that have piled up since the last iteration
        let events = backend.poll_events().await?;
        // this loops throug said events and prints them
        for ev in events {
            match ev {
                ChatEvent::Message { from, body, .. } => println!("{}: {}", from, body),
                ChatEvent::System(text) => println!("[system]: {}", text),
            }
        }

        sleep(Duration::from_millis(50)).await;
    }
}
