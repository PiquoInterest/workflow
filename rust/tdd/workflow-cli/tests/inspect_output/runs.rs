use std::collections::BTreeMap;

use workflow_cli_tdd::output::{
    AnalyticsPageInfoOutput, DescribeRunBehavior, ListRunsObservation, Page, Pagination,
    RunListOptions, RunOutputPage, RunWorld, RunsListCall, list_runs,
};

use super::support::{fields, page_info, run, run_output};

fn list(
    describe_run: DescribeRunBehavior,
    page: Page<workflow_cli_tdd::output::RunRecord>,
) -> ListRunsObservation {
    list_runs(
        &RunWorld {
            analytics_page: page,
            describe_run,
        },
        &RunListOptions {
            json: true,
            ..RunListOptions::default()
        },
    )
}

#[test]
fn preserves_analytics_page_metadata_in_json_output() {
    let input = run("run-1");
    let info = page_info();
    let page = Page {
        data: vec![input.clone()],
        cursor: None,
        has_more: false,
        page_info: Some(info.clone()),
    };

    let observation = list(DescribeRunBehavior::Absent, page);
    assert_eq!(
        observation.analytics_call,
        RunsListCall {
            workflow_name: None,
            status: None,
            pagination: Pagination::descending(None, 20),
        }
    );
    assert_eq!(observation.stdout_write_count, 1);
    assert_eq!(
        observation.output,
        RunOutputPage {
            data: vec![run_output(&input)],
            cursor: None,
            has_more: false,
            page_info: Some(AnalyticsPageInfoOutput {
                current_lookback_days: info.current_lookback_days,
                max_lookback_days: info.max_lookback_days,
                current_window_start: info.current_window_start.0,
                max_window_start: info.max_window_start.0,
                upgrade_available: info.upgrade_available,
            }),
        }
    );
}

#[test]
fn includes_world_specific_fields_when_describe_run_exists() {
    let input = run("wrun_41KX206BTK10M0C31CMN2AS1JS");
    let observation = list(
        DescribeRunBehavior::Sync(fields(&[("region", Some("sfo1")), ("shard", Some("a"))])),
        Page::new(vec![input.clone()], None, false),
    );

    assert_eq!(observation.describe_calls, vec![input.run_id]);
    assert_eq!(
        observation.output.data[0].world_fields,
        fields(&[("region", Some("sfo1")), ("shard", Some("a"))])
    );
}

#[test]
fn preserves_null_describe_run_values() {
    let input = run("wrun_malformed");
    let observation = list(
        DescribeRunBehavior::Sync(fields(&[("region", None)])),
        Page::new(vec![input], None, false),
    );

    assert!(
        observation.output.data[0]
            .world_fields
            .contains_key("region")
    );
    assert_eq!(observation.output.data[0].world_fields["region"], None);
}

#[test]
fn describe_run_cannot_overwrite_canonical_run_fields() {
    let input = run("wrun_41KX206BTK10M0C31CMN2AS1JS");
    let observation = list(
        DescribeRunBehavior::Sync(fields(&[
            ("status", Some("hacked")),
            ("runId", Some("nope")),
            ("region", Some("sfo1")),
        ])),
        Page::new(vec![input.clone()], None, false),
    );

    let output = &observation.output.data[0];
    assert_eq!(output.status, "running");
    assert_eq!(output.run_id, input.run_id);
    assert_eq!(output.world_fields, fields(&[("region", Some("sfo1"))]));
}

#[test]
fn throwing_describe_run_contributes_no_fields() {
    let input = run("wrun_41KX206BTK10M0C31CMN2AS1JS");
    let observation = list(
        DescribeRunBehavior::Throw,
        Page::new(vec![input], None, false),
    );

    assert_eq!(observation.output.data[0].status, "running");
    assert!(observation.output.data[0].world_fields.is_empty());
}

#[test]
fn supports_async_describe_run_implementations() {
    let input = run("wrun_41KX206BTK10M0C31CMN2AS1JS");
    let observation = list(
        DescribeRunBehavior::Async(fields(&[("region", Some("sfo1"))])),
        Page::new(vec![input], None, false),
    );

    assert_eq!(
        observation.output.data[0].world_fields,
        fields(&[("region", Some("sfo1"))])
    );
}

#[test]
fn rejecting_async_describe_run_contributes_no_fields() {
    let input = run("wrun_41KX206BTK10M0C31CMN2AS1JS");
    let observation = list(
        DescribeRunBehavior::Reject,
        Page::new(vec![input], None, false),
    );

    assert_eq!(observation.output.data[0].status, "running");
    assert!(observation.output.data[0].world_fields.is_empty());
}

#[test]
fn world_without_describe_run_adds_no_world_fields() {
    let input = run("wrun_01KX2M5N3RBNC12RYWYYH4WWQJ");
    let observation = list(
        DescribeRunBehavior::Absent,
        Page::new(vec![input], None, false),
    );

    assert_eq!(observation.describe_calls, Vec::<String>::new());
    assert_eq!(observation.output.data[0].world_fields, BTreeMap::new());
}
