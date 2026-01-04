use std::cell::UnsafeCell;
use std::sync::Once;

/// A thread-safe lazy initialization wrapper
pub struct Lazy<T, F = Box<dyn FnOnce() -> T + Send + Sync>> {
    init: Once,
    data: UnsafeCell<Option<T>>,
    factory: UnsafeCell<Option<F>>,
}

// Safety: Lazy is Sync if T is Sync and F is Send (because F moves into T construct).
unsafe impl<T: Sync, F: Send> Sync for Lazy<T, F> {}
unsafe impl<T: Send, F: Send> Send for Lazy<T, F> {}

impl<T, F> Lazy<T, F>
where
    F: FnOnce() -> T,
{
    /// Create a new lazy value
    pub const fn new(f: F) -> Self {
        Self {
            init: Once::new(),
            data: UnsafeCell::new(None),
            factory: UnsafeCell::new(Some(f)),
        }
    }

    /// Get the value, initializing it if necessary
    pub fn get(&self) -> &T {
        self.init.call_once(|| {
            // SAFETY: this block is executed only once.
            // We can safely read/write the UnsafeCells.
            unsafe {
                let factory_ptr = self.factory.get();
                if let Some(f) = (*factory_ptr).take() {
                    let value = f();
                    *self.data.get() = Some(value);
                }
            }
        });

        // SAFETY: self.init ensures data is initialized and visible.
        unsafe {
            (*self.data.get()).as_ref().unwrap()
        }
    }
}

impl<T, F> std::ops::Deref for Lazy<T, F>
where
    F: FnOnce() -> T,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}
