use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::basis::*;

pub trait EventEmitter {
    fn on<F>(&self, event: &str, callback: F) -> ListenerId
    where
        F: Fn(&[Box<dyn Any>]) + 'static;

    fn once<F>(&self, event: &str, callback: F) -> ListenerId
    where
        F: Fn(&[Box<dyn Any>]) + 'static;

    fn off(&self, event: &str, id: ListenerId) -> bool;

    fn off_all(&self, event: &str);

    fn emit(&self, event: &str, args: Vec<Box<dyn Any>>);
}

pub trait AsyncEventEmitter {}


#[derive(Clone)]
pub struct SingleThreadEventEmitter {
    listeners: Rc<RefCell<HashMap<String, Vec<Listener>>>>,
    id_counter: Rc<RefCell<ListenerId>>,
}

impl SingleThreadEventEmitter {
    pub fn new() -> Self {
        Self {
            listeners: Rc::new(RefCell::new(HashMap::new())),
            id_counter: Rc::new(RefCell::new(1)),
        }
    }

    fn get_id(&self) -> ListenerId {
        let mut id = self.id_counter.borrow_mut();
        let old_value = *id;
        *id += 1;
        old_value
    }
}

impl EventEmitter for SingleThreadEventEmitter {
    fn on<F>(&self, event: &str, callback: F) -> ListenerId
    where
        F: Fn(&[Box<dyn std::any::Any>]) + 'static,
    {
        let id = self.get_id();
        let listener = Listener {
            id,
            callback: Rc::new(callback),
            once: false,
        };

        self.listeners
            .borrow_mut()
            .entry(event.to_string())
            .or_default()
            .push(listener);

        id
    }

    fn once<F>(&self, event: &str, callback: F) -> ListenerId
    where
        F: Fn(&[Box<dyn std::any::Any>]) + 'static,
    {
        let id = self.get_id();
        let listener = Listener {
            id,
            callback: Rc::new(callback),
            once: true,
        };

        self.listeners
            .borrow_mut()
            .entry(event.to_string())
            .or_default()
            .push(listener);

        id
    }

    fn off(&self, event: &str, id: ListenerId) -> bool {
        let mut listeners = self.listeners.borrow_mut();

        if let Some(ls) = listeners.get_mut(event) {
            let len = ls.len();
            ls.retain(|lsnr| lsnr.id != id);

            return len != ls.len();
        }

        false
    }

    fn off_all(&self, event: &str) {
        self.listeners.borrow_mut().remove(event);
    }

    fn emit(&self, event: &str, args: Vec<Box<dyn std::any::Any>>) {
        let callbacks: Vec<Rc<dyn Fn(&[Box<dyn std::any::Any>])>> = {
            let listeners = self.listeners.borrow();
            
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
            let mut listeners = self.listeners.borrow_mut();
            if let Some(listener) = listeners.get_mut(event) {
                listener.retain(|l| !l.once);
            }
        }
    }
}
