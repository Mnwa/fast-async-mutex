use crate::inner::Inner;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

/// An async `ordered` RwLock.
/// It will be work with any async runtime in `Rust`, it may be a `tokio`, `smol`, `async-std`, etc..
#[derive(Debug)]
pub struct RwLock<T: ?Sized> {
    readers: AtomicUsize,
    inner: Inner<T>,
}

impl<T> RwLock<T> {
    /// Create a new `UnorderedRWLock`
    #[inline]
    pub const fn new(data: T) -> RwLock<T> {
        RwLock {
            readers: AtomicUsize::new(0),
            inner: Inner::new(data),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Acquires the mutex for are write.
    ///
    /// Returns a guard that releases the mutex and wake the next locker when it will be dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use fast_async_mutex::rwlock::RwLock;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mutex = RwLock::new(10);
    ///     let mut guard = mutex.write().await;
    ///     *guard += 1;
    ///     assert_eq!(*guard, 11);
    /// }
    /// ```
    #[inline]
    pub fn write(&self) -> RwLockWriteGuardFuture<T> {
        RwLockWriteGuardFuture {
            mutex: &self,
            is_realized: false,
        }
    }

    /// Acquires the mutex for are write.
    ///
    /// Returns a guard that releases the mutex and wake the next locker when it will be dropped.
    /// `WriteLockOwnedGuard` have a `'static` lifetime, but requires the `Arc<RWLock<T>>` type
    ///
    /// # Examples
    ///
    /// ```
    /// use fast_async_mutex::rwlock::RwLock;
    /// use std::sync::Arc;
    /// #[tokio::main]
    /// async fn main() {
    ///     let mutex = Arc::new(RwLock::new(10));
    ///     let mut guard = mutex.write_owned().await;
    ///     *guard += 1;
    ///     assert_eq!(*guard, 11);
    /// }
    /// ```
    #[inline]
    pub fn write_owned(self: &Arc<Self>) -> RwLockWriteOwnedGuardFuture<T> {
        RwLockWriteOwnedGuardFuture {
            mutex: self.clone(),
            is_realized: false,
        }
    }

    /// Acquires the mutex for are read.
    ///
    /// Returns a guard that releases the mutex and wake the next locker when it will be dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use fast_async_mutex::rwlock::RwLock;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mutex = RwLock::new(10);
    ///     let guard = mutex.read().await;
    ///     let guard2 = mutex.read().await;
    ///     assert_eq!(*guard, *guard2);
    /// }
    /// ```
    #[inline]
    pub fn read(&self) -> RwLockReadGuardFuture<T> {
        RwLockReadGuardFuture {
            mutex: &self,
            is_realized: false,
        }
    }

    /// Acquires the mutex for are write.
    ///
    /// Returns a guard that releases the mutex and wake the next locker when it will be dropped.
    /// `WriteLockOwnedGuard` have a `'static` lifetime, but requires the `Arc<RWLock<T>>` type
    ///
    /// # Examples
    ///
    /// ```
    /// use fast_async_mutex::rwlock::RwLock;
    /// use std::sync::Arc;
    /// #[tokio::main]
    /// async fn main() {
    ///     let mutex = Arc::new(RwLock::new(10));
    ///     let guard = mutex.read().await;
    ///     let guard2 = mutex.read().await;
    ///     assert_eq!(*guard, *guard2);
    /// }
    /// ```
    #[inline]
    pub fn read_owned(self: &Arc<Self>) -> RwLockReadOwnedGuardFuture<T> {
        RwLockReadOwnedGuardFuture {
            mutex: self.clone(),
            is_realized: false,
        }
    }

    #[inline]
    fn unlock_reader(&self) {
        if self.readers.fetch_sub(1, Ordering::Release) == 1 {
            self.inner.unlock()
        } else {
            self.inner.try_wake(null_mut())
        }
    }

    #[inline]
    fn add_reader(&self) {
        self.readers.fetch_add(1, Ordering::Release);
    }

    #[inline]
    fn try_acquire_reader(&self) -> bool {
        self.readers.load(Ordering::Acquire) > 0 || self.inner.try_acquire()
    }
}

/// The Simple Write Lock Guard
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally borrows the RWLock, so the mutex will not be dropped while a guard exists.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
#[derive(Debug)]
pub struct RwLockWriteGuard<'a, T: ?Sized> {
    mutex: &'a RwLock<T>,
}

#[derive(Debug)]
pub struct RwLockWriteGuardFuture<'a, T: ?Sized> {
    mutex: &'a RwLock<T>,
    is_realized: bool,
}

