pub mod basic_types;
pub mod event_emitters;
pub mod traits;

#[cfg(test)]
mod single_thread_tests {
    use crate::event_emitters::SingleThreadEventEmitter;
    use crate::traits::EventEmitter;
    use std::any::Any;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_basic_on_and_emit() {
        let emitter = SingleThreadEventEmitter::new();
        let call_count = Rc::new(RefCell::new(0));

        let call_count_clone = call_count.clone();
        emitter.on("test", move |_args| {
            *call_count_clone.borrow_mut() += 1;
        });

        emitter.emit("test", vec![]);
        emitter.emit("test", vec![]);

        assert_eq!(*call_count.borrow(), 2);
    }

    #[test]
    fn test_event_arguments() {
        let emitter = SingleThreadEventEmitter::new();
        let received_args = Rc::new(RefCell::new(Vec::<String>::new()));

        let received_args_clone = received_args.clone();
        emitter.on("data", move |args| {
            for arg in args {
                if let Some(s) = arg.downcast_ref::<String>() {
                    received_args_clone.borrow_mut().push(s.clone());
                }
            }
        });

        let args = vec![
            Box::new("hello".to_string()) as Box<dyn Any>,
            Box::new("world".to_string()) as Box<dyn Any>,
        ];

        emitter.emit("data", args);

        assert_eq!(*received_args.borrow(), vec!["hello", "world"]);
    }

    #[test]
    fn test_once_listener() {
        let emitter = SingleThreadEventEmitter::new();
        let call_count = Rc::new(RefCell::new(0));

        let call_count_clone = call_count.clone();
        emitter.once("once_test", move |_args| {
            *call_count_clone.borrow_mut() += 1;
        });

        emitter.emit("once_test", vec![]);
        emitter.emit("once_test", vec![]); // 第二次应该不会触发

        assert_eq!(*call_count.borrow(), 1);
    }

    #[test]
    fn test_off_removal() {
        let emitter = SingleThreadEventEmitter::new();
        let call_count = Rc::new(RefCell::new(0));

        let call_count_clone = call_count.clone();
        let id = emitter.on("removable", move |_args| {
            *call_count_clone.borrow_mut() += 1;
        });

        emitter.emit("removable", vec![]);
        assert_eq!(*call_count.borrow(), 1);

        // 移除监听器
        assert!(emitter.off("removable", id));

        emitter.emit("removable", vec![]);
        assert_eq!(*call_count.borrow(), 1); // 应该还是1，没有增加
    }

    #[test]
    fn test_off_nonexistent() {
        let emitter = SingleThreadEventEmitter::new();

        // 尝试移除不存在的监听器
        assert!(!emitter.off("nonexistent", 999));
    }

    #[test]
    fn test_off_all() {
        let emitter = SingleThreadEventEmitter::new();
        let call_count1 = Rc::new(RefCell::new(0));
        let call_count2 = Rc::new(RefCell::new(0));

        let count1_clone = call_count1.clone();
        let count2_clone = call_count2.clone();

        emitter.on("multi", move |_args| {
            *count1_clone.borrow_mut() += 1;
        });

        emitter.on("multi", move |_args| {
            *count2_clone.borrow_mut() += 1;
        });

        emitter.emit("multi", vec![]);
        assert_eq!(*call_count1.borrow(), 1);
        assert_eq!(*call_count2.borrow(), 1);

        // 移除所有监听器
        emitter.off_all("multi");

        emitter.emit("multi", vec![]);
        assert_eq!(*call_count1.borrow(), 1); // 没有增加
        assert_eq!(*call_count2.borrow(), 1); // 没有增加
    }

    #[test]
    fn test_multiple_events() {
        let emitter = SingleThreadEventEmitter::new();
        let event1_count = Rc::new(RefCell::new(0));
        let event2_count = Rc::new(RefCell::new(0));

        let count1_clone = event1_count.clone();
        let count2_clone = event2_count.clone();

        emitter.on("event1", move |_args| {
            *count1_clone.borrow_mut() += 1;
        });

        emitter.on("event2", move |_args| {
            *count2_clone.borrow_mut() += 1;
        });

        emitter.emit("event1", vec![]);
        emitter.emit("event2", vec![]);
        emitter.emit("event1", vec![]);

        assert_eq!(*event1_count.borrow(), 2);
        assert_eq!(*event2_count.borrow(), 1);
    }

    #[test]
    fn test_listener_id_uniqueness() {
        let emitter = SingleThreadEventEmitter::new();

        let id1 = emitter.on("test", |_args| {});
        let id2 = emitter.on("test", |_args| {});
        let id3 = emitter.once("test", |_args| {});

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_empty_args() {
        let emitter = SingleThreadEventEmitter::new();
        let called = Rc::new(RefCell::new(false));

        let called_clone = called.clone();
        emitter.on("empty", move |args| {
            assert!(args.is_empty());
            *called_clone.borrow_mut() = true;
        });

        emitter.emit("empty", vec![]);
        assert!(*called.borrow());
    }

    #[test]
    fn test_nonexistent_event_emit() {
        let emitter = SingleThreadEventEmitter::new();

        emitter.emit("nonexistent", vec![]);
    }

    #[test]
    fn test_multiple_data_types() {
        let emitter = SingleThreadEventEmitter::new();
        let results = Rc::new(RefCell::new(Vec::<String>::new()));

        let results_clone = results.clone();
        emitter.on("mixed", move |args| {
            for arg in args {
                if let Some(num) = arg.downcast_ref::<i32>() {
                    results_clone.borrow_mut().push(format!("i32: {}", num));
                } else if let Some(b) = arg.downcast_ref::<bool>() {
                    results_clone.borrow_mut().push(format!("bool: {}", b));
                } else if let Some(s) = arg.downcast_ref::<&str>() {
                    results_clone.borrow_mut().push(format!("str: {}", s));
                }
            }
        });

        let args = vec![
            Box::new(42) as Box<dyn Any>,
            Box::new(true) as Box<dyn Any>,
            Box::new("hello") as Box<dyn Any>,
        ];

        emitter.emit("mixed", args);

        let expected = vec!["i32: 42", "bool: true", "str: hello"];
        assert_eq!(*results.borrow(), expected);
    }
}
