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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WireEvent {
    Chat {
        v: u8,
        id: Uuid,
        ts: DateTime<Utc>,
        from: String,
        room: RoomId,
        body: String,
    },
    Join {
        v: u8,
        from: String,
        room: RoomId,
    },
    Leave {
        v: u8,
        from: String,
        room: RoomId,
    },
    System {
        v: u8,
        text: String,
    },
}

impl WireEvent {
    pub fn version(&self) -> u8 {
        match self {
            WireEvent::Chat { v, .. }
            | WireEvent::Join { v, .. }
            | WireEvent::Leave { v, .. }
            | WireEvent::System { v, .. } => *v,
        }
    }

    pub fn into_chat_event(self) -> ChatEvent {
        match self {
            WireEvent::Chat {
                id,
                ts,
                from,
                room,
                body,
                ..
            } => ChatEvent::Message {
                id,
                ts,
                from,
                room,
                body,
            },
            WireEvent::Join { from, room, .. } => {
                ChatEvent::System(format!("{} joined {}", from, room))
            }
            WireEvent::Leave { from, room, .. } => {
                ChatEvent::System(format!("{} left {}", from, room))
            }
            WireEvent::System { text, .. } => ChatEvent::System(text),
        }
    }
}
