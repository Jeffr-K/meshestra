use crate::di::Container;
use std::ops::Deref;
use std::sync::{Arc, Mutex, Once, PoisonError};

/// A wrapper for lazy-initialized services to handle circular dependencies.
///
/// `Lazy<T>` holds a reference to the DI `Container` and resolves the actual
/// service `T` only when it's first accessed. This allows breaking dependency
/// cycles during container setup.
///
/// # Panics
///
/// It will panic at runtime if the requested service `T` is not registered in the
/// container when the `Lazy<T>` is first dereferenced.
pub struct Lazy<T: 'static + Send + Sync> {
    container: Container,
    instance: Mutex<Option<Arc<T>>>,
    once: Once,
}

impl<T: 'static + Send + Sync> Lazy<T> {
    /// Creates a new `Lazy<T>`.
    ///
    /// This is typically called by the `#[derive(Injectable)]` macro.
    pub fn new(container: &Container) -> Self {
        Self {
            container: container.clone(),
            instance: Mutex::new(None),
            once: Once::new(),
        }
    }

    /// Internal method to initialize the service.
    fn init(&self) {
        self.once.call_once(|| {
            let resolved = self.container.resolve::<T>().unwrap_or_else(|e| {
                panic!(
                    "Failed to lazily resolve dependency '{}': {}",
                    std::any::type_name::<T>(),
                    e
                )
            });
            *self.instance.lock().unwrap_or_else(PoisonError::into_inner) = Some(resolved);
        });
    }
}

impl<T: 'static + Send + Sync> Deref for Lazy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Ensure the instance is initialized.
        self.init();

        // The following logic is a bit tricky to avoid holding the lock for too long.
        // We get the instance, which is an Arc. We can then safely return a reference
        // to the value inside the Arc without holding the lock.
        // `MutexGuard` will be dropped, releasing the lock.
        let guard = self.instance.lock().unwrap_or_else(PoisonError::into_inner);

        // We use `Arc::as_ptr` and unsafe code to extend the lifetime of the reference.
        // This is safe because the `Arc` is stored within the `Lazy` struct and will not be
        // dropped while the `Lazy` struct itself is alive. The reference returned by `deref`
        // cannot outlive the `Lazy` struct.
        unsafe { &*Arc::as_ptr(guard.as_ref().unwrap()) }
    }
}

impl<T: 'static + Send + Sync> Clone for Lazy<T> {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
            instance: Mutex::new(self.instance.lock().unwrap().clone()),
            once: Once::new(), // Each clone gets its own Once to handle initialization locally
        }
    }
}
