use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

/// An async `ordered` RwLock.
/// It will be works with any async runtime in `Rust`, it may be a `tokio`, `smol`, `async-std` and etc..
///
/// The main difference with the standard `RwLock` is ordered mutex will check an ordering of blocking.
/// This way has some guaranties of mutex execution order, but it's a little bit slowly than original mutex.
/// Also ordered mutex is an unstable on thread pool runtimes.
pub struct OrderedRwLock<T: ?Sized> {
    state: AtomicUsize,
    current: AtomicUsize,
    readers: AtomicUsize,
    waker: AtomicPtr<Waker>,
    data: UnsafeCell<T>,
}

impl<T> OrderedRwLock<T> {
    /// Create a new `UnorderedRWLock`
    #[inline]
    pub const fn new(data: T) -> OrderedRwLock<T> {
        OrderedRwLock {
            state: AtomicUsize::new(0),
            current: AtomicUsize::new(0),
            readers: AtomicUsize::new(0),
            waker: AtomicPtr::new(null_mut()),
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> OrderedRwLock<T> {
    /// Acquires the mutex for are write.
    ///
    /// Returns a guard that releases the mutex and wake the next locker when it will be dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use fast_async_mutex::rwlock_ordered::OrderedRwLock;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mutex = OrderedRwLock::new(10);
    ///     let mut guard = mutex.write().await;
    ///     *guard += 1;
    ///     assert_eq!(*guard, 11);
    /// }
    /// ```
    #[inline]
    pub fn write(&self) -> OrderedRwLockWriteGuardFuture<T> {
        OrderedRwLockWriteGuardFuture {
            mutex: &self,
            id: self.state.fetch_add(1, Ordering::AcqRel),
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
    /// use fast_async_mutex::rwlock_ordered::OrderedRwLock;
    /// use std::sync::Arc;
    /// #[tokio::main]
    /// async fn main() {
    ///     let mutex = Arc::new(OrderedRwLock::new(10));
    ///     let mut guard = mutex.write_owned().await;
    ///     *guard += 1;
    ///     assert_eq!(*guard, 11);
    /// }
    /// ```
    #[inline]
    pub fn write_owned(self: &Arc<Self>) -> OrderedRwLockWriteOwnedGuardFuture<T> {
        OrderedRwLockWriteOwnedGuardFuture {
            mutex: self.clone(),
            id: self.state.fetch_add(1, Ordering::AcqRel),
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
    /// use fast_async_mutex::rwlock_ordered::OrderedRwLock;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mutex = OrderedRwLock::new(10);
    ///     let guard = mutex.read().await;
    ///     let guard2 = mutex.read().await;
    ///     assert_eq!(*guard, *guard2);
    /// }
    /// ```
    #[inline]
    pub fn read(&self) -> OrderedRwLockReadGuardFuture<T> {
        OrderedRwLockReadGuardFuture {
            mutex: &self,
            id: self.state.fetch_add(1, Ordering::AcqRel),
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
    /// use fast_async_mutex::rwlock_ordered::OrderedRwLock;
    /// use std::sync::Arc;
    /// #[tokio::main]
    /// async fn main() {
    ///     let mutex = Arc::new(OrderedRwLock::new(10));
    ///     let guard = mutex.read().await;
    ///     let guard2 = mutex.read().await;
    ///     assert_eq!(*guard, *guard2);
    /// }
    /// ```
    #[inline]
    pub fn read_owned(self: &Arc<Self>) -> OrderedRwLockReadOwnedGuardFuture<T> {
        OrderedRwLockReadOwnedGuardFuture {
            mutex: self.clone(),
            id: self.state.fetch_add(1, Ordering::AcqRel),
            is_realized: false,
        }
    }

    #[inline]
    fn unlock(&self) {
        self.current.fetch_add(1, Ordering::AcqRel);

        let waker_ptr = self.waker.swap(null_mut(), Ordering::AcqRel);
        if !waker_ptr.is_null() {
            unsafe { Box::from_raw(waker_ptr).wake() }
        }
    }

    #[inline]
    fn store_waker(&self, waker: &Waker) {
        let _ = self.waker.compare_exchange_weak(
            null_mut(),
            Box::into_raw(Box::new(waker.clone())),
            Ordering::AcqRel,
            Ordering::Relaxed,
        );
    }
}

/// The Simple Write Lock Guard
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally borrows the RWLock, so the mutex will not be dropped while a guard exists.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
pub struct OrderedRwLockWriteGuard<'a, T: ?Sized> {
    mutex: &'a OrderedRwLock<T>,
}

pub struct OrderedRwLockWriteGuardFuture<'a, T: ?Sized> {
    mutex: &'a OrderedRwLock<T>,
    id: usize,
    is_realized: bool,
}

/// An owned handle to a held RWLock.
/// This guard is only available from a RWLock that is wrapped in an `Arc`. It is identical to `WriteLockGuard`, except that rather than borrowing the `RWLock`, it clones the `Arc`, incrementing the reference count. This means that unlike `WriteLockGuard`, it will have the `'static` lifetime.
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally keeps a reference-couned pointer to the original `RWLock`, so even if the lock goes away, the guard remains valid.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
pub struct OrderedRwLockWriteOwnedGuard<T: ?Sized> {
    mutex: Arc<OrderedRwLock<T>>,
}

pub struct OrderedRwLockWriteOwnedGuardFuture<T: ?Sized> {
    mutex: Arc<OrderedRwLock<T>>,
    id: usize,
    is_realized: bool,
}

/// The Simple Write Lock Guard
/// As long as you have this guard, you have shared access to the underlying `T`. The guard internally borrows the `RWLock`, so the mutex will not be dropped while a guard exists.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
pub struct OrderedRwLockReadGuard<'a, T: ?Sized> {
    mutex: &'a OrderedRwLock<T>,
}

pub struct OrderedRwLockReadGuardFuture<'a, T: ?Sized> {
    mutex: &'a OrderedRwLock<T>,
    id: usize,
    is_realized: bool,
}

/// An owned handle to a held RWLock.
/// This guard is only available from a RWLock that is wrapped in an `Arc`. It is identical to `WriteLockGuard`, except that rather than borrowing the `RWLock`, it clones the `Arc`, incrementing the reference count. This means that unlike `WriteLockGuard`, it will have the `'static` lifetime.
/// As long as you have this guard, you have shared access to the underlying `T`. The guard internally keeps a reference-couned pointer to the original `RWLock`, so even if the lock goes away, the guard remains valid.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
pub struct OrderedRwLockReadOwnedGuard<T: ?Sized> {
    mutex: Arc<OrderedRwLock<T>>,
}

pub struct OrderedRwLockReadOwnedGuardFuture<T: ?Sized> {
    mutex: Arc<OrderedRwLock<T>>,
    id: usize,
    is_realized: bool,
}

unsafe impl<T> Send for OrderedRwLock<T> where T: Send + ?Sized {}
unsafe impl<T> Sync for OrderedRwLock<T> where T: Send + Sync + ?Sized {}

unsafe impl<T> Send for OrderedRwLockReadGuard<'_, T> where T: ?Sized + Send {}
unsafe impl<T> Send for OrderedRwLockReadOwnedGuard<T> where T: ?Sized + Send {}

unsafe impl<T> Sync for OrderedRwLockReadGuard<'_, T> where T: Send + Sync + ?Sized {}
unsafe impl<T> Sync for OrderedRwLockReadOwnedGuard<T> where T: Send + Sync + ?Sized {}

unsafe impl<T> Send for OrderedRwLockWriteGuard<'_, T> where T: ?Sized + Send {}
unsafe impl<T> Send for OrderedRwLockWriteOwnedGuard<T> where T: ?Sized + Send {}

unsafe impl<T> Sync for OrderedRwLockWriteGuard<'_, T> where T: Send + Sync + ?Sized {}
unsafe impl<T> Sync for OrderedRwLockWriteOwnedGuard<T> where T: Send + Sync + ?Sized {}

impl<'a, T: ?Sized> Future for OrderedRwLockWriteGuardFuture<'a, T> {
    type Output = OrderedRwLockWriteGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::Acquire);

        if current == self.id {
            self.is_realized = true;
            Poll::Ready(OrderedRwLockWriteGuard { mutex: self.mutex })
        } else {
            if Some(current) == self.id.checked_sub(1) {
                self.mutex.store_waker(cx.waker())
            }
            Poll::Pending
        }
    }
}

impl<T: ?Sized> Future for OrderedRwLockWriteOwnedGuardFuture<T> {
    type Output = OrderedRwLockWriteOwnedGuard<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::Acquire);
        if current == self.id {
            self.is_realized = true;
            Poll::Ready(OrderedRwLockWriteOwnedGuard {
                mutex: self.mutex.clone(),
            })
        } else {
            if Some(current) == self.id.checked_sub(1) {
                self.mutex.store_waker(cx.waker())
            }

            Poll::Pending
        }
    }
}

impl<T: ?Sized> Deref for OrderedRwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for OrderedRwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Deref for OrderedRwLockWriteOwnedGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for OrderedRwLockWriteOwnedGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for OrderedRwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.unlock()
    }
}

