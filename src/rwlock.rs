use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

/// An async RwLock.
/// It will be works with any async runtime in `Rust`, it may be a `tokio`, `smol`, `async-std` and etc..
pub struct RwLock<T: ?Sized> {
    state: AtomicUsize,
    current: AtomicUsize,
    readers: AtomicUsize,
    waker: AtomicPtr<Waker>,
    data: UnsafeCell<T>,
}

impl<T> RwLock<T> {
    /// Create a new `UnorderedRWLock`
    #[inline]
    pub const fn new(data: T) -> RwLock<T> {
        RwLock {
            state: AtomicUsize::new(0),
            current: AtomicUsize::new(0),
            readers: AtomicUsize::new(0),
            waker: AtomicPtr::new(null_mut()),
            data: UnsafeCell::new(data),
        }
    }

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
            id: self.state.fetch_add(1, Ordering::AcqRel),
            is_realized: false,
        }
    }
}

/// The Simple Write Lock Guard
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally borrows the RWLock, so the mutex will not be dropped while a guard exists.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
pub struct RwLockWriteGuard<'a, T: ?Sized> {
    mutex: &'a RwLock<T>,
}

pub struct RwLockWriteGuardFuture<'a, T: ?Sized> {
    mutex: &'a RwLock<T>,
    id: usize,
    is_realized: bool,
}

/// An owned handle to a held RWLock.
/// This guard is only available from a RWLock that is wrapped in an `Arc`. It is identical to `WriteLockGuard`, except that rather than borrowing the `RWLock`, it clones the `Arc`, incrementing the reference count. This means that unlike `WriteLockGuard`, it will have the `'static` lifetime.
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally keeps a reference-couned pointer to the original `RWLock`, so even if the lock goes away, the guard remains valid.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
pub struct RwLockWriteOwnedGuard<T: ?Sized> {
    mutex: Arc<RwLock<T>>,
}

pub struct RwLockWriteOwnedGuardFuture<T: ?Sized> {
    mutex: Arc<RwLock<T>>,
    id: usize,
    is_realized: bool,
}

/// The Simple Write Lock Guard
/// As long as you have this guard, you have shared access to the underlying `T`. The guard internally borrows the `RWLock`, so the mutex will not be dropped while a guard exists.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
pub struct RwLockReadGuard<'a, T: ?Sized> {
    mutex: &'a RwLock<T>,
}

pub struct RwLockReadGuardFuture<'a, T: ?Sized> {
    mutex: &'a RwLock<T>,
    id: usize,
    is_realized: bool,
}

/// An owned handle to a held RWLock.
/// This guard is only available from a RWLock that is wrapped in an `Arc`. It is identical to `WriteLockGuard`, except that rather than borrowing the `RWLock`, it clones the `Arc`, incrementing the reference count. This means that unlike `WriteLockGuard`, it will have the `'static` lifetime.
/// As long as you have this guard, you have shared access to the underlying `T`. The guard internally keeps a reference-couned pointer to the original `RWLock`, so even if the lock goes away, the guard remains valid.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
pub struct RwLockReadOwnedGuard<T: ?Sized> {
    mutex: Arc<RwLock<T>>,
}

pub struct RwLockReadOwnedGuardFuture<T: ?Sized> {
    mutex: Arc<RwLock<T>>,
    id: usize,
    is_realized: bool,
}

unsafe impl<T> Send for RwLock<T> where T: Send + ?Sized {}
unsafe impl<T> Sync for RwLock<T> where T: Send + Sync + ?Sized {}

unsafe impl<T> Send for RwLockReadGuard<'_, T> where T: ?Sized + Send {}
unsafe impl<T> Send for RwLockReadOwnedGuard<T> where T: ?Sized + Send {}

unsafe impl<T> Sync for RwLockReadGuard<'_, T> where T: Send + Sync + ?Sized {}
unsafe impl<T> Sync for RwLockReadOwnedGuard<T> where T: Send + Sync + ?Sized {}

unsafe impl<T> Send for RwLockWriteGuard<'_, T> where T: ?Sized + Send {}
unsafe impl<T> Send for RwLockWriteOwnedGuard<T> where T: ?Sized + Send {}

unsafe impl<T> Sync for RwLockWriteGuard<'_, T> where T: Send + Sync + ?Sized {}
unsafe impl<T> Sync for RwLockWriteOwnedGuard<T> where T: Send + Sync + ?Sized {}

impl<'a, T: ?Sized> Future for RwLockWriteGuardFuture<'a, T> {
    type Output = RwLockWriteGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::Acquire);

        if current == self.id {
            self.is_realized = true;
            Poll::Ready(RwLockWriteGuard { mutex: self.mutex })
        } else {
            if Some(current) == self.id.checked_sub(1) {
                self.mutex
                    .waker
                    .swap(cx.waker() as *const Waker as *mut Waker, Ordering::AcqRel);
            }
            Poll::Pending
        }
    }
}

impl<T: ?Sized> Future for RwLockWriteOwnedGuardFuture<T> {
    type Output = RwLockWriteOwnedGuard<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::Acquire);
        if current == self.id {
            self.is_realized = true;
            Poll::Ready(RwLockWriteOwnedGuard {
                mutex: self.mutex.clone(),
            })
        } else {
            if Some(current) == self.id.checked_sub(1) {
                self.mutex
                    .waker
                    .store(cx.waker() as *const Waker as *mut Waker, Ordering::Release);
            }

            Poll::Pending
        }
    }
}

impl<T: ?Sized> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Deref for RwLockWriteOwnedGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for RwLockWriteOwnedGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.current.fetch_add(1, Ordering::AcqRel);

        wake_ptr(&self.mutex.waker)
    }
}

