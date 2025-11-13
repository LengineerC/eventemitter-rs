use std::fmt::Debug;
use crate::types::*;

pub enum Callback {
    Sync(SyncCallback),
    Async(AsyncCallback),
}

pub enum ThreadSafeCallback {
    Sync(SyncThreadSafeCallback),
    Async(AsyncThreadSafeCallback),
}

pub struct Listener {
    pub id: ListenerId,
    pub callback: SyncCallback,
    pub once: bool,
}

impl Debug for Listener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Listener")
            .field("id", &self.id)
            .field("once", &self.once)
            .finish()
    }
}

pub struct ThreadSafeListener {
    pub id: ListenerId,
    pub callback: SyncThreadSafeCallback,
    pub once: bool,
}

impl Debug for ThreadSafeListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadSafeListener")
            .field("id", &self.id)
            .field("once", &self.once)
            .finish()
    }
}
