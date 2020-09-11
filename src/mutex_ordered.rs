use crate::inner::OrderedInner;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

/// An async `ordered` mutex.
/// It will be work with any async runtime in `Rust`, it may be a `tokio`, `smol`, `async-std`, etc..
///
/// The main difference with the standard `Mutex` is ordered mutex will check an ordering of blocking.
/// This way has some guaranties of mutex execution order, but it's a little bit slowly than original mutex.
///
/// The Ordered Mutex has its mechanism of locking order when you have concurrent access to data.
/// It will work well when you needed step by step data locking like sending UDP packages in a specific order.
#[derive(Debug)]
pub struct OrderedMutex<T: ?Sized> {
    inner: OrderedInner<T>,
}

impl<T> OrderedMutex<T> {
    /// Create a new `OrderedMutex`
    #[inline]
    pub const fn new(data: T) -> OrderedMutex<T> {
        OrderedMutex {
            inner: OrderedInner::new(data),
        }
    }
}

impl<T: ?Sized> OrderedMutex<T> {
    /// Acquires the mutex.
    ///
    /// Returns a guard that releases the mutex and wake the next locker when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use fast_async_mutex::mutex_ordered::OrderedMutex;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mutex = OrderedMutex::new(10);
    ///     let guard = mutex.lock().await;
    ///     assert_eq!(*guard, 10);
    /// }
    /// ```
    #[inline]
    pub fn lock(&self) -> OrderedMutexGuardFuture<T> {
        OrderedMutexGuardFuture {
            mutex: &self,
            id: self.inner.generate_id(),
            is_realized: false,
        }
    }

    /// Acquires the mutex.
    ///
    /// Returns a guard that releases the mutex and wake the next locker when dropped.
    /// `OrderedMutexOwnedGuard` have a `'static` lifetime, but requires the `Arc<OrderedMutex<T>>` type
    ///
    /// # Examples
    ///
    /// ```
    /// use fast_async_mutex::mutex_ordered::OrderedMutex;
    /// use std::sync::Arc;
    /// #[tokio::main]
    /// async fn main() {
    ///     let mutex = Arc::new(OrderedMutex::new(10));
    ///     let guard = mutex.lock_owned().await;
    ///     assert_eq!(*guard, 10);
    /// }
    /// ```
    #[inline]
    pub fn lock_owned(self: &Arc<Self>) -> OrderedMutexOwnedGuardFuture<T> {
        OrderedMutexOwnedGuardFuture {
            mutex: self.clone(),
            id: self.inner.generate_id(),
            is_realized: false,
        }
    }
}

/// The Simple OrderedMutex Guard
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally borrows the OrderedMutex, so the mutex will not be dropped while a guard exists.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
#[derive(Debug)]
pub struct OrderedMutexGuard<'a, T: ?Sized> {
    mutex: &'a OrderedMutex<T>,
}

#[derive(Debug)]
pub struct OrderedMutexGuardFuture<'a, T: ?Sized> {
    mutex: &'a OrderedMutex<T>,
    id: usize,
    is_realized: bool,
}

/// An owned handle to a held OrderedMutex.
/// This guard is only available from a OrderedMutex that is wrapped in an `Arc`. It is identical to `OrderedMutexGuard`, except that rather than borrowing the `OrderedMutex`, it clones the `Arc`, incrementing the reference count. This means that unlike `OrderedMutexGuard`, it will have the `'static` lifetime.
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally keeps a reference-couned pointer to the original `OrderedMutex`, so even if the lock goes away, the guard remains valid.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
#[derive(Debug)]
pub struct OrderedMutexOwnedGuard<T: ?Sized> {
    mutex: Arc<OrderedMutex<T>>,
}

#[derive(Debug)]
pub struct OrderedMutexOwnedGuardFuture<T: ?Sized> {
    mutex: Arc<OrderedMutex<T>>,
    id: usize,
    is_realized: bool,
}

