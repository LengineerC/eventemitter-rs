use event_bus::*;
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
