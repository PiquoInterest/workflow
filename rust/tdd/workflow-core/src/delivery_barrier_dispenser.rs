fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/delivery-barrier-dispenser.test.ts implementation pending")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryBarrierScenario {
    ParkedPayloadBeforeSuspension,
    MultipleParkedSegments,
    RejectedPromiseQueue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingStep {
    pub name: String,
    pub correlation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeliveryBarrierObservation {
    pub suspended: bool,
    pub pending_steps: Vec<PendingStep>,
    pub registry_size_before_release: usize,
    pub registry_size_after_release: usize,
    pub delivery_idle_before_release: bool,
    pub delivery_idle_after_release: bool,
}

/// Drives the future delivery-barrier safety-net through one production shape.
pub fn run_delivery_barrier_scenario(
    scenario: DeliveryBarrierScenario,
) -> DeliveryBarrierObservation {
    let _ = scenario;
    pending()
}
