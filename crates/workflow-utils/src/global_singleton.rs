use std::any::Any;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

type ErasedValue = dyn Any + Send + Sync;
type RegistryStore = BTreeMap<RegistryKey, Arc<RegistrySlot>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RegistryKey {
    name: String,
    shape_version: u32,
}

impl RegistryKey {
    fn new(name: &str, shape_version: u32) -> Self {
        Self {
            name: name.to_owned(),
            shape_version,
        }
    }
}

#[derive(Default)]
struct RegistrySlot {
    value: OnceLock<Arc<ErasedValue>>,
}

impl fmt::Debug for RegistrySlot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RegistrySlot")
            .field("initialized", &self.value.get().is_some())
            .finish()
    }
}

static PROCESS_REGISTRY: OnceLock<Mutex<RegistryStore>> = OnceLock::new();

fn process_registry() -> &'static Mutex<RegistryStore> {
    PROCESS_REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn lock_registry() -> MutexGuard<'static, RegistryStore> {
    process_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A handle to the process-wide, versioned singleton registry.
///
/// Every handle uses the same backing store. Values are keyed by both their
/// stable name and shape version, mirroring the TypeScript `Symbol.for()`
/// contract while retaining Rust's runtime type checks.
#[derive(Debug, Default, Clone, Copy)]
pub struct GlobalSingletonRegistry;

impl GlobalSingletonRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Returns the existing value for `name` and `shape_version`, or initializes
    /// it exactly once for the process.
    ///
    /// Initialization is coordinated per key, so unrelated keys do not hold the
    /// registry lock while user code runs. A key reused with a different Rust
    /// type fails closed rather than exposing a type-confused value.
    pub fn global_singleton<T, F>(&self, name: &str, shape_version: u32, create: F) -> Arc<T>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        let key = RegistryKey::new(name, shape_version);
        let slot = {
            let mut registry = lock_registry();
            Arc::clone(
                registry
                    .entry(key)
                    .or_insert_with(|| Arc::new(RegistrySlot::default())),
            )
        };

        let erased = Arc::clone(slot.value.get_or_init(|| {
            let created: Arc<ErasedValue> = Arc::new(create());
            created
        }));

        Arc::downcast::<T>(erased).unwrap_or_else(|_| {
            panic!(
                "global singleton `{name}/v{shape_version}` was initialized with a different Rust type"
            )
        })
    }

    /// Reads an initialized singleton without running a factory.
    #[must_use]
    pub fn get<T>(&self, name: &str, shape_version: u32) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let key = RegistryKey::new(name, shape_version);
        let slot = {
            let registry = lock_registry();
            registry.get(&key).cloned()
        }?;
        let erased = Arc::clone(slot.value.get()?);
        Arc::downcast::<T>(erased).ok()
    }

    /// Removes one name/version entry so its next access creates fresh state.
    ///
    /// Existing `Arc` holders remain valid, matching the TypeScript test seam's
    /// orphan-reference behavior.
    pub fn reset_for_test(&self, name: &str, shape_version: u32) {
        lock_registry().remove(&RegistryKey::new(name, shape_version));
    }
}
