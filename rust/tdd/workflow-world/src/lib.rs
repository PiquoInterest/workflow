#![forbid(unsafe_code)]

use std::collections::BTreeMap;

pub type Environment = BTreeMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtcTimestamp {
    pub unix_millis: i64,
    pub iso8601: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateInput {
    Text(String),
    UnixMillis(i64),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NullableDateInput {
    #[default]
    Missing,
    Null,
    Value(DateInput),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NullableTimestamp {
    #[default]
    Missing,
    Null,
    Value(UtcTimestamp),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsRunInput {
    pub run_id: String,
    pub status: String,
    pub deployment_id: String,
    pub workflow_name: String,
    pub created_at: DateInput,
    pub updated_at: DateInput,
    pub started_at: NullableDateInput,
    pub completed_at: NullableDateInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsRun {
    pub run_id: String,
    pub status: String,
    pub deployment_id: String,
    pub workflow_name: String,
    pub created_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
    pub started_at: NullableTimestamp,
    pub completed_at: NullableTimestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsEventInput {
    pub run_id: String,
    pub event_id: String,
    pub event_type: String,
    pub workflow_name: String,
    pub deployment_id: String,
    pub run_created_at: DateInput,
    pub created_at: DateInput,
    pub vercel_id: Option<String>,
    pub request_id: Option<String>,
    pub compute_instance_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsEvent {
    pub run_id: String,
    pub event_id: String,
    pub event_type: String,
    pub workflow_name: String,
    pub deployment_id: String,
    pub run_created_at: UtcTimestamp,
    pub created_at: UtcTimestamp,
    pub vercel_id: Option<String>,
    pub request_id: Option<String>,
    pub compute_instance_id: Option<String>,
}

pub fn parse_analytics_run(input: AnalyticsRunInput) -> Result<AnalyticsRun, String> {
    let _ = input;
    panic!("TDD RED: packages/world/src/analytics.test.ts implementation pending")
}

pub fn parse_analytics_event(input: AnalyticsEventInput) -> Result<AnalyticsEvent, String> {
    let _ = input;
    panic!("TDD RED: packages/world/src/analytics.test.ts implementation pending")
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EventDataInput {
    pub token: Option<String>,
    pub token_retention_until: Option<DateInput>,
    pub step_name: Option<String>,
    pub owner_message_id: Option<String>,
    pub cancel_reason: Option<String>,
    pub sealed: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EventData {
    pub token: Option<String>,
    pub token_retention_until: Option<UtcTimestamp>,
    pub step_name: Option<String>,
    pub owner_message_id: Option<String>,
    pub cancel_reason: Option<String>,
    pub sealed: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateEventInput {
    pub event_type: String,
    pub correlation_id: Option<String>,
    pub spec_version: Option<u32>,
    pub event_data: Option<EventDataInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEventInput {
    pub event_type: String,
    pub run_id: String,
    pub event_id: String,
    pub correlation_id: Option<String>,
    pub created_at: DateInput,
    pub spec_version: Option<u32>,
    pub event_data: Option<EventDataInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEvent {
    pub event_type: String,
    pub run_id: Option<String>,
    pub event_id: Option<String>,
    pub correlation_id: Option<String>,
    pub created_at: Option<UtcTimestamp>,
    pub spec_version: Option<u32>,
    pub event_data: Option<EventData>,
}

pub fn parse_create_event(input: CreateEventInput) -> Result<ParsedEvent, String> {
    let _ = input;
    panic!("TDD RED: packages/world/src/events.test.ts implementation pending")
}

pub fn parse_stored_event(input: StoredEventInput) -> Result<ParsedEvent, String> {
    let _ = input;
    panic!("TDD RED: packages/world/src/events.test.ts implementation pending")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageInfoInput {
    pub current_lookback_days: u32,
    pub max_lookback_days: u32,
    pub current_window_start: String,
    pub max_window_start: String,
    pub upgrade_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageInfo {
    pub current_lookback_days: u32,
    pub max_lookback_days: u32,
    pub current_window_start: UtcTimestamp,
    pub max_window_start: UtcTimestamp,
    pub upgrade_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginatedResponseInput<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
    pub has_more: bool,
    pub page_info: Option<PageInfoInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
    pub has_more: bool,
    pub page_info: Option<PageInfo>,
}

pub fn parse_paginated_response<T>(
    input: PaginatedResponseInput<T>,
) -> Result<PaginatedResponse<T>, String> {
    let _ = input;
    panic!("TDD RED: packages/world/src/shared.test.ts implementation pending")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveRun {
    pub run_id: String,
    pub workflow_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryLogLevel {
    Debug,
    Warn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryLogEntry {
    pub level: RecoveryLogLevel,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecoveryReport {
    pub enqueued: usize,
    pub logs: Vec<RecoveryLogEntry>,
}

pub fn reenqueue_active_runs<F>(
    runs: &[ActiveRun],
    environment: &Environment,
    explicit_namespace: Option<&str>,
    label: &str,
    debug_selector: Option<&str>,
    enqueue: F,
) -> RecoveryReport
where
    F: FnMut(&str, &str) -> Result<(), String>,
{
    let _ = (
        runs,
        environment,
        explicit_namespace,
        label,
        debug_selector,
        enqueue,
    );
    panic!("TDD RED: packages/world/src/recovery.test.ts implementation pending")
}

pub const NODE_HTTP_ENV_VAR: &str = "WORKFLOW_NODE_HTTP";
pub const NODE_HTTP_DEFAULT: bool = false;

#[must_use]
pub fn is_node_http_enabled(environment: Option<&Environment>) -> bool {
    let _ = environment;
    panic!("TDD RED: packages/world/src/node-http-flag.test.ts implementation pending")
}
