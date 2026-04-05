use dashmap::DashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

pub type WsSender = mpsc::UnboundedSender<String>;

struct ConnectionEntry {
    sender: WsSender,
}

/// Registry tracking all active WebSocket connections per user.
pub struct ConnectionRegistry {
    connections: DashMap<Uuid, Vec<ConnectionEntry>>,
}

impl ConnectionRegistry {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
        }
    }

    pub fn add(&self, user_id: Uuid, sender: WsSender) {
        self.connections
            .entry(user_id)
            .or_default()
            .push(ConnectionEntry { sender });
    }

    pub fn remove(&self, user_id: Uuid) {
        self.connections.remove(&user_id);
    }

    pub fn send_to_user(&self, user_id: &Uuid, message: &str) {
        if let Some(entries) = self.connections.get(user_id) {
            for entry in entries.value() {
                let _ = entry.sender.send(message.to_string());
            }
        }
    }

    pub fn broadcast_to_room(&self, user_ids: &[Uuid], message: &str) {
        for uid in user_ids {
            self.send_to_user(uid, message);
        }
    }

    pub fn connection_count(&self) -> usize {
        self.connections
            .iter()
            .map(|entry| entry.value().len())
            .sum()
    }
}

impl Default for ConnectionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ConnectionRegistry {
    fn clone(&self) -> Self {
        // ConnectionRegistry is meant to be shared via Arc, not cloned.
        // This creates a new empty registry.
        Self::new()
    }
}
