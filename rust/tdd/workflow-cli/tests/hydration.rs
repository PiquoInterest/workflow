use workflow_cli_tdd::hydration::{
    CliError, EventResource, HydratedError, dehydrate_step_error, hydrate_resource_io,
    is_encrypted_ref,
};

#[test]
fn displays_and_decrypts_encrypted_event_errors() {
    let raw_key = [7_u8; 32];
    let error = CliError {
        message: "step failed".to_owned(),
    };
    let encrypted_error = dehydrate_step_error(&error, "run_1", &raw_key).unwrap();
    let event = EventResource {
        run_id: "run_1".to_owned(),
        event_id: "event_1".to_owned(),
        event_type: "step_failed".to_owned(),
        error: encrypted_error,
    };

    let encrypted = hydrate_resource_io(&event, None).unwrap();
    assert!(is_encrypted_ref(&encrypted.error));

    let decrypted = hydrate_resource_io(&event, Some(&raw_key)).unwrap();
    match decrypted.error {
        HydratedError::Error(error) => assert_eq!(error.message, "step failed"),
        HydratedError::EncryptedRef => panic!("encrypted payload was not decrypted"),
    }
}
