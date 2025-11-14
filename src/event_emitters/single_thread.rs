use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;

use crate::basis::*;
use crate::types::*;

pub trait EventEmitter {
    fn on<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(Args) + 'static;

    fn once<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(Args) + 'static;

    fn off(&self, event: &str, id: HandlerId) -> bool;

    fn off_all(&self, event: &str);

    fn emit(&self, event: &str, args: Rc<Vec<Arg>>);
}

pub trait AsyncEventEmitter {
    fn on_async<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(Args) -> Pin<Box<dyn Future<Output = ()>>> + 'static;

    fn once_async<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(Args) -> Pin<Box<dyn Future<Output = ()>>> + 'static;
}

#[derive(Clone)]
pub struct SingleThreadEventEmitter {
    listeners: Rc<RefCell<HashMap<String, Vec<Handler>>>>,
    id_counter: Rc<RefCell<HandlerId>>,
}

impl SingleThreadEventEmitter {
    pub fn new() -> Self {
        Self {
            listeners: Rc::new(RefCell::new(HashMap::new())),
            id_counter: Rc::new(RefCell::new(1)),
        }
    }

    fn get_id(&self) -> HandlerId {
        let mut id = self.id_counter.borrow_mut();
        let old_value = *id;
        *id += 1;
        old_value
    }
}

impl EventEmitter for SingleThreadEventEmitter {
    fn on<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(Args) + 'static,
    {
        let id = self.get_id();
        let handler = Handler {
            id,
            callback: Callback::Sync(Rc::new(callback)),
            once: false,
        };

        self.listeners
            .borrow_mut()
            .entry(event.to_string())
            .or_default()
            .push(handler);

        id
    }

    fn once<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(Args) + 'static,
    {
        let id = self.get_id();
        let handler = Handler {
            id,
            callback: Callback::Sync(Rc::new(callback)),
            once: true,
        };

        self.listeners
            .borrow_mut()
            .entry(event.to_string())
            .or_default()
            .push(handler);

        id
    }

    fn off(&self, event: &str, id: HandlerId) -> bool {
        let mut listeners = self.listeners.borrow_mut();

        if let Some(handlers) = listeners.get_mut(event) {
            let len = handlers.len();
            handlers.retain(|h| h.id != id);

            return len != handlers.len();
        }

        false
    }

    fn off_all(&self, event: &str) {
        self.listeners.borrow_mut().remove(event);
    }

    fn emit(&self, event: &str, args: Rc<Vec<Arg>>) {
        let callbacks: Vec<Callback> = self
            .listeners
            .borrow()
            .get(event)
            .map(|handlers| handlers.iter().map(|h| h.callback.clone()).collect())
            .unwrap_or_default();

        for callback in &callbacks {
            if let Callback::Sync(cb) = callback {
                cb(args.clone());
            }
        }

        for callback in callbacks {
            if let Callback::Async(cb) = callback {
                let args_clone = args.clone();
                tokio::task::spawn_local(async move {
                    cb(args_clone).await;
                });
            }
        }

        let mut listeners = self.listeners.borrow_mut();
        let handlers_opt = listeners.get_mut(event);
        if let Some(handlers) = handlers_opt {
            handlers.retain(|h| !h.once);
        }
    }
}

impl AsyncEventEmitter for SingleThreadEventEmitter {
    fn on_async<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(Args) -> Pin<Box<dyn Future<Output = ()>>> + 'static,
    {
        let id = self.get_id();
        let handler = Handler {
            id,
            callback: Callback::Async(Rc::new(callback)),
            once: false,
        };

        self.listeners
            .borrow_mut()
            .entry(event.to_string())
            .or_default()
            .push(handler);

        id
    }

    fn once_async<F>(&self, event: &str, callback: F) -> HandlerId
    where
        F: Fn(Args) -> Pin<Box<dyn Future<Output = ()>>> + 'static,
    {
        let id = self.get_id();
        let handler = Handler {
            id,
            callback: Callback::Async(Rc::new(callback)),
            once: true,
        };

        self.listeners
            .borrow_mut()
            .entry(event.to_string())
            .or_default()
            .push(handler);

        id
    }
}
