use workflow_core_tdd::benchmark::{
    BENCH_METHODOLOGY_VERSION, BenchArgument, BenchmarkConfig, BenchmarkObservation,
    BenchmarkScenario, MetricKind, REPLAY_CADENCE_EVE, REPLAY_CADENCE_GATEWAY,
    RTT_INDEX_BUCKETS, ReturnIntegrity, run_benchmark_scenario,
};

fn observe(scenario: BenchmarkScenario) -> (BenchmarkConfig, BenchmarkObservation) {
    let config = BenchmarkConfig::default();
    let observation = run_benchmark_scenario(scenario, &config);
    assert_common_contract(&observation);
    (config, observation)
}

fn assert_common_contract(observation: &BenchmarkObservation) {
    assert_eq!(observation.methodology_version, BENCH_METHODOLOGY_VERSION);
    assert_eq!(observation.preflight.workflow_fn, "benchSequentialStepsWorkflow");
    assert_eq!(observation.preflight.arguments, vec![BenchArgument::Integer(1)]);
    assert!(observation.deployment_clock_anchor);
    assert!(observation.trigger_response_validated);
    assert!(observation.negative_clock_skew_clamped);
    assert_eq!(observation.preflight_timeout_ms, 180_000);
    assert!(observation.plan.run_sequentially);
    assert_eq!(observation.plan.zero_success_abort_attempts, 3);
    assert_eq!(
        observation.plan.max_attempts(),
        observation.plan.iterations + observation.plan.extra_attempts
    );
}

