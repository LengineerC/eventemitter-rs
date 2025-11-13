use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use crate::basis::*;
use crate::types::*;

pub trait ThreadSafeEventEmitter: Send + Sync {
    fn on<F>(&self, event: &str, callback: F) -> ListenerId
    where
        F: Fn(ThreadSafeArgs) + Send + Sync + 'static;

    fn once<F>(&self, event: &str, callback: F) -> ListenerId
    where
        F: Fn(ThreadSafeArgs) + Send + Sync + 'static;

    fn off(&self, event: &str, id: ListenerId) -> bool;

    fn off_all(&self, event: &str);

    fn emit(&self, event: &str, args: Vec<ThreadSafeArg>);
}

pub trait ThreadSafeAsyncEventEmitter {}

#[derive(Clone)]
pub struct MultiThreadEventEmitter {
    listeners: Arc<Mutex<HashMap<String, Vec<ThreadSafeListener>>>>,
    id_counter: Arc<AtomicU64>,
}

impl MultiThreadEventEmitter {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(HashMap::new())),
            id_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    fn get_id(&self) -> ListenerId {
        self.id_counter.fetch_add(1, Ordering::SeqCst)
    }
}

impl ThreadSafeEventEmitter for MultiThreadEventEmitter {
    fn on<F>(&self, event: &str, callback: F) -> ListenerId
    where
        F: Fn(ThreadSafeArgs) + Send + Sync + 'static,
    {
        let id = self.get_id();
        let listener = ThreadSafeListener {
            id,
            callback: Arc::new(callback),
            once: false,
        };

        let mut listeners = self.listeners.lock().unwrap();
        listeners
            .entry(event.to_string())
            .or_default()
            .push(listener);

        id
    }

    fn once<F>(&self, event: &str, callback: F) -> ListenerId
    where
        F: Fn(ThreadSafeArgs) + Send + Sync + 'static,
    {
        let id = self.get_id();
        let listener = ThreadSafeListener {
            id,
            callback: Arc::new(callback),
            once: true,
        };

        let mut listeners = self.listeners.lock().unwrap();
        listeners
            .entry(event.to_string())
            .or_default()
            .push(listener);

        id
    }

    fn off(&self, event: &str, id: ListenerId) -> bool {
        let mut listeners = self.listeners.lock().unwrap();

        if let Some(ls) = listeners.get_mut(event) {
            let len = ls.len();
            ls.retain(|l| l.id != id);

            return len != ls.len();
        }

        false
    }

    fn off_all(&self, event: &str) {
        self.listeners.lock().unwrap().remove(event);
    }

    fn emit(&self, event: &str, args: Vec<ThreadSafeArg>) {
        let callbacks: Vec<SyncThreadSafeCallback> = {
            let listeners = self.listeners.lock().unwrap();

            if let Some(listener) = listeners.get(event) {
                listener.iter().map(|l| l.callback.clone()).collect()
            } else {
                Vec::new()
            }
        };

        for callback in &callbacks {
            callback(&args);
        }

        if !callbacks.is_empty() {
            let mut listeners = self.listeners.lock().unwrap();

            if let Some(listener) = listeners.get_mut(event) {
                listener.retain(|l| !l.once);
            }
        }
    }
}
