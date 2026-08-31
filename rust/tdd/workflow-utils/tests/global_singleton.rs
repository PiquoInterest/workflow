use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use workflow_utils_tdd::GlobalSingletonRegistry;

const NAME: &str = "@workflow/utils//globalSingletonTest";

#[derive(Debug, PartialEq, Eq)]
struct CounterState {
    calls: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct ShapeState {
    shape: &'static str,
}

#[derive(Debug, Default)]
struct TransportState {
    transports: BTreeMap<String, String>,
}

#[test]
fn returns_the_same_object_for_repeated_calls() {
    let registry = GlobalSingletonRegistry::new();
    let first = registry.global_singleton(NAME, 1, || CounterState { calls: 0 });
    let second = registry.global_singleton(NAME, 1, || CounterState { calls: 0 });
    assert!(Arc::ptr_eq(&first, &second));
}

#[test]
fn runs_the_factory_exactly_once() {
    let registry = GlobalSingletonRegistry::new();
    let factory_runs = AtomicUsize::new(0);

    for _ in 0..3 {
        registry.global_singleton(NAME, 1, || CounterState {
            calls: factory_runs.fetch_add(1, Ordering::SeqCst) + 1,
        });
    }

    assert_eq!(factory_runs.load(Ordering::SeqCst), 1);
}

#[test]
fn mutations_are_visible_to_every_holder() {
    let registry = GlobalSingletonRegistry::new();
    let copy_a = registry.global_singleton(NAME, 1, || Mutex::new(TransportState::default()));
    let copy_b = registry.global_singleton(NAME, 1, || Mutex::new(TransportState::default()));

    copy_a
        .lock()
        .unwrap()
        .transports
        .insert("run_1".to_owned(), "ws".to_owned());

    assert_eq!(
        copy_b
            .lock()
            .unwrap()
            .transports
            .get("run_1")
            .map(String::as_str),
        Some("ws")
    );
}

#[test]
fn reaches_across_call_sites_through_the_process_registry() {
    let registry = GlobalSingletonRegistry::new();
    let created = registry.global_singleton(NAME, 1, || ShapeState { shape: "shared" });
    let from_registry = registry
        .get::<ShapeState>(NAME, 1)
        .expect("well-known name/version key must expose the shared state");

    assert!(Arc::ptr_eq(&created, &from_registry));
}

#[test]
fn different_shape_versions_do_not_share_state() {
    let registry = GlobalSingletonRegistry::new();
    let v1 = registry.global_singleton(NAME, 1, || ShapeState { shape: "old" });
    let v2 = registry.global_singleton(NAME, 2, || ShapeState { shape: "new" });

    assert!(!Arc::ptr_eq(&v1, &v2));
    assert_eq!(v1.shape, "old");
    assert_eq!(v2.shape, "new");
}

#[test]
fn different_names_do_not_share_state() {
    let registry = GlobalSingletonRegistry::new();
    let a = registry.global_singleton(&format!("{NAME}/a"), 1, || ShapeState { shape: "a" });
    let b = registry.global_singleton(&format!("{NAME}/b"), 1, || ShapeState { shape: "b" });

    assert!(!Arc::ptr_eq(&a, &b));
    registry.reset_for_test(&format!("{NAME}/a"), 1);
    registry.reset_for_test(&format!("{NAME}/b"), 1);
}

#[test]
fn reset_makes_the_next_call_build_a_fresh_object() {
    let registry = GlobalSingletonRegistry::new();
    let before = registry.global_singleton(NAME, 1, || ShapeState { shape: "first" });

    registry.reset_for_test(NAME, 1);
    let after = registry.global_singleton(NAME, 1, || ShapeState { shape: "second" });

    assert!(!Arc::ptr_eq(&before, &after));
    assert_eq!(after.shape, "second");
}

#[test]
fn reset_only_clears_the_named_version() {
    let registry = GlobalSingletonRegistry::new();
    let v1 = registry.global_singleton(NAME, 1, || ShapeState { shape: "old" });
    let v2 = registry.global_singleton(NAME, 2, || ShapeState { shape: "new" });

    registry.reset_for_test(NAME, 1);

    let rebuilt = registry.global_singleton(NAME, 1, || ShapeState { shape: "rebuilt" });
    let untouched = registry.global_singleton(NAME, 2, || ShapeState { shape: "unused" });
    assert!(!Arc::ptr_eq(&rebuilt, &v1));
    assert!(Arc::ptr_eq(&untouched, &v2));
}

#[test]
fn reset_is_a_no_op_when_nothing_was_created() {
    let registry = GlobalSingletonRegistry::new();
    registry.reset_for_test(NAME, 1);
}
