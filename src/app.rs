use crate::backend::p2p::P2PBackend;
use crate::backend::ChatBackend;
use crate::cli::{Cli, Command};
use crate::protocol::{ChatEvent, RoomId};

use chrono::Local;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Server { port, username } => {
            println!("Starting server on port: {} as '{}'", port, username);

            let backend = P2PBackend::listen(port, username).await?;

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

            let backend = P2PBackend::connect(&host, port, username).await?;

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
                Err(e) => {
                    eprintln!("stdin read error: {}", e);
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

    let mut current_room = RoomId::default();

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

            if let Some(rest) = msg.strip_prefix("/join ") {
                let trimmed = rest.trim();

                if trimmed.is_empty() {
                    println!("[system]: usage: /join <room>");
                    continue;
                }

                let room = RoomId::new(trimmed);

                if room == current_room {
                    println!("[system]: already in {}", current_room);
                    continue;
                }

                if current_room != RoomId::default() {
                    backend.leave_room(&current_room).await?;
                }

                current_room = room;
                backend.join_room(&current_room).await?;
                println!("[system]: joined {}", current_room);
                continue;
            }

            if msg == "/leave" {
                if current_room == RoomId::default() {
                    println!("[system]: already in default room");
                    continue;
                }
                backend.leave_room(&current_room).await?;
                println!("[system]: left {}, back to default", current_room);
                current_room = RoomId::default();
                backend.join_room(&current_room).await?;
                continue;
            }

            if backend.send_message(&current_room, &msg).await.is_err() {
                println!("disconnected, quitting loop");
                return Ok(());
            };
        }

        // this grabs all the events that have piled up since the last iteration
        let events = backend.poll_events().await?;
        // this loops throug said events and prints them
        for ev in events {
            match ev {
                ChatEvent::Message {
                    id,
                    ts,
                    room,
                    from,
                    body,
                } => {
                    println!(
                        "{} | {} [{}] {}: {}",
                        ts.with_timezone(&Local).format("%m/%d/%Y %H:%M"),
                        id,
                        room,
                        from,
                        body
                    )
                }
                ChatEvent::System(text) => println!("[system]: {}", text),
            }
        }

        sleep(Duration::from_millis(50)).await;
    }
}
