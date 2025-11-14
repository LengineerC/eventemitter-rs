use crate::event_emitters::{MultiThreadEventEmitter, ThreadSafeEventEmitter};
use event_bus::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::task;

#[test]
fn test_basic_emit_and_on() {
    let emitter = MultiThreadEventEmitter::new();
    let result = Arc::new(Mutex::new(None));

    let result_clone = result.clone();
    emitter.on("hello", move |args| {
        let msg = args[0].downcast_ref::<String>().unwrap();
        *result_clone.lock().unwrap() = Some(msg.clone());
    });

    // emitter.emit("hello", Arc::new(vec![Box::new("world".to_string())]));
    emitter.emit("hello", ts_args!["world".to_string()]);

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

    emitter.emit("ping", Arc::new(vec![]));
    emitter.emit("ping", Arc::new(vec![])); // 第二次 emit 不应触发

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

    emitter.emit("count", Arc::new(vec![]));
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
    emitter.emit("count", Arc::new(vec![]));
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
                e.emit("add", Arc::new(vec![Box::new(i), Box::new(1)]));
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
        Arc::new(vec![
            Box::new("Hello".to_string()),
            Box::new("Rust".to_string()),
        ]),
    );

    assert_eq!(&*output.lock().unwrap(), "Hello Rust");
}

#[test]
fn test_no_panic_when_event_not_found() {
    let emitter = MultiThreadEventEmitter::new();
    emitter.emit("non_existent_event", Arc::new(vec![]));
    // just ensure it doesn't panic
    assert!(true);
}

#[tokio::test]
async fn test_sync_on_and_emit() {
    let emitter = MultiThreadEventEmitter::new().set_handle(tokio::runtime::Handle::current());
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    emitter.on("sync_event", move |_args| {
        let mut val = counter_clone.lock().unwrap();
        *val += 1;
    });

    emitter.emit("sync_event", Arc::new(vec![]));
    emitter.emit("sync_event", Arc::new(vec![]));

    assert_eq!(*counter.lock().unwrap(), 2);
}

#[tokio::test]
async fn test_async_on_and_emit() {
    let emitter = MultiThreadEventEmitter::new().set_handle(tokio::runtime::Handle::current());
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    emitter.on_async("async_event", move |_args| {
        let counter_clone = counter_clone.clone();
        Box::pin(async move {
            let mut val = counter_clone.lock().unwrap();
            *val += 1;
        })
    });

    emitter.emit("async_event", Arc::new(vec![]));
    // 等待所有异步任务执行
    task::yield_now().await;

    assert_eq!(*counter.lock().unwrap(), 1);
}

#[tokio::test]
async fn test_sync_and_async_mixed() {
    let emitter = MultiThreadEventEmitter::new().set_handle(tokio::runtime::Handle::current());
    let sync_counter = Arc::new(Mutex::new(0));
    let async_counter = Arc::new(Mutex::new(0));

    let sync_clone = sync_counter.clone();
    emitter.on("mixed_event", move |_args| {
        let mut val = sync_clone.lock().unwrap();
        *val += 1;
    });

    let async_clone = async_counter.clone();
    emitter.on_async("mixed_event", move |_args| {
        let async_clone = async_clone.clone();
        Box::pin(async move {
            let mut val = async_clone.lock().unwrap();
            *val += 1;
        })
    });

    emitter.emit("mixed_event", Arc::new(vec![]));
    task::yield_now().await;

    assert_eq!(*sync_counter.lock().unwrap(), 1);
    assert_eq!(*async_counter.lock().unwrap(), 1);
}

#[tokio::test]
async fn test_once_sync() {
    let emitter = MultiThreadEventEmitter::new().set_handle(tokio::runtime::Handle::current());
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    emitter.once("once_event", move |_args| {
        let mut val = counter_clone.lock().unwrap();
        *val += 1;
    });

    emitter.emit("once_event", Arc::new(vec![]));
    emitter.emit("once_event", Arc::new(vec![]));

    assert_eq!(*counter.lock().unwrap(), 1);
}

#[tokio::test]
async fn test_once_async() {
    let emitter = MultiThreadEventEmitter::new().set_handle(tokio::runtime::Handle::current());
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    emitter.once_async("once_async_event", move |_args| {
        let counter_clone = counter_clone.clone();
        Box::pin(async move {
            let mut val = counter_clone.lock().unwrap();
            *val += 1;
        })
    });

    emitter.emit("once_async_event", Arc::new(vec![]));
    task::yield_now().await;

    emitter.emit("once_async_event", Arc::new(vec![]));
    task::yield_now().await;

    assert_eq!(*counter.lock().unwrap(), 1);
}

#[tokio::test]
async fn test_off_and_off_all() {
    let emitter = MultiThreadEventEmitter::new().set_handle(tokio::runtime::Handle::current());
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    let id = emitter.on("remove_event", move |_args| {
        let mut val = counter_clone.lock().unwrap();
        *val += 1;
    });

    emitter.emit("remove_event", Arc::new(vec![]));
    assert_eq!(*counter.lock().unwrap(), 1);

    assert!(emitter.off("remove_event", id));
    emitter.emit("remove_event", Arc::new(vec![]));
    assert_eq!(*counter.lock().unwrap(), 1); // 没增加

    let counter2 = Arc::new(Mutex::new(0));
    let counter2_clone = counter2.clone();
    emitter.on("multi_event", move |_args| {
        let mut val = counter2_clone.lock().unwrap();
        *val += 1;
    });
    emitter.off_all("multi_event");
    emitter.emit("multi_event", Arc::new(vec![]));
    assert_eq!(*counter2.lock().unwrap(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn multithread_event_test() {
    let emitter =
        Arc::new(MultiThreadEventEmitter::new().set_handle(tokio::runtime::Handle::current()));

    let counter = Arc::new(tokio::sync::Mutex::new(0usize));

    // -------- 注册同步回调 --------
    emitter.on("e1", |_| {
        // println!("[sync] received");
    });

    emitter.on("e1", |_| {
        // println!("[sync2] received");
    });

    // -------- 注册异步回调 --------
    {
        let counter = counter.clone();
        emitter.on_async("e1", move |_| {
            let counter = counter.clone();
            Box::pin(async move {
                tokio::time::sleep(Duration::from_millis(5)).await;
                let mut lock = counter.lock().await;
                *lock += 1;
            })
        });
    }

    {
        let counter = counter.clone();
        emitter.on_async("e1", move |_| {
            let counter = counter.clone();
            Box::pin(async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                let mut lock = counter.lock().await;
                *lock += 1;
            })
        });
    }

    // -------- 开启 10 个线程并发 emit --------
    let mut handles = vec![];

    for _ in 0..10 {
        let emitter = emitter.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                emitter.emit("e1", Arc::new(vec![]));
            }
        }));
    }

    // 等待所有线程结束
    for h in handles {
        h.join().unwrap();
    }

    // 等待 async handler 完成（最多等待 3 秒）
    tokio::time::sleep(Duration::from_secs(3)).await;

    // -------- 验证 async handler 总运行次数 --------
    let result = *counter.lock().await;

    // 每次 emit 会触发 2 个 async handler
    let expected = 10 * 100 * 2;

    assert_eq!(
        result, expected,
        "Async callbacks incomplete: expected {}, got {}",
        expected, result
    );
}
