use std::cell::UnsafeCell;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
use std::task::Waker;

#[derive(Debug)]
pub(crate) struct Inner<T: ?Sized> {
    is_acquired: AtomicBool,
    waker: AtomicPtr<Waker>,
    pub(crate) data: UnsafeCell<T>,
}

impl<T> Inner<T> {
    #[inline]
    pub const fn new(data: T) -> Inner<T> {
        Inner {
            is_acquired: AtomicBool::new(false),
            waker: AtomicPtr::new(null_mut()),
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> Inner<T> {
    #[inline]
    pub(crate) fn unlock(&self) {
        self.is_acquired.store(false, Ordering::SeqCst);

        self.try_wake(null_mut())
    }

    #[inline]
    pub(crate) fn store_waker(&self, waker: &Waker) {
        self.try_wake(Box::into_raw(Box::new(waker.clone())));
    }

    #[inline]
    pub(crate) fn try_wake(&self, waker_ptr: *mut Waker) {
        let waker_ptr = self.waker.swap(waker_ptr, Ordering::AcqRel);

        if !waker_ptr.is_null() {
            unsafe { Box::from_raw(waker_ptr).wake() }
        }
    }

    #[inline]
    pub(crate) fn try_acquire(&self) -> bool {
        !self.is_acquired.swap(true, Ordering::AcqRel)
    }

    #[inline]
    pub(crate) fn try_acquire_reader(&self, readers: usize) -> bool {
        readers > 0 || self.try_acquire()
    }
}

#[derive(Debug)]
pub(crate) struct OrderedInner<T: ?Sized> {
    pub(crate) state: AtomicUsize,
    pub(crate) current: AtomicUsize,
    waker: AtomicPtr<Waker>,
    pub(crate) data: UnsafeCell<T>,
}

impl<T> OrderedInner<T> {
    #[inline]
    pub const fn new(data: T) -> OrderedInner<T> {
        OrderedInner {
            state: AtomicUsize::new(0),
            current: AtomicUsize::new(0),
            waker: AtomicPtr::new(null_mut()),
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> OrderedInner<T> {
    #[inline]
    pub(crate) fn generate_id(&self) -> usize {
        self.state.fetch_add(1, Ordering::Release)
    }

    #[inline]
    pub(crate) fn unlock(&self) {
        self.current.fetch_add(1, Ordering::AcqRel);

        self.try_wake(null_mut())
    }

    #[inline]
    pub(crate) fn store_waker(&self, waker: &Waker) {
        self.try_wake(Box::into_raw(Box::new(waker.clone())));
    }

    #[inline]
    pub(crate) fn try_wake(&self, waker_ptr: *mut Waker) {
        let waker_ptr = self.waker.swap(waker_ptr, Ordering::AcqRel);

        if !waker_ptr.is_null() {
            unsafe { Box::from_raw(waker_ptr).wake() }
        }
    }

    #[inline]
    pub(crate) fn try_acquire(&self, id: usize) -> bool {
        id == self.current.load(Ordering::Acquire)
    }

    #[inline]
    pub(crate) fn try_acquire_reader(&self, id: usize, readers: usize) -> bool {
        id == self.current.load(Ordering::Acquire) + readers
    }
}