impl<T: ?Sized> Drop for RwLockWriteOwnedGuard<T> {
    fn drop(&mut self) {
        self.mutex.current.fetch_add(1, Ordering::AcqRel);

        wake_ptr(&self.mutex.waker)
    }
}

impl<T: ?Sized> Drop for RwLockWriteGuardFuture<'_, T> {
    fn drop(&mut self) {
        if !self.is_realized {
            self.mutex.current.fetch_add(1, Ordering::AcqRel);

            wake_ptr(&self.mutex.waker)
        }
    }
}

impl<T: ?Sized> Drop for RwLockWriteOwnedGuardFuture<T> {
    fn drop(&mut self) {
        if !self.is_realized {
            self.mutex.current.fetch_add(1, Ordering::AcqRel);

            wake_ptr(&self.mutex.waker)
        }
    }
}
impl<'a, T: ?Sized> Future for RwLockReadGuardFuture<'a, T> {
    type Output = RwLockReadGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::Acquire);
        let readers = self.mutex.readers.load(Ordering::Acquire);
        if current + readers == self.id {
            self.is_realized = true;
            self.mutex.readers.fetch_add(1, Ordering::Release);
            Poll::Ready(RwLockReadGuard { mutex: self.mutex })
        } else {
            if Some(current + readers) == self.id.checked_sub(1) {
                self.mutex
                    .waker
                    .swap(cx.waker() as *const Waker as *mut Waker, Ordering::AcqRel);
            }
            Poll::Pending
        }
    }
}

impl<T: ?Sized> Future for RwLockReadOwnedGuardFuture<T> {
    type Output = RwLockReadOwnedGuard<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::Acquire);
        let readers = self.mutex.readers.load(Ordering::Acquire);
        if current + readers == self.id {
            self.is_realized = true;
            self.mutex.readers.fetch_add(1, Ordering::Release);
            Poll::Ready(RwLockReadOwnedGuard {
                mutex: self.mutex.clone(),
            })
        } else {
            if Some(current + readers) == self.id.checked_sub(1) {
                self.mutex
                    .waker
                    .store(cx.waker() as *const Waker as *mut Waker, Ordering::Release);
            }

            Poll::Pending
        }
    }
}

impl<T: ?Sized> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> Deref for RwLockReadOwnedGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.current.fetch_add(1, Ordering::AcqRel);
        self.mutex.readers.fetch_sub(1, Ordering::Release);

        wake_ptr(&self.mutex.waker)
    }
}

impl<T: ?Sized> Drop for RwLockReadOwnedGuard<T> {
    fn drop(&mut self) {
        self.mutex.current.fetch_add(1, Ordering::AcqRel);
        self.mutex.readers.fetch_sub(1, Ordering::Release);

        wake_ptr(&self.mutex.waker)
    }
}

impl<T: ?Sized> Drop for RwLockReadGuardFuture<'_, T> {
    fn drop(&mut self) {
        if !self.is_realized {
            self.mutex.current.fetch_add(1, Ordering::AcqRel);

            wake_ptr(&self.mutex.waker)
        }
    }
}

impl<T: ?Sized> Drop for RwLockReadOwnedGuardFuture<T> {
    fn drop(&mut self) {
        if !self.is_realized {
            self.mutex.current.fetch_add(1, Ordering::AcqRel);

            wake_ptr(&self.mutex.waker)
        }
    }
}

#[inline]
fn wake_ptr(waker_ptr: &AtomicPtr<Waker>) {
    unsafe {
        if let Some(waker_ptr) = waker_ptr.load(Ordering::Acquire).as_ref() {
            waker_ptr.wake_by_ref();
        }
    }
}

impl<T: Debug> Debug for RwLock<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLock")
            .field("state", &self.state)
            .field("current", &self.current)
            .field("waker", &self.waker)
            .field("data", &self.data)
            .field("readers", &self.readers)
            .finish()
    }
}

impl<T: Debug> Debug for RwLockWriteGuardFuture<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLockWriteGuardFuture")
            .field("mutex", &self.mutex)
            .field("id", &self.id)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for RwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLockWriteGuard")
            .field("mutex", &self.mutex)
            .finish()
    }
}

impl<T: Debug> Debug for RwLockWriteOwnedGuardFuture<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLockWriteOwnedGuardFuture")
            .field("mutex", &self.mutex)
            .field("id", &self.id)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for RwLockWriteOwnedGuard<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLockWriteOwnedGuard")
            .field("mutex", &self.mutex)
            .finish()
    }
}

impl<T: Debug> Debug for RwLockReadGuardFuture<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLockReadGuardFuture")
            .field("mutex", &self.mutex)
            .field("id", &self.id)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for RwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLockReadGuard")
            .field("mutex", &self.mutex)
            .finish()
    }
}

impl<T: Debug> Debug for RwLockReadOwnedGuardFuture<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLockReadOwnedGuardFuture")
            .field("mutex", &self.mutex)
            .field("id", &self.id)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for RwLockReadOwnedGuard<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLockReadOwnedGuard")
            .field("mutex", &self.mutex)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard, RwLockWriteOwnedGuard};
    use futures::{FutureExt, StreamExt, TryStreamExt};
    use std::ops::AddAssign;
    use std::sync::atomic::AtomicUsize;
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
    async fn test_overflow() {
        let mut c = RwLock::new(String::from("lol"));

        c.state = AtomicUsize::new(usize::max_value());
        c.current = AtomicUsize::new(usize::max_value());

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
}
