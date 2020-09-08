use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

/// An async `unordered` mutex.
/// It will be works with any async runtime in `Rust`, it may be a `tokio`, `smol`, `async-std` and etc..
///
/// The main difference with the standard `Mutex` is unordered mutex will not check an ordering of blocking.
/// This way is much faster, but there are some risks what someone mutex lock will be executed much later.
pub struct UnorderedMutex<T: ?Sized> {
    is_acquired: AtomicBool,
    waker: AtomicPtr<Waker>,
    data: UnsafeCell<T>,
}

impl<T> UnorderedMutex<T> {
    #[inline]
    pub const fn new(data: T) -> UnorderedMutex<T> {
        UnorderedMutex {
            is_acquired: AtomicBool::new(false),
            waker: AtomicPtr::new(null_mut()),
            data: UnsafeCell::new(data),
        }
    }

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
    pub const fn lock(&self) -> UnorderedMutexGuardFuture<T> {
        UnorderedMutexGuardFuture {
            mutex: &self,
            is_realized: false,
        }
    }

    /// Acquires the mutex.
    ///
    /// Returns a guard that releases the mutex and wake the next locker when dropped.
    /// `MutexOwnedGuard` have a `'static` lifetime, but requires the `Arc<Mutex<T>>` type
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
    pub fn lock_owned(self: &Arc<Self>) -> UnorderedMutexOwnedGuardFuture<T> {
        UnorderedMutexOwnedGuardFuture {
            mutex: self.clone(),
            is_realized: false,
        }
    }
}

/// The Simple Mutex Guard
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally borrows the Mutex, so the mutex will not be dropped while a guard exists.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
pub struct UnorderedMutexGuard<'a, T: ?Sized> {
    mutex: &'a UnorderedMutex<T>,
}

pub struct UnorderedMutexGuardFuture<'a, T: ?Sized> {
    mutex: &'a UnorderedMutex<T>,
    is_realized: bool,
}

/// An owned handle to a held Mutex.
/// This guard is only available from a Mutex that is wrapped in an `Arc`. It is identical to `MutexGuard`, except that rather than borrowing the `Mutex`, it clones the `Arc`, incrementing the reference count. This means that unlike `MutexGuard`, it will have the `'static` lifetime.
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally keeps a reference-couned pointer to the original `Mutex`, so even if the lock goes away, the guard remains valid.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
pub struct UnorderedMutexOwnedGuard<T: ?Sized> {
    mutex: Arc<UnorderedMutex<T>>,
}

pub struct UnorderedMutexOwnedGuardFuture<T: ?Sized> {
    mutex: Arc<UnorderedMutex<T>>,
    is_realized: bool,
}

unsafe impl<T: ?Sized + Send> Send for UnorderedMutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for UnorderedMutex<T> {}

unsafe impl<T: ?Sized + Send> Send for UnorderedMutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Send> Sync for UnorderedMutexGuard<'_, T> {}

unsafe impl<T: ?Sized + Send> Send for UnorderedMutexOwnedGuard<T> {}
unsafe impl<T: ?Sized + Send> Sync for UnorderedMutexOwnedGuard<T> {}

impl<'a, T: ?Sized> Future for UnorderedMutexGuardFuture<'a, T> {
    type Output = UnorderedMutexGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.mutex.is_acquired.swap(true, Ordering::AcqRel) {
            self.is_realized = true;
            Poll::Ready(UnorderedMutexGuard { mutex: self.mutex })
        } else {
            self.mutex
                .waker
                .swap(cx.waker() as *const Waker as *mut Waker, Ordering::AcqRel);
            Poll::Pending
        }
    }
}

impl<T: ?Sized> Future for UnorderedMutexOwnedGuardFuture<T> {
    type Output = UnorderedMutexOwnedGuard<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.mutex.is_acquired.swap(true, Ordering::AcqRel) {
            self.is_realized = true;
            Poll::Ready(UnorderedMutexOwnedGuard {
                mutex: self.mutex.clone(),
            })
        } else {
            self.mutex
                .waker
                .swap(cx.waker() as *const Waker as *mut Waker, Ordering::AcqRel);
            Poll::Pending
        }
    }
}

