use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::sync::Arc;
use tokio::sync::broadcast;

/// A simple in-memory event bus
#[derive(Clone)]
pub struct EventBus {
    // Map of Event Type -> Broadcast Sender
    channels: Arc<DashMap<TypeId, broadcast::Sender<Arc<dyn Any + Send + Sync>>>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
        }
    }

    /// Publish an event
    pub fn publish<E: Clone + Send + Sync + 'static>(&self, event: E) {
        let type_id = TypeId::of::<E>();
        if let Some(sender) = self.channels.get(&type_id) {
            let _ = sender.send(Arc::new(event));
        }
    }

    /// Subscribe to an event
    pub fn subscribe<E: Clone + Send + Sync + 'static>(
        &self,
    ) -> broadcast::Receiver<Arc<dyn Any + Send + Sync>> {
        let type_id = TypeId::of::<E>();
        let sender = self.channels.entry(type_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(100);
            tx
        });
        sender.subscribe()
    }
}