impl<T: ?Sized> Drop for OrderedRwLockWriteOwnedGuard<T> {
    fn drop(&mut self) {
        self.mutex.unlock()
    }
}

impl<T: ?Sized> Drop for OrderedRwLockWriteGuardFuture<'_, T> {
    fn drop(&mut self) {
        if !self.is_realized {
            self.mutex.unlock()
        }
    }
}

impl<T: ?Sized> Drop for OrderedRwLockWriteOwnedGuardFuture<T> {
    fn drop(&mut self) {
        if !self.is_realized {
            self.mutex.unlock()
        }
    }
}
impl<'a, T: ?Sized> Future for OrderedRwLockReadGuardFuture<'a, T> {
    type Output = OrderedRwLockReadGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::Acquire);
        let readers = self.mutex.readers.load(Ordering::Acquire);
        if current + readers == self.id {
            self.is_realized = true;
            self.mutex.readers.fetch_add(1, Ordering::Release);
            Poll::Ready(OrderedRwLockReadGuard { mutex: self.mutex })
        } else {
            if Some(current + readers) == self.id.checked_sub(1) {
                self.mutex.store_waker(cx.waker())
            }
            Poll::Pending
        }
    }
}

impl<T: ?Sized> Future for OrderedRwLockReadOwnedGuardFuture<T> {
    type Output = OrderedRwLockReadOwnedGuard<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::Acquire);
        let readers = self.mutex.readers.load(Ordering::Acquire);
        if current + readers == self.id {
            self.is_realized = true;
            self.mutex.readers.fetch_add(1, Ordering::Release);
            Poll::Ready(OrderedRwLockReadOwnedGuard {
                mutex: self.mutex.clone(),
            })
        } else {
            if Some(current + readers) == self.id.checked_sub(1) {
                self.mutex.store_waker(cx.waker())
            }

            Poll::Pending
        }
    }
}

