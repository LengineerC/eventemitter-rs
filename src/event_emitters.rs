use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::basic_types::*;
use crate::traits::*;

#[derive(Clone)]
pub struct SingleThreadEventEmitter {
    listeners: Rc<RefCell<HashMap<String, Vec<Listener>>>>,
    id_counter: Rc<RefCell<ListenerId>>,
}

impl SingleThreadEventEmitter {
    pub fn new() -> Self {
        Self {
            listeners: Rc::new(RefCell::new(HashMap::new())),
            id_counter: Rc::new(RefCell::new(0)),
        }
    }

    fn get_id(&self) -> ListenerId {
        let mut cur_id = self.id_counter.borrow_mut();
        *cur_id += 1;
        *cur_id
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
        // let listener_binding = self.listeners.borrow();
        // let listeners_opt = listener_binding.get(event).clone();

        // if let Some(listeners) = listeners_opt {
        //     for l in listeners {
        //         (l.callback)(&args);
        //     }
        // }

        // if let Some(listeners) = self.listeners.borrow_mut().get_mut(event) {
        //     listeners.retain(|l| !l.once);
        // }

        let callbacks: Vec<Rc<dyn Fn(&[Box<dyn std::any::Any>])>> = {
            let listeners = self.listeners.borrow();
            if let Some(listeners) = listeners.get(event) {
                listeners.iter().map(|l| l.callback.clone()).collect()
            } else {
                Vec::new()
            }
        };

        for callback in &callbacks {
            callback(&args);
        }

        if !callbacks.is_empty() {
            let mut listeners = self.listeners.borrow_mut();
            if let Some(listeners) = listeners.get_mut(event) {
                listeners.retain(|l| !l.once);
            }
        }
    }
}
