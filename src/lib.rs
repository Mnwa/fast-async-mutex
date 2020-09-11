//! `fast_async_mutex` it is a lib which provide asynchronous locking mechanisms, which used spinlock algorithm.
//! It's maybe very efficient because when mutex tries to acquire data unsuccessfully, these returning control to an async runtime back.
//! This lib built only on atomics and don't use others std synchronous data structures, which make this lib so fast.

/// The simple Mutex, which will provide unique access to you data between multiple threads/futures.
pub mod mutex;

/// The Ordered Mutex has its mechanism of locking order when you have concurrent access to data.
/// It will work well when you needed step by step data locking like sending UDP packages in a specific order.
pub mod mutex_ordered;

/// The RW Lock mechanism accepts you get concurrent shared access to your data without waiting.
/// And get unique access with locks like a Mutex.
pub mod rwlock;

/// The RW Lock mechanism accepts you get shared access to your data without locking.
/// The Ordered RW Lock will be locking all reads, which starting after write and unlocking them only when write will realize.
/// It may be slow down the reads speed, but decrease time to write on systems, where it is critical.
///
/// **BUT RW Lock has some limitations. You should avoid acquiring the second reading before realizing first inside the one future.
/// Because it can happen that between your readings a write from another thread will acquire the mutex, and you will get a deadlock.**
pub mod rwlock_ordered;

pub(crate) mod inner;
pub(crate) mod utils;
