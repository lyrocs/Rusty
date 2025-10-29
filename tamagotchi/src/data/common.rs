/// Common data types and utilities
///
/// Lazy initialization pattern and shared data structures.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

/// Lazy-initialized static data structure
pub struct LazyData<T> {
    initialized: AtomicBool,
    data: UnsafeCell<Option<T>>,
}

unsafe impl<T> Sync for LazyData<T> {}

impl<T> LazyData<T> {
    pub const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            data: UnsafeCell::new(None),
        }
    }

    pub fn get_or_init<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if !self.initialized.load(Ordering::Acquire) {
            unsafe {
                *self.data.get() = Some(init());
            }
            self.initialized.store(true, Ordering::Release);
        }
        unsafe { (*self.data.get()).as_ref().unwrap() }
    }
}
