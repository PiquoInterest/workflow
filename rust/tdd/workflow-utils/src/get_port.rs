use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkflowPortOptions {
    pub timeout: Duration,
}

impl Default for WorkflowPortOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(1_000),
        }
    }
}

#[must_use]
pub fn get_port() -> Option<u16> {
    panic!("TDD RED: packages/utils/src/get-port.test.ts implementation pending")
}

#[must_use]
pub fn get_all_ports() -> Vec<u16> {
    panic!("TDD RED: packages/utils/src/get-port.test.ts implementation pending")
}

#[must_use]
pub fn get_workflow_port(options: WorkflowPortOptions) -> Option<u16> {
    let _ = options;
    panic!("TDD RED: packages/utils/src/get-port.test.ts implementation pending")
}

#[must_use]
pub fn parse_windows_netstat_ports_for_pid(output: &str, process_id: u32) -> Vec<u16> {
    let _ = (output, process_id);
    panic!("TDD RED: packages/utils/src/get-port.test.ts implementation pending")
}
