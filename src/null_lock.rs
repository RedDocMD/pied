use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

/// This is a cheating Mutex!
/// It doesn't actually do any mutual exclusion - merely
/// satisfies the Rust typesystem.
/// Switch to spin when MMU is set up!
pub struct Mutex<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

// On a multi-core system, this wouldn't work!
unsafe impl<T> Send for Mutex<T> where T: Send + ?Sized {}
unsafe impl<T> Sync for Mutex<T> where T: Send + ?Sized {}

impl<T> Mutex<T> {
    pub const fn new(val: T) -> Self {
        Self {
            data: UnsafeCell::new(val),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        MutexGuard {
            ptr: self.data.get(),
        }
    }
}

pub struct MutexGuard<T>
where
    T: ?Sized,
{
    ptr: *mut T,
}

impl<T> Deref for MutexGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for MutexGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}
