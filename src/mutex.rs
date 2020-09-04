use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

pub struct Mutex<T: 'static + ?Sized> {
    state: AtomicUsize,
    current: AtomicUsize,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub fn new(data: T) -> Mutex<T> {
        Mutex {
            state: Default::default(),
            current: Default::default(),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuardFuture<T> {
        MutexGuardFuture {
            mutex: &self,
            id: self.state.fetch_add(1, Ordering::Acquire),
        }
    }

    pub fn lock_owned(self: &Arc<Self>) -> MutexOwnedGuardFuture<T> {
        MutexOwnedGuardFuture {
            mutex: self.clone(),
            id: self.state.fetch_add(1, Ordering::Acquire),
        }
    }
}

pub struct MutexGuard<'a, T: 'static + ?Sized> {
    mutex: &'a Mutex<T>,
}

unsafe impl<T: ?Sized + Send> Send for MutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Send> Sync for MutexGuard<'_, T> {}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let ptr = self.mutex.data.get();
        assert!(!ptr.is_null());

        unsafe { &*ptr }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let ptr = self.mutex.data.get();
        assert!(!ptr.is_null());

        unsafe { &mut *ptr }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.current.fetch_add(1, Ordering::Acquire);
    }
}

pub struct MutexGuardFuture<'a, T: 'static + ?Sized> {
    mutex: &'a Mutex<T>,
    id: usize,
}

impl<'a, T: ?Sized> Future for MutexGuardFuture<'a, T> {
    type Output = MutexGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.current.load(Ordering::Relaxed) == self.id {
            Poll::Ready(MutexGuard { mutex: self.mutex })
        } else {
            Poll::Pending
        }
    }
}

pub struct MutexOwnedGuard<T: 'static + ?Sized> {
    mutex: Arc<Mutex<T>>,
}

unsafe impl<T: ?Sized + Send> Send for MutexOwnedGuard<T> {}
unsafe impl<T: ?Sized + Send> Sync for MutexOwnedGuard<T> {}

impl<T: ?Sized> Deref for MutexOwnedGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let ptr = self.mutex.data.get();
        assert!(!ptr.is_null());

        unsafe { &*ptr }
    }
}

impl<T: ?Sized> DerefMut for MutexOwnedGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let ptr = self.mutex.data.get();
        assert!(!ptr.is_null());

        unsafe { &mut *ptr }
    }
}

impl<T: ?Sized> Drop for MutexOwnedGuard<T> {
    fn drop(&mut self) {
        self.mutex.current.fetch_add(1, Ordering::Acquire);
    }
}

pub struct MutexOwnedGuardFuture<T: 'static + ?Sized> {
    mutex: Arc<Mutex<T>>,
    id: usize,
}

impl<'a, T: ?Sized> Future for MutexOwnedGuardFuture<T> {
    type Output = MutexOwnedGuard<T>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.current.load(Ordering::Relaxed) == self.id {
            Poll::Ready(MutexOwnedGuard {
                mutex: self.mutex.clone(),
            })
        } else {
            Poll::Pending
        }
    }
}

impl<T: Debug> Debug for Mutex<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mutex")
            .field("state", &self.state)
            .field("current", &self.current)
            .field("data", &self.data)
            .finish()
    }
}

impl<T: Debug> Debug for MutexGuardFuture<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutexGuardFuture")
            .field("mutex", &self.mutex)
            .field("id", &self.id)
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
    use std::ops::AddAssign;
    use std::sync::Arc;

    #[tokio::test(core_threads = 12)]
    async fn test_mutex() {
        let c = Mutex::new(0);

        for _ in 0..25 {
            tokio::join!(
                async {
                    let mut co: MutexGuard<i32> = c.lock().await;
                    *co += 1;
                },
                async {
                    let mut c: MutexGuard<i32> = c.lock().await;
                    *c += 1;
                },
                async {
                    let mut c: MutexGuard<i32> = c.lock().await;
                    *c += 1;
                },
                async {
                    let mut c: MutexGuard<i32> = c.lock().await;
                    *c += 1;
                },
            );
        }

        let co = c.lock().await;
        assert_eq!(*co, 100)
    }

    #[tokio::test(core_threads = 12)]
    async fn test_owned_mutex() {
        let c = Arc::new(Mutex::new(0));

        for _ in 0..25 {
            tokio::join!(
                async {
                    let mut co: MutexOwnedGuard<i32> = c.lock_owned().await;
                    *co += 1;
                },
                async {
                    let mut c: MutexOwnedGuard<i32> = c.lock_owned().await;
                    *c += 1;
                },
                async {
                    let mut c: MutexOwnedGuard<i32> = c.lock_owned().await;
                    *c += 1;
                },
                async {
                    let mut c: MutexOwnedGuard<i32> = c.lock_owned().await;
                    *c += 1;
                },
            );
        }

        let co = c.lock_owned().await;
        assert_eq!(*co, 100)
    }

    #[tokio::test]
    async fn test_container() {
        let c = Mutex::new(String::from("lol"));

        let mut co: MutexGuard<String> = c.lock().await;
        co.add_assign("lol");

        assert_eq!(*co, "lollol");
    }
}
