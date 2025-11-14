use std::{any::Any, pin::Pin, rc::Rc, sync::Arc};

pub type HandlerId = u64;

pub type Arg = Box<dyn Any>;
pub type ThreadSafeArg = Box<dyn Any + Send + Sync>;

pub type Args<'a> = &'a [Arg];
pub type ThreadSafeArgs<'a> = &'a [ThreadSafeArg];

pub type SyncCallback = Rc<dyn Fn(Args)>;
pub type SyncThreadSafeCallback = Arc<dyn Fn(ThreadSafeArgs) + Send + Sync>;
pub type AsyncCallback = Rc<dyn Fn(Args) -> Pin<Box<dyn Future<Output = ()>>>>;
pub type AsyncThreadSafeCallback =
    Arc<dyn Fn(ThreadSafeArgs) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync>;
