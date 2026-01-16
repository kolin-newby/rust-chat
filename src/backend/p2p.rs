use anyhow::Context;
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::backend::ChatBackend;
use crate::protocol::ChatEvent;

pub struct P2PBackend {
    username: String,
    writer: OwnedWriteHalf,
    events_rx: mpsc::Receiver<ChatEvent>,
}

impl P2PBackend {
    pub async fn connect(host: &str, port: u16, username: String) -> anyhow::Result<Self> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr)
            .await
            .with_context(|| format!("Failed to connect to: {}", addr))?;

        Self::from_stream(stream, username).await
    }

    async fn from_stream(stream: TcpStream, username: String) -> anyhow::Result<Self> {
        let (reader, writer) = stream.into_split();
        let (events_tx, events_rx) = mpsc::channel::<ChatEvent>(256);

        tokio::spawn(async move {
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            let _ = events_tx.send(ChatEvent::System("connected".into())).await;

            loop {
                // make sure we start with an empty line each iteration
                line.clear();

                // reads a full line from the reader and breaks if there is an error
                let bytes = match reader.read_line(&mut line).await {
                    Ok(n) => n,
                    Err(_) => break,
                };

                // breaks on EOF, there are not bytes read
                if bytes == 0 {
                    break;
                }

                // trims off the '\n' and '\r' characters off of the end of the lines
                let body = line.trim_end_matches(&['\r', '\n'][..]).to_string();
                // if there ius no body no need to do anything, just move on to the next iteration
                if body.is_empty() {
                    continue;
                }

                // parses the read body into a ChatEvent::Message structure TODO: wire-up the actual 'from' and 'room' values
                let event = ChatEvent::Message {
                    from: "peer".to_string(),
                    room: "default".to_string(),
                    body,
                };

                // sends parsed event and breaks if that errors
                if events_tx.send(event).await.is_err() {
                    break;
                }
            }

            // sends System message when connection is closed
            let _ = events_tx
                .send(ChatEvent::System("connection closed".into()))
                .await;
        });

        // returns a P2PBackend struct when the async task is successfully spawned, does not clock rest of program
        Ok(Self {
            username,
            writer,
            events_rx,
        })
    }
}

#[async_trait]
impl ChatBackend for P2PBackend {
    async fn poll_events(&mut self) -> anyhow::Result<Vec<ChatEvent>> {
        let mut events = Vec::new();

        while let Ok(ev) = self.events_rx.try_recv() {
            events.push(ev);
        }

        Ok(events)
    }

    async fn join_room(&mut self, _room: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn send_message(&mut self, _room: &str, body: &str) -> anyhow::Result<()> {
        self.writer.write_all(body.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;
        Ok(())
    }
}
