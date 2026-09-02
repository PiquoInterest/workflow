#[must_use]
pub fn frame(title: &str, contents: &[&str]) -> String {
    let mut result = title.to_owned();
    for (index, content) in contents.iter().enumerate() {
        let is_last = index + 1 == contents.len();
        let first_prefix = if is_last { "╰▶ " } else { "├▶ " };
        let continuation_prefix = if is_last { "   " } else { "│  " };

        for (line_index, line) in content.split('\n').enumerate() {
            result.push('\n');
            result.push_str(if line_index == 0 {
                first_prefix
            } else {
                continuation_prefix
            });
            result.push_str(line);
        }
    }
    result
}

#[must_use]
pub fn code(token: &str) -> String {
    format!("<i><dim>`</dim>{token}<dim>`</dim></i>")
}

#[must_use]
pub fn hint(text: &str) -> String {
    format!("<blue><b>hint:</b> {text}</blue>")
}

#[must_use]
pub fn note(lines: &[&str]) -> String {
    format!("<blue><b>note:</b> {}</blue>", lines.join("\n"))
}

#[must_use]
pub fn help(text: &str) -> String {
    format!("<cyan><b>help:</b> {text}</cyan>")
}

#[must_use]
pub fn docs(url: &str) -> String {
    format!("<blue><b>docs:</b> {url}</blue>")
}

/// Render one inline marker without trusting caller-supplied widths.
///
/// TypeScript obtains these offsets from the interpolated token itself. The
/// Rust adapter receives them explicitly, so it clamps both values to the
/// first source line instead of allocating caller-controlled padding or
/// panicking on arithmetic overflow.
#[must_use]
pub fn inline_annotation(
    source: &str,
    token_start: usize,
    token_len: usize,
    explanation: &str,
) -> String {
    let (first_line, remaining) = source
        .split_once('\n')
        .map_or((source, None), |(first, rest)| (first, Some(rest)));
    let first_line_len = first_line.chars().count();
    let start = token_start.min(first_line_len);
    let available = first_line_len.saturating_sub(start);
    let width = token_len.max(1).min(available.max(1));
    let midpoint = width / 2;

    let mut underline = " ".repeat(start);
    underline.push_str(&"─".repeat(midpoint));
    underline.push('┬');
    underline.push_str(&"─".repeat(width.saturating_sub(midpoint + 1)));

    let mut annotation = " ".repeat(start.saturating_add(midpoint));
    annotation.push_str("╰▶ ");
    annotation.push_str(explanation);

    let mut result = format!("{first_line}\n{underline}\n{annotation}");
    if let Some(remaining) = remaining {
        result.push('\n');
        result.push_str(remaining);
    }
    result
}
