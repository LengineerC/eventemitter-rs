pub mod basis;
pub mod event_emitters;

#[cfg(test)]
mod single_thread_tests {
    use super::*;
    use crate::event_emitters::*;
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

#[cfg(test)]
mod multi_thread_tests {
    use super::*;
    use crate::event_emitters::{MultiThreadEventEmitter, ThreadSafeEventEmitter};
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_basic_emit_and_on() {
        let emitter = MultiThreadEventEmitter::new();
        let result = Arc::new(Mutex::new(None));

        let result_clone = result.clone();
        emitter.on("hello", move |args| {
            let msg = args[0].downcast_ref::<String>().unwrap();
            *result_clone.lock().unwrap() = Some(msg.clone());
        });

        emitter.emit("hello", vec![Box::new("world".to_string())]);

        assert_eq!(result.lock().unwrap().clone(), Some("world".to_string()));
    }

    #[test]
    fn test_once_listener_removed_after_first_emit() {
        let emitter = MultiThreadEventEmitter::new();
        let counter = Arc::new(Mutex::new(0));

        let counter_clone = counter.clone();
        emitter.once("ping", move |_| {
            *counter_clone.lock().unwrap() += 1;
        });

        emitter.emit("ping", vec![]);
        emitter.emit("ping", vec![]); // 第二次 emit 不应触发

        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_off_removes_specific_listener() {
        let emitter = MultiThreadEventEmitter::new();
        let counter = Arc::new(Mutex::new(0));

        let counter_clone = counter.clone();
        let id1 = emitter.on("count", move |_| {
            *counter_clone.lock().unwrap() += 1;
        });

        let counter_clone2 = counter.clone();
        emitter.on("count", move |_| {
            *counter_clone2.lock().unwrap() += 10;
        });

        emitter.off("count", id1);

        emitter.emit("count", vec![]);
        assert_eq!(*counter.lock().unwrap(), 10); // 只触发了第二个 listener
    }

    #[test]
    fn test_off_all_removes_all_listeners() {
        let emitter = MultiThreadEventEmitter::new();
        let counter = Arc::new(Mutex::new(0));

        let counter_clone = counter.clone();
        emitter.on("count", move |_| {
            *counter_clone.lock().unwrap() += 1;
        });

        emitter.off_all("count");
        emitter.emit("count", vec![]);
        assert_eq!(*counter.lock().unwrap(), 0);
    }

    #[test]
    fn test_thread_safety_with_multiple_threads() {
        let emitter = Arc::new(MultiThreadEventEmitter::new());
        let sum = Arc::new(Mutex::new(0));

        let sum_clone = sum.clone();
        emitter.on("add", move |args| {
            let a = args[0].downcast_ref::<i32>().unwrap();
            let b = args[1].downcast_ref::<i32>().unwrap();
            *sum_clone.lock().unwrap() += a + b;
        });

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let e = emitter.clone();
                thread::spawn(move || {
                    e.emit("add", vec![Box::new(i), Box::new(1)]);
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        // 0+1 + 1+1 + 2+1 + ... + 9+1 = 55
        assert_eq!(*sum.lock().unwrap(), 55);
    }

    #[test]
    fn test_emit_with_multiple_args_types() {
        let emitter = MultiThreadEventEmitter::new();
        let output = Arc::new(Mutex::new(String::new()));

        let out_clone = output.clone();
        emitter.on("concat", move |args| {
            let a = args[0].downcast_ref::<String>().unwrap();
            let b = args[1].downcast_ref::<String>().unwrap();
            *out_clone.lock().unwrap() = format!("{a} {b}");
        });

        emitter.emit(
            "concat",
            vec![Box::new("Hello".to_string()), Box::new("Rust".to_string())],
        );

        assert_eq!(&*output.lock().unwrap(), "Hello Rust");
    }

    #[test]
    fn test_no_panic_when_event_not_found() {
        let emitter = MultiThreadEventEmitter::new();
        emitter.emit("non_existent_event", vec![]);
        // just ensure it doesn't panic
        assert!(true);
    }
}
