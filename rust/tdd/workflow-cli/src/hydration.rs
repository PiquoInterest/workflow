#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedStepError {
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventResource {
    pub run_id: String,
    pub event_id: String,
    pub event_type: String,
    pub error: EncryptedStepError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HydratedError {
    EncryptedRef,
    Error(CliError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HydratedEventResource {
    pub run_id: String,
    pub event_id: String,
    pub event_type: String,
    pub error: HydratedError,
}

pub fn dehydrate_step_error(
    error: &CliError,
    run_id: &str,
    raw_key: &[u8; 32],
) -> Result<EncryptedStepError, String> {
    let _ = (error, run_id, raw_key);
    panic!("TDD RED: packages/cli/src/lib/inspect/hydration.test.ts implementation pending")
}

pub fn hydrate_resource_io(
    resource: &EventResource,
    raw_key: Option<&[u8; 32]>,
) -> Result<HydratedEventResource, String> {
    let _ = (resource, raw_key);
    panic!("TDD RED: packages/cli/src/lib/inspect/hydration.test.ts implementation pending")
}

pub const fn is_encrypted_ref(value: &HydratedError) -> bool {
    matches!(value, HydratedError::EncryptedRef)
}
