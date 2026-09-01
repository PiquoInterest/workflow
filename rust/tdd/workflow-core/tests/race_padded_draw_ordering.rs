use std::collections::BTreeSet;

use workflow_core_tdd::race_padded_draw_ordering::{
    CORRELATION_IDS, DrawBinding, RECORDED_EVENT_COUNT, RacePaddedReplayObservation,
    ReplayPassObservation, ReplayTemperature, replay_race_padded_draws,
};

fn expected_bindings() -> Vec<DrawBinding> {
    vec![
        DrawBinding::new(CORRELATION_IDS[0], "settleW"),
        DrawBinding::new(CORRELATION_IDS[1], "sleepW"),
        DrawBinding::new(CORRELATION_IDS[2], "settleS"),
        DrawBinding::new(CORRELATION_IDS[3], "sleepS"),
        DrawBinding::new(CORRELATION_IDS[4], "recoverW"),
        DrawBinding::new(CORRELATION_IDS[5], "finalizeS"),
        DrawBinding::new(CORRELATION_IDS[6], "finalizeW"),
    ]
}

fn assert_recorded_order(pass: &ReplayPassObservation) {
    assert!(pass.suspended);
    assert_eq!(pass.corruption_error, None);
    assert_eq!(pass.event_count, RECORDED_EVENT_COUNT);
    assert_eq!(pass.event_index, pass.event_count);

    let mut pending = pass.pending_steps.clone();
    pending.sort();
    assert_eq!(
        pending,
        vec![
            "finalizeS".to_owned(),
            "finalizeW".to_owned(),
            "settleW".to_owned(),
        ]
    );
    assert_eq!(pass.bindings, expected_bindings());
}

#[test]
fn cold_replay_reproduces_every_recorded_draw_binding() {
    let observation = replay_race_padded_draws(ReplayTemperature::Cold);
    assert_eq!(observation.passes.len(), 1);
    assert_recorded_order(&observation.passes[0]);
}

#[test]
fn warm_replay_sharing_the_payload_cache_preserves_log_order() {
    let observation = replay_race_padded_draws(ReplayTemperature::WarmSharedCache);
    assert_eq!(observation.passes.len(), 2);
    assert!(observation.hydration_calls > 0);
    assert_recorded_order(&observation.passes[0]);
    assert_recorded_order(&observation.passes[1]);
}

/// Rust-only strengthening of WF-RUST-023: one logical entity must not be
/// minted under multiple correlation IDs, and one ID must not bind two entities.
#[test]
fn cold_and_warm_passes_keep_exact_single_owner_correlation_bindings() {
    let RacePaddedReplayObservation { passes, .. } =
        replay_race_padded_draws(ReplayTemperature::WarmSharedCache);
    assert_eq!(passes.len(), 2);

    for pass in &passes {
        let correlation_ids: BTreeSet<&str> = pass
            .bindings
            .iter()
            .map(|binding| binding.correlation_id.as_str())
            .collect();
        assert_eq!(correlation_ids.len(), pass.bindings.len());
        assert_eq!(
            pass.bindings
                .iter()
                .filter(|binding| binding.entity == "finalizeS")
                .count(),
            1
        );
        assert_eq!(
            pass.bindings
                .iter()
                .filter(|binding| binding.entity == "finalizeW")
                .count(),
            1
        );
    }

    assert_eq!(passes[0].bindings, passes[1].bindings);
}
