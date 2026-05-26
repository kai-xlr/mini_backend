pub enum ChatEvent {
    ClientConnected(String),
    MessageReceived { sender: String, body: String },
    MessageBroadcast(String),
    ClientDisconnected(String),
}

pub struct RecordedEvent {
    pub timestamp: u64,
    pub event: ChatEvent,
}
