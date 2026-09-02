#[must_use]
pub fn pluralize<'a>(singular: &'a str, plural: &'a str, count: f64) -> &'a str {
    if count == 1.0 { singular } else { plural }
}
