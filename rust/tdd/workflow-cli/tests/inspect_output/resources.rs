use workflow_cli_tdd::output::{
    EventWorld, Page, Pagination, ResourceListCall, ResourceListOptions, StepWorld, WaitWorld,
    get_observability_upgrade_required_message, list_events, list_steps, list_waits,
};

use super::support::{event, event_output, page_info, step, step_output, wait, wait_output};

#[test]
fn list_steps_passes_cursor_and_preserves_array_output() {
    let input = step();
    let observation = list_steps(
        &StepWorld {
            analytics_page: Some(Page::new(
                vec![input.clone()],
                Some("next-step-cursor"),
                true,
            )),
            storage_page: None,
        },
        &ResourceListOptions {
            json: true,
            run_id: "run-1".to_owned(),
            cursor: Some("step-cursor".to_owned()),
            limit: Some(1),
            ..ResourceListOptions::default()
        },
    );

    assert_eq!(
        observation.analytics_call,
        Some(ResourceListCall {
            run_id: "run-1".to_owned(),
            correlation_id: None,
            pagination: Pagination::descending(Some("step-cursor"), 1),
            resolve_data: None,
        })
    );
    assert_eq!(observation.output, vec![step_output(&input)]);
    assert_eq!(observation.stdout_write_count, 1);
}

#[test]
fn list_steps_falls_back_to_storage_when_first_analytics_page_is_empty() {
    let mut input = step();
    input.step_name = "step//./src/workflows/test//doWork".to_owned();
    let observation = list_steps(
        &StepWorld {
            analytics_page: Some(Page::new(Vec::new(), None, false)),
            storage_page: Some(Page::new(vec![input.clone()], None, false)),
        },
        &ResourceListOptions {
            json: true,
            run_id: "run-1".to_owned(),
            ..ResourceListOptions::default()
        },
    );

    assert!(observation.analytics_call.is_some());
    assert_eq!(
        observation.storage_call,
        Some(ResourceListCall {
            run_id: "run-1".to_owned(),
            correlation_id: None,
            pagination: Pagination::descending(None, 20),
            resolve_data: Some("none".to_owned()),
        })
    );
    assert_eq!(observation.output, vec![step_output(&input)]);
}

#[test]
fn list_events_passes_cursor_and_preserves_array_output() {
    let input = event(true);
    let observation = list_events(
        &EventWorld {
            analytics_page: Some(Page::new(
                vec![input.clone()],
                Some("next-event-cursor"),
                true,
            )),
            storage_page: None,
        },
        &ResourceListOptions {
            json: true,
            run_id: "run-1".to_owned(),
            cursor: Some("event-cursor".to_owned()),
            limit: Some(1),
            ..ResourceListOptions::default()
        },
    );

    assert_eq!(
        observation.analytics_call,
        Some(ResourceListCall {
            run_id: "run-1".to_owned(),
            correlation_id: None,
            pagination: Pagination::descending(Some("event-cursor"), 1),
            resolve_data: None,
        })
    );
    assert_eq!(observation.output, vec![event_output(&input)]);
    assert_eq!(observation.stdout_write_count, 1);
}

#[test]
fn list_events_falls_back_to_storage_when_first_analytics_page_is_empty() {
    let input = event(false);
    let observation = list_events(
        &EventWorld {
            analytics_page: Some(Page::new(Vec::new(), None, false)),
            storage_page: Some(Page::new(vec![input.clone()], None, false)),
        },
        &ResourceListOptions {
            json: true,
            run_id: "run-1".to_owned(),
            ..ResourceListOptions::default()
        },
    );

    assert!(observation.analytics_call.is_some());
    assert_eq!(
        observation.storage_call,
        Some(ResourceListCall {
            run_id: "run-1".to_owned(),
            correlation_id: None,
            pagination: Pagination::descending(None, 20),
            resolve_data: Some("none".to_owned()),
        })
    );
    assert_eq!(observation.output, vec![event_output(&input)]);
}

#[test]
fn list_waits_passes_cursor_and_preserves_array_output() {
    let input = wait();
    let mut analytics_page = Page::new(
        vec![input.clone()],
        Some("next-wait-cursor"),
        true,
    );
    analytics_page.page_info = Some(page_info());
    let observation = list_waits(
        &WaitWorld { analytics_page },
        &ResourceListOptions {
            json: true,
            run_id: "run-1".to_owned(),
            cursor: Some("wait-cursor".to_owned()),
            limit: Some(1),
            ..ResourceListOptions::default()
        },
    );

    assert_eq!(
        observation.analytics_call,
        Some(ResourceListCall {
            run_id: "run-1".to_owned(),
            correlation_id: None,
            pagination: Pagination::descending(Some("wait-cursor"), 1),
            resolve_data: None,
        })
    );
    assert_eq!(observation.output, vec![wait_output(&input)]);
    assert_eq!(observation.stdout_write_count, 1);
}

#[test]
fn list_waits_surfaces_observability_upgrade_hint_in_table_mode() {
    let input = wait();
    let mut analytics_page = Page::new(vec![input], None, false);
    analytics_page.page_info = Some(page_info());
    let observation = list_waits(
        &WaitWorld { analytics_page },
        &ResourceListOptions {
            json: false,
            run_id: "run-1".to_owned(),
            ..ResourceListOptions::default()
        },
    );

    assert!(
        observation
            .logs
            .join("\n")
            .contains(get_observability_upgrade_required_message())
    );
}
