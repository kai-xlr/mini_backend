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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_event_store_all_types() {
        let cases: Vec<(&str, &str, fn(&ChatEvent) -> bool)> = vec![
            ("ClientConnected", "127.0.0.1:8080", |e| matches!(e, ChatEvent::ClientConnected(a) if a == "127.0.0.1:8080")),
            ("ClientDisconnected", "127.0.0.1:8080", |e| matches!(e, ChatEvent::ClientDisconnected(a) if a == "127.0.0.1:8080")),
            ("MessageBroadcast", "hello world", |e| matches!(e, ChatEvent::MessageBroadcast(m) if m == "hello world")),
            ("MessageReceived", "127.0.0.1:9999: hi", |e| matches!(e, ChatEvent::MessageReceived { sender, body } if sender == "127.0.0.1:9999" && body == "hi")),
        ];

        for (event_type, details, check) in cases {
            let event = ChatEvent::from_event_store(event_type, details);
            assert!(event.is_some(), "expected Some for {}/{}", event_type, details);
            assert!(check(&event.unwrap()), "check failed for {}/{}", event_type, details);
        }
    }

    #[test]
    fn test_from_event_store_invalid_type_returns_none() {
        assert!(ChatEvent::from_event_store("UnknownEvent", "data").is_none());
        assert!(ChatEvent::from_event_store("", "data").is_none());
        assert!(ChatEvent::from_event_store("ClientConnectedd", "data").is_none());
    }

    #[test]
    fn test_from_event_store_malformed_message_received() {
        assert!(ChatEvent::from_event_store("MessageReceived", "no-separator").is_none());
        assert!(ChatEvent::from_event_store("MessageReceived", "").is_none());
    }

    #[test]
    fn test_event_type_details_roundtrip() {
        let events = vec![
            ChatEvent::ClientConnected("addr".into()),
            ChatEvent::ClientDisconnected("addr".into()),
            ChatEvent::MessageBroadcast("msg".into()),
            ChatEvent::MessageReceived { sender: "127.0.0.1:9999".into(), body: "hello".into() },
        ];

        for original in events {
            let et = original.event_type();
            let det = original.details();
            let recovered = ChatEvent::from_event_store(et, &det).unwrap();
            assert_eq!(original.event_type(), recovered.event_type());
            assert_eq!(original.details(), recovered.details());
        }
    }
}
