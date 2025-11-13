use std::{any::Any, fmt::Debug, pin::Pin, rc::Rc};

pub type ListenerId = u64;

pub enum Callback {
    Sync(Rc<dyn Fn(&[Box<dyn Any>])>),
    // Async(Rc<dyn Fn(&[Box<dyn Any>]) -> Pin<Box<dyn Future<Output = ()>>>>),
}

pub struct Listener {
    pub id: ListenerId,
    pub callback: Rc<dyn Fn(&[Box<dyn Any>])>,
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
