use workflow_builders_tdd::swc_plugin::SwcMode;

use super::support::{TestRoot, build, options, passthrough, set_entries_to_bundle, write_file};

fn assert_rewrites_extension(input_ext: &str, output_ext: &str) {
    let root = TestRoot::new(&format!("rewrite-{}", &input_ext[1..]));
    let outdir = root.join("out");
    let dependency = root.write(&format!("src/dep{input_ext}"), "export const dep = {};");
    let step = root.write(
        "src/step.ts",
        "import { dep } from './dep';\nconsole.log(dep);",
    );
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    options.outdir = Some(outdir);
    options.rewrite_ts_extensions = true;
    set_entries_to_bundle(&mut options, &[&step]);
    passthrough(&mut options, &step);
    passthrough(&mut options, &dependency);

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(observation.output.contains(&format!("/dep{output_ext}")));
    assert!(!observation.output.contains(&format!("/dep{input_ext}")));
}

#[test]
fn rewrites_externalized_ts_imports_to_js_when_enabled() {
    assert_rewrites_extension(".ts", ".js");
}

#[test]
fn rewrites_externalized_tsx_imports_to_js_when_enabled() {
    assert_rewrites_extension(".tsx", ".js");
}

#[test]
fn rewrites_externalized_mts_imports_to_mjs_when_enabled() {
    assert_rewrites_extension(".mts", ".mjs");
}

#[test]
fn rewrites_externalized_cts_imports_to_cjs_when_enabled() {
    assert_rewrites_extension(".cts", ".cjs");
}

#[test]
fn bundles_path_aliased_project_local_imports_inline() {
    let root = TestRoot::new("alias-local");
    let outdir = root.join("out");
    let config = root.write(
        "src/lib/config.ts",
        "export const config = { value: \"hello-from-config\" };",
    );
    let step = root.write(
        "src/step.ts",
        "import { config } from '@/lib/config';\nconsole.log(config);",
    );
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    options.outdir = Some(outdir);
    options.rewrite_ts_extensions = true;
    options.aliases.insert("@".to_owned(), root.join("src"));
    set_entries_to_bundle(&mut options, &[&step]);
    passthrough(&mut options, &step);
    passthrough(&mut options, &config);

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(observation.output.contains("hello-from-config"));
    assert!(!observation.output.contains("@/lib/config"));
    assert!(!observation.output.contains("/lib/config.js"));
    assert!(!observation.output.contains("/lib/config.ts"));
}

#[test]
fn bundles_transitive_aliased_imports_inside_aliased_helpers() {
    let root = TestRoot::new("alias-transitive");
    let outdir = root.join("out");
    let providers = root.write(
        "src/lib/providers.ts",
        "export const providerName = \"anthropic\";",
    );
    let client_factory = root.write(
        "src/lib/client-factory.ts",
        "import { providerName } from '@my-pkg/lib/providers';\n\
         export const client = { provider: providerName };",
    );
    let step = root.write(
        "src/step.ts",
        "import { client } from '@my-pkg/lib/client-factory';\nconsole.log(client);",
    );
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    options.outdir = Some(outdir);
    options.rewrite_ts_extensions = true;
    options
        .aliases
        .insert("@my-pkg".to_owned(), root.join("src"));
    set_entries_to_bundle(&mut options, &[&step]);
    for path in [&providers, &client_factory, &step] {
        passthrough(&mut options, path);
    }

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(observation.output.contains("anthropic"));
    assert!(!observation.output.contains("@my-pkg/lib/providers"));
    assert!(!observation.output.contains("@my-pkg/lib/client-factory"));
}

#[test]
fn does_not_relativize_node_builtins() {
    let root = TestRoot::new("node-builtins");
    let outdir = root.join("out");
    let step = root.write(
        "src/step.ts",
        "import { createHash } from 'crypto';\n\
         import { join } from 'node:path';\n\
         console.log(createHash, join);",
    );
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    options.outdir = Some(outdir);
    set_entries_to_bundle(&mut options, &[&step]);
    passthrough(&mut options, &step);

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(
        observation.output.contains("from \"crypto\"")
            || observation.output.contains("from 'crypto'")
    );
    assert!(
        observation.output.contains("from \"node:path\"")
            || observation.output.contains("from 'node:path'")
    );
    assert!(!observation.output.contains("from \"./crypto\""));
    assert!(!observation.output.contains("from './crypto'"));
    assert!(!observation.output.contains("from \"./node:path\""));
    assert!(!observation.output.contains("from './node:path'"));
}

#[test]
fn does_not_externalize_aliased_imports_that_resolve_into_node_modules() {
    let root = TestRoot::new("alias-node-modules");
    let outdir = root.join("out");
    let package = root.write(
        "node_modules/some-pkg/index.js",
        "export const pkg = \"hello\";",
    );
    let step = root.write(
        "src/step.ts",
        "import { pkg } from '@pkg';\nconsole.log(pkg);",
    );
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    options.outdir = Some(outdir);
    options.aliases.insert("@pkg".to_owned(), package.clone());
    set_entries_to_bundle(&mut options, &[&step]);
    passthrough(&mut options, &step);
    passthrough(&mut options, &package);

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(observation.output.contains("hello"));
    assert!(!observation.output.contains("from \"node_modules"));
    assert!(!observation.output.contains("from 'node_modules"));
}