/// An owned handle to a held RWLock.
/// This guard is only available from a RWLock that is wrapped in an `Arc`. It is identical to `WriteLockGuard`, except that rather than borrowing the `RWLock`, it clones the `Arc`, incrementing the reference count. This means that unlike `WriteLockGuard`, it will have the `'static` lifetime.
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally keeps a reference-couned pointer to the original `RWLock`, so even if the lock goes away, the guard remains valid.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
#[derive(Debug)]
pub struct RwLockWriteOwnedGuard<T: ?Sized> {
    mutex: Arc<RwLock<T>>,
}

#[derive(Debug)]
pub struct RwLockWriteOwnedGuardFuture<T: ?Sized> {
    mutex: Arc<RwLock<T>>,
    is_realized: bool,
}

/// The Simple Write Lock Guard
/// As long as you have this guard, you have shared access to the underlying `T`. The guard internally borrows the `RWLock`, so the mutex will not be dropped while a guard exists.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
#[derive(Debug)]
pub struct RwLockReadGuard<'a, T: ?Sized> {
    mutex: &'a RwLock<T>,
}

#[derive(Debug)]
pub struct RwLockReadGuardFuture<'a, T: ?Sized> {
    mutex: &'a RwLock<T>,
    is_realized: bool,
}

/// An owned handle to a held RWLock.
/// This guard is only available from a RWLock that is wrapped in an `Arc`. It is identical to `WriteLockGuard`, except that rather than borrowing the `RWLock`, it clones the `Arc`, incrementing the reference count. This means that unlike `WriteLockGuard`, it will have the `'static` lifetime.
/// As long as you have this guard, you have shared access to the underlying `T`. The guard internally keeps a reference-couned pointer to the original `RWLock`, so even if the lock goes away, the guard remains valid.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
#[derive(Debug)]
pub struct RwLockReadOwnedGuard<T: ?Sized> {
    mutex: Arc<RwLock<T>>,
}

#[derive(Debug)]
pub struct RwLockReadOwnedGuardFuture<T: ?Sized> {
    mutex: Arc<RwLock<T>>,
    is_realized: bool,
}

impl<'a, T: ?Sized> Future for RwLockWriteGuardFuture<'a, T> {
    type Output = RwLockWriteGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.inner.try_acquire() {
            self.is_realized = true;
            Poll::Ready(RwLockWriteGuard { mutex: self.mutex })
        } else {
            self.mutex.inner.store_waker(cx.waker());
            Poll::Pending
        }
    }
}

impl<T: ?Sized> Future for RwLockWriteOwnedGuardFuture<T> {
    type Output = RwLockWriteOwnedGuard<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.inner.try_acquire() {
            self.is_realized = true;
            Poll::Ready(RwLockWriteOwnedGuard {
                mutex: self.mutex.clone(),
            })
        } else {
            self.mutex.inner.store_waker(cx.waker());
            Poll::Pending
        }
    }
}

impl<'a, T: ?Sized> Future for RwLockReadGuardFuture<'a, T> {
    type Output = RwLockReadGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.try_acquire_reader() {
            self.is_realized = true;
            self.mutex.add_reader();
            Poll::Ready(RwLockReadGuard { mutex: self.mutex })
        } else {
            self.mutex.inner.store_waker(cx.waker());
            Poll::Pending
        }
    }
}

impl<T: ?Sized> Future for RwLockReadOwnedGuardFuture<T> {
    type Output = RwLockReadOwnedGuard<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.try_acquire_reader() {
            self.is_realized = true;
            self.mutex.add_reader();
            Poll::Ready(RwLockReadOwnedGuard {
                mutex: self.mutex.clone(),
            })
        } else {
            self.mutex.inner.store_waker(cx.waker());
            Poll::Pending
        }
    }
}

crate::impl_send_sync_rwlock!(
    RwLock,
    RwLockReadGuard,
    RwLockReadOwnedGuard,
    RwLockWriteGuard,
    RwLockWriteOwnedGuard
);

