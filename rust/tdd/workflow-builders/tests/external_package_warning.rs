use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::discovery::{
    ExternalPackageWarningOptions, ExternalPackageWarningSession,
    discover_with_external_package_warnings,
};

struct TestRoot(PathBuf);

impl TestRoot {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("workflow-{label}-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_file(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn project(root: &TestRoot) -> PathBuf {
    let project = root.0.join("project");
    write_file(&project.join("index.ts"), "export const x = 1;\n");
    project
}

fn package(project: &Path, name: &str, package_json: &str, source: &str) {
    let package_dir = project.join("node_modules").join(name);
    write_file(&package_dir.join("package.json"), package_json);
    write_file(&package_dir.join("index.js"), source);
}

fn discover(
    session: &mut ExternalPackageWarningSession,
    project: &Path,
    external_packages: &[&str],
    input_name: &str,
    outdir_name: &str,
) -> Vec<String> {
    discover_with_external_package_warnings(
        session,
        &ExternalPackageWarningOptions {
            working_dir: project.to_path_buf(),
            inputs: vec![project.join(input_name)],
            outdir: project.join(outdir_name),
            external_packages: external_packages
                .iter()
                .map(|package| (*package).to_owned())
                .collect(),
        },
    )
    .unwrap()
}

fn first_warning<'a>(warnings: &'a [String], package: &str) -> &'a str {
    warnings
        .iter()
        .find(|warning| warning.contains(package))
        .unwrap()
}

#[test]
fn warns_when_an_external_package_depends_on_workflow_serde() {
    let root = TestRoot::new("external-serde-dependency");
    let project = project(&root);
    package(
        &project,
        "my-serde-pkg",
        r#"{"name":"my-serde-pkg","version":"1.0.0","main":"index.js","dependencies":{"@workflow/serde":"^1.0.0"}}"#,
        "export class Foo {}\n",
    );

    let mut session = ExternalPackageWarningSession::default();
    let warnings = discover(
        &mut session,
        &project,
        &["my-serde-pkg"],
        "index.ts",
        "out",
    );
    let warning = first_warning(&warnings, "my-serde-pkg");
    assert!(warning.contains("serverExternalPackages"));
    assert!(warning.contains("serialization classes"));
}

#[test]
fn warns_when_external_package_source_contains_serde_symbols() {
    let root = TestRoot::new("external-serde-symbols");
    let project = project(&root);
    package(
        &project,
        "my-symbol-pkg",
        r#"{"name":"my-symbol-pkg","version":"1.0.0","main":"index.js"}"#,
        r#"export class Bar {
  static [Symbol.for('workflow-serialize')](instance) { return {}; }
  static [Symbol.for('workflow-deserialize')](data) { return new Bar(); }
}
"#,
    );

    let mut session = ExternalPackageWarningSession::default();
    let warnings = discover(
        &mut session,
        &project,
        &["my-symbol-pkg"],
        "index.ts",
        "out",
    );
    assert!(first_warning(&warnings, "my-symbol-pkg").contains("serialization classes"));
}

#[test]
fn warns_when_external_package_contains_use_step() {
    let root = TestRoot::new("external-use-step");
    let project = project(&root);
    package(
        &project,
        "my-step-pkg",
        r#"{"name":"my-step-pkg","version":"1.0.0","main":"index.js"}"#,
        "export async function doWork() {\n  \"use step\";\n  return 42;\n}\n",
    );

    let mut session = ExternalPackageWarningSession::default();
    let warnings = discover(
        &mut session,
        &project,
        &["my-step-pkg"],
        "index.ts",
        "out",
    );
    assert!(first_warning(&warnings, "my-step-pkg").contains("\"use step\" functions"));
}

#[test]
fn warns_when_external_package_contains_use_workflow() {
    let root = TestRoot::new("external-use-workflow");
    let project = project(&root);
    package(
        &project,
        "my-workflow-pkg",
        r#"{"name":"my-workflow-pkg","version":"1.0.0","main":"index.js"}"#,
        "export async function runJob() {\n  \"use workflow\";\n  return \"done\";\n}\n",
    );

    let mut session = ExternalPackageWarningSession::default();
    let warnings = discover(
        &mut session,
        &project,
        &["my-workflow-pkg"],
        "index.ts",
        "out",
    );
    assert!(
        first_warning(&warnings, "my-workflow-pkg").contains("\"use workflow\" functions")
    );
}

