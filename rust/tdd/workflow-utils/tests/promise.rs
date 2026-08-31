use std::cell::Cell;
use workflow_utils_tdd::{Deferred, once};

#[test]
fn creates_a_deferred_value_with_resolvers() {
    let deferred = Deferred::<String>::new();
    assert!(deferred.is_pending());
}

#[test]
fn resolves_the_deferred_value() {
    let deferred = Deferred::<String>::new();
    deferred.resolve("test".to_owned());
    assert_eq!(deferred.wait().unwrap(), "test");
}

#[test]
fn rejects_the_deferred_value() {
    let deferred = Deferred::<String>::new();
    deferred.reject("test error");
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
