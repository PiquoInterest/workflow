use workflow_core_tdd::region_routing::{observe_cross_region_stream, resume_hook_in_region};

#[test]
fn sfo1_reader_sees_chunks_of_an_in_progress_iad1_stream() {
    let chunk_count = 5;
    let observation = observe_cross_region_stream("iad1", "sfo1", chunk_count);

    assert!(observation.run_id.starts_with("wrun_"));
    assert_eq!(observation.writer_tagged_region, "iad1");
    assert_eq!(observation.tail_index_before_read, chunk_count - 1);
    assert_eq!(observation.server_status_before_read, "running");
    assert_eq!(observation.reader_region, "sfo1");
    assert_eq!(observation.reader_tail_index, chunk_count - 1);
    assert_eq!(observation.return_value, "done");
}

fn assert_hook_round_trip(region: &str) {
    let label = format!("e2e-region-hook-{region}");
    let observation = resume_hook_in_region(region, &label);

    assert!(observation.run_id.starts_with("wrun_"));
    assert!(observation.is_tagged);
    assert_eq!(observation.tagged_region, region);
    assert_eq!(observation.hook_run_id, observation.run_id);
    assert_eq!(observation.metadata_custom_data, label);
    assert_eq!(
        observation.payload_messages,
        vec!["one".to_owned(), "two".to_owned()]
    );
    assert_eq!(observation.server_status, "completed");
}

#[test]
fn sfo1_hook_resolves_by_token_and_resumes_its_owning_run() {
    assert_hook_round_trip("sfo1");
}

#[test]
fn fra1_hook_resolves_by_token_and_resumes_its_owning_run() {
    assert_hook_round_trip("fra1");
}
