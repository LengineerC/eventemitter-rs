use std::fmt::Debug;
use crate::types::*;

#[derive(Clone)]
pub enum Callback {
    Sync(SyncCallback),
    Async(AsyncCallback),
}

#[derive(Clone)]
pub enum ThreadSafeCallback {
    Sync(SyncThreadSafeCallback),
    Async(AsyncThreadSafeCallback),
}

pub struct Handler {
    pub id: HandlerId,
    pub callback: Callback,
    pub once: bool,
}

impl Debug for Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handler")
            .field("id", &self.id)
            .field("once", &self.once)
            .finish()
    }
}

pub struct ThreadSafeHandler {
    pub id: HandlerId,
    pub callback: ThreadSafeCallback,
    pub once: bool,
}

impl Debug for ThreadSafeHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadSafeHandler")
            .field("id", &self.id)
            .field("once", &self.once)
            .finish()
    }
}
