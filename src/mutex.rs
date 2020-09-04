use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

pub struct Mutex<T: ?Sized> {
    state: AtomicUsize,
    current: AtomicUsize,
    waker_send: Sender<Waker>,
    waker_recv: Receiver<Waker>,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    #[inline]
    pub fn new(data: T) -> Mutex<T> {
        let (waker_send, waker_recv) = channel();
        Mutex {
            state: Default::default(),
            current: Default::default(),
            data: UnsafeCell::new(data),
            waker_send,
            waker_recv,
        }
    }

    #[inline]
    pub fn lock(&self) -> MutexGuardFuture<T> {
        MutexGuardFuture {
            mutex: &self,
            id: self.state.fetch_add(1, Ordering::Acquire),
        }
    }

    #[inline]
    pub fn lock_owned(self: &Arc<Self>) -> MutexOwnedGuardFuture<T> {
        MutexOwnedGuardFuture {
            mutex: self.clone(),
            id: self.state.fetch_add(1, Ordering::Acquire),
        }
    }
}

pub struct MutexGuard<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
}

pub struct MutexGuardFuture<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
    id: usize,
}

pub struct MutexOwnedGuard<T: ?Sized> {
    mutex: Arc<Mutex<T>>,
}

pub struct MutexOwnedGuardFuture<T: ?Sized> {
    mutex: Arc<Mutex<T>>,
    id: usize,
}

unsafe impl<T: ?Sized + Send> Send for MutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Send> Sync for MutexGuard<'_, T> {}

unsafe impl<T: ?Sized + Send> Send for MutexOwnedGuard<T> {}
unsafe impl<T: ?Sized + Send> Sync for MutexOwnedGuard<T> {}

impl<'a, T: ?Sized> Future for MutexGuardFuture<'a, T> {
    type Output = MutexGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.current.load(Ordering::Relaxed) == self.id {
            Poll::Ready(MutexGuard { mutex: self.mutex })
        } else {
            if self.mutex.current.load(Ordering::Relaxed) == self.id - 1 {
                self.mutex
                    .waker_send
                    .send(cx.waker().clone())
                    .expect("cannot to send waker");
            }
            Poll::Pending
        }
    }
}

impl<T: ?Sized> Future for MutexOwnedGuardFuture<T> {
    type Output = MutexOwnedGuard<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.current.load(Ordering::Relaxed) == self.id {
            Poll::Ready(MutexOwnedGuard {
                mutex: self.mutex.clone(),
            })
        } else {
            if self.mutex.current.load(Ordering::Relaxed) == self.id - 1 {
                self.mutex
                    .waker_send
                    .send(cx.waker().clone())
                    .expect("cannot to send waker");
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
        self.mutex.current.fetch_add(1, Ordering::Acquire);

        if let Ok(waker) = self.mutex.waker_recv.try_recv() {
            waker.wake();
        }
    }
}

impl<T: ?Sized> Drop for MutexOwnedGuard<T> {
    fn drop(&mut self) {
        self.mutex.current.fetch_add(1, Ordering::Acquire);

        if let Ok(waker) = self.mutex.waker_recv.try_recv() {
            waker.wake();
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
    use futures::{FutureExt, StreamExt};
    use std::ops::AddAssign;
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::time::{delay_for, Duration};

    #[tokio::test(core_threads = 12)]
    async fn test_mutex() {
        let expected_result = 100;
        let c = Mutex::new(0);

        let ins = Instant::now();

        futures::stream::iter(0..expected_result)
            .then(|i| c.lock().map(move |co| (i, co)))
            .for_each_concurrent(None, |(i, mut co)| async move {
                delay_for(Duration::from_millis(expected_result - i)).await;
                *co += 1;
            })
            .await;

        println!("{:?}", ins.elapsed());

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
}
