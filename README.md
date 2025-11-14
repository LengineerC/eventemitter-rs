# nodevent

A Node.js-style event bus in Rust with synchronous and asynchronous support (via **Tokio**).

## Installation

```toml
# Cargo.toml
[dependencies]
nodevent = { path = "../path_to_your_crate" }
tokio = { version = "1", features = ["full"] }
```

---

## 1. Synchronous Event Handling

```rust
use nodevent::{MultiThreadEventEmitter, args};

fn main() {
    let emitter = MultiThreadEventEmitter::new();

    // Register a synchronous listener
    emitter.on("hello", |args| {
        let msg = args[0].downcast_ref::<String>().unwrap();
        println!("Hello received: {}", msg);
    });

    // Emit an event
    emitter.emit("hello", args!["world"]);
}
```

* `on(event, callback)`: Registers a persistent listener.
* `once(event, callback)`: Registers a listener that will be removed after first call.
* `off(event, id)`: Removes a specific listener by its ID.
* `off_all(event)`: Removes all listeners for an event.

---

## 2. Thread-Safe / Multi-Threaded Event Handling

```rust
use nodevent::{MultiThreadEventEmitter, ts_args};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    let emitter = MultiThreadEventEmitter::new().set_handle(rt.handle().clone());
    let counter = Arc::new(Mutex::new(0));

    let counter_clone = counter.clone();
    emitter.on("increment", move |args| {
        let n = args[0].downcast_ref::<i32>().unwrap();
        *counter_clone.lock().unwrap() += n;
    });

    emitter.emit("increment", ts_args![5]);
    println!("Counter: {}", counter.lock().unwrap());
}
```

* Use `ts_args!` to pass arguments in a thread-safe way (`Arc<Vec<Box<dyn Any + Send + Sync>>>`).
* Multi-threaded emitters require a `tokio::runtime::Handle` to run async tasks.

---

## 3. Asynchronous Event Handling

```rust
use nodevent::{MultiThreadEventEmitter, ts_args};
use tokio::runtime::Runtime;

async fn async_test(n: i32) {
    println!("Async event received: {}", n);
}

fn main() {
    let rt = Runtime::new().unwrap();
    let emitter = MultiThreadEventEmitter::new().set_handle(rt.handle().clone());

    // Register an async listener
    emitter.on_async("async_event", |args| {
        let n: i32 = *args[0].downcast_ref().unwrap();
        Box::pin(async move {
            async_test(n).await;
        })
    });

    // Register a once-only async listener
    emitter.once_async("async_once", |args| {
        let n: i32 = *args[0].downcast_ref().unwrap();
        Box::pin(async move {
            async_test(n).await;
            println!("Async once done: {}", n);
        })
    });

    emitter.emit("async_event", ts_args![42]);
    emitter.emit("async_once", ts_args![100]);

    // Wait for async tasks to finish
    rt.block_on(async {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    });
}
```

* `on_async` / `once_async` require closures returning `Pin<Box<dyn Future<Output = ()> + Send + Sync>>`.
* Use `Box::pin(async move { ... })` inside the closure.

---

## 4. Macros

```rust
use nodevent::{args, ts_args};

// Synchronous args (single-threaded)
let sync_args = args![1, "hello", true];

// Thread-safe args (multi-threaded)
let safe_args = ts_args![42, "world", false];
```

* `args!`: Wraps values into `Box<dyn Any>` inside `Rc<Vec<_>>`.
* `ts_args!`: Wraps values into `Box<dyn Any + Send + Sync>` inside `Arc<Vec<_>>`.

---

## 5. Summary

| Feature               | Macro / Method            | Thread Safety         |
| --------------------- | ------------------------- | --------------------- |
| Synchronous listener  | `on` / `once`             | Single-thread         |
| Asynchronous listener | `on_async` / `once_async` | Multi-thread          |
| Emit event            | `emit`                    | Single/Multi-thread   |
| Arguments             | `args!` / `ts_args!`      | Single / Multi-thread |

This crate provides a flexible Node.js-style event bus in Rust, with full async support via **Tokio** and optional multi-threaded safety.
