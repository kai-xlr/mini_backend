pub enum ChatEvent {
    ClientConnected(String),
    MessageReceived { sender: String, body: String },
    MessageBroadcast(String),
    ClientDisconnected(String),
}

#[allow(dead_code)]
impl ChatEvent {
    pub fn event_type(&self) -> &str {
        match self {
            ChatEvent::ClientConnected(_) => "ClientConnected",
            ChatEvent::MessageReceived { .. } => "MessageReceived",
            ChatEvent::MessageBroadcast(_) => "MessageBroadcast",
            ChatEvent::ClientDisconnected(_) => "ClientDisconnected",
        }
    }

    pub fn details(&self) -> String {
        match self {
            ChatEvent::ClientConnected(addr) => addr.clone(),
            ChatEvent::MessageReceived { sender, body } => format!("{}: {}", sender, body),
            ChatEvent::MessageBroadcast(msg) => msg.clone(),
            ChatEvent::ClientDisconnected(addr) => addr.clone(),
        }
    }

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
    pub sequence_id: u64,
    pub timestamp: u64,
    pub event: ChatEvent,
}
