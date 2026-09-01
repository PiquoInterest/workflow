#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedName {
    pub short_name: String,
    pub module_specifier: String,
    pub function_name: String,
}

#[must_use]
pub fn parse_workflow_name(name: &str) -> Option<ParsedName> {
    parse_name("workflow", name)
}

#[must_use]
pub fn parse_step_name(name: &str) -> Option<ParsedName> {
    parse_name("step", name)
}

#[must_use]
pub fn parse_class_name(name: &str) -> Option<ParsedName> {
    parse_name("class", name)
}

#[must_use]
pub fn format_step_name(name: &str) -> String {
    format_parsed_name(parse_step_name(name), name)
}

#[must_use]
pub fn format_workflow_name(name: &str) -> String {
    format_parsed_name(parse_workflow_name(name), name)
}

#[must_use]
pub fn workflow_display_name(name: &str) -> String {
    parse_workflow_name(name)
        .map(|parsed| parsed.short_name)
        .or_else(|| short_name_from_sanitized("workflow", name))
        .unwrap_or_else(|| name.to_owned())
}

#[must_use]
pub fn step_display_name(name: &str) -> String {
    parse_step_name(name)
        .map(|parsed| parsed.short_name)
        .or_else(|| short_name_from_sanitized("step", name))
        .unwrap_or_else(|| name.to_owned())
}

fn parse_name(tag: &str, name: &str) -> Option<ParsedName> {
    let mut parts = name.split("//");
    let prefix = parts.next()?;
    let module_specifier = parts.next()?;
    let function_parts: Vec<_> = parts.collect();

    if prefix != tag || module_specifier.is_empty() || function_parts.is_empty() {
        return None;
    }

    let function_name = function_parts.join("//");
    let mut short_name = function_name
        .split('/')
        .next_back()
        .unwrap_or_default()
        .to_owned();
    let module_short_name = module_short_name(module_specifier);

    if matches!(short_name.as_str(), "default" | "__default")
        && !module_short_name.is_empty()
    {
        short_name = module_short_name;
    }

    Some(ParsedName {
        short_name,
        module_specifier: module_specifier.to_owned(),
        function_name,
    })
}

fn module_short_name(module_specifier: &str) -> String {
    if module_specifier.starts_with("./") {
        return module_specifier
            .split('/')
            .next_back()
            .unwrap_or_default()
            .to_owned();
    }

    let parts: Vec<_> = module_specifier.split('@').collect();
    let without_version = parts
        .get(..parts.len().saturating_sub(1))
        .unwrap_or_default()
        .join("@");
    let package_name = if without_version.is_empty() {
        parts.first().copied().unwrap_or_default()
    } else {
        without_version.as_str()
    };

    package_name
        .split('/')
        .next_back()
        .unwrap_or_default()
        .to_owned()
}

fn short_name_from_sanitized(tag: &str, name: &str) -> Option<String> {
    let expected_prefix = format!("{tag}--");
    if !name.starts_with(&expected_prefix) {
        return None;
    }

    let segments: Vec<_> = name.split("--").filter(|segment| !segment.is_empty()).collect();
    let function_part = segments.last().copied()?;
    let mut short_name = function_part
        .split('-')
        .filter(|segment| !segment.is_empty())
        .next_back()
        .unwrap_or_default()
        .to_owned();

    if matches!(short_name.as_str(), "default" | "__default") {
        let module_short_name = segments
            .get(segments.len().saturating_sub(2))
            .and_then(|segment| {
                segment
                    .split('-')
                    .filter(|part| !part.is_empty())
                    .next_back()
            });
        if let Some(module_short_name) = module_short_name {
            if module_short_name != tag {
                short_name = module_short_name.to_owned();
            }
        }
    }

    if short_name.is_empty() {
        None
    } else {
        Some(short_name)
    }
}

fn format_parsed_name(parsed: Option<ParsedName>, fallback: &str) -> String {
    match parsed {
        Some(parsed) => format!(
            "{} ({})",
            escape_single_line(&parsed.short_name),
            escape_single_line(&parsed.module_specifier)
        ),
        None => escape_single_line(fallback),
    }
}

fn escape_single_line(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{0000}'..='\u{001f}' | '\u{007f}'..='\u{009f}' | '\u{2028}' | '\u{2029}' => {
                use std::fmt::Write;
                write!(&mut escaped, "\\u{:04x}", character as u32)
                    .expect("writing to a String cannot fail");
            }
            _ => escaped.push(character),
        }
    }
    escaped
}
