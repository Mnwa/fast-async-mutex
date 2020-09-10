//! `fast_async_mutex` it is a lib which provide asynchronous locking mechanisms, which used spinlock algorithm.
//! It's maybe very efficient because when mutex tries to acquire data unsuccessfully, these returning control to an async runtime back.
//! This lib built only on atomics and don't use others std synchronous data structures, which make this lib so fast.

/// The fast async mutex which uses spinlock algorithm with using waker
pub mod mutex;
/// The fast async mutex which uses spinlock algorithm with using waker
/// This realisation will check an order of mutex acquiring.
pub mod mutex_ordered;
/// RwLock realisation which uses spinlock algorithm with using waker
pub mod rwlock;
/// RwLock realisation which uses spinlock algorithm with using waker
/// This realisation will check an order of mutex acquiring.
pub mod rwlock_ordered;

pub(crate) mod utils;
