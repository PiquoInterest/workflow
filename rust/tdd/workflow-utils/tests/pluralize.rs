use workflow_utils_tdd::pluralize;

#[test]
fn returns_singular_form_when_count_is_one() {
    assert_eq!(pluralize("step", "steps", 1.0), "step");
    assert_eq!(pluralize("retry", "retries", 1.0), "retry");
    assert_eq!(pluralize("hook", "hooks", 1.0), "hook");
}

#[test]
fn returns_plural_form_when_count_is_zero() {
    assert_eq!(pluralize("step", "steps", 0.0), "steps");
    assert_eq!(pluralize("retry", "retries", 0.0), "retries");
}

#[test]
fn returns_plural_form_when_count_is_greater_than_one() {
    assert_eq!(pluralize("step", "steps", 2.0), "steps");
    assert_eq!(pluralize("retry", "retries", 3.0), "retries");
    assert_eq!(pluralize("hook", "hooks", 100.0), "hooks");
}

#[test]
fn supports_irregular_has_have_forms() {
    assert_eq!(pluralize("has", "have", 1.0), "has");
    assert_eq!(pluralize("has", "have", 2.0), "have");
    assert_eq!(pluralize("has", "have", 0.0), "have");
}
