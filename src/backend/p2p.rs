use anyhow::Context;
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

use crate::backend::ChatBackend;
use crate::protocol::{ChatEvent, WireEnvelope, PROTOCOL_VERSION};

pub struct P2PBackend {
    username: String,
    writer: OwnedWriteHalf,
    events_rx: mpsc::Receiver<ChatEvent>,
}

impl P2PBackend {
    pub async fn listen(port: u16, username: String) -> anyhow::Result<Self> {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr)
            .await
            .with_context(|| format!("Failed to bind TCP Listener to address: {}", addr))?;

        let (stream, peer_addr) = listener
            .accept()
            .await
            .context("failed to accept incoming connection")?;

        println!("Connected with peer at: {}", peer_addr);

        Self::from_stream(stream, username).await
    }

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
                let line_str = line.trim_end_matches(&['\r', '\n'][..]).to_string();
                // if there ius no body no need to do anything, just move on to the next iteration
                if line_str.is_empty() {
                    continue;
                }

                // we use chat events here when the JSON can't be parsed or there is a protocol version mismatch.
                let event = match serde_json::from_str::<WireEnvelope>(&line_str) {
                    Ok(env) => {
                        let incoming_version = env.version();
                        if incoming_version == PROTOCOL_VERSION {
                            env.into_chat_event()
                        } else {
                            ChatEvent::System(format!(
                                "chat protocol version mismatch: {} -> {} | {}",
                                incoming_version, PROTOCOL_VERSION, line_str
                            ))
                        }
                    }
                    Err(e) => ChatEvent::System(format!(
                        "invalid JSON error: {} | message: {}",
                        e, line_str
                    )),
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

    async fn join_room(&mut self, room: &str) -> anyhow::Result<()> {
        let wire = WireEnvelope::join(&self.username, room);
        let json = serde_json::to_string(&wire)?;
        self.writer.write_all(json.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;
        Ok(())
    }

    async fn leave_room(&mut self, room: &str) -> anyhow::Result<()> {
        let wire = WireEnvelope::leave(&self.username, room);
        let json = serde_json::to_string(&wire)?;
        self.writer.write_all(json.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;
        Ok(())
    }

    async fn send_message(&mut self, room: &str, body: &str) -> anyhow::Result<()> {
        let wire = WireEnvelope::chat(&self.username, room, body);
        let json = serde_json::to_string(&wire)?;
        self.writer.write_all(json.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;
        Ok(())
    }
}