impl<T: ?Sized> Deref for UnorderedMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for UnorderedMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Deref for UnorderedMutexOwnedGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for UnorderedMutexOwnedGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for UnorderedMutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.is_acquired.store(false, Ordering::Release);

        wake_ptr(&self.mutex.waker)
    }
}

impl<T: ?Sized> Drop for UnorderedMutexOwnedGuard<T> {
    fn drop(&mut self) {
        self.mutex.is_acquired.store(false, Ordering::Release);

        wake_ptr(&self.mutex.waker)
    }
}

impl<T: ?Sized> Drop for UnorderedMutexGuardFuture<'_, T> {
    fn drop(&mut self) {
        if !self.is_realized {
            self.mutex.is_acquired.store(false, Ordering::SeqCst);

            wake_ptr(&self.mutex.waker)
        }
    }
}

impl<T: ?Sized> Drop for UnorderedMutexOwnedGuardFuture<T> {
    fn drop(&mut self) {
        if !self.is_realized {
            self.mutex.is_acquired.store(false, Ordering::SeqCst);

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

impl<T: Debug> Debug for UnorderedMutex<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mutex")
            .field("is_acquired", &self.is_acquired)
            .field("waker", &self.waker)
            .field("data", &self.data)
            .finish()
    }
}

impl<T: Debug> Debug for UnorderedMutexGuardFuture<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutexGuardFuture")
            .field("mutex", &self.mutex)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for UnorderedMutexGuard<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutexGuard")
            .field("mutex", &self.mutex)
            .finish()
    }
}

impl<T: Debug> Debug for UnorderedMutexOwnedGuardFuture<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutexOwnedGuardFuture")
            .field("mutex", &self.mutex)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for UnorderedMutexOwnedGuard<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutexOwnedGuard")
            .field("mutex", &self.mutex)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::mutex_unordered::{UnorderedMutex, UnorderedMutexGuard, UnorderedMutexOwnedGuard};
    use futures::{FutureExt, StreamExt, TryStreamExt};
    use std::ops::AddAssign;
    use std::sync::Arc;
    use tokio::time::{delay_for, Duration};

    #[tokio::test(core_threads = 12)]
    async fn test_mutex() {
        let c = UnorderedMutex::new(0);

        futures::stream::iter(0..10000)
            .for_each_concurrent(None, |_| async {
                let mut co: UnorderedMutexGuard<i32> = c.lock().await;
                *co += 1;
            })
            .await;

        let co = c.lock().await;
        assert_eq!(*co, 10000)
    }

    #[tokio::test(core_threads = 12)]
    async fn test_mutex_delay() {
        let expected_result = 100;
        let c = UnorderedMutex::new(0);

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
        let c = Arc::new(UnorderedMutex::new(0));

        futures::stream::iter(0..10000)
            .for_each_concurrent(None, |_| async {
                let mut co: UnorderedMutexOwnedGuard<i32> = c.lock_owned().await;
                *co += 1;
            })
            .await;

        let co = c.lock_owned().await;
        assert_eq!(*co, 10000)
    }

    #[tokio::test]
    async fn test_container() {
        let c = UnorderedMutex::new(String::from("lol"));

        let mut co: UnorderedMutexGuard<String> = c.lock().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[tokio::test]
    async fn test_timeout() {
        let c = UnorderedMutex::new(String::from("lol"));

        let co: UnorderedMutexGuard<String> = c.lock().await;

        futures::stream::iter(0..10000i32)
            .then(|_| tokio::time::timeout(Duration::from_nanos(1), c.lock()))
            .try_for_each_concurrent(None, |_c| futures::future::ok(()))
            .await
            .expect_err("timout must be");

        drop(co);

        let mut co: UnorderedMutexGuard<String> = c.lock().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }
}
