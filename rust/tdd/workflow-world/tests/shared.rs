use workflow_world_tdd::{
    PageInfoInput, PaginatedResponseInput, UtcTimestamp, parse_paginated_response,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Item {
    id: String,
}

#[test]
fn preserves_optional_analytics_page_metadata() {
    let response = parse_paginated_response(PaginatedResponseInput {
        data: vec![Item {
            id: "item_1".to_owned(),
        }],
        cursor: None,
        has_more: false,
        page_info: Some(PageInfoInput {
            current_lookback_days: 2,
            max_lookback_days: 30,
            current_window_start: "2026-06-29T00:00:00.000Z".to_owned(),
            max_window_start: "2026-06-01T00:00:00.000Z".to_owned(),
            upgrade_available: true,
        }),
    })
    .unwrap();

    assert_eq!(response.data[0].id, "item_1");
    assert_eq!(response.cursor, None);
    assert!(!response.has_more);
    assert_eq!(
        response.page_info.unwrap(),
        workflow_world_tdd::PageInfo {
            current_lookback_days: 2,
            max_lookback_days: 30,
            current_window_start: UtcTimestamp {
                unix_millis: 1_782_691_200_000,
                iso8601: "2026-06-29T00:00:00.000Z".to_owned(),
            },
            max_window_start: UtcTimestamp {
                unix_millis: 1_780_272_000_000,
                iso8601: "2026-06-01T00:00:00.000Z".to_owned(),
            },
            upgrade_available: true,
        }
    );
}
