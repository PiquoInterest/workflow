use workflow_core_tdd::world_singleton::initialize_runtime_world;

#[test]
fn route_initialization_reuses_the_runtime_world_singleton() {
    let observation = initialize_runtime_world();
    assert_eq!(observation.response_status, 204);
    assert_eq!(observation.runtime_world_id, observation.created_world_id);
    assert_eq!(observation.local_world_create_calls, 1);
}
