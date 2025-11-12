use crate::basic_types::*;
use std::any::Any;

pub trait EventEmitter {
    fn on<F>(&self, event: &str, callback: F) -> ListenerId
    where
        F: Fn(&[Box<dyn Any>]) + 'static;

    fn once<F>(&self, event: &str, callback: F) -> ListenerId
    where
        F: Fn(&[Box<dyn Any>]) + Send + Sync + 'static;

    fn off(&self, event: &str, id: ListenerId) -> bool;

    fn off_all(&self, event: &str);

    fn emit(&self, event: &str, data: Vec<Box<dyn Any>>);
}

pub trait AsyncEventEmitter {}