#[test]
fn does_not_warn_for_packages_without_workflow_patterns() {
    let root = TestRoot::new("external-plain");
    let project = project(&root);
    package(
        &project,
        "plain-pkg",
        r#"{"name":"plain-pkg","version":"1.0.0","main":"index.js"}"#,
        "export const hello = \"world\";\n",
    );

    let mut session = ExternalPackageWarningSession::default();
    let warnings = discover(
        &mut session,
        &project,
        &["plain-pkg"],
        "index.ts",
        "out",
    );
    assert!(warnings.is_empty());
}

#[test]
fn does_not_warn_for_server_only_or_client_only_pseudo_packages() {
    let root = TestRoot::new("external-pseudo-packages");
    let project = project(&root);

    let mut session = ExternalPackageWarningSession::default();
    let warnings = discover(
        &mut session,
        &project,
        &["server-only", "client-only"],
        "index.ts",
        "out",
    );
    assert!(warnings.is_empty());
}

#[test]
fn does_not_warn_when_external_packages_is_empty() {
    let root = TestRoot::new("external-empty");
    let project = project(&root);

    let mut session = ExternalPackageWarningSession::default();
    let warnings = discover(&mut session, &project, &[], "index.ts", "out");
    assert!(warnings.is_empty());
}

#[test]
fn warns_only_once_per_package_across_discovery_calls() {
    let root = TestRoot::new("external-dedupe");
    let project = project(&root);
    write_file(&project.join("other.ts"), "export const y = 2;\n");
    package(
        &project,
        "my-serde-pkg",
        r#"{"name":"my-serde-pkg","version":"1.0.0","main":"index.js","dependencies":{"@workflow/serde":"^1.0.0"}}"#,
        "export class Foo {}\n",
    );

    let mut session = ExternalPackageWarningSession::default();
    let mut warnings = discover(
        &mut session,
        &project,
        &["my-serde-pkg"],
        "index.ts",
        "out",
    );
    warnings.extend(discover(
        &mut session,
        &project,
        &["my-serde-pkg"],
        "other.ts",
        "out2",
    ));

    assert_eq!(
        warnings
            .iter()
            .filter(|warning| warning.contains("my-serde-pkg"))
            .count(),
        1
    );
}

#[test]
fn warning_lists_multiple_detected_issues() {
    let root = TestRoot::new("external-multiple-issues");
    let project = project(&root);
    package(
        &project,
        "multi-pkg",
        r#"{"name":"multi-pkg","version":"1.0.0","main":"index.js","dependencies":{"@workflow/serde":"^1.0.0"}}"#,
        r#"export async function doWork() {
  "use step";
  return 42;
}
export class Foo {
  static [Symbol.for('workflow-serialize')](instance) { return {}; }
  static [Symbol.for('workflow-deserialize')](data) { return new Foo(); }
}
"#,
    );

    let mut session = ExternalPackageWarningSession::default();
    let warnings = discover(
        &mut session,
        &project,
        &["multi-pkg"],
        "index.ts",
        "out",
    );
    let warning = first_warning(&warnings, "multi-pkg");
    assert!(warning.contains("\"use step\" functions"));
    assert!(warning.contains("serialization classes"));
}

#[test]
fn warns_via_entry_file_detection_without_a_serde_dependency() {
    let root = TestRoot::new("external-entry-detection");
    let project = project(&root);
    package(
        &project,
        "no-serde-dep-pkg",
        r#"{"name":"no-serde-dep-pkg","version":"1.0.0","main":"index.js"}"#,
        "export async function doWork() {\n  \"use step\";\n  return 42;\n}\n",
    );

    let mut session = ExternalPackageWarningSession::default();
    let warnings = discover(
        &mut session,
        &project,
        &["no-serde-dep-pkg"],
        "index.ts",
        "out",
    );
    let warning = first_warning(&warnings, "no-serde-dep-pkg");
    assert!(warning.contains("\"use step\" functions"));
}
