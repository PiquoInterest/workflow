use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use workflow_utils::GlobalSingletonRegistry;

const NAME: &str = "@workflow/utils//globalSingletonTest";

fn case_name(case: &str) -> String {
    format!("{NAME}/{case}")
}

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
    let name = case_name("repeated-calls");
    let registry = GlobalSingletonRegistry::new();
    let first = registry.global_singleton(&name, 1, || CounterState { calls: 0 });
    let second = registry.global_singleton(&name, 1, || CounterState { calls: 0 });
    assert!(Arc::ptr_eq(&first, &second));
    registry.reset_for_test(&name, 1);
}

#[test]
fn runs_the_factory_exactly_once() {
    let name = case_name("factory-once");
    let registry = GlobalSingletonRegistry::new();
    let factory_runs = AtomicUsize::new(0);

    for _ in 0..3 {
        registry.global_singleton(&name, 1, || CounterState {
            calls: factory_runs.fetch_add(1, Ordering::SeqCst) + 1,
        });
    }

    assert_eq!(factory_runs.load(Ordering::SeqCst), 1);
    registry.reset_for_test(&name, 1);
}

#[test]
fn mutations_are_visible_to_every_holder() {
    let name = case_name("shared-mutations");
    let registry = GlobalSingletonRegistry::new();
    let copy_a =
        registry.global_singleton(&name, 1, || Mutex::new(TransportState::default()));
    let copy_b =
        registry.global_singleton(&name, 1, || Mutex::new(TransportState::default()));

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
    registry.reset_for_test(&name, 1);
}

#[test]
fn reaches_across_registry_handles_through_the_process_registry() {
    let name = case_name("process-registry");
    let registry_a = GlobalSingletonRegistry::new();
    let registry_b = GlobalSingletonRegistry::new();
    let created = registry_a.global_singleton(&name, 1, || ShapeState { shape: "shared" });
    let from_registry = registry_b
        .get::<ShapeState>(&name, 1)
        .expect("well-known name/version key must expose the shared state");

    assert!(Arc::ptr_eq(&created, &from_registry));
    registry_a.reset_for_test(&name, 1);
}

#[test]
fn different_shape_versions_do_not_share_state() {
    let name = case_name("shape-versions");
    let registry = GlobalSingletonRegistry::new();
    let v1 = registry.global_singleton(&name, 1, || ShapeState { shape: "old" });
    let v2 = registry.global_singleton(&name, 2, || ShapeState { shape: "new" });

    assert!(!Arc::ptr_eq(&v1, &v2));
    assert_eq!(v1.shape, "old");
    assert_eq!(v2.shape, "new");
    registry.reset_for_test(&name, 1);
    registry.reset_for_test(&name, 2);
}

#[test]
fn different_names_do_not_share_state() {
    let base = case_name("different-names");
    let name_a = format!("{base}/a");
    let name_b = format!("{base}/b");
    let registry = GlobalSingletonRegistry::new();
    let a = registry.global_singleton(&name_a, 1, || ShapeState { shape: "a" });
    let b = registry.global_singleton(&name_b, 1, || ShapeState { shape: "b" });

    assert!(!Arc::ptr_eq(&a, &b));
    registry.reset_for_test(&name_a, 1);
    registry.reset_for_test(&name_b, 1);
}

#[test]
fn reset_makes_the_next_call_build_a_fresh_object() {
    let name = case_name("reset-fresh");
    let registry = GlobalSingletonRegistry::new();
    let before = registry.global_singleton(&name, 1, || ShapeState { shape: "first" });

    registry.reset_for_test(&name, 1);
    let after = registry.global_singleton(&name, 1, || ShapeState { shape: "second" });

    assert!(!Arc::ptr_eq(&before, &after));
    assert_eq!(after.shape, "second");
    registry.reset_for_test(&name, 1);
}

#[test]
fn reset_only_clears_the_named_version() {
    let name = case_name("reset-version");
    let registry = GlobalSingletonRegistry::new();
    let v1 = registry.global_singleton(&name, 1, || ShapeState { shape: "old" });
    let v2 = registry.global_singleton(&name, 2, || ShapeState { shape: "new" });

    registry.reset_for_test(&name, 1);

    let rebuilt = registry.global_singleton(&name, 1, || ShapeState { shape: "rebuilt" });
    let untouched = registry.global_singleton(&name, 2, || ShapeState { shape: "unused" });
    assert!(!Arc::ptr_eq(&rebuilt, &v1));
    assert!(Arc::ptr_eq(&untouched, &v2));
    registry.reset_for_test(&name, 1);
    registry.reset_for_test(&name, 2);
}

#[test]
fn reset_is_a_no_op_when_nothing_was_created() {
    let name = case_name("reset-no-op");
    let registry = GlobalSingletonRegistry::new();
    registry.reset_for_test(&name, 1);
}

#[test]
fn concurrent_callers_run_the_factory_once() {
    let name = Arc::new(case_name("concurrent-factory"));
    let registry = Arc::new(GlobalSingletonRegistry::new());
    let factory_runs = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(8));

    let handles = (0..8)
        .map(|_| {
            let name = Arc::clone(&name);
            let registry = Arc::clone(&registry);
            let factory_runs = Arc::clone(&factory_runs);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                registry.global_singleton(name.as_str(), 1, || {
                    factory_runs.fetch_add(1, Ordering::SeqCst);
                    42_u64
                })
            })
        })
        .collect::<Vec<_>>();

    let values = handles
        .into_iter()
        .map(|handle| handle.join().expect("singleton worker must not panic"))
        .collect::<Vec<_>>();

    assert_eq!(factory_runs.load(Ordering::SeqCst), 1);
    for value in &values[1..] {
        assert!(Arc::ptr_eq(&values[0], value));
    }
    registry.reset_for_test(name.as_str(), 1);
}

#[test]
fn a_key_reused_with_a_different_type_fails_closed() {
    let name = case_name("type-mismatch");
    let registry = GlobalSingletonRegistry::new();
    let _number = registry.global_singleton(&name, 1, || 7_u64);

    assert!(registry.get::<String>(&name, 1).is_none());
    let mismatch = std::panic::catch_unwind(|| {
        registry.global_singleton(&name, 1, || "wrong type".to_owned());
    });
    assert!(mismatch.is_err());
    registry.reset_for_test(&name, 1);
}
