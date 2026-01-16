pub mod p2p;

use crate::protocol::ChatEvent;
use async_trait::async_trait;

#[async_trait]
pub trait ChatBackend {
    ///tell the backend which room/channel to use
    async fn join_room(&mut self, room: &str) -> anyhow::Result<()>;

    ///send a message to the active room
    async fn send_message(&mut self, room: &str, body: &str) -> anyhow::Result<()>;

    ///query the backend for changes
    async fn poll_events(&mut self) -> anyhow::Result<Vec<ChatEvent>>;
}
