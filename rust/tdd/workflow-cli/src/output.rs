use std::collections::BTreeMap;

pub const EXPIRED_DATA_MESSAGE: &str = "<data expired>";
pub const OBSERVABILITY_UPGRADE_MESSAGE: &str =
    "Upgrade Observability Plus to inspect data outside the current lookback window.";

fn pending<T>() -> T {
    panic!("TDD RED: packages/cli/src/lib/inspect/output.test.ts implementation pending")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateValue(pub String);

impl DateValue {
    pub fn iso(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pagination {
    pub sort_order: String,
    pub cursor: Option<String>,
    pub limit: usize,
}

impl Pagination {
    pub fn descending(cursor: Option<&str>, limit: usize) -> Self {
        Self {
            sort_order: "desc".to_owned(),
            cursor: cursor.map(str::to_owned),
            limit,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsPageInfo {
    pub current_lookback_days: u32,
    pub max_lookback_days: u32,
    pub current_window_start: DateValue,
    pub max_window_start: DateValue,
    pub upgrade_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsPageInfoOutput {
    pub current_lookback_days: u32,
    pub max_lookback_days: u32,
    pub current_window_start: String,
    pub max_window_start: String,
    pub upgrade_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
    pub has_more: bool,
    pub page_info: Option<AnalyticsPageInfo>,
}

impl<T> Page<T> {
    pub fn new(data: Vec<T>, cursor: Option<&str>, has_more: bool) -> Self {
        Self {
            data,
            cursor: cursor.map(str::to_owned),
            has_more,
            page_info: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ApiError {
    pub status: Option<u16>,
    pub code: Option<String>,
    pub body_error: Option<String>,
}

pub fn has_expired_data(expired_at_epoch_ms: Option<i64>, now_epoch_ms: i64) -> bool {
    let _ = (expired_at_epoch_ms, now_epoch_ms);
    pending()
}

pub fn format_table_value(
    property: &str,
    value: &str,
    expired_at_epoch_ms: Option<i64>,
    now_epoch_ms: i64,
) -> String {
    let _ = (property, value, expired_at_epoch_ms, now_epoch_ms);
    pending()
}

pub fn is_observability_upgrade_required_error(error: &ApiError) -> bool {
    let _ = error;
    pending()
}

pub fn get_observability_upgrade_required_message() -> &'static str {
    pending()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecord {
    pub run_id: String,
    pub status: String,
    pub deployment_id: String,
    pub workflow_name: String,
    pub spec_version: u32,
    pub attributes: BTreeMap<String, String>,
    pub created_at: DateValue,
    pub updated_at: DateValue,
    pub started_at: Option<DateValue>,
    pub completed_at: Option<DateValue>,
    pub error_code: Option<String>,
    pub workflow_core_version: Option<String>,
    pub workflow_encryption_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutputRecord {
    pub run_id: String,
    pub status: String,
    pub deployment_id: String,
    pub workflow_name: String,
    pub spec_version: u32,
    pub attributes: BTreeMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_code: Option<String>,
    pub workflow_core_version: Option<String>,
    pub workflow_encryption_enabled: bool,
    pub world_fields: BTreeMap<String, Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DescribeRunBehavior {
    #[default]
    Absent,
    Sync(BTreeMap<String, Option<String>>),
    Async(BTreeMap<String, Option<String>>),
    Throw,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunWorld {
    pub analytics_page: Page<RunRecord>,
    pub describe_run: DescribeRunBehavior,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunListOptions {
    pub json: bool,
    pub workflow_name: Option<String>,
    pub status: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunsListCall {
    pub workflow_name: Option<String>,
    pub status: Option<String>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutputPage {
    pub data: Vec<RunOutputRecord>,
    pub cursor: Option<String>,
    pub has_more: bool,
    pub page_info: Option<AnalyticsPageInfoOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListRunsObservation {
    pub analytics_call: RunsListCall,
    pub output: RunOutputPage,
    pub stdout_write_count: usize,
    pub describe_calls: Vec<String>,
}

pub fn list_runs(world: &RunWorld, options: &RunListOptions) -> ListRunsObservation {
    let _ = (world, options);
    pending()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRecord {
    pub run_id: String,
    pub step_id: String,
    pub step_name: String,
    pub status: String,
    pub attempt: u64,
    pub created_at: DateValue,
    pub updated_at: DateValue,
    pub started_at: Option<DateValue>,
    pub completed_at: Option<DateValue>,
    pub retry_after: Option<DateValue>,
    pub error_code: Option<String>,
    pub workflow_core_version: Option<String>,
    pub workflow_encryption_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepOutputRecord {
    pub run_id: String,
    pub step_id: String,
    pub step_name: String,
    pub status: String,
    pub attempt: u64,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub retry_after: Option<String>,
    pub error_code: Option<String>,
    pub workflow_core_version: Option<String>,
    pub workflow_encryption_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRecord {
    pub run_id: String,
    pub event_id: String,
    pub event_type: String,
    pub correlation_id: Option<String>,
    pub entity_id: Option<String>,
    pub step_name: Option<String>,
    pub workflow_name: Option<String>,
    pub deployment_id: Option<String>,
    pub spec_version: Option<u32>,
    pub run_created_at: Option<DateValue>,
    pub created_at: DateValue,
    pub event_data: Option<String>,
    pub region: Option<String>,
    pub vercel_id: Option<String>,
    pub request_id: Option<String>,
    pub resume_at: Option<DateValue>,
    pub retry_after: Option<DateValue>,
    pub error_code: Option<String>,
    pub workflow_core_version: Option<String>,
    pub is_webhook: bool,
    pub is_system: bool,
    pub workflow_encryption_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventOutputRecord {
    pub run_id: String,
    pub event_id: String,
    pub event_type: String,
    pub correlation_id: Option<String>,
    pub entity_id: Option<String>,
    pub step_name: Option<String>,
    pub workflow_name: Option<String>,
    pub deployment_id: Option<String>,
    pub spec_version: Option<u32>,
    pub run_created_at: Option<String>,
    pub created_at: String,
    pub event_data: Option<String>,
    pub region: Option<String>,
    pub vercel_id: Option<String>,
    pub request_id: Option<String>,
    pub resume_at: Option<String>,
    pub retry_after: Option<String>,
    pub error_code: Option<String>,
    pub workflow_core_version: Option<String>,
    pub is_webhook: bool,
    pub is_system: bool,
    pub workflow_encryption_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitRecord {
    pub run_id: String,
    pub wait_id: String,
    pub status: String,
    pub resume_at: DateValue,
    pub created_at: DateValue,
    pub updated_at: DateValue,
    pub completed_at: Option<DateValue>,
    pub workflow_core_version: Option<String>,
    pub workflow_encryption_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitOutputRecord {
    pub run_id: String,
    pub wait_id: String,
    pub status: String,
    pub resume_at: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub workflow_core_version: Option<String>,
    pub workflow_encryption_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceListOptions {
    pub json: bool,
    pub run_id: String,
    pub correlation_id: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceListCall {
    pub run_id: String,
    pub correlation_id: Option<String>,
    pub pagination: Pagination,
    pub resolve_data: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceListObservation<T> {
    pub analytics_call: Option<ResourceListCall>,
    pub storage_call: Option<ResourceListCall>,
    pub output: Vec<T>,
    pub stdout_write_count: usize,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepWorld {
    pub analytics_page: Option<Page<StepRecord>>,
    pub storage_page: Option<Page<StepRecord>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventWorld {
    pub analytics_page: Option<Page<EventRecord>>,
    pub storage_page: Option<Page<EventRecord>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitWorld {
    pub analytics_page: Page<WaitRecord>,
}

pub fn list_steps(
    world: &StepWorld,
    options: &ResourceListOptions,
) -> ResourceListObservation<StepOutputRecord> {
    let _ = (world, options);
    pending()
}

pub fn list_events(
    world: &EventWorld,
    options: &ResourceListOptions,
) -> ResourceListObservation<EventOutputRecord> {
    let _ = (world, options);
    pending()
}

pub fn list_waits(
    world: &WaitWorld,
    options: &ResourceListOptions,
) -> ResourceListObservation<WaitOutputRecord> {
    let _ = (world, options);
    pending()
}
