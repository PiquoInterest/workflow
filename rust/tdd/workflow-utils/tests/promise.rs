use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use workflow_utils::{Deferred, once};

#[test]
fn creates_a_deferred_value_with_resolvers() {
    let deferred = Deferred::<String>::new();
    assert!(deferred.is_pending());
}

#[test]
fn resolves_the_deferred_value() {
    let deferred = Deferred::<String>::new();
    deferred.resolve("test".to_owned());
    assert!(!deferred.is_pending());
    assert_eq!(deferred.wait().unwrap(), "test");
}

#[test]
fn rejects_the_deferred_value() {
    let deferred = Deferred::<String>::new();
    deferred.reject("test error");
    assert!(!deferred.is_pending());
    assert_eq!(deferred.wait().unwrap_err(), "test error");
}

#[test]
fn lazy_once_calls_the_factory_only_once() {
    let call_count = Cell::new(0);
    let value = once(|| {
        call_count.set(call_count.get() + 1);
        "result".to_owned()
    });

    assert_eq!(value.value().as_str(), "result");
    assert_eq!(call_count.get(), 1);

    assert_eq!(value.value().as_str(), "result");
    assert_eq!(call_count.get(), 1);
}

#[test]
fn lazy_once_caches_the_result() {
    let next = Cell::new(0_u64);
    let value = once(|| {
        next.set(next.get() + 1);
        next.get()
    });
    let first = *value.value();
    let second = *value.value();
    assert_eq!(first, second);
    assert_eq!(next.get(), 1);
}

#[test]
fn only_the_first_settlement_changes_the_result() {
    let resolved = Deferred::<String>::new();
    resolved.resolve("first".to_owned());
    resolved.reject("late rejection");
    assert_eq!(resolved.wait().unwrap(), "first");

    let rejected = Deferred::<String>::new();
    rejected.reject("first error");
    rejected.resolve("late value".to_owned());
    assert_eq!(rejected.wait().unwrap_err(), "first error");
}

#[test]
fn a_cloned_resolver_can_settle_a_waiting_value() {
    let deferred = Deferred::<String>::new();
    let resolver = deferred.clone();
    let worker = std::thread::spawn(move || resolver.resolve("from worker".to_owned()));

    assert_eq!(deferred.wait().unwrap(), "from worker");
    worker.join().expect("resolver worker must not panic");
}

#[test]
fn lazy_once_retries_after_a_panicking_initializer() {
    let attempts = Cell::new(0_u32);
    let value = once(|| {
        attempts.set(attempts.get() + 1);
        assert!(attempts.get() > 1, "first initialization fails");
        attempts.get()
    });

    let first = catch_unwind(AssertUnwindSafe(|| {
        let _ = value.value();
    }));
    assert!(first.is_err());
    assert_eq!(*value.value(), 2);
    assert_eq!(*value.value(), 2);
    assert_eq!(attempts.get(), 2);
}
