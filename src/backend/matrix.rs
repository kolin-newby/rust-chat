use std::collections::HashMap;

use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use matrix_sdk::{
    config::SyncSettings,
    room::Room,
    ruma::{
        events::room::message::{MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent},
        OwnedRoomId, RoomOrAliasId, ServerName,
    },
    Client,
};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    backend::ChatBackend,
    protocol::{ChatEvent, RoomId},
};

pub struct MatrixBackend {
    client: Client,
    events_rx: mpsc::Receiver<ChatEvent>,
    room_map: HashMap<RoomId, OwnedRoomId>,
}

impl MatrixBackend {
    pub async fn login(
        homeserver: &ServerName,
        user_id: &str,
        password: &str,
        insecure: bool,
    ) -> anyhow::Result<Self> {
        // Insecure mode connects directly to the given host over HTTP rather
        // than going through .well-known discovery: a local test homeserver's
        // own well-known response can still claim an https:// base_url (as
        // Conduit's does), which would silently pull us back to HTTPS even
        // though we asked to skip TLS.
        let builder = if insecure {
            Client::builder().homeserver_url(format!("http://{homeserver}"))
        } else {
            Client::builder().server_name(homeserver)
        };
        let client = builder.build().await?;

        client
            .matrix_auth()
            .login_username(user_id, password)
            .initial_device_display_name("rust-chat")
            .send()
            .await?;

        let own_user_id = client
            .user_id()
            .context("client has no user_id after login")?
            .to_owned();

        // catch up on room state before wiring up the live handler, otherwise the
        // first sync replays existing room history through poll_events
        client.sync_once(SyncSettings::new()).await?;

        let (events_tx, events_rx) = mpsc::channel::<ChatEvent>(256);

        let handler_user_id = own_user_id.clone();
        let handler_events_tx = events_tx.clone();
        client.add_event_handler(move |ev: OriginalSyncRoomMessageEvent, room: Room| {
            let events_tx = handler_events_tx.clone();
            let own_user_id = handler_user_id.clone();
            async move {
                // we sent this message ourselves; app.rs doesn't expect an echo of its own sends
                if ev.sender == own_user_id {
                    return;
                }

                let MessageType::Text(text) = ev.content.msgtype else {
                    return;
                };

                let ts = ev
                    .origin_server_ts
                    .to_system_time()
                    .map(DateTime::<Utc>::from)
                    .unwrap_or_else(Utc::now);

                let event = ChatEvent::Message {
                    // matrix event ids are opaque strings, not UUIDs, so mint a local one
                    id: Uuid::new_v4(),
                    ts,
                    from: ev.sender.to_string(),
                    room: RoomId::new(room.room_id().to_string()),
                    body: text.body,
                };

                let _ = events_tx.send(event).await;
            }
        });

        let sync_client = client.clone();
        tokio::spawn(async move {
            let result = sync_client.sync(SyncSettings::new()).await;
            let text = match result {
                Ok(()) => "matrix sync loop ended".to_string(),
                Err(e) => format!("matrix sync loop ended: {}", e),
            };
            let _ = events_tx.send(ChatEvent::System(text)).await;
        });

        Ok(Self {
            client,
            events_rx,
            room_map: HashMap::new(),
        })
    }
}

#[async_trait]
impl ChatBackend for MatrixBackend {
    async fn poll_events(&mut self) -> anyhow::Result<Vec<ChatEvent>> {
        let mut events = Vec::new();

        while let Ok(ev) = self.events_rx.try_recv() {
            events.push(ev);
        }

        Ok(events)
    }

    async fn join_room(&mut self, room: &RoomId) -> anyhow::Result<()> {
        let room_or_alias = RoomOrAliasId::parse(room.as_str())
            .with_context(|| format!("'{}' is not a valid room id or alias", room))?;

        let joined = self
            .client
            .join_room_by_id_or_alias(&room_or_alias, &[])
            .await
            .with_context(|| format!("failed to join '{}'", room))?;

        self.room_map
            .insert(room.clone(), joined.room_id().to_owned());

        Ok(())
    }

    async fn leave_room(&mut self, room: &RoomId) -> anyhow::Result<()> {
        let Some(room_id) = self.room_map.remove(room) else {
            anyhow::bail!("not currently in room '{}'", room);
        };

        if let Some(matrix_room) = self.client.get_room(&room_id) {
            matrix_room.leave().await?;
        }

        Ok(())
    }

    async fn send_message(&mut self, room: &RoomId, body: &str) -> anyhow::Result<()> {
        let room_id = self
            .room_map
            .get(room)
            .with_context(|| format!("not currently in room '{}', join it first", room))?;

        let matrix_room = self
            .client
            .get_room(room_id)
            .context("joined room is no longer known to the client")?;

        matrix_room
            .send(RoomMessageEventContent::text_plain(body))
            .await?;

        Ok(())
    }
}
