use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use crate::basis::*;
use crate::types::*;

pub trait ThreadSafeEventEmitter: Send + Sync {
    fn on<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(ThreadSafeArgs) + Send + Sync + 'static;

    fn once<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(ThreadSafeArgs) + Send + Sync + 'static;

    fn off(&self, event: &str, id: HandlerId) -> bool;

    fn off_all(&self, event: &str);

    fn emit(&self, event: &str, args: Arc<Vec<ThreadSafeArg>>);
}

pub trait ThreadSafeAsyncEventEmitter {
    fn on_async<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(ThreadSafeArgs) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>>
            + Send
            + Sync
            + 'static;

    fn once_async<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(ThreadSafeArgs) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>>
            + Send
            + Sync
            + 'static;
}

#[derive(Clone)]
pub struct MultiThreadEventEmitter {
    listeners: Arc<Mutex<HashMap<String, Vec<ThreadSafeHandler>>>>,
    id_counter: Arc<AtomicU64>,
}

impl MultiThreadEventEmitter {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(HashMap::new())),
            id_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    fn get_id(&self) -> HandlerId {
        self.id_counter.fetch_add(1, Ordering::SeqCst)
    }
}

impl ThreadSafeEventEmitter for MultiThreadEventEmitter {
    fn on<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(ThreadSafeArgs) + Send + Sync + 'static,
    {
        let id = self.get_id();
        let handler = ThreadSafeHandler {
            id,
            callback: ThreadSafeCallback::Sync(Arc::new(callback)),
            once: false,
        };

        let mut listeners = self.listeners.lock().unwrap();
        listeners
            .entry(event.to_string())
            .or_default()
            .push(handler);

        id
    }

    fn once<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(ThreadSafeArgs) + Send + Sync + 'static,
    {
        let id = self.get_id();
        let handler = ThreadSafeHandler {
            id,
            callback: ThreadSafeCallback::Sync(Arc::new(callback)),
            once: true,
        };

        let mut listeners = self.listeners.lock().unwrap();
        listeners
            .entry(event.to_string())
            .or_default()
            .push(handler);

        id
    }

    fn off(&self, event: &str, id: HandlerId) -> bool {
        let mut listeners = self.listeners.lock().unwrap();

        if let Some(handlers) = listeners.get_mut(event) {
            let len = handlers.len();
            handlers.retain(|h| h.id != id);

            return len != handlers.len();
        }

        false
    }

    fn off_all(&self, event: &str) {
        self.listeners.lock().unwrap().remove(event);
    }

    fn emit(&self, event: &str, args: Arc<Vec<ThreadSafeArg>>) {
        let callbacks: Vec<ThreadSafeCallback> = self
            .listeners
            .lock()
            .unwrap()
            .get(event)
            .map(|handlers| handlers.iter().map(|h| h.callback.clone()).collect())
            .unwrap_or_default();

        for callback in &callbacks {
            if let ThreadSafeCallback::Sync(cb) = callback {
                cb(&args);
            }
        }

        for callback in callbacks {
            if let ThreadSafeCallback::Async(cb) = callback {
                let args_clone = args.clone();
                tokio::spawn(async move {
                    cb(&args_clone).await;
                });
            }
        }

        let mut listeners = self.listeners.lock().unwrap();
        let handlers_opt = listeners.get_mut(event);
        if let Some(handlers) = handlers_opt {
            handlers.retain(|h| !h.once);
        }
    }
}

impl ThreadSafeAsyncEventEmitter for MultiThreadEventEmitter {
    fn on_async<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(ThreadSafeArgs) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>>
            + Send
            + Sync
            + 'static,
    {
        let id = self.get_id();
        let handler = ThreadSafeHandler {
            id,
            callback: ThreadSafeCallback::Async(Arc::new(callback)),
            once: false,
        };

        self.listeners
            .lock()
            .unwrap()
            .entry(event.to_string())
            .or_default()
            .push(handler);

        id
    }

    fn once_async<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(ThreadSafeArgs) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>>
            + Send
            + Sync
            + 'static,
    {
        let id = self.get_id();
        let handler = ThreadSafeHandler {
            id,
            callback: ThreadSafeCallback::Async(Arc::new(callback)),
            once: true,
        };

        self.listeners
            .lock()
            .unwrap()
            .entry(event.to_string())
            .or_default()
            .push(handler);

        id
    }
}
