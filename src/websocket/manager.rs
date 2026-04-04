use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;
use super::events::WsEvent;

const CHANNEL_CAPACITY: usize = 64;

#[derive(Clone)]
pub struct WsManager {
    channels: Arc<DashMap<Uuid, broadcast::Sender<WsEvent>>>,
}

impl WsManager {
    pub fn new() -> Self {
        Self { channels: Arc::new(DashMap::new()) }
    }

    pub fn subscribe(&self, user_id: Uuid) -> broadcast::Receiver<WsEvent> {
        let entry = self.channels.entry(user_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
            tx
        });
        entry.subscribe()
    }

    pub fn send_to_user(&self, user_id: Uuid, event: WsEvent) {
        if let Some(sender) = self.channels.get(&user_id) {
            let _ = sender.send(event);
        }
    }

    pub fn remove_user(&self, user_id: &Uuid) {
        self.channels.remove(user_id);
    }
}

impl Default for WsManager {
    fn default() -> Self { Self::new() }
}