#[test]
fn externalizes_nested_bare_packages_that_only_resolve_from_a_bundled_package() {
    let root = TestRoot::new("nested-native");
    let outdir = root.join("out");
    let parent_dir = root.join("node_modules/parent-pkg");
    write_file(
        &parent_dir.join("package.json"),
        r#"{"name":"parent-pkg","main":"index.js"}"#,
    );
    let parent = parent_dir.join("index.js");
    write_file(
        &parent,
        "const native = require('optional-native');\nexports.value = native.value;",
    );
    let native_dir = parent_dir.join("node_modules/optional-native");
    write_file(
        &native_dir.join("package.json"),
        r#"{"name":"optional-native","main":"binding.node"}"#,
    );
    write_file(&native_dir.join("binding.node"), "");
    let step = root.write(
        "src/step.ts",
        "import { value } from 'parent-pkg';\nconsole.log(value);",
    );
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    options.outdir = Some(outdir);
    set_entries_to_bundle(&mut options, &[&step, &parent]);
    passthrough(&mut options, &step);
    passthrough(&mut options, &parent);

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(observation.output.contains("optional-native"));
    assert!(!observation.output.contains("binding.node"));
}

fn assert_preserves_extension(input_ext: &str) {
    let root = TestRoot::new(&format!("preserve-{}", &input_ext[1..]));
    let outdir = root.join("out");
    let dependency = root.write(&format!("src/dep{input_ext}"), "export const dep = {};");
    let step = root.write(
        "src/step.ts",
        "import { dep } from './dep';\nconsole.log(dep);",
    );
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    options.outdir = Some(outdir);
    set_entries_to_bundle(&mut options, &[&step]);
    passthrough(&mut options, &step);
    passthrough(&mut options, &dependency);

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(observation.output.contains(&format!("/dep{input_ext}")));
}

#[test]
fn preserves_externalized_ts_extensions_by_default() {
    assert_preserves_extension(".ts");
}

#[test]
fn preserves_externalized_tsx_extensions_by_default() {
    assert_preserves_extension(".tsx");
}

#[test]
fn preserves_externalized_mts_extensions_by_default() {
    assert_preserves_extension(".mts");
}

#[test]
fn preserves_externalized_cts_extensions_by_default() {
    assert_preserves_extension(".cts");
}

fn transitive_local_fixture(
    root: &TestRoot,
) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let helper = root.write(
        "shared/helpers.ts",
        "export const HELPER_VALUE = \"from-helper\";",
    );
    let constants = root.write(
        "shared/constants.ts",
        "import { HELPER_VALUE } from './helpers';\n\
         export const CATEGORIES = [HELPER_VALUE];",
    );
    let step = root.write(
        "server/workflows/my-workflow.ts",
        "import { CATEGORIES } from '../../shared/constants';\n\
         export async function myStep() {\n\
           'use step';\n\
           return CATEGORIES[0];\n\
         }",
    );
    (step, constants, helper)
}

#[test]
fn bundles_transitive_local_typescript_dependencies_with_extensionless_imports() {
    let root = TestRoot::new("transitive-local-bundled");
    let (step, constants, helper) = transitive_local_fixture(&root);
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    options.outdir = Some(root.join("out"));
    options.bundle_transitive_local_step_dependencies = true;
    set_entries_to_bundle(&mut options, &[&step]);
    for path in [&step, &constants, &helper] {
        passthrough(&mut options, path);
    }

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(observation.output.contains("from-helper"));
    assert!(!observation.output.contains("shared/constants"));
    assert!(!observation.output.contains("shared/helpers"));
}

#[test]
fn externalizes_transitive_local_typescript_dependencies_by_default() {
    let root = TestRoot::new("transitive-local-external");
    let (step, constants, helper) = transitive_local_fixture(&root);
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    options.outdir = Some(root.join("out"));
    set_entries_to_bundle(&mut options, &[&step]);
    for path in [&step, &constants, &helper] {
        passthrough(&mut options, path);
    }

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(!observation.output.contains("from-helper"));
    assert!(observation.output.contains("shared/constants.ts"));
}

#[test]
fn keeps_ordinary_package_dependencies_external_when_reachable_from_a_bundled_step() {
    let root = TestRoot::new("ordinary-package-external");
    let package_dir = root.join("node_modules/plain-pkg");
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"plain-pkg","main":"index.js"}"#,
    );
    let package = package_dir.join("index.js");
    write_file(&package, "export const value = \"from-package\";");
    let step = root.write(
        "server/workflows/my-workflow.ts",
        "import { value } from 'plain-pkg';\n\
         export async function myStep() {\n\
           'use step';\n\
           return value;\n\
         }",
    );
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    options.outdir = Some(root.join("out"));
    options.bundle_transitive_local_step_dependencies = true;
    set_entries_to_bundle(&mut options, &[&step]);
    passthrough(&mut options, &step);
    passthrough(&mut options, &package);

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(
        observation.output.contains("from \"plain-pkg\"")
            || observation.output.contains("from 'plain-pkg'")
    );
    assert!(!observation.output.contains("from-package"));
}

#[test]
fn bundles_package_parents_that_lead_to_discovered_workflow_entries() {
    let root = TestRoot::new("package-parent-bundled");
    let package_dir = root.join("node_modules/workflow-pkg");
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"workflow-pkg","main":"index.js"}"#,
    );
    let package_index = package_dir.join("index.js");
    let package_serde = package_dir.join("serde.js");
    write_file(&package_serde, "export const value = \"from-serde\";");
    write_file(&package_index, "export { value } from './serde.js';");
    let step = root.write(
        "server/workflows/my-workflow.ts",
        "import { value } from 'workflow-pkg';\n\
         export async function myStep() {\n\
           'use step';\n\
           return value;\n\
         }",
    );
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    options.outdir = Some(root.join("out"));
    set_entries_to_bundle(&mut options, &[&step, &package_serde]);
    for path in [&step, &package_index, &package_serde] {
        passthrough(&mut options, path);
    }

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(observation.output.contains("from-serde"));
    assert!(!observation.output.contains("from \"workflow-pkg\""));
    assert!(!observation.output.contains("from 'workflow-pkg'"));
}
