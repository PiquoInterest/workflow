use std::collections::BTreeMap;

use workflow_cli_tdd::output::{
    AnalyticsPageInfo, DateValue, EventOutputRecord, EventRecord, Page, RunOutputRecord, RunRecord,
    StepOutputRecord, StepRecord, WaitOutputRecord, WaitRecord,
};

pub fn date(value: &str) -> DateValue {
    DateValue::iso(value)
}

pub fn page_info() -> AnalyticsPageInfo {
    AnalyticsPageInfo {
        current_lookback_days: 2,
        max_lookback_days: 30,
        current_window_start: date("2026-06-28T00:00:00.000Z"),
        max_window_start: date("2026-06-01T00:00:00.000Z"),
        upgrade_available: true,
    }
}

pub fn run(run_id: &str) -> RunRecord {
    RunRecord {
        run_id: run_id.to_owned(),
        status: "running".to_owned(),
        deployment_id: "dep-1".to_owned(),
        workflow_name: "workflow//./src/workflows/test//myWorkflow".to_owned(),
        spec_version: 2,
        attributes: BTreeMap::new(),
        created_at: date("2026-06-30T00:00:00.000Z"),
        updated_at: date("2026-06-30T00:00:00.000Z"),
        started_at: Some(date("2026-06-30T00:00:01.000Z")),
        completed_at: None,
        error_code: None,
        workflow_core_version: None,
        workflow_encryption_enabled: false,
    }
}

pub fn run_output(input: &RunRecord) -> RunOutputRecord {
    RunOutputRecord {
        run_id: input.run_id.clone(),
        status: input.status.clone(),
        deployment_id: input.deployment_id.clone(),
        workflow_name: input.workflow_name.clone(),
        spec_version: input.spec_version,
        attributes: input.attributes.clone(),
        created_at: input.created_at.0.clone(),
        updated_at: input.updated_at.0.clone(),
        started_at: input.started_at.as_ref().map(|value| value.0.clone()),
        completed_at: input.completed_at.as_ref().map(|value| value.0.clone()),
        error_code: input.error_code.clone(),
        workflow_core_version: input.workflow_core_version.clone(),
        workflow_encryption_enabled: input.workflow_encryption_enabled,
        world_fields: BTreeMap::new(),
    }
}

pub fn step() -> StepRecord {
    StepRecord {
        run_id: "run-1".to_owned(),
        step_id: "step-1".to_owned(),
        step_name: "doWork".to_owned(),
        status: "completed".to_owned(),
        attempt: 1,
        created_at: date("2026-06-30T00:00:00.000Z"),
        updated_at: date("2026-06-30T00:00:02.000Z"),
        started_at: Some(date("2026-06-30T00:00:01.000Z")),
        completed_at: Some(date("2026-06-30T00:00:02.000Z")),
        retry_after: None,
        error_code: None,
        workflow_core_version: None,
        workflow_encryption_enabled: false,
    }
}

pub fn step_output(input: &StepRecord) -> StepOutputRecord {
    StepOutputRecord {
        run_id: input.run_id.clone(),
        step_id: input.step_id.clone(),
        step_name: input.step_name.clone(),
        status: input.status.clone(),
        attempt: input.attempt,
        created_at: input.created_at.0.clone(),
        updated_at: input.updated_at.0.clone(),
        started_at: input.started_at.as_ref().map(|value| value.0.clone()),
        completed_at: input.completed_at.as_ref().map(|value| value.0.clone()),
        retry_after: input.retry_after.as_ref().map(|value| value.0.clone()),
        error_code: input.error_code.clone(),
        workflow_core_version: input.workflow_core_version.clone(),
        workflow_encryption_enabled: input.workflow_encryption_enabled,
    }
}

pub fn event(analytics: bool) -> EventRecord {
    EventRecord {
        run_id: "run-1".to_owned(),
        event_id: "event-1".to_owned(),
        event_type: "step_completed".to_owned(),
        correlation_id: Some("step-1".to_owned()),
        entity_id: analytics.then(|| "step-1".to_owned()),
        step_name: Some("doWork".to_owned()),
        workflow_name: analytics.then(|| "workflow//./src/workflows/test//myWorkflow".to_owned()),
        deployment_id: analytics.then(|| "dep-1".to_owned()),
        spec_version: analytics.then_some(2),
        run_created_at: analytics.then(|| date("2026-06-30T00:00:00.000Z")),
        created_at: date("2026-06-30T00:00:02.000Z"),
        event_data: (!analytics).then(|| "{\"stepName\":\"doWork\"}".to_owned()),
        region: None,
        vercel_id: None,
        request_id: None,
        resume_at: None,
        retry_after: None,
        error_code: None,
        workflow_core_version: None,
        is_webhook: false,
        is_system: false,
        workflow_encryption_enabled: false,
    }
}

pub fn event_output(input: &EventRecord) -> EventOutputRecord {
    EventOutputRecord {
        run_id: input.run_id.clone(),
        event_id: input.event_id.clone(),
        event_type: input.event_type.clone(),
        correlation_id: input.correlation_id.clone(),
        entity_id: input.entity_id.clone(),
        step_name: input.step_name.clone(),
        workflow_name: input.workflow_name.clone(),
        deployment_id: input.deployment_id.clone(),
        spec_version: input.spec_version,
        run_created_at: input.run_created_at.as_ref().map(|value| value.0.clone()),
        created_at: input.created_at.0.clone(),
        event_data: input.event_data.clone(),
        region: input.region.clone(),
        vercel_id: input.vercel_id.clone(),
        request_id: input.request_id.clone(),
        resume_at: input.resume_at.as_ref().map(|value| value.0.clone()),
        retry_after: input.retry_after.as_ref().map(|value| value.0.clone()),
        error_code: input.error_code.clone(),
        workflow_core_version: input.workflow_core_version.clone(),
        is_webhook: input.is_webhook,
        is_system: input.is_system,
        workflow_encryption_enabled: input.workflow_encryption_enabled,
    }
}

pub fn wait() -> WaitRecord {
    WaitRecord {
        run_id: "run-1".to_owned(),
        wait_id: "wait-1".to_owned(),
        status: "waiting".to_owned(),
        resume_at: date("2026-06-30T00:01:00.000Z"),
        created_at: date("2026-06-30T00:00:00.000Z"),
        updated_at: date("2026-06-30T00:00:00.000Z"),
        completed_at: None,
        workflow_core_version: None,
        workflow_encryption_enabled: false,
    }
}

pub fn wait_output(input: &WaitRecord) -> WaitOutputRecord {
    WaitOutputRecord {
        run_id: input.run_id.clone(),
        wait_id: input.wait_id.clone(),
        status: input.status.clone(),
        resume_at: input.resume_at.0.clone(),
        created_at: input.created_at.0.clone(),
        updated_at: input.updated_at.0.clone(),
        completed_at: input.completed_at.as_ref().map(|value| value.0.clone()),
        workflow_core_version: input.workflow_core_version.clone(),
        workflow_encryption_enabled: input.workflow_encryption_enabled,
    }
}

pub fn fields(values: &[(&str, Option<&str>)]) -> BTreeMap<String, Option<String>> {
    values
        .iter()
        .map(|(key, value)| ((*key).to_owned(), value.map(str::to_owned)))
        .collect()
}

pub fn page<T>(data: Vec<T>, cursor: Option<&str>, has_more: bool) -> Page<T> {
    Page::new(data, cursor, has_more)
}
