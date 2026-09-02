use workflow_ai_tdd::{IteratorCase, exercise_stream_iterator};

#[test]
fn converts_an_async_generator_to_a_readable_stream() {
    let observation = exercise_stream_iterator(IteratorCase::GeneratorToStream);
    assert_eq!(observation.values, vec![1, 2, 3]);
    assert!(observation.done);
}

#[test]
fn yields_to_the_macrotask_queue_between_browser_chunks() {
    let observation = exercise_stream_iterator(IteratorCase::BrowserMacrotaskYield);
    assert_eq!(observation.values, vec![1, 2]);
    assert_eq!(observation.macrotask_yields, 2);
}

#[test]
fn skips_macrotask_yield_outside_browsers() {
    let observation = exercise_stream_iterator(IteratorCase::NonBrowserNoYield);
    assert_eq!(observation.values, vec![1, 2, 3]);
    assert_eq!(observation.macrotask_yields, 0);
    assert!(observation.done);
}

#[test]
fn aborts_after_the_first_chunk() {
    let observation = exercise_stream_iterator(IteratorCase::AbortAfterFirst);
    assert_eq!(observation.values, vec![1]);
    assert!(observation.error.is_some());
}

#[test]
fn rejects_an_already_aborted_signal_before_yielding() {
    let observation = exercise_stream_iterator(IteratorCase::AlreadyAborted);
    assert!(observation.values.is_empty());
    assert!(observation.error.is_some());
}

#[test]
fn propagates_generator_errors() {
    let observation = exercise_stream_iterator(IteratorCase::GeneratorError);
    assert_eq!(observation.values, vec![1]);
    assert_eq!(observation.error.as_deref(), Some("generator error"));
}

#[test]
fn converts_a_readable_stream_to_an_async_iterator() {
    let observation = exercise_stream_iterator(IteratorCase::StreamToIterator);
    assert_eq!(observation.values, vec![1, 2, 3]);
    assert!(observation.done);
}
