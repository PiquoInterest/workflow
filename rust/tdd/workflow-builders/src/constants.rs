#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowQueueTrigger {
    pub trigger_type: String,
    pub topic: String,
    pub consumer: String,
    pub retry_after_seconds: u64,
    pub initial_delay_seconds: u64,
    pub max_concurrency: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkflowEntrypointOptions {
    pub namespace: Option<String>,
    pub base_path: Option<String>,
    pub route_module_body_started_at: Option<String>,
}

pub fn create_workflow_queue_trigger(
    explicit_namespace: Option<&str>,
    environment_namespace: Option<&str>,
) -> WorkflowQueueTrigger {
    let _ = (explicit_namespace, environment_namespace);
    panic!("TDD RED: packages/builders/src/constants.test.ts implementation pending")
}

pub fn get_workflow_queue_trigger(
    explicit_namespace: Option<&str>,
    environment_namespace: Option<&str>,
    sequential_replays: Option<&str>,
) -> WorkflowQueueTrigger {
    let _ = (
        explicit_namespace,
        environment_namespace,
        sequential_replays,
    );
    panic!("TDD RED: packages/builders/src/constants.test.ts implementation pending")
}

pub fn create_workflow_entrypoint_options_code(
    options: &WorkflowEntrypointOptions,
    environment_namespace: Option<&str>,
) -> String {
    let _ = (options, environment_namespace);
    panic!("TDD RED: packages/builders/src/constants.test.ts implementation pending")
}
