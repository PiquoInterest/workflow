use workflow_world_tdd::{
    Environment, NODE_HTTP_DEFAULT, NODE_HTTP_ENV_VAR, is_node_http_enabled,
};

#[test]
fn follows_the_compiled_default_when_unset() {
    assert_eq!(is_node_http_enabled(Some(&Environment::new())), NODE_HTTP_DEFAULT);
}

#[test]
fn is_switchable_in_both_directions() {
    for (raw, expected) in [("1", true), ("0", false), ("true", true), ("false", false)] {
        let environment = Environment::from([(NODE_HTTP_ENV_VAR.to_owned(), raw.to_owned())]);
        assert_eq!(is_node_http_enabled(Some(&environment)), expected);
    }
}

#[test]
fn uses_the_process_environment_when_no_map_is_supplied() {
    // The production implementation reads process.env lazily. The Rust core
    // represents that source with `None`; a later JavaScript binding test must
    // additionally exercise live process-global mutation in an isolated child.
    assert_eq!(is_node_http_enabled(None), NODE_HTTP_DEFAULT);
}
