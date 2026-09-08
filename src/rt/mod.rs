//! Runtime components
//!
//! This module provides traits and types that allow hyper to be runtime-agnostic.
//! By abstracting over async runtimes, hyper can work with different executors, timers, and IO transports.
//!
//! The main components in this module are:
//!
//! - **Executors**: Traits for spawning and running futures, enabling integration with any async runtime.
//! - **Timers**: Abstractions for sleeping and scheduling tasks, allowing time-based operations to be runtime-independent.
//! - **IO Transports**: Traits for asynchronous reading and writing, so hyper can work with various IO backends.
//!
//! By implementing these traits, you can customize how hyper interacts with your chosen runtime environment.
//!
//! To learn more, [check out the runtime guide](https://hyper.rs/guides/1/init/runtime/).

pub mod bounds;
mod io;
mod timer;

pub use self::io::{Read, ReadBuf, ReadBufCursor, Write};
pub use self::timer::{Sleep, Timer};

/// Receives a one-way notification that an HTTP/2 connection should stop being reused.
///
/// Implementations must return promptly and must not block or panic. Hyper calls
/// the observer at most once per connection, outside its internal PING lock.
/// The observer must not retain the connection or its request sender.
/// Notification does not close the connection or cancel existing streams.
pub trait KeepAliveObserver: std::fmt::Debug + Send + Sync {
    /// The keep-alive PING exceeded the configured reuse timeout without an ACK.
    fn on_reuse_timeout(&self);
}

/// An executor of futures.
///
/// This trait allows Hyper to abstract over async runtimes. Implement this trait for your own type.
///
/// # Example
///
/// ```
/// # use hyper::rt::Executor;
/// # use std::future::Future;
/// #[derive(Clone)]
/// struct TokioExecutor;
///
/// impl<F> Executor<F> for TokioExecutor
/// where
///     F: Future + Send + 'static,
///     F::Output: Send + 'static,
/// {
///     fn execute(&self, future: F) {
///         tokio::spawn(future);
///     }
/// }
/// ```
pub trait Executor<Fut> {
    /// Place the future into the executor to be run.
    fn execute(&self, fut: Fut);
}