impl<'a, T: ?Sized> Future for OrderedMutexGuardFuture<'a, T> {
    type Output = OrderedMutexGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.inner.try_acquire(self.id) {
            self.is_realized = true;
            Poll::Ready(OrderedMutexGuard { mutex: self.mutex })
        } else {
            self.mutex.inner.store_waker(cx.waker());
            Poll::Pending
        }
    }
}

impl<T: ?Sized> Future for OrderedMutexOwnedGuardFuture<T> {
    type Output = OrderedMutexOwnedGuard<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.inner.try_acquire(self.id) {
            self.is_realized = true;
            Poll::Ready(OrderedMutexOwnedGuard {
                mutex: self.mutex.clone(),
            })
        } else {
            self.mutex.inner.store_waker(cx.waker());
            Poll::Pending
        }
    }
}

crate::impl_send_sync_mutex!(OrderedMutex, OrderedMutexGuard, OrderedMutexOwnedGuard);

crate::impl_deref_mut!(OrderedMutexGuard, 'a);
crate::impl_deref_mut!(OrderedMutexOwnedGuard);

crate::impl_drop_guard!(OrderedMutexGuard, 'a, unlock);
crate::impl_drop_guard!(OrderedMutexOwnedGuard, unlock);
crate::impl_drop_guard_future!(OrderedMutexGuardFuture, 'a, unlock);
crate::impl_drop_guard_future!(OrderedMutexOwnedGuardFuture, unlock);

#[cfg(test)]
mod tests {
    use crate::mutex_ordered::{OrderedMutex, OrderedMutexGuard, OrderedMutexOwnedGuard};
    use futures::executor::block_on;
    use futures::{FutureExt, StreamExt, TryStreamExt};
    use std::ops::AddAssign;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;
    use tokio::time::{delay_for, Duration};

    #[tokio::test(core_threads = 12)]
    async fn test_mutex() {
        let c = OrderedMutex::new(0);

        futures::stream::iter(0..10000)
            .for_each_concurrent(None, |_| async {
                let mut co: OrderedMutexGuard<i32> = c.lock().await;
                *co += 1;
            })
            .await;

        let co = c.lock().await;
        assert_eq!(*co, 10000)
    }

    #[tokio::test(core_threads = 12)]
    async fn test_mutex_delay() {
        let expected_result = 100;
        let c = OrderedMutex::new(0);

        futures::stream::iter(0..expected_result)
            .then(|i| c.lock().map(move |co| (i, co)))
            .for_each_concurrent(None, |(i, mut co)| async move {
                delay_for(Duration::from_millis(expected_result - i)).await;
                *co += 1;
            })
            .await;

        let co = c.lock().await;
        assert_eq!(*co, expected_result)
    }

    #[tokio::test(core_threads = 12)]
    async fn test_owned_mutex() {
        let c = Arc::new(OrderedMutex::new(0));

        futures::stream::iter(0..10000)
            .for_each_concurrent(None, |_| async {
                let mut co: OrderedMutexOwnedGuard<i32> = c.lock_owned().await;
                *co += 1;
            })
            .await;

        let co = c.lock_owned().await;
        assert_eq!(*co, 10000)
    }

    #[tokio::test]
    async fn test_container() {
        let c = OrderedMutex::new(String::from("lol"));

        let mut co: OrderedMutexGuard<String> = c.lock().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[tokio::test]
    async fn test_overflow() {
        let mut c = OrderedMutex::new(String::from("lol"));

        c.inner.state = AtomicUsize::new(usize::max_value());
        c.inner.current = AtomicUsize::new(usize::max_value());

        let mut co: OrderedMutexGuard<String> = c.lock().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[tokio::test]
    async fn test_timeout() {
        let c = OrderedMutex::new(String::from("lol"));

        let co: OrderedMutexGuard<String> = c.lock().await;

        futures::stream::iter(0..10000i32)
            .then(|_| tokio::time::timeout(Duration::from_nanos(1), c.lock()))
            .try_for_each_concurrent(None, |_c| futures::future::ok(()))
            .await
            .expect_err("timout must be");

        drop(co);

        let mut co: OrderedMutexGuard<String> = c.lock().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[test]
    fn multithreading_test() {
        let num = 100;
        let mutex = Arc::new(OrderedMutex::new(0));
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