crate::impl_deref_mut!(RwLockWriteGuard, 'a);
crate::impl_deref_mut!(RwLockWriteOwnedGuard);
crate::impl_deref!(RwLockReadGuard, 'a);
crate::impl_deref!(RwLockReadOwnedGuard);

crate::impl_drop_guard!(RwLockWriteGuard, 'a, unlock);
crate::impl_drop_guard!(RwLockWriteOwnedGuard, unlock);
crate::impl_drop_guard_self!(RwLockReadGuard, 'a, unlock_reader);
crate::impl_drop_guard_self!(RwLockReadOwnedGuard, unlock_reader);

crate::impl_drop_guard_future!(RwLockWriteGuardFuture, 'a, unlock);
crate::impl_drop_guard_future!(RwLockWriteOwnedGuardFuture, unlock);
crate::impl_drop_guard_future!(RwLockReadGuardFuture, 'a, unlock);
crate::impl_drop_guard_future!(RwLockReadOwnedGuardFuture, unlock);

#[cfg(test)]
mod tests {
    use crate::rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard, RwLockWriteOwnedGuard};
    use futures::executor::block_on;
    use futures::{FutureExt, StreamExt, TryStreamExt};
    use std::ops::AddAssign;
    use std::sync::Arc;
    use tokio::time::{delay_for, Duration};

    #[tokio::test(core_threads = 12)]
    async fn test_mutex() {
        let c = RwLock::new(0);

        futures::stream::iter(0..10000)
            .for_each_concurrent(None, |_| async {
                let mut co: RwLockWriteGuard<i32> = c.write().await;
                *co += 1;
            })
            .await;

        let co = c.write().await;
        assert_eq!(*co, 10000)
    }

    #[tokio::test(core_threads = 12)]
    async fn test_mutex_delay() {
        let expected_result = 100;
        let c = RwLock::new(0);

        futures::stream::iter(0..expected_result)
            .then(|i| c.write().map(move |co| (i, co)))
            .for_each_concurrent(None, |(i, mut co)| async move {
                delay_for(Duration::from_millis(expected_result - i)).await;
                *co += 1;
            })
            .await;

        let co = c.write().await;
        assert_eq!(*co, expected_result)
    }

    #[tokio::test(core_threads = 12)]
    async fn test_owned_mutex() {
        let c = Arc::new(RwLock::new(0));

        futures::stream::iter(0..10000)
            .for_each_concurrent(None, |_| async {
                let mut co: RwLockWriteOwnedGuard<i32> = c.write_owned().await;
                *co += 1;
            })
            .await;

        let co = c.write_owned().await;
        assert_eq!(*co, 10000)
    }

    #[tokio::test(core_threads = 12)]
    async fn test_container() {
        let c = RwLock::new(String::from("lol"));

        let mut co: RwLockWriteGuard<String> = c.write().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[tokio::test(core_threads = 12)]
    async fn test_timeout() {
        let c = RwLock::new(String::from("lol"));

        let co: RwLockWriteGuard<String> = c.write().await;

        futures::stream::iter(0..10000i32)
            .then(|_| tokio::time::timeout(Duration::from_nanos(1), c.write()))
            .try_for_each_concurrent(None, |_c| futures::future::ok(()))
            .await
            .expect_err("timout must be");

        drop(co);

        let mut co: RwLockWriteGuard<String> = c.write().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[tokio::test(core_threads = 12)]
    async fn test_concurrent_reading() {
        let c = RwLock::new(String::from("lol"));

        let co: RwLockReadGuard<String> = c.read().await;

        futures::stream::iter(0..10000i32)
            .then(|_| c.read())
            .inspect(|c| assert_eq!(*co, **c))
            .for_each_concurrent(None, |_c| futures::future::ready(()))
            .await;

        assert!(matches!(
            tokio::time::timeout(Duration::from_millis(1), c.write()).await,
            Err(_)
        ));

        let co2: RwLockReadGuard<String> = c.read().await;
        assert_eq!(*co, *co2);
    }

    #[tokio::test(core_threads = 12)]
    async fn test_concurrent_reading_writing() {
        let c = RwLock::new(String::from("lol"));

        let co: RwLockReadGuard<String> = c.read().await;
        let co2: RwLockReadGuard<String> = c.read().await;
        assert_eq!(*co, *co2);

        drop(co);
        drop(co2);

        let mut co: RwLockWriteGuard<String> = c.write().await;

        assert!(matches!(
            tokio::time::timeout(Duration::from_millis(1), c.read()).await,
            Err(_)
        ));

        *co += "lol";

        drop(co);

        let co: RwLockReadGuard<String> = c.read().await;
        let co2: RwLockReadGuard<String> = c.read().await;
        assert_eq!(*co, "lollol");
        assert_eq!(*co, *co2);
    }

    #[test]
    fn multithreading_test() {
        let num = 100;
        let mutex = Arc::new(RwLock::new(0));
        let ths: Vec<_> = (0..num)
            .map(|i| {
                let mutex = mutex.clone();
                std::thread::spawn(move || {
                    block_on(async {
                        if i % 2 == 0 {
                            let mut lock = mutex.write().await;
                            *lock += 1;
                            drop(lock)
                        } else {
                            let lock1 = mutex.read().await;
                            let lock2 = mutex.read().await;
                            assert_eq!(*lock1, *lock2);
                            drop(lock1);
                            drop(lock2);
                        }
                    })
                })
            })
            .collect();

        for thread in ths {
            thread.join().unwrap();
        }

        block_on(async {
            let lock = mutex.read().await;
            assert_eq!(num / 2, *lock)
        })
    }
}
