#[macro_use]
mod deref {
    #[macro_export]
    macro_rules! impl_deref_mut {
        ($struct_name:ident) => {
            $crate::impl_deref!($struct_name);
            impl<T: ?Sized> std::ops::DerefMut for $struct_name<T> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut *self.mutex.data.get() }
                }
            }
        };
        ($struct_name:ident, $lifetime:lifetime) => {
            $crate::impl_deref!($struct_name, $lifetime);
            impl<$lifetime, T: ?Sized> std::ops::DerefMut for $struct_name<$lifetime, T> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut *self.mutex.data.get() }
                }
            }
        };
    }

    #[macro_export]
    macro_rules! impl_deref {
        ($struct_name:ident) => {
            impl<T: ?Sized> std::ops::Deref for $struct_name<T> {
                type Target = T;

                fn deref(&self) -> &Self::Target {
                    unsafe { &*self.mutex.data.get() }
                }
            }
        };
        ($struct_name:ident, $lifetime:lifetime) => {
            impl<$lifetime, T: ?Sized> std::ops::Deref for $struct_name<$lifetime, T> {
                type Target = T;

                fn deref(&self) -> &Self::Target {
                    unsafe { &*self.mutex.data.get() }
                }
            }
        };
    }
}

#[macro_use]
mod drop {
    #[macro_export]
    macro_rules! impl_drop_guard {
        ($struct_name:ident, $unlock_fn:ident) => {
            impl<T: ?Sized> Drop for $struct_name<T> {
                fn drop(&mut self) {
                    self.mutex.$unlock_fn()
                }
            }
        };
        ($struct_name:ident, $lifetime:lifetime, $unlock_fn:ident) => {
            impl<$lifetime, T: ?Sized> Drop for $struct_name<$lifetime, T> {
                fn drop(&mut self) {
                    self.mutex.$unlock_fn()
                }
            }
        };
    }

    #[macro_export]
    macro_rules! impl_drop_guard_future {
        ($struct_name:ident, $unlock_fn:ident) => {
            impl<T: ?Sized> Drop for $struct_name<T> {
                fn drop(&mut self) {
                    if !self.is_realized {
                        self.mutex.$unlock_fn()
                    }
                }
            }
        };
        ($struct_name:ident, $lifetime:lifetime, $unlock_fn:ident) => {
            impl<$lifetime, T: ?Sized> Drop for $struct_name<$lifetime, T> {
                fn drop(&mut self) {
                    if !self.is_realized {
                        self.mutex.$unlock_fn()
                    }
                }
            }
        };
    }
}

#[macro_use]
mod sync {
    #[macro_export]
    macro_rules! impl_send_sync_rwlock {
        ($mutex_name:ident, $read_guard:ident, $read_guard_owned:ident, $write_guard:ident, $write_guard_owned:ident) => {
            unsafe impl<T> Send for $mutex_name<T> where T: Send + ?Sized {}
            unsafe impl<T> Sync for $mutex_name<T> where T: Send + Sync + ?Sized {}

            unsafe impl<T> Send for $read_guard<'_, T> where T: ?Sized + Send {}
            unsafe impl<T> Sync for $read_guard<'_, T> where T: Send + Sync + ?Sized {}

            unsafe impl<T> Send for $read_guard_owned<T> where T: ?Sized + Send {}
            unsafe impl<T> Sync for $read_guard_owned<T> where T: Send + Sync + ?Sized {}

            unsafe impl<T> Send for $write_guard<'_, T> where T: ?Sized + Send {}
            unsafe impl<T> Sync for $write_guard<'_, T> where T: Send + Sync + ?Sized {}

            unsafe impl<T> Send for $write_guard_owned<T> where T: ?Sized + Send {}
            unsafe impl<T> Sync for $write_guard_owned<T> where T: Send + Sync + ?Sized {}
        };
    }

    #[macro_export]
    macro_rules! impl_send_sync_mutex {
        ($mutex_name:ident, $mutex_guard:ident, $mutex_guard_owned:ident) => {
            unsafe impl<T> Send for $mutex_name<T> where T: Send + ?Sized {}
            unsafe impl<T> Sync for $mutex_name<T> where T: Send + Sync + ?Sized {}

            unsafe impl<T> Send for $mutex_guard<'_, T> where T: ?Sized + Send {}
            unsafe impl<T> Sync for $mutex_guard<'_, T> where T: Send + Sync + ?Sized {}

            unsafe impl<T> Send for $mutex_guard_owned<T> where T: ?Sized + Send {}
            unsafe impl<T> Sync for $mutex_guard_owned<T> where T: Send + Sync + ?Sized {}
        };
    }
}
