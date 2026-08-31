use serde_json::json;
use workflow_world::hooks::{
    HOOK_RESUME_DEDUP_VERSION, HOOK_RESUME_INPUT_VERSION, HookLookupProtocolFields,
    HookResumeCapabilities, HookResumeContext, PersistedHookProtocolFields,
    parse_hook_resume_capabilities, parse_hook_resume_context,
};

fn full_context() -> serde_json::Value {
    json!({
        "deploymentId": "deployment_1",
        "workflowName": "processOrder",
        "runSpecVersion": 5,
        "workflowCoreVersion": "5.0.0",
        "traceCarrier": { "traceparent": "00-abc-def-01" },
        "encryptionPublicKey": "ZmFrZS1wdWJsaWMta2V5",
        "hookResumeInputVersion": 1,
    })
}

#[test]
fn protocol_versions_are_stable() {
    assert_eq!(HOOK_RESUME_INPUT_VERSION, 1);
    assert_eq!(HOOK_RESUME_DEDUP_VERSION, 1);
}

#[test]
fn context_matches_typescript_unknown_field_stripping() {
    let mut input = full_context();
    input["resumeCapabilities"] = json!({ "hookResumeDedupVersion": 1 });
    input["unexpected"] = json!("not persisted");

    let parsed = parse_hook_resume_context(input).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap(), full_context());
}

#[test]
fn context_rejects_non_string_trace_values() {
    let mut input = full_context();
    input["traceCarrier"] = json!({ "traceparent": 42 });
    assert!(parse_hook_resume_context(input).is_err());
}

#[test]
fn capabilities_match_typescript_unknown_field_stripping() {
    let parsed = parse_hook_resume_capabilities(json!({
        "hookResumeDedupVersion": 1,
        "staleServerField": true,
    }))
    .unwrap();

    assert_eq!(
        serde_json::to_value(parsed).unwrap(),
        json!({ "hookResumeDedupVersion": 1 })
    );
}

#[test]
fn capabilities_require_a_bounded_integer_version() {
    assert!(parse_hook_resume_capabilities(json!({})).is_err());
    for invalid in [
        json!({ "hookResumeDedupVersion": "1" }),
        json!({ "hookResumeDedupVersion": 0 }),
        json!({ "hookResumeDedupVersion": -1 }),
        json!({ "hookResumeDedupVersion": 1.5 }),
        json!({ "hookResumeDedupVersion": u64::from(u32::MAX) + 1 }),
    ] {
        assert!(parse_hook_resume_capabilities(invalid).is_err());
    }

    let parsed = parse_hook_resume_capabilities(json!({
        "hookResumeDedupVersion": u32::MAX,
    }))
    .unwrap();
    assert_eq!(parsed.hook_resume_dedup_version, u32::MAX);
}

#[test]
fn context_versions_require_bounded_integers() {
    for key in ["runSpecVersion", "hookResumeInputVersion"] {
        for invalid in [
            json!(0),
            json!(-1),
            json!(1.5),
            json!(u64::from(u32::MAX) + 1),
        ] {
            let mut input = full_context();
            input[key] = invalid;
            assert!(parse_hook_resume_context(input).is_err());
        }
    }
}

#[test]
fn transient_capabilities_cannot_survive_persistence_conversion() {
    let context: HookResumeContext = parse_hook_resume_context(full_context()).unwrap();
    let capabilities: HookResumeCapabilities =
        parse_hook_resume_capabilities(json!({ "hookResumeDedupVersion": 1 })).unwrap();
    let lookup = HookLookupProtocolFields {
        persisted: PersistedHookProtocolFields {
            resume_context: Some(context),
        },
        resume_capabilities: Some(capabilities),
    };

    assert!(
        serde_json::to_value(&lookup)
            .unwrap()
            .get("resumeCapabilities")
            .is_some()
    );

    let persisted = lookup.into_persisted();
    let persisted_json = serde_json::to_value(persisted).unwrap();
    assert!(persisted_json.get("resumeContext").is_some());
    assert!(persisted_json.get("resumeCapabilities").is_none());
}
