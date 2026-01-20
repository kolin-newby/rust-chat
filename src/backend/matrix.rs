use async_trait::async_trait;
use std::collections::HashMap;

use matrix_sdk::{Client, ServerName};
use tokio::sync::mpsc;

use crate::{
    backend::ChatBackend,
    protocol::{ChatEvent, RoomId},
};

pub struct MatrixBackend {
    client: Client,
    events_rx: mpsc::Receiver<ChatEvent>,
    user_id: String,
    room_map: HashMap<String, RoomId>,
}

impl MatrixBackend {
    pub async fn login(
        homeserver: &ServerName,
        access_token: &str,
        user_id: &str,
    ) -> anyhow::Result<Self> {
        let client = Client::builder().server_name(homeserver).build().await?;

        client.matrix_auth()
    }
}

#[async_trait]
impl ChatBackend for MatrixBackend {
    async fn poll_events(&mut self) -> anyhow::Result<Vec<ChatEvent>> {
        Ok(Vec::new())
    }

    async fn join_room(&mut self, room: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn leave_room(&mut self, room: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn send_message(&mut self, room: &str, body: &str) -> anyhow::Result<()> {
        Ok(())
    }
}