impl<T: ?Sized> Deref for OrderedRwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> Deref for OrderedRwLockReadOwnedGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for OrderedRwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.readers.fetch_sub(1, Ordering::Release);
        self.mutex.unlock()
    }
}

impl<T: ?Sized> Drop for OrderedRwLockReadOwnedGuard<T> {
    fn drop(&mut self) {
        self.mutex.readers.fetch_sub(1, Ordering::Release);
        self.mutex.unlock()
    }
}

impl<T: ?Sized> Drop for OrderedRwLockReadGuardFuture<'_, T> {
    fn drop(&mut self) {
        if !self.is_realized {
            self.mutex.unlock()
        }
    }
}

impl<T: ?Sized> Drop for OrderedRwLockReadOwnedGuardFuture<T> {
    fn drop(&mut self) {
        if !self.is_realized {
            self.mutex.unlock()
        }
    }
}

impl<T: Debug> Debug for OrderedRwLock<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderedRwLock")
            .field("state", &self.state)
            .field("current", &self.current)
            .field("waker", &self.waker)
            .field("data", &self.data)
            .field("readers", &self.readers)
            .finish()
    }
}

impl<T: Debug> Debug for OrderedRwLockWriteGuardFuture<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderedRwLockWriteGuardFuture")
            .field("mutex", &self.mutex)
            .field("id", &self.id)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for OrderedRwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderedRwLockWriteGuard")
            .field("mutex", &self.mutex)
            .finish()
    }
}

impl<T: Debug> Debug for OrderedRwLockWriteOwnedGuardFuture<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderedRwLockWriteOwnedGuardFuture")
            .field("mutex", &self.mutex)
            .field("id", &self.id)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for OrderedRwLockWriteOwnedGuard<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderedRwLockWriteOwnedGuard")
            .field("mutex", &self.mutex)
            .finish()
    }
}

impl<T: Debug> Debug for OrderedRwLockReadGuardFuture<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderedRwLockReadGuardFuture")
            .field("mutex", &self.mutex)
            .field("id", &self.id)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for OrderedRwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderedRwLockReadGuard")
            .field("mutex", &self.mutex)
            .finish()
    }
}

