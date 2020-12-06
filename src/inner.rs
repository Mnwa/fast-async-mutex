use std::cell::UnsafeCell;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
use std::task::Waker;

#[derive(Debug)]
pub(crate) struct Inner<T: ?Sized> {
    is_acquired: AtomicBool,
    pub(crate) data: UnsafeCell<T>,
}

impl<T> Inner<T> {
    #[inline]
    pub const fn new(data: T) -> Inner<T> {
        Inner {
            is_acquired: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> Inner<T> {
    #[inline]
    pub(crate) fn unlock(&self) {
        self.is_acquired.store(false, Ordering::Release);
    }

    #[inline]
    pub(crate) fn store_waker(&self, waker: &Waker) {
        waker.wake_by_ref();
    }

    #[inline]
    pub(crate) fn try_acquire(&self) -> bool {
        self.is_acquired
            .compare_exchange_weak(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
    }
}

#[derive(Debug)]
pub(crate) struct OrderedInner<T: ?Sized> {
    pub(crate) state: AtomicUsize,
    pub(crate) current: AtomicUsize,
    pub(crate) data: UnsafeCell<T>,
}

impl<T> OrderedInner<T> {
    #[inline]
    pub const fn new(data: T) -> OrderedInner<T> {
        OrderedInner {
            state: AtomicUsize::new(0),
            current: AtomicUsize::new(0),
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> OrderedInner<T> {
    #[inline]
    pub(crate) fn generate_id(&self) -> usize {
        self.state.fetch_add(1, Ordering::Relaxed)
    }

    #[inline]
    pub(crate) fn unlock(&self) {
        self.current.fetch_add(1, Ordering::Release);
    }

    #[inline]
    pub(crate) fn store_waker(&self, waker: &Waker) {
        waker.wake_by_ref();
    }

    #[inline]
    pub(crate) fn try_acquire(&self, id: usize) -> bool {
        id == self.current.load(Ordering::Acquire)
    }
}
