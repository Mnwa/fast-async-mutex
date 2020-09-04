use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

/// An async mutex.
/// It will be works with any async runtime in Rust, it may be a Tokio, Smoll, Async-std etc..
pub struct Mutex<T: ?Sized> {
    state: AtomicUsize,
    current: AtomicUsize,
    waker: AtomicPtr<Waker>,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    #[inline]
    pub fn new(data: T) -> Mutex<T> {
        Mutex {
            state: Default::default(),
            current: Default::default(),
            waker: Default::default(),
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
    pub fn lock(&self) -> MutexGuardFuture<T> {
        MutexGuardFuture {
            mutex: &self,
            id: self.state.fetch_add(1, Ordering::SeqCst),
            is_realized: Default::default(),
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
    pub fn lock_owned(self: &Arc<Self>) -> MutexOwnedGuardFuture<T> {
        MutexOwnedGuardFuture {
            mutex: self.clone(),
            id: self.state.fetch_add(1, Ordering::SeqCst),
            is_realized: Default::default(),
        }
    }
}

/// The Simple Mutex Guard
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally borrows the Mutex, so the mutex will not be dropped while a guard exists.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
pub struct MutexGuard<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
}

pub struct MutexGuardFuture<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
    id: usize,
    is_realized: AtomicBool,
}

/// An owned handle to a held Mutex.
/// This guard is only available from a Mutex that is wrapped in an `Arc`. It is identical to `MutexGuard`, except that rather than borrowing the `Mutex`, it clones the `Arc`, incrementing the reference count. This means that unlike `MutexGuard`, it will have the `'static` lifetime.
/// As long as you have this guard, you have exclusive access to the underlying `T`. The guard internally keeps a reference-couned pointer to the original `Mutex`, so even if the lock goes away, the guard remains valid.
/// The lock is automatically released and waked the next locker whenever the guard is dropped, at which point lock will succeed yet again.
pub struct MutexOwnedGuard<T: ?Sized> {
    mutex: Arc<Mutex<T>>,
}

pub struct MutexOwnedGuardFuture<T: ?Sized> {
    mutex: Arc<Mutex<T>>,
    id: usize,
    is_realized: AtomicBool,
}

unsafe impl<T: ?Sized + Send> Send for MutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Send> Sync for MutexGuard<'_, T> {}

unsafe impl<T: ?Sized + Send> Send for MutexOwnedGuard<T> {}
unsafe impl<T: ?Sized + Send> Sync for MutexOwnedGuard<T> {}

impl<'a, T: ?Sized> Future for MutexGuardFuture<'a, T> {
    type Output = MutexGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::SeqCst);
        if current == self.id {
            self.is_realized.store(true, Ordering::SeqCst);
            Poll::Ready(MutexGuard { mutex: self.mutex })
        } else {
            if Some(current) == self.id.checked_sub(1) {
                let _ = self.mutex.waker.compare_exchange_weak(
                    null_mut(),
                    Box::into_raw(Box::new(cx.waker().clone())),
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
            }
            Poll::Pending
        }
    }
}

impl<T: ?Sized> Future for MutexOwnedGuardFuture<T> {
    type Output = MutexOwnedGuard<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::SeqCst);
        if current == self.id {
            self.is_realized.store(true, Ordering::SeqCst);
            Poll::Ready(MutexOwnedGuard {
                mutex: self.mutex.clone(),
            })
        } else {
            if Some(current) == self.id.checked_sub(1) {
                let _ = self.mutex.waker.compare_exchange_weak(
                    null_mut(),
                    Box::into_raw(Box::new(cx.waker().clone())),
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
            }

            Poll::Pending
        }
    }
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.mutex.data.get().as_ref().expect("mutex: empty ptr") }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.mutex.data.get().as_mut().expect("mutex: empty ptr") }
    }
}

impl<T: ?Sized> Deref for MutexOwnedGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.mutex.data.get().as_ref().expect("mutex: empty ptr") }
    }
}

impl<T: ?Sized> DerefMut for MutexOwnedGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.mutex.data.get().as_mut().expect("mutex: empty ptr") }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.current.fetch_add(1, Ordering::SeqCst);

        let waker_ptr = self.mutex.waker.swap(null_mut(), Ordering::SeqCst);
        if !waker_ptr.is_null() {
            unsafe {
                let waker = waker_ptr.read();
                waker.wake();
            }
        }
    }
}

impl<T: ?Sized> Drop for MutexOwnedGuard<T> {
    fn drop(&mut self) {
        self.mutex.current.fetch_add(1, Ordering::SeqCst);

        let waker_ptr = self.mutex.waker.swap(null_mut(), Ordering::SeqCst);
        if !waker_ptr.is_null() {
            unsafe {
                let waker = waker_ptr.read();
                waker.wake();
            }
        }
    }
}

impl<T: ?Sized> Drop for MutexGuardFuture<'_, T> {
    fn drop(&mut self) {
        if !self.is_realized.load(Ordering::SeqCst) {
            self.mutex.current.fetch_add(1, Ordering::SeqCst);
        }
    }
}

impl<T: ?Sized> Drop for MutexOwnedGuardFuture<T> {
    fn drop(&mut self) {
        if !self.is_realized.load(Ordering::SeqCst) {
            self.mutex.current.fetch_add(1, Ordering::SeqCst);
        }
    }
}

impl<T: Debug> Debug for Mutex<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mutex")
            .field("state", &self.state)
            .field("current", &self.current)
            .field("waker", &self.waker)
            .field("data", &self.data)
            .finish()
    }
}

impl<T: Debug> Debug for MutexGuardFuture<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutexGuardFuture")
            .field("mutex", &self.mutex)
            .field("id", &self.id)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutexGuardFuture")
            .field("mutex", &self.mutex)
            .finish()
    }
}

impl<T: Debug> Debug for MutexOwnedGuardFuture<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutexGuardFuture")
            .field("mutex", &self.mutex)
            .field("id", &self.id)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for MutexOwnedGuard<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutexGuardFuture")
            .field("mutex", &self.mutex)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::mutex::{Mutex, MutexGuard, MutexOwnedGuard};
    use futures::{FutureExt, StreamExt, TryStreamExt};
    use std::ops::AddAssign;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;
    use tokio::time::{delay_for, Duration};

    #[tokio::test(core_threads = 12)]
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

    #[tokio::test(core_threads = 12)]
    async fn test_mutex_delay() {
        let expected_result = 100;
        let c = Mutex::new(0);

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
    async fn test_overflow() {
        let mut c = Mutex::new(String::from("lol"));

        c.state = AtomicUsize::new(usize::max_value());
        c.current = AtomicUsize::new(usize::max_value());

        let mut co: MutexGuard<String> = c.lock().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[tokio::test]
    async fn test_timeout() {
        let mut c = Mutex::new(String::from("lol"));

        c.state = AtomicUsize::new(usize::max_value());
        c.current = AtomicUsize::new(usize::max_value());

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
}
