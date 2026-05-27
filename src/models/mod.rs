pub enum ChatEvent {
    ClientConnected(String),
    MessageReceived { sender: String, body: String },
    MessageBroadcast(String),
    ClientDisconnected(String),
}

impl ChatEvent {
    pub fn from_event_store(event_type: &str, details: &str) -> Option<Self> {
        match event_type {
            "ClientConnected" => Some(ChatEvent::ClientConnected(details.to_string())),
            "ClientDisconnected" => Some(ChatEvent::ClientDisconnected(details.to_string())),
            "MessageBroadcast" => Some(ChatEvent::MessageBroadcast(details.to_string())),
            "MessageReceived" => {
                let (sender, body) = details.split_once(": ")?;
                Some(ChatEvent::MessageReceived {
                    sender: sender.to_string(),
                    body: body.to_string(),
                })
            }
            _ => None,
        }
    }
}

pub struct RecordedEvent {
    pub timestamp: u64,
    pub event: ChatEvent,
}
