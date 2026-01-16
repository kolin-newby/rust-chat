pub type RoomId = String;

#[derive(Debug, Clone)]
pub enum ChatEvent {
    Message {
        from: String,
        room: RoomId,
        body: String,
    },

    System(String),
}
