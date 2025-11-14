/// let params = args![1, "hello", true];
#[macro_export]
macro_rules! args {
    ($($v:expr),* $(,)?) => {
        std::rc::Rc::new(vec![$(Box::new($v) as Box<dyn std::any::Any>),*])
    };
}

/// let params = ts_args![1, "hello", true];
#[macro_export]
macro_rules! ts_args {
    ($($v:expr),* $(,)?) => {
        std::sync::Arc::new(vec![$(Box::new($v) as Box<dyn std::any::Any + Send + Sync>),*])
    };
}
