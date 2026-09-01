use workflow_errors_tdd::ansi;

#[test]
fn frame_renders_a_single_line_title_with_no_contents() {
    assert_eq!(
        ansi::frame("something went wrong", &[]),
        "something went wrong"
    );
}

#[test]
fn frame_renders_a_single_content_line_with_last_branch() {
    assert_eq!(
        ansi::frame("something went wrong", &["here is why"]),
        "something went wrong\n╰▶ here is why"
    );
}

#[test]
fn frame_renders_multiple_contents_with_middle_and_last_branches() {
    assert_eq!(
        ansi::frame("something went wrong", &["first reason", "second reason"]),
        "something went wrong\n├▶ first reason\n╰▶ second reason"
    );
}

#[test]
fn frame_indents_continuation_lines_under_their_branch() {
    assert_eq!(
        ansi::frame("title", &["first\nwith two lines", "last\nalso two lines"]),
        "title\n├▶ first\n│  with two lines\n╰▶ last\n   also two lines"
    );
}

#[test]
fn code_wraps_a_token_in_dim_backticks_and_italics() {
    assert_eq!(ansi::code("fn()"), "<i><dim>`</dim>fn()<dim>`</dim></i>");
}

#[test]
fn hint_renders_a_hint_line() {
    assert_eq!(
        ansi::hint("try reloading"),
        "<blue><b>hint:</b> try reloading</blue>"
    );
}

#[test]
fn note_renders_a_note_line() {
    assert_eq!(
        ansi::note(&["read more:", "https://example.com"]),
        "<blue><b>note:</b> read more:\nhttps://example.com</blue>"
    );
}

#[test]
fn help_renders_a_help_line() {
    assert_eq!(
        ansi::help("run `wf inspect run run_123`"),
        "<cyan><b>help:</b> run `wf inspect run run_123`</cyan>"
    );
}

#[test]
fn docs_renders_a_docs_line() {
    assert_eq!(
        ansi::docs("https://workflow-sdk.dev/docs/api-reference/workflow/sleep"),
        "<blue><b>docs:</b> https://workflow-sdk.dev/docs/api-reference/workflow/sleep</blue>"
    );
}

#[test]
fn inline_underlines_a_single_token_on_a_single_line() {
    assert_eq!(
        ansi::inline_annotation("function hello()", 9, 5, "name not allowed"),
        "function hello()\n         ──┬──\n           ╰▶ name not allowed"
    );
}

#[test]
fn inline_preserves_subsequent_lines_unchanged() {
    assert_eq!(
        ansi::inline_annotation("const x = 1\nconst y = 2", 6, 1, "unused"),
        "const x = 1\n      ┬\n      ╰▶ unused\nconst y = 2"
    );
}