impl<T: Debug> Debug for OrderedRwLockReadOwnedGuardFuture<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderedRwLockReadOwnedGuardFuture")
            .field("mutex", &self.mutex)
            .field("id", &self.id)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for OrderedRwLockReadOwnedGuard<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderedRwLockReadOwnedGuard")
            .field("mutex", &self.mutex)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::rwlock_ordered::{
        OrderedRwLock, OrderedRwLockReadGuard, OrderedRwLockWriteGuard,
        OrderedRwLockWriteOwnedGuard,
    };
    use futures::{FutureExt, StreamExt, TryStreamExt};
    use std::ops::AddAssign;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;
    use tokio::time::{delay_for, Duration};

    #[tokio::test(core_threads = 12)]
    async fn test_mutex() {
        let c = OrderedRwLock::new(0);

        futures::stream::iter(0..10000)
            .for_each_concurrent(None, |_| async {
                let mut co: OrderedRwLockWriteGuard<i32> = c.write().await;
                *co += 1;
            })
            .await;

        let co = c.write().await;
        assert_eq!(*co, 10000)
    }

    #[tokio::test(core_threads = 12)]
    async fn test_mutex_delay() {
        let expected_result = 100;
        let c = OrderedRwLock::new(0);

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
        let c = Arc::new(OrderedRwLock::new(0));

        futures::stream::iter(0..10000)
            .for_each_concurrent(None, |_| async {
                let mut co: OrderedRwLockWriteOwnedGuard<i32> = c.write_owned().await;
                *co += 1;
            })
            .await;

        let co = c.write_owned().await;
        assert_eq!(*co, 10000)
    }

    #[tokio::test(core_threads = 12)]
    async fn test_container() {
        let c = OrderedRwLock::new(String::from("lol"));

        let mut co: OrderedRwLockWriteGuard<String> = c.write().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[tokio::test(core_threads = 12)]
    async fn test_overflow() {
        let mut c = OrderedRwLock::new(String::from("lol"));

        c.state = AtomicUsize::new(usize::max_value());
        c.current = AtomicUsize::new(usize::max_value());

        let mut co: OrderedRwLockWriteGuard<String> = c.write().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[tokio::test(core_threads = 12)]
    async fn test_timeout() {
        let c = OrderedRwLock::new(String::from("lol"));

        let co: OrderedRwLockWriteGuard<String> = c.write().await;

        futures::stream::iter(0..10000i32)
            .then(|_| tokio::time::timeout(Duration::from_nanos(1), c.write()))
            .try_for_each_concurrent(None, |_c| futures::future::ok(()))
            .await
            .expect_err("timout must be");

        drop(co);

        let mut co: OrderedRwLockWriteGuard<String> = c.write().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[tokio::test(core_threads = 12)]
    async fn test_concurrent_reading() {
        let c = OrderedRwLock::new(String::from("lol"));

        let co: OrderedRwLockReadGuard<String> = c.read().await;

        futures::stream::iter(0..10000i32)
            .then(|_| c.read())
            .inspect(|c| assert_eq!(*co, **c))
            .for_each_concurrent(None, |_c| futures::future::ready(()))
            .await;

        assert!(matches!(
            tokio::time::timeout(Duration::from_millis(1), c.write()).await,
            Err(_)
        ));

        let co2: OrderedRwLockReadGuard<String> = c.read().await;
        assert_eq!(*co, *co2);
    }

    #[tokio::test(core_threads = 12)]
    async fn test_concurrent_reading_writing() {
        let c = OrderedRwLock::new(String::from("lol"));

        let co: OrderedRwLockReadGuard<String> = c.read().await;
        let co2: OrderedRwLockReadGuard<String> = c.read().await;
        assert_eq!(*co, *co2);

        drop(co);
        drop(co2);

        let mut co: OrderedRwLockWriteGuard<String> = c.write().await;

        assert!(matches!(
            tokio::time::timeout(Duration::from_millis(1), c.read()).await,
            Err(_)
        ));

        *co += "lol";

        drop(co);

        let co: OrderedRwLockReadGuard<String> = c.read().await;
        let co2: OrderedRwLockReadGuard<String> = c.read().await;
        assert_eq!(*co, "lollol");
        assert_eq!(*co, *co2);
    }
}
