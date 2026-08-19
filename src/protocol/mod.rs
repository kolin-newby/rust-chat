use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PROTOCOL_VERSION: u8 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RoomId(String);

impl RoomId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for RoomId {
    fn default() -> Self {
        Self("default".to_string())
    }
}

impl fmt::Display for RoomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

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
                    room: room.clone(),
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

    pub fn chat(from: &str, room: &RoomId, body: &str) -> Self {
        Self {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            ts: Utc::now(),
            from: from.to_string(),
            room: Some(room.clone()),
            content: WireContent::Chat {
                body: body.to_string(),
            },
        }
    }

    pub fn join(from: &str, room: &RoomId) -> Self {
        Self {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            ts: Utc::now(),
            from: from.to_string(),
            room: Some(room.clone()),
            content: WireContent::Join,
        }
    }

    pub fn leave(from: &str, room: &RoomId) -> Self {
        Self {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            ts: Utc::now(),
            from: from.to_string(),
            room: Some(room.clone()),
            content: WireContent::Leave,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_with_room_produces_message_event() {
        let envelope = WireEnvelope::chat("alice", &RoomId::new("general"), "hello");

        let event = envelope.clone().into_chat_event();

        match event {
            ChatEvent::Message {
                id,
                from,
                room,
                body,
                ..
            } => {
                assert_eq!(id, envelope.id);
                assert_eq!(from, "alice");
                assert_eq!(room, RoomId::new("general"));
                assert_eq!(body, "hello");
            }
            other => panic!("expected ChatEvent::Message, got {:?}", other),
        }
    }

    #[test]
    fn chat_without_room_produces_system_error() {
        let mut envelope = WireEnvelope::chat("alice", &RoomId::new("general"), "hello");
        envelope.room = None;

        let event = envelope.into_chat_event();

        match event {
            ChatEvent::System(text) => assert_eq!(text, "missing <room> for Chat"),
            other => panic!("expected ChatEvent::System, got {:?}", other),
        }
    }

    #[test]
    fn join_with_room_produces_system_notice() {
        let envelope = WireEnvelope::join("bob", &RoomId::new("general"));

        let event = envelope.into_chat_event();

        match event {
            ChatEvent::System(text) => assert_eq!(text, "bob joined general"),
            other => panic!("expected ChatEvent::System, got {:?}", other),
        }
    }

    #[test]
    fn join_without_room_produces_system_error() {
        let mut envelope = WireEnvelope::join("bob", &RoomId::new("general"));
        envelope.room = None;

        let event = envelope.into_chat_event();

        match event {
            ChatEvent::System(text) => assert_eq!(text, "missing <room> for Join"),
            other => panic!("expected ChatEvent::System, got {:?}", other),
        }
    }

    #[test]
    fn leave_with_room_produces_system_notice() {
        let envelope = WireEnvelope::leave("bob", &RoomId::new("general"));

        let event = envelope.into_chat_event();

        match event {
            ChatEvent::System(text) => assert_eq!(text, "bob left general"),
            other => panic!("expected ChatEvent::System, got {:?}", other),
        }
    }

    #[test]
    fn leave_without_room_produces_system_error() {
        let mut envelope = WireEnvelope::leave("bob", &RoomId::new("general"));
        envelope.room = None;

        let event = envelope.into_chat_event();

        match event {
            ChatEvent::System(text) => assert_eq!(text, "missing <room> for Leave"),
            other => panic!("expected ChatEvent::System, got {:?}", other),
        }
    }

    #[test]
    fn system_content_passes_text_through_regardless_of_room() {
        let envelope = WireEnvelope {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            ts: Utc::now(),
            from: "server".to_string(),
            room: None,
            content: WireContent::System {
                text: "connected".to_string(),
            },
        };

        let event = envelope.into_chat_event();

        match event {
            ChatEvent::System(text) => assert_eq!(text, "connected"),
            other => panic!("expected ChatEvent::System, got {:?}", other),
        }
    }

    #[test]
    fn chat_constructor_sets_protocol_version_and_body() {
        let envelope = WireEnvelope::chat("alice", &RoomId::new("general"), "hi");

        assert_eq!(envelope.version(), PROTOCOL_VERSION);
        assert_eq!(envelope.room, Some(RoomId::new("general")));
        assert!(matches!(envelope.content, WireContent::Chat { ref body } if body == "hi"));
    }

    #[test]
    fn room_id_default_is_default_room() {
        assert_eq!(RoomId::default().as_str(), "default");
    }

    #[test]
    fn room_id_displays_as_its_string() {
        let room = RoomId::new("general");
        assert_eq!(room.to_string(), "general");
    }

    #[test]
    fn wire_envelope_round_trips_through_json() {
        let original = WireEnvelope::chat("alice", &RoomId::new("general"), "hello");

        let json = serde_json::to_string(&original).unwrap();
        let decoded: WireEnvelope = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.id, original.id);
        assert_eq!(decoded.room, original.room);
        assert!(matches!(decoded.content, WireContent::Chat { ref body } if body == "hello"));
    }
}
