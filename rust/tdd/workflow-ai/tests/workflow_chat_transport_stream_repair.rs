use workflow_ai_tdd::{ChatRepairCase, exercise_chat_repair};

#[test]
fn raw_interleaved_text_is_fatal_to_the_consumer() {
    let observation = exercise_chat_repair(ChatRepairCase::RawInterleavedText);
    assert!(
        observation
            .consumer_error
            .as_deref()
            .is_some_and(|message| message.contains(
                "Received text-delta for missing text part with ID \"0\""
            ))
    );
}

#[test]
fn repairs_interleaved_text_without_losing_content() {
    let observation = exercise_chat_repair(ChatRepairCase::RepairedInterleavedText);
    assert!(observation.consumer_error.is_none());
    assert_eq!(observation.text, "Hello world");
}

#[test]
fn drops_a_replayed_duplicate_tail_without_erroring() {
    let observation = exercise_chat_repair(ChatRepairCase::DuplicateTextTail);
    assert!(observation.consumer_error.is_none());
    assert_eq!(observation.text, "Hello world");
}

#[test]
fn passes_well_formed_multi_step_streams_through_unchanged() {
    let observation = exercise_chat_repair(ChatRepairCase::WellFormedMultiStep);
    assert!(observation.consumer_error.is_none());
    assert_eq!(observation.text, "onetwo");
}

#[test]
fn raw_interleaved_reasoning_is_fatal_to_the_consumer() {
    let observation = exercise_chat_repair(ChatRepairCase::RawInterleavedReasoning);
    assert!(
        observation
            .consumer_error
            .as_deref()
            .is_some_and(|message| message.contains(
                "Received reasoning-delta for missing reasoning part with ID \"0\""
            ))
    );
}

#[test]
fn repairs_interleaved_reasoning_without_losing_content() {
    let observation = exercise_chat_repair(ChatRepairCase::RepairedInterleavedReasoning);
    assert!(observation.consumer_error.is_none());
    assert_eq!(observation.reasoning, "Thinking...");
}
