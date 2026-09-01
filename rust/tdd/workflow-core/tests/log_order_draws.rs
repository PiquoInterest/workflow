use std::collections::{BTreeMap, BTreeSet};

use workflow_core_tdd::log_order_draws::{
    DrawMode, EntityBinding, observe_blocked_branch_draws,
};

fn by_id(bindings: &[EntityBinding]) -> BTreeMap<&str, &str> {
    bindings
        .iter()
        .map(|binding| (binding.correlation_id.as_str(), binding.entity.as_str()))
        .collect()
}

fn finalize_ids(bindings: &[EntityBinding]) -> BTreeSet<&str> {
    bindings
        .iter()
        .filter(|binding| binding.is_finalize_step())
        .map(|binding| binding.correlation_id.as_str())
        .collect()
}

#[test]
fn arrival_order_control_rebinds_a_finalize_ordinal_under_extension() {
    let observation = observe_blocked_branch_draws(DrawMode::ArrivalOrder);
    let fresh_ids: BTreeSet<&str> = observation
        .longer
        .iter()
        .map(|binding| binding.correlation_id.as_str())
        .collect();
    let rebound = observation
        .shorter
        .iter()
        .filter(|binding| binding.is_finalize_step())
        .filter(|binding| !fresh_ids.contains(binding.correlation_id.as_str()))
        .count();
    assert!(rebound > 0);
}

#[test]
fn log_order_keeps_shared_bindings_and_finalize_ids_stable_under_extension() {
    let observation = observe_blocked_branch_draws(DrawMode::LogOrder);
    let stale = by_id(&observation.shorter);
    let fresh = by_id(&observation.longer);
    for (id, binding) in stale {
        if let Some(extended) = fresh.get(id) {
            assert_eq!(*extended, binding, "{id} rebound under extension");
        }
    }
    assert_eq!(
        finalize_ids(&observation.longer),
        finalize_ids(&observation.shorter)
    );
}

#[test]
fn log_order_is_stable_across_every_dense_prefix() {
    let observation = observe_blocked_branch_draws(DrawMode::LogOrder);
    for window in observation.dense_prefixes.windows(2) {
        let previous = by_id(&window[0]);
        let current = by_id(&window[1]);
        for (id, binding) in previous {
            if let Some(extended) = current.get(id) {
                assert_eq!(*extended, binding, "{id} rebound between prefixes");
            }
        }
    }
}

#[test]
fn log_order_is_deterministic_for_the_same_prefix() {
    let observation = observe_blocked_branch_draws(DrawMode::LogOrder);
    assert!(observation.repeated_full.len() >= 2);
    assert_eq!(observation.repeated_full[0], observation.repeated_full[1]);
}
