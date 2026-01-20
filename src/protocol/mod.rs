use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PROTOCOL_VERSION: u8 = 2;

pub type RoomId = String;

#[derive(Debug, Clone)]
pub enum ChatEvent {
    Message {
        id: Uuid,
        ts: DateTime<Utc>,
        from: String,
        room: RoomId,
        body: String,
    },

    System(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireEnvelope {
    pub v: u8,
    pub id: Uuid,
    pub ts: DateTime<Utc>,
    pub from: String,
    pub room: Option<RoomId>,
    #[serde(flatten)]
    pub content: WireContent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WireContent {
    Chat { body: String },
    Join,
    Leave,
    System { text: String },
}

impl WireEnvelope {
    pub fn version(&self) -> u8 {
        self.v
    }

    pub fn into_chat_event(self) -> ChatEvent {
        let WireEnvelope {
            id,
            ts,
            from,
            room,
            content,
            ..
        } = self;

        match content {
            WireContent::Chat { body } => {
                let Some(room) = room.as_ref() else {
                    return ChatEvent::System("missing <room> for Chat".to_string());
                };
                ChatEvent::Message {
                    id,
                    ts,
                    from,
                    room: room.to_string(),
                    body,
                }
            }
            WireContent::Join => {
                let Some(room) = room.as_ref() else {
                    return ChatEvent::System("missing <room> for Join".to_string());
                };
                ChatEvent::System(format!("{} joined {}", from, room))
            }
            WireContent::Leave => {
                let Some(room) = room.as_ref() else {
                    return ChatEvent::System("missing <room> for Leave".to_string());
                };
                ChatEvent::System(format!("{} left {}", from, room))
            }
            WireContent::System { text } => ChatEvent::System(text),
        }
    }

    pub fn chat(from: &str, room: &str, body: &str) -> Self {
        Self {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            ts: Utc::now(),
            from: from.to_string(),
            room: Some(room.to_string()),
            content: WireContent::Chat {
                body: body.to_string(),
            },
        }
    }

    pub fn join(from: &str, room: &str) -> Self {
        Self {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            ts: Utc::now(),
            from: from.to_string(),
            room: Some(room.to_string()),
            content: WireContent::Join,
        }
    }

    pub fn leave(from: &str, room: &str) -> Self {
        Self {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            ts: Utc::now(),
            from: from.to_string(),
            room: Some(room.to_string()),
            content: WireContent::Leave,
        }
    }
}
