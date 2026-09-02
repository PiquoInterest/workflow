use std::cell::OnceCell;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

struct DeferredInner<T> {
    settlement: Mutex<Option<Result<T, String>>>,
    ready: Condvar,
}

impl<T> DeferredInner<T> {
    fn lock_settlement(&self) -> MutexGuard<'_, Option<Result<T, String>>> {
        self.settlement
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// A one-shot deferred value corresponding to the resolver state behind a
/// JavaScript promise.
///
/// Cloning creates another resolver handle. Exactly one waiter consumes the
/// settled value, and only the first resolve or reject call changes the state.
pub struct Deferred<T> {
    inner: Arc<DeferredInner<T>>,
}

impl<T> Deferred<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DeferredInner {
                settlement: Mutex::new(None),
                ready: Condvar::new(),
            }),
        }
    }

    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.inner.lock_settlement().is_none()
    }

    /// Resolves the deferred value unless it was already settled.
    pub fn resolve(&self, value: T) {
        let mut settlement = self.inner.lock_settlement();
        if settlement.is_some() {
            return;
        }
        *settlement = Some(Ok(value));
        drop(settlement);
        self.inner.ready.notify_all();
    }

    /// Rejects the deferred value unless it was already settled.
    pub fn reject(&self, reason: impl Into<String>) {
        let reason = reason.into();
        let mut settlement = self.inner.lock_settlement();
        if settlement.is_some() {
            return;
        }
        *settlement = Some(Err(reason));
        drop(settlement);
        self.inner.ready.notify_all();
    }

    /// Blocks until the value is settled, then consumes the one-shot result.
    pub fn wait(self) -> Result<T, String> {
        let mut settlement = self.inner.lock_settlement();
        loop {
            if let Some(result) = settlement.take() {
                return result;
            }
            settlement = self
                .inner
                .ready
                .wait(settlement)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }
}

impl<T> Default for Deferred<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for Deferred<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> fmt::Debug for Deferred<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Deferred")
            .field("pending", &self.is_pending())
            .finish_non_exhaustive()
    }
}

/// Lazily computes and memoizes a value on first access.
pub struct OnceValue<T, F> {
    value: OnceCell<T>,
    initializer: F,
}

/// Creates a lazily evaluated, memoized value.
///
/// `F` is callable more than once so a panicking initialization remains
/// retryable, matching the TypeScript getter, which is replaced only after a
/// successful return.
#[must_use]
pub fn once<T, F>(initializer: F) -> OnceValue<T, F>
where
    F: Fn() -> T,
{
    OnceValue {
        value: OnceCell::new(),
        initializer,
    }
}

impl<T, F> OnceValue<T, F>
where
    F: Fn() -> T,
{
    #[must_use]
    pub fn value(&self) -> &T {
        self.value.get_or_init(|| (self.initializer)())
    }
}

impl<T, F> fmt::Debug for OnceValue<T, F> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OnceValue")
            .field("initialized", &self.value.get().is_some())
            .finish_non_exhaustive()
    }
}
