#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeWindow {
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsPageInfo {
    pub current_lookback_days: u32,
    pub max_lookback_days: u32,
    pub current_window_start: String,
    pub max_window_start: String,
    pub upgrade_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InspectPageResponse {
    pub page_info: Option<AnalyticsPageInfo>,
}

pub fn parse_time_input(value: &str, flag_name: &str, now_epoch_ms: i64) -> Result<String, String> {
    let _ = (value, flag_name, now_epoch_ms);
    panic!("TDD RED: packages/cli/src/lib/inspect/time-window.test.ts implementation pending")
}

pub fn resolve_time_window(
    since: Option<&str>,
    until: Option<&str>,
    now_epoch_ms: i64,
) -> Result<Option<TimeWindow>, String> {
    let _ = (since, until, now_epoch_ms);
    panic!("TDD RED: packages/cli/src/lib/inspect/time-window.test.ts implementation pending")
}

pub fn plan_window_start_from_response(response: Option<&InspectPageResponse>) -> Option<String> {
    let _ = response;
    panic!("TDD RED: packages/cli/src/lib/inspect/time-window.test.ts implementation pending")
}
