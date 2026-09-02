use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::pseudo_package::{
    BundleObservation, BundleOptions, PSEUDO_PACKAGES, bundle_with_pseudo_package_support,
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

    fn entry(&self, source: &str) -> PathBuf {
        let path = self.0.join("workflow.ts");
        fs::write(&path, source).unwrap();
        path
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

fn write_package(root: &Path, name: &str, source: &str) {
    let package_dir = root.join("node_modules").join(name);
    write_file(&package_dir.join("index.js"), source);
    write_file(
        &package_dir.join("package.json"),
        &format!(r#"{{"name":"{name}","main":"index.js"}}"#),
    );
}

fn bundle(
    root: &TestRoot,
    source: &str,
    use_pseudo_package_plugin: bool,
    external_packages: &[&str],
) -> Result<BundleObservation, String> {
    bundle_with_pseudo_package_support(&BundleOptions {
        entry_file: root.entry(source),
        use_pseudo_package_plugin,
        external_packages: external_packages
            .iter()
            .map(|package| (*package).to_owned())
            .collect::<BTreeSet<_>>(),
    })
}

fn assert_import_absent(output: &str, package: &str) {
    assert!(!output.contains(&format!("require('{package}')")));
    assert!(!output.contains(&format!("require(\"{package}\")")));
}

#[test]
fn replaces_server_only_import_with_an_empty_module() {
    let root = TestRoot::new("pseudo-server-only");
    let observation = bundle(
        &root,
        r#"
        import 'server-only';
        export function workflow() {
          return "hello";
        }
        "#,
        true,
        &[],
    )
    .unwrap();

    assert!(observation.errors.is_empty());
    assert_import_absent(&observation.output, "server-only");
}

#[test]
fn handles_server_only_alongside_other_code() {
    let root = TestRoot::new("pseudo-server-only-code");
    let observation = bundle(
        &root,
        r#"
        import 'server-only';
        const x = 42;
        export function workflow() {
          return x;
        }
        "#,
        true,
        &[],
    )
    .unwrap();

    assert!(observation.errors.is_empty());
    assert_import_absent(&observation.output, "server-only");
    assert!(observation.output.contains("42"));
}

#[test]
fn replaces_client_only_import_with_an_empty_module() {
    let root = TestRoot::new("pseudo-client-only");
    let observation = bundle(
        &root,
        "import 'client-only';\nexport const x = 1;\n",
        true,
        &[],
    )
    .unwrap();

    assert!(observation.errors.is_empty());
    assert_import_absent(&observation.output, "client-only");
}

#[test]
fn replaces_compiled_server_only_import() {
    let root = TestRoot::new("pseudo-compiled-server-only");
    let observation = bundle(
        &root,
        "import 'next/dist/compiled/server-only';\nexport const x = 1;\n",
        true,
        &[],
    )
    .unwrap();

    assert!(observation.errors.is_empty());
    assert_import_absent(&observation.output, "next/dist/compiled/server-only");
}

#[test]
fn replaces_compiled_client_only_import() {
    let root = TestRoot::new("pseudo-compiled-client-only");
    let observation = bundle(
        &root,
        "import 'next/dist/compiled/client-only';\nexport const x = 1;\n",
        true,
        &[],
    )
    .unwrap();

    assert!(observation.errors.is_empty());
    assert_import_absent(&observation.output, "next/dist/compiled/client-only");
}

#[test]
fn handles_server_only_and_client_only_in_the_same_file() {
    let root = TestRoot::new("pseudo-both");
    let observation = bundle(
        &root,
        r#"
        import 'server-only';
        import 'client-only';
        export function workflow() {
          return "mixed";
        }
        "#,
        true,
        &[],
    )
    .unwrap();

    assert!(observation.errors.is_empty());
    assert_import_absent(&observation.output, "server-only");
    assert_import_absent(&observation.output, "client-only");
}

#[test]
fn a_missing_server_only_package_is_not_silently_resolved_without_the_plugin() {
    let root = TestRoot::new("pseudo-without-plugin");
    let result = bundle(
        &root,
        "import 'server-only';\nexport const x = 1;\n",
        false,
        &[],
    );

    if let Err(error) = result {
        assert!(error.contains("Could not resolve \"server-only\""));
    }
}

#[test]
fn pseudo_package_set_contains_only_the_four_next_marker_packages() {
    assert_eq!(
        PSEUDO_PACKAGES,
        [
            "server-only",
            "client-only",
            "next/dist/compiled/server-only",
            "next/dist/compiled/client-only",
        ]
    );
}

#[test]
fn inlines_dynamic_imports_when_the_package_is_not_external() {
    let root = TestRoot::new("dynamic-inline");
    write_package(&root.0, "my-test-package", "export const testValue = 42;");
    let observation = bundle(
        &root,
        r#"
        export async function workflow() {
          const pkg = await import('my-test-package');
          return pkg.testValue;
        }
        "#,
        false,
        &[],
    )
    .unwrap();

    assert!(observation.errors.is_empty());
    assert!(observation.output.contains("42"));
}

#[test]
fn leaves_dynamic_imports_in_place_when_the_package_is_external() {
    let root = TestRoot::new("dynamic-external");
    write_package(&root.0, "external-package", "export const value = 100;");
    let observation = bundle(
        &root,
        r#"
        export async function workflow() {
          const pkg = await import('external-package');
          return pkg.value;
        }
        "#,
        false,
        &["external-package"],
    )
    .unwrap();

    assert!(observation.errors.is_empty());
    assert!(
        observation.output.contains("import('external-package')")
            || observation.output.contains("import(\"external-package\")")
    );
    assert!(!observation.output.contains("100"));
}

#[test]
fn inlines_nested_dynamic_imports_from_dependencies() {
    let root = TestRoot::new("dynamic-nested");
    write_package(&root.0, "inner-package", "export const innerValue = 999;");
    write_package(
        &root.0,
        "outer-package",
        r#"
        export async function loadInner() {
          const inner = await import('inner-package');
          return inner.innerValue;
        }
        "#,
    );
    let observation = bundle(
        &root,
        r#"
        import { loadInner } from 'outer-package';
        export async function workflow() {
          return loadInner();
        }
        "#,
        false,
        &[],
    )
    .unwrap();

    assert!(observation.errors.is_empty());
    assert!(observation.output.contains("999"));
}
