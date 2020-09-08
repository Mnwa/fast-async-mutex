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

    #[inline]
    pub fn write(&self) -> WriteLockGuardFuture<T> {
        WriteLockGuardFuture {
            mutex: &self,
            id: self.state.fetch_add(1, Ordering::AcqRel),
            is_realized: false,
        }
    }

    #[inline]
    pub fn write_owned(self: &Arc<Self>) -> WriteLockOwnedGuardFuture<T> {
        WriteLockOwnedGuardFuture {
            mutex: self.clone(),
            id: self.state.fetch_add(1, Ordering::AcqRel),
            is_realized: false,
        }
    }

    #[inline]
    pub fn read(&self) -> ReadLockGuardFuture<T> {
        ReadLockGuardFuture {
            mutex: &self,
            id: self.state.fetch_add(1, Ordering::AcqRel),
            is_realized: false,
        }
    }

    #[inline]
    pub fn read_owned(self: &Arc<Self>) -> ReadLockOwnedGuardFuture<T> {
        ReadLockOwnedGuardFuture {
            mutex: self.clone(),
            id: self.state.fetch_add(1, Ordering::AcqRel),
            is_realized: false,
        }
    }
}

pub struct WriteLockGuard<'a, T: ?Sized> {
    mutex: &'a RwLock<T>,
}

pub struct WriteLockGuardFuture<'a, T: ?Sized> {
    mutex: &'a RwLock<T>,
    id: usize,
    is_realized: bool,
}

pub struct WriteLockOwnedGuard<T: ?Sized> {
    mutex: Arc<RwLock<T>>,
}

pub struct WriteLockOwnedGuardFuture<T: ?Sized> {
    mutex: Arc<RwLock<T>>,
    id: usize,
    is_realized: bool,
}

pub struct ReadLockGuard<'a, T: ?Sized> {
    mutex: &'a RwLock<T>,
}

pub struct ReadLockGuardFuture<'a, T: ?Sized> {
    mutex: &'a RwLock<T>,
    id: usize,
    is_realized: bool,
}

pub struct ReadLockOwnedGuard<T: ?Sized> {
    mutex: Arc<RwLock<T>>,
}

pub struct ReadLockOwnedGuardFuture<T: ?Sized> {
    mutex: Arc<RwLock<T>>,
    id: usize,
    is_realized: bool,
}

unsafe impl<T: ?Sized + Send> Send for RwLock<T> {}
unsafe impl<T: ?Sized + Send> Sync for RwLock<T> {}

unsafe impl<T: ?Sized + Send> Send for WriteLockGuard<'_, T> {}
unsafe impl<T: ?Sized + Send> Sync for WriteLockGuard<'_, T> {}

unsafe impl<T: ?Sized + Send> Send for WriteLockOwnedGuard<T> {}
unsafe impl<T: ?Sized + Send> Sync for WriteLockOwnedGuard<T> {}

unsafe impl<T: ?Sized + Send> Send for ReadLockGuard<'_, T> {}
unsafe impl<T: ?Sized + Send> Sync for ReadLockGuard<'_, T> {}

unsafe impl<T: ?Sized + Send> Send for ReadLockOwnedGuard<T> {}
unsafe impl<T: ?Sized + Send> Sync for ReadLockOwnedGuard<T> {}