fn assert_metric_kinds(observation: &BenchmarkObservation, expected: &[MetricKind]) {
    let actual = observation
        .plan
        .metrics
        .iter()
        .map(|metric| metric.kind)
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn scenario_one_noop_step_turbo_records_ttfs() {
    let (config, observation) = observe(BenchmarkScenario::StepTurbo);
    assert_eq!(observation.plan.trigger.workflow_fn, "benchStepWorkflow");
    assert!(observation.plan.trigger.arguments.is_empty());
    assert_eq!(observation.plan.iterations, config.stream_iterations);
    assert_eq!(observation.plan.warmup_iterations, config.warmup_iterations);
    assert_eq!(
        observation.plan.return_integrity,
        ReturnIntegrity::StepTimings { exact_count: 1 }
    );
    assert_eq!(observation.plan.timeout_ms, config.run_timeout_ms);
    assert_metric_kinds(&observation, &[MetricKind::Ttfs]);
    assert_eq!(observation.plan.metrics[0].targets.unwrap().p75_ms, 200);
    assert_eq!(observation.plan.metrics[0].targets.unwrap().p90_ms, 300);
    assert_eq!(observation.plan.metrics[0].targets.unwrap().p99_ms, 600);
}

#[test]
fn scenario_one_streaming_step_turbo_records_ttfs() {
    let (config, observation) = observe(BenchmarkScenario::StreamingStepTurbo);
    assert_eq!(observation.plan.trigger.workflow_fn, "benchStreamWorkflow");
    assert!(observation.plan.trigger.arguments.is_empty());
    assert_eq!(observation.plan.iterations, config.stream_iterations);
    assert_eq!(observation.plan.warmup_iterations, config.warmup_iterations);
    assert_eq!(
        observation.plan.return_integrity,
        ReturnIntegrity::StepTimings { exact_count: 1 }
    );
    assert_eq!(observation.plan.timeout_ms, config.run_timeout_ms);
    assert_metric_kinds(&observation, &[MetricKind::Ttfs]);
}

#[test]
fn scenario_hook_and_step_non_turbo_records_ttfs() {
    let (config, observation) = observe(BenchmarkScenario::HookAndStepNonTurbo);
    assert_eq!(
        observation.plan.trigger.workflow_fn,
        "benchHookStreamWorkflow"
    );
    assert!(observation.plan.trigger.arguments.is_empty());
    assert_eq!(observation.plan.iterations, config.stream_iterations);
    assert_eq!(observation.plan.warmup_iterations, config.warmup_iterations);
    assert_eq!(
        observation.plan.return_integrity,
        ReturnIntegrity::StepTimings { exact_count: 1 }
   );
    assert_eq!(observation.plan.timeout_ms, config.run_timeout_ms);
    assert_metric_kinds(&observation, &[MetricKind::Ttfs]);
}

#[test]
fn scenario_paced_control_records_stream_buckets_and_write_slip() {
    let (config, observation) = observe(BenchmarkScenario::PacedControl);
    assert_eq!(observation.plan.trigger.workflow_fn, "benchCrttWorkflow");
    assert_eq!(
        observation.plan.trigger.arguments,
        vec![
            BenchArgument::Integer(config.crtt_chunk_count() as u64),
            BenchArgument::Number(config.crtt_interval_ms()),
            BenchArgument::Text("llm".to_owned()),
        ]
    );
    assert_eq!(observation.plan.iterations, config.crtt_iterations);
    assert_eq!(
        observation.plan.return_integrity,
        ReturnIntegrity::ChunkRtt {
            exact_received: config.crtt_chunk_count(),
        }
    );
    assert_eq!(
        observation.plan.timeout_ms,
        config.run_timeout_ms
            + config.crtt_chunk_count() as u64 * config.crtt_interval_ms().ceil() as u64
    );
    assert_metric_kinds(
        &observation,
        &[
            MetricKind::Stream,
            MetricKind::CrttDetail,
            MetricKind::CrttDetail,
            MetricKind::CrttDetail,
            MetricKind::WriteSlip,
        ],
    );
    let buckets = observation
        .plan
        .metrics
        .iter()
        .filter_map(|metric| metric.bucket.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(buckets, RTT_INDEX_BUCKETS.to_vec());
}

#[test]
fn scenario_size_sweep_records_size_profile_and_write_slip() {
    let (config, observation) = observe(BenchmarkScenario::SizeSweep);
    assert_eq!(observation.plan.trigger.workflow_fn, "benchCrttWorkflow");
    assert_eq!(
        observation.plan.trigger.arguments,
        vec![
            BenchArgument::Integer(config.crtt_chunk_count() as u64),
            BenchArgument::Number(config.crtt_interval_ms()),
            BenchArgument::Text("sweep".to_owned()),
        ]
    );
    assert_eq!(observation.plan.iterations, config.crtt_iterations);
    assert_eq!(
        observation.plan.return_integrity,
        ReturnIntegrity::ChunkRtt {
            exact_received: config.crtt_chunk_count(),
        }
    );
    assert_eq!(
        observation.plan.timeout_ms,
        config.run_timeout_ms
            + config.crtt_chunk_count() as u64 * config.crtt_interval_ms().ceil() as u64
    );
    assert_metric_kinds(
        &observation,
        &[MetricKind::Stream, MetricKind::WriteSlip],
    );
    assert_eq!(observation.plan.metrics[0].group.as_deref(), Some("sweep"));
}

#[test]
fn scenario_replay_gateway_reality_preserves_cadence_identity() {
    let (config, observation) = observe(BenchmarkScenario::ReplayGatewayReality);
    assert_eq!(observation.plan.trigger.workflow_fn, "benchReplayWorkflow");
    assert_eq!(
        observation.plan.trigger.arguments,
        vec![
            BenchArgument::Text(REPLAY_CADENCE_GATEWAY.to_owned()),
            BenchArgument::Integer(1),
        ]
    );
    assert_eq!(observation.plan.iterations, config.replay_gateway_iterations);
    assert_eq!(observation.plan.warmup_iterations, 1);
    assert_eq!(
        observation.plan.return_integrity,
        ReturnIntegrity::ChunkRtt {
            exact_received: config.replay_gateway_events,
        }
    );
    assert!(observation.plan.cadence_semantic_hash_required);
    assert_eq!(
        observation.plan.timeout_ms,
        config.replay_timeout_ms(config.replay_gateway_span_ms, 1)
    );
    assert_metric_kinds(
        &observation,
        &[MetricKind::Stream, MetricKind::WriteSlip],
    );
}

#[test]
fn scenario_replay_eve_reality_preserves_cadence_identity() {
    let (config, observation) = observe(BenchmarkScenario::ReplayEveReality);
    assert_eq!(observation.plan.trigger.workflow_fn, "benchReplayWorkflow");
    assert_eq!(
        observation.plan.trigger.arguments,
        vec![
            BenchArgument::Text(REPLAY_CADENCE_EVE.to_owned()),
            BenchArgument::Integer(1),
        ]
    );
    assert_eq!(observation.plan.iterations, config.replay_reality_iterations);
    assert_eq!(observation.plan.warmup_iterations, 0);
    assert_eq!(
        observation.plan.return_integrity,
        ReturnIntegrity::ChunkRtt {
            exact_received: config.replay_eve_events,
        }
    );
    assert!(observation.plan.cadence_semantic_hash_required);
    assert_eq!(
        observation.plan.timeout_ms,
        config.replay_timeout_ms(config.replay_eve_span_ms, 1)
    );
}

#[test]
fn scenario_replay_eve_stress_uses_configured_speed() {
    let (config, observation) = observe(BenchmarkScenario::ReplayEveStress);
    assert_eq!(observation.plan.trigger.workflow_fn, "benchReplayWorkflow");
    assert_eq!(
        observation.plan.trigger.arguments,
        vec![
            BenchArgument::Text(REPLAY_CADENCE_EVE.to_owned()),
            BenchArgument::Integer(config.replay_speed as u64),
        ]
    );
    assert_eq!(observation.plan.iterations, config.replay_eve_iterations);
    assert_eq!(observation.plan.warmup_iterations, 0);
    assert_eq!(
        observation.plan.return_integrity,
        ReturnIntegrity::ChunkRtt {
            exact_received: config.replay_eve_events,
        }
    );
    assert!(observation.plan.cadence_semantic_hash_required);
    assert_eq!(
        observation.plan.timeout_ms,
        config.replay_timeout_ms(config.replay_eve_span_ms, config.replay_speed)
    );
}

#[test]
fn scenario_fanout_records_first_and_last_completion_latency() {
    let (config, observation) = observe(BenchmarkScenario::FanOut);
    assert_eq!(
        observation.plan.trigger.workflow_fn,
        "benchFanOutStepsWorkflow"
    );
    assert_eq!(
        observation.plan.trigger.arguments,
        vec![BenchArgument::Integer(config.fanout_step_count as u64)]
    );
    assert_eq!(observation.plan.iterations, config.fanout_iterations);
    assert_eq!(observation.plan.warmup_iterations, 0);
    assert_eq!(
        observation.plan.return_integrity,
        ReturnIntegrity::StepTimings {
            exact_count: config.fanout_step_count,
        }
    );
    assert_eq!(
        observation.plan.timeout_ms,
        config.count_scaled_timeout_ms(config.fanout_step_count)
    );
    assert_metric_kinds(
        &observation,
        &[MetricKind::FanOutTtfs, MetricKind::FanOutTtls],
    );
}

#[test]
fn scenario_sequential_splits_step_gaps_and_records_workflow_overhead() {
    let (config, observation) = observe(BenchmarkScenario::Sequential);
    assert_eq!(
        observation.plan.trigger.workflow_fn,
        "benchSequentialStepsWorkflow"
    );
    assert_eq!(
        observation.plan.trigger.arguments,
        vec![BenchArgument::Integer(config.sequential_step_count as u64)]
    );
    assert_eq!(observation.plan.iterations, config.sequential_iterations);
    assert_eq!(observation.plan.warmup_iterations, 0);
    assert_eq!(observation.plan.extra_attempts, 2);
    assert_eq!(
        observation.plan.timeout_ms,
        config.count_scaled_timeout_ms(config.sequential_step_count)
    );
    assert_eq!(
        observation.plan.return_integrity,
        ReturnIntegrity::StepTimings {
            exact_count: config.sequential_step_count,
        }
    );
    assert_metric_kinds(
        &observation,
        &[
            MetricKind::StsoInline,
            MetricKind::StsoQueueHop,
            MetricKind::WorkflowOverhead,
        ],
    );
}
