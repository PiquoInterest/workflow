use workflow_core_tdd::build_errors::{
    BuildFixture, ExternalPackageFixture, analyze_workflow_build,
};

const DOC_LINK: &str = "workflow-sdk.dev/err/node-js-module-in-workflow";
const STEP_SUGGESTION: &str = "Move this function into a step function";

#[test]
fn reports_a_helpful_direct_node_module_violation() {
    let fixture = BuildFixture::new(
        "test_node_violation.ts",
        r#"
import { readFileSync } from 'fs';

export async function nodeModuleViolationWorkflow() {
  'use workflow';
  const content = readFileSync('package.json', 'utf8');
  return content;
}
"#,
    );

    let observation = analyze_workflow_build(&fixture);
    assert!(!observation.succeeded);
    assert!(
        observation
            .output
            .contains("You are attempting to use \"fs\" which is a Node.js module.")
    );
    assert!(observation.output.contains("test_node_violation.ts"));
    assert!(observation.output.contains(STEP_SUGGESTION));
    assert!(observation.output.contains(DOC_LINK));
}

#[test]
fn attributes_transitive_node_usage_to_the_top_level_external_package() {
    let package_name = "workflow-test-dual-entry-package";
    let mut fixture = BuildFixture::new(
        "test_blob_violation.ts",
        &format!(
            r#"
import {{ getPlatform }} from '{package_name}';

export async function blobViolationWorkflow() {{
  'use workflow';
  return getPlatform();
}}
"#
        ),
    );
    fixture.external_packages.push(ExternalPackageFixture {
        package_name: package_name.to_owned(),
        module_entry_source: r#"
import os from 'os';
export function getPlatform() {
  return os.platform();
}
"#
        .to_owned(),
        main_entry_source: r#"
module.exports = {
  getPlatform() {
    return 'cjs';
  },
};
"#
        .to_owned(),
    });

    let observation = analyze_workflow_build(&fixture);
    assert!(!observation.succeeded);
    assert!(observation.output.contains(&format!(
        "You are attempting to use \"{package_name}\" which depends on Node.js modules."
    )));
    assert!(observation.output.contains("test_blob_violation.ts"));
    assert!(observation.output.contains(STEP_SUGGESTION));
    assert!(observation.output.contains(DOC_LINK));
}

#[test]
fn reports_a_helpful_bun_module_violation() {
    let fixture = BuildFixture::new(
        "test_bun_violation.ts",
        r#"
import { serve } from 'bun';

export async function bunViolationWorkflow() {
  'use workflow';
  const server = serve({ port: 3000, fetch: () => new Response('ok') });
  return server.port;
}
"#,
    );

    let observation = analyze_workflow_build(&fixture);
    assert!(!observation.succeeded);
    assert!(
        observation
            .output
            .contains("You are attempting to use \"bun\" which is a Bun module.")
    );
    assert!(observation.output.contains("test_bun_violation.ts"));
    assert!(observation.output.contains(STEP_SUGGESTION));
}

#[test]
fn reports_every_direct_node_module_violation_in_one_build() {
    let fixture = BuildFixture::new(
        "test_multiple_violations.ts",
        r#"
import { readFileSync } from 'fs';
import { join } from 'path';
import { createHash } from 'crypto';

export async function multipleViolationsWorkflow() {
  'use workflow';
  const content = readFileSync(join('/', 'package.json'), 'utf8');
  const hash = createHash('sha256').update(content).digest('hex');
  return hash;
}
"#,
    );

    let observation = analyze_workflow_build(&fixture);
    assert!(!observation.succeeded);
    assert!(
        observation
            .output
            .contains("\"fs\" which is a Node.js module")
    );
    assert!(
        observation
            .output
            .contains("\"path\" which is a Node.js module")
    );
    assert!(
        observation
            .output
            .contains("\"crypto\" which is a Node.js module")
    );
    assert!(observation.output.contains("test_multiple_violations.ts"));
    assert_eq!(
        observation.reported_modules,
        vec!["fs".to_owned(), "path".to_owned(), "crypto".to_owned()]
    );
}