impl<'a, T: ?Sized> Future for WriteLockGuardFuture<'a, T> {
    type Output = WriteLockGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::Acquire);

        if current == self.id {
            self.is_realized = true;
            Poll::Ready(WriteLockGuard { mutex: self.mutex })
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

impl<T: ?Sized> Future for WriteLockOwnedGuardFuture<T> {
    type Output = WriteLockOwnedGuard<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::Acquire);
        if current == self.id {
            self.is_realized = true;
            Poll::Ready(WriteLockOwnedGuard {
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

impl<T: ?Sized> Deref for WriteLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for WriteLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Deref for WriteLockOwnedGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for WriteLockOwnedGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for WriteLockGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.current.fetch_add(1, Ordering::AcqRel);

        wake_ptr(&self.mutex.waker)
    }
}

impl<T: ?Sized> Drop for WriteLockOwnedGuard<T> {
    fn drop(&mut self) {
        self.mutex.current.fetch_add(1, Ordering::AcqRel);

        wake_ptr(&self.mutex.waker)
    }
}

impl<T: ?Sized> Drop for WriteLockGuardFuture<'_, T> {
    fn drop(&mut self) {
        if !self.is_realized {
            self.mutex.current.fetch_add(1, Ordering::AcqRel);

            wake_ptr(&self.mutex.waker)
        }
    }
}

impl<T: ?Sized> Drop for WriteLockOwnedGuardFuture<T> {
    fn drop(&mut self) {
        if !self.is_realized {
            self.mutex.current.fetch_add(1, Ordering::AcqRel);

            wake_ptr(&self.mutex.waker)
        }
    }
}
impl<'a, T: ?Sized> Future for ReadLockGuardFuture<'a, T> {
    type Output = ReadLockGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::Acquire);
        let readers = self.mutex.readers.load(Ordering::Acquire);
        if current + readers == self.id {
            self.is_realized = true;
            self.mutex.readers.fetch_add(1, Ordering::Release);
            Poll::Ready(ReadLockGuard { mutex: self.mutex })
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

impl<T: ?Sized> Future for ReadLockOwnedGuardFuture<T> {
    type Output = ReadLockOwnedGuard<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current = self.mutex.current.load(Ordering::Acquire);
        let readers = self.mutex.readers.load(Ordering::Acquire);
        if current + readers == self.id {
            self.is_realized = true;
            self.mutex.readers.fetch_add(1, Ordering::Release);
            Poll::Ready(ReadLockOwnedGuard {
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

impl<T: ?Sized> Deref for ReadLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> Deref for ReadLockOwnedGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for ReadLockGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.current.fetch_add(1, Ordering::AcqRel);
        self.mutex.readers.fetch_sub(1, Ordering::Release);

        wake_ptr(&self.mutex.waker)
    }
}

impl<T: ?Sized> Drop for ReadLockOwnedGuard<T> {
    fn drop(&mut self) {
        self.mutex.current.fetch_add(1, Ordering::AcqRel);
        self.mutex.readers.fetch_sub(1, Ordering::Release);

        wake_ptr(&self.mutex.waker)
    }
}

impl<T: ?Sized> Drop for ReadLockGuardFuture<'_, T> {
    fn drop(&mut self) {
        if !self.is_realized {
            self.mutex.current.fetch_add(1, Ordering::AcqRel);

            wake_ptr(&self.mutex.waker)
        }
    }
}

impl<T: ?Sized> Drop for ReadLockOwnedGuardFuture<T> {
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
        f.debug_struct("Mutex")
            .field("state", &self.state)
            .field("current", &self.current)
            .field("waker", &self.waker)
            .field("data", &self.data)
            .finish()
    }
}

impl<T: Debug> Debug for WriteLockGuardFuture<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutexGuardFuture")
            .field("mutex", &self.mutex)
            .field("id", &self.id)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for WriteLockGuard<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutexGuard")
            .field("mutex", &self.mutex)
            .finish()
    }
}

impl<T: Debug> Debug for WriteLockOwnedGuardFuture<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutexOwnedGuardFuture")
            .field("mutex", &self.mutex)
            .field("id", &self.id)
            .field("is_realized", &self.is_realized)
            .finish()
    }
}

impl<T: Debug> Debug for WriteLockOwnedGuard<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutexOwnedGuard")
            .field("mutex", &self.mutex)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::rwlock::{ReadLockGuard, RwLock, WriteLockGuard, WriteLockOwnedGuard};
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
                let mut co: WriteLockGuard<i32> = c.write().await;
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
                let mut co: WriteLockOwnedGuard<i32> = c.write_owned().await;
                *co += 1;
            })
            .await;

        let co = c.write_owned().await;
        assert_eq!(*co, 10000)
    }

    #[tokio::test]
    async fn test_container() {
        let c = RwLock::new(String::from("lol"));

        let mut co: WriteLockGuard<String> = c.write().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[tokio::test]
    async fn test_overflow() {
        let mut c = RwLock::new(String::from("lol"));

        c.state = AtomicUsize::new(usize::max_value());
        c.current = AtomicUsize::new(usize::max_value());

        let mut co: WriteLockGuard<String> = c.write().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[tokio::test]
    async fn test_timeout() {
        let c = RwLock::new(String::from("lol"));

        let co: WriteLockGuard<String> = c.write().await;

        futures::stream::iter(0..10000i32)
            .then(|_| tokio::time::timeout(Duration::from_nanos(1), c.write()))
            .try_for_each_concurrent(None, |_c| futures::future::ok(()))
            .await
            .expect_err("timout must be");

        drop(co);

        let mut co: WriteLockGuard<String> = c.write().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }

    #[tokio::test]
    async fn test_concurrent_reading() {
        let c = RwLock::new(String::from("lol"));

        let co: ReadLockGuard<String> = c.read().await;

        futures::stream::iter(0..10000i32)
            .then(|_| c.read())
            .inspect(|c| assert_eq!(*co, **c))
            .for_each_concurrent(None, |_c| futures::future::ready(()))
            .await;

        assert!(matches!(
            tokio::time::timeout(Duration::from_millis(1), c.write()).await,
            Err(_)
        ));

        let co2: ReadLockGuard<String> = c.read().await;
        assert_eq!(*co, *co2);
    }

    #[tokio::test]
    async fn test_concurrent_reading_writing() {
        let c = RwLock::new(String::from("lol"));

        let co: ReadLockGuard<String> = c.read().await;
        let co2: ReadLockGuard<String> = c.read().await;
        assert_eq!(*co, *co2);

        drop(co);
        drop(co2);

        let mut co: WriteLockGuard<String> = c.write().await;

        assert!(matches!(
            tokio::time::timeout(Duration::from_millis(1), c.read()).await,
            Err(_)
        ));

        *co += "lol";

        drop(co);

        let co: ReadLockGuard<String> = c.read().await;
        let co2: ReadLockGuard<String> = c.read().await;
        assert_eq!(*co, "lollol");
        assert_eq!(*co, *co2);
    }
}
