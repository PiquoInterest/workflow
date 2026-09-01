use std::collections::BTreeMap;

use workflow_core_tdd::route_bundle_isolation::{
    BUILD_TIMEOUT_MS, HARNESS_SCRIPT, HARNESS_TIMEOUT_MS, HarnessParseError, HarnessResult,
    HarnessValidationError, NONEXISTENT_TOKEN, OUTPUT_LIMIT_BYTES, RESULT_MARKER,
    ROUTE_BUNDLE_PATH, extract_harness_payload, route_bundle_plan, run_route_bundle_isolation,
    sanitized_isolation_environment, validate_harness_result,
};

fn environment() -> BTreeMap<String, String> {
    [
        ("PATH", "/usr/bin"),
        ("FORCE_COLOR", "1"),
        ("VERCEL", "1"),
        ("VERCEL_ENV", "production"),
        ("VERCEL_DEPLOYMENT_ID", "dpl_unsafe"),
        ("VERCEL_PROJECT_ID", "prj_unsafe"),
        ("NODE_OPTIONS", "--require ./preload.js"),
        ("NODE_PATH", "./shadow-node-modules"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect()
}

#[test]
fn plan_builds_and_invokes_the_exact_isolated_route_without_a_shell() {
    let plan = route_bundle_plan();
    assert_eq!(plan.project, "nextjs-turbopack");
    assert_eq!(plan.build.program, "pnpm");
    assert_eq!(plan.build.args, vec!["build".to_owned()]);
    assert_eq!(plan.build.timeout_ms, BUILD_TIMEOUT_MS);
    assert_eq!(plan.build.output_limit_bytes, OUTPUT_LIMIT_BYTES);
    assert!(plan.build.kill_process_group_on_timeout);
    assert!(!plan.build.use_shell);

    assert_eq!(plan.harness.program, "current-node");
    assert_eq!(plan.harness.args[0], "-e");
    assert_eq!(plan.harness.args[1], HARNESS_SCRIPT);
    assert_eq!(plan.harness.args[2], ROUTE_BUNDLE_PATH);
    assert_eq!(plan.harness.timeout_ms, HARNESS_TIMEOUT_MS);
    assert_eq!(plan.harness.output_limit_bytes, OUTPUT_LIMIT_BYTES);
    assert!(plan.harness.kill_process_group_on_timeout);
    assert!(!plan.harness.use_shell);

    assert_eq!(plan.route_bundle_path, ROUTE_BUNDLE_PATH);
    assert_eq!(plan.result_marker, RESULT_MARKER);
    assert_eq!(plan.nonexistent_token, NONEXISTENT_TOKEN);
    assert_eq!(plan.expected_status, 500);
    assert_eq!(plan.expected_body_fragment, "Hook not found");
    assert!(plan.reject_duplicate_result_records);
    assert!(plan.require_exact_marker_prefix);
}

#[test]
fn environment_strips_vercel_and_node_preload_state_but_preserves_unrelated_values() {
    let sanitized = sanitized_isolation_environment(&environment());
    for key in [
        "VERCEL",
        "VERCEL_ENV",
        "VERCEL_DEPLOYMENT_ID",
        "VERCEL_PROJECT_ID",
        "NODE_OPTIONS",
        "NODE_PATH",
    ] {
        assert!(!sanitized.contains_key(key), "{key} leaked into isolation process");
    }
    assert_eq!(sanitized.get("FORCE_COLOR").map(String::as_str), Some("0"));
    assert_eq!(sanitized.get("PATH").map(String::as_str), Some("/usr/bin"));
}

#[test]
fn exact_result_prefix_is_accepted_amid_unrelated_bundle_noise() {
    let stdout = format!(
        "route log before\n{RESULT_MARKER}{{\"status\":500,\"body\":\"Hook not found\"}}\nroute log after\n"
    );
    assert_eq!(
        extract_harness_payload(&stdout),
        Ok("{\"status\":500,\"body\":\"Hook not found\"}".to_owned())
    );
}

#[test]
fn marker_substrings_in_untrusted_route_output_cannot_spoof_the_result() {
    let stdout = format!(
        "attacker log: prefix-{RESULT_MARKER}{{\"status\":200}}\n{RESULT_MARKER}{{\"status\":500}}\n"
    );
    assert_eq!(
        extract_harness_payload(&stdout),
        Ok("{\"status\":500}".to_owned())
    );
}

#[test]
fn marker_substring_without_an_exact_prefix_is_reported_as_missing() {
    let stdout = format!("noise before {RESULT_MARKER}{{\"status\":500}}\n");
    assert!(matches!(
        extract_harness_payload(&stdout),
        Err(HarnessParseError::MissingResult { .. })
    ));
}

#[test]
fn missing_result_diagnostic_uses_a_bounded_stdout_preview() {
    let stdout = "x".repeat(2_000);
    let error = extract_harness_payload(&stdout).unwrap_err();
    let HarnessParseError::MissingResult { ref stdout_preview } = error else {
        panic!("unexpected parse error")
    };
    assert_eq!(stdout_preview.chars().count(), 512);
    assert!(error.to_string().contains("bounded stdout preview"));
}

#[test]
fn duplicate_exact_result_records_are_rejected_instead_of_taking_the_last_one() {
    let stdout = format!(
        "{RESULT_MARKER}{{\"status\":500}}\n{RESULT_MARKER}{{\"status\":200}}\n"
    );
    assert_eq!(
        extract_harness_payload(&stdout),
        Err(HarnessParseError::DuplicateResults)
    );
}

#[test]
fn empty_exact_result_payload_is_rejected() {
    assert_eq!(
        extract_harness_payload(&format!("{RESULT_MARKER}\n")),
        Err(HarnessParseError::EmptyPayload)
    );
}

#[test]
fn healthy_missing_token_response_proves_the_world_resolved_inside_the_bundle() {
    let result = HarnessResult {
        status: Some(500),
        body: Some("Hook not found: route-bundle-isolation-nonexistent-token".to_owned()),
        ..HarnessResult::default()
    };
    assert_eq!(validate_harness_result(&result), Ok(()));
}

#[test]
fn harness_errors_fail_before_response_assertions() {
    let result = HarnessResult {
        harness_error: Some("route bundle did not expose routeModule.userland.POST".to_owned()),
        export_keys: vec!["default".to_owned()],
        ..HarnessResult::default()
    };
    assert_eq!(
        validate_harness_result(&result),
        Err(HarnessValidationError::HarnessError(
            "route bundle did not expose routeModule.userland.POST".to_owned()
        ))
    );
}

#[test]
fn dynamic_require_fallback_is_the_original_regression_and_is_rejected() {
    let result = HarnessResult {
        status: Some(500),
        body: Some("Cannot find module as expression is too dynamic".to_owned()),
        ..HarnessResult::default()
    };
    assert_eq!(
        validate_harness_result(&result),
        Err(HarnessValidationError::DynamicRequireFallback)
    );
}

#[test]
fn missing_world_initialization_is_rejected_loudly() {
    let result = HarnessResult {
        status: Some(500),
        body: Some("world runtime was not initialized".to_owned()),
        ..HarnessResult::default()
    };
    assert_eq!(
        validate_harness_result(&result),
        Err(HarnessValidationError::WorldRuntimeNotInitialized)
    );
}

#[test]
fn success_status_cannot_masquerade_as_the_expected_missing_hook_failure() {
    let result = HarnessResult {
        status: Some(200),
        body: Some("Hook not found".to_owned()),
        ..HarnessResult::default()
    };
    assert_eq!(
        validate_harness_result(&result),
        Err(HarnessValidationError::UnexpectedStatus(Some(200)))
    );
}

#[test]
fn missing_body_is_rejected_before_status_or_fragment_checks() {
    let result = HarnessResult {
        status: Some(500),
        ..HarnessResult::default()
    };
    assert_eq!(
        validate_harness_result(&result),
        Err(HarnessValidationError::MissingBody)
    );
}

#[test]
fn unrelated_server_error_does_not_prove_isolated_world_resolution() {
    let result = HarnessResult {
        status: Some(500),
        body: Some("Internal Server Error".to_owned()),
        ..HarnessResult::default()
    };
    assert_eq!(
        validate_harness_result(&result),
        Err(HarnessValidationError::MissingHookNotFound)
    );
}

#[test]
fn real_isolated_route_bundle_must_complete_the_contract() {
    let source_environment = environment();
    let observation = run_route_bundle_isolation(&source_environment);
    assert_eq!(observation.plan, route_bundle_plan());
    assert_eq!(
        observation.build_environment,
        sanitized_isolation_environment(&source_environment)
    );
    assert_eq!(
        observation.harness_environment,
        sanitized_isolation_environment(&source_environment)
    );
    assert_eq!(validate_harness_result(&observation.result), Ok(()));
    assert!(extract_harness_payload(&observation.stdout).is_ok());
}
