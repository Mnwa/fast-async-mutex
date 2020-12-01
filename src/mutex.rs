use crate::inner::Inner;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

/// The simple Mutex, which will provide unique access to you data between multiple threads/futures.
#[derive(Debug)]
pub struct Mutex<T: ?Sized> {
    inner: Inner<T>,
}

impl<T> Mutex<T> {
    /// Create a new `Mutex`
    #[inline]
    pub const fn new(data: T) -> Mutex<T> {
        Mutex {
            inner: Inner::new(data),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Acquires the mutex.
    ///
    /// Returns a guard that releases the mutex and wake the next locker when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use fast_async_mutex::mutex::Mutex;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mutex = Mutex::new(10);
    ///     let guard = mutex.lock().await;
    ///     assert_eq!(*guard, 10);
    /// }
    /// ```
    #[inline]
    pub const fn lock(&self) -> MutexGuardFuture<T> {
        MutexGuardFuture {
            mutex: &self,
            is_realized: false,
        }
    }

    /// Acquires the mutex.
    ///
    /// Returns a guard that releases the mutex and wake the next locker when dropped.
    /// `MutexOwnedGuardFuture` have a `'static` lifetime, but requires the `Arc<Mutex<T>>` type
    ///
    /// # Examples
    ///
    /// ```
    /// use fast_async_mutex::mutex::Mutex;
    /// use std::sync::Arc;
    /// #[tokio::main]
    /// async fn main() {
    ///     let mutex = Arc::new(Mutex::new(10));
    ///     let guard = mutex.lock_owned().await;
    ///     assert_eq!(*guard, 10);
    /// }
    /// ```
    #[inline]
    pub fn lock_owned(self: &Arc<Self>) -> MutexOwnedGuardFuture<T> {
        MutexOwnedGuardFuture {
            mutex: self.clone(),
            is_realized: false,
        }
    }
}

/// The Simple Mutex Guard
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally borrows the Mutex, so the mutex will not be dropped while a guard exists.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
#[derive(Debug)]
pub struct MutexGuard<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
}

#[derive(Debug)]
pub struct MutexGuardFuture<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
    is_realized: bool,
}

/// An owned handle to a held Mutex.
/// This guard is only available from a Mutex that is wrapped in an `Arc`. It is identical to `MutexGuard`, except that rather than borrowing the `Mutex`, it clones the `Arc`, incrementing the reference count. This means that unlike `MutexGuard`, it will have the `'static` lifetime.
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally keeps a reference-couned pointer to the original `Mutex`, so even if the lock goes away, the guard remains valid.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
#[derive(Debug)]
pub struct MutexOwnedGuard<T: ?Sized> {
    mutex: Arc<Mutex<T>>,
}

#[derive(Debug)]
pub struct MutexOwnedGuardFuture<T: ?Sized> {
    mutex: Arc<Mutex<T>>,
    is_realized: bool,
}

impl<'a, T: ?Sized> Future for MutexGuardFuture<'a, T> {
    type Output = MutexGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.inner.try_acquire() {
            self.is_realized = true;
            Poll::Ready(MutexGuard { mutex: self.mutex })
        } else {
            self.mutex.inner.store_waker(cx.waker());
            Poll::Pending
        }
    }
}

impl<T: ?Sized> Future for MutexOwnedGuardFuture<T> {
    type Output = MutexOwnedGuard<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.inner.try_acquire() {
            self.is_realized = true;
            Poll::Ready(MutexOwnedGuard {
                mutex: self.mutex.clone(),
            })
        } else {
            self.mutex.inner.store_waker(cx.waker());
            Poll::Pending
        }
    }
}

crate::impl_send_sync_mutex!(Mutex, MutexGuard, MutexOwnedGuard);

crate::impl_deref_mut!(MutexGuard, 'a);
crate::impl_deref_mut!(MutexOwnedGuard);

crate::impl_drop_guard!(MutexGuard, 'a, unlock);
crate::impl_drop_guard!(MutexOwnedGuard, unlock);
crate::impl_drop_guard_future!(MutexGuardFuture, 'a, unlock);
crate::impl_drop_guard_future!(MutexOwnedGuardFuture, unlock);

#[cfg(test)]
mod tests {
    use crate::mutex::{Mutex, MutexGuard, MutexOwnedGuard};
    use futures::executor::block_on;
    use futures::{FutureExt, StreamExt, TryStreamExt};
    use std::ops::AddAssign;
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    #[tokio::test(flavor = "multi_thread", worker_threads = 12)]
    async fn test_mutex() {
        let c = Mutex::new(0);

        futures::stream::iter(0..10000)
            .for_each_concurrent(None, |_| async {
                let mut co: MutexGuard<i32> = c.lock().await;
                *co += 1;
            })
            .await;

        let co = c.lock().await;
        assert_eq!(*co, 10000)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 12)]
    async fn test_mutex_delay() {
        let expected_result = 100;
        let c = Mutex::new(0);

        futures::stream::iter(0..expected_result)
            .then(|i| c.lock().map(move |co| (i, co)))
            .for_each_concurrent(None, |(i, mut co)| async move {
                sleep(Duration::from_millis(expected_result - i)).await;
                *co += 1;
            })
            .await;

        let co = c.lock().await;
        assert_eq!(*co, expected_result)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 12)]
    async fn test_owned_mutex() {
        let c = Arc::new(Mutex::new(0));

        futures::stream::iter(0..10000)
            .for_each_concurrent(None, |_| async {
                let mut co: MutexOwnedGuard<i32> = c.lock_owned().await;
                *co += 1;
            })
            .await;

        let co = c.lock_owned().await;
        assert_eq!(*co, 10000)
    }

    #[tokio::test]
    async fn test_container() {
        let c = Mutex::new(String::from("lol"));

        let mut co: MutexGuard<String> = c.lock().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[tokio::test]
    async fn test_timeout() {
        let c = Mutex::new(String::from("lol"));

        let co: MutexGuard<String> = c.lock().await;

        futures::stream::iter(0..10000i32)
            .then(|_| tokio::time::timeout(Duration::from_nanos(1), c.lock()))
            .try_for_each_concurrent(None, |_c| futures::future::ok(()))
            .await
            .expect_err("timout must be");

        drop(co);

        let mut co: MutexGuard<String> = c.lock().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[test]
    fn multithreading_test() {
        let num = 100;
        let mutex = Arc::new(Mutex::new(0));
        let ths: Vec<_> = (0..num)
            .map(|_| {
                let mutex = mutex.clone();
                std::thread::spawn(move || {
                    block_on(async {
                        let mut lock = mutex.lock().await;
                        *lock += 1;
                    })
                })
            })
            .collect();

        for thread in ths {
            thread.join().unwrap();
        }

        block_on(async {
            let lock = mutex.lock().await;
            assert_eq!(num, *lock)
        })
    }
}
