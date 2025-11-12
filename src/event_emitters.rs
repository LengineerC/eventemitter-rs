use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::basic_types::*;
use crate::traits::*;

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
}
