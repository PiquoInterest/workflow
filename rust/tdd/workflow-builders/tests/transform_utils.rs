use workflow_builders_tdd::transform_utils::{
    WorkflowPatternDetection, detect_workflow_patterns, matches_use_step, matches_use_workflow,
    matches_workflow_serde_computed_property, matches_workflow_serde_import,
    matches_workflow_serde_symbol, should_transform_file,
};

fn no_patterns() -> WorkflowPatternDetection {
    WorkflowPatternDetection::default()
}

#[test]
fn use_workflow_matches_single_quotes() {
    assert!(matches_use_workflow("'use workflow';"));
    assert!(matches_use_workflow("'use workflow'"));
}

#[test]
fn use_workflow_matches_double_quotes() {
    assert!(matches_use_workflow("\"use workflow\";"));
    assert!(matches_use_workflow("\"use workflow\""));
}

#[test]
fn use_workflow_matches_leading_whitespace() {
    assert!(matches_use_workflow("  'use workflow';"));
    assert!(matches_use_workflow("\t\"use workflow\";"));
}

#[test]
fn use_workflow_does_not_match_inline_usage() {
    assert!(!matches_use_workflow("const x = 'use workflow';"));
}

#[test]
fn use_step_matches_single_quotes() {
    assert!(matches_use_step("'use step';"));
    assert!(matches_use_step("'use step'"));
}

#[test]
fn use_step_matches_double_quotes() {
    assert!(matches_use_step("\"use step\";"));
    assert!(matches_use_step("\"use step\""));
}

#[test]
fn serde_import_matches_single_quotes() {
    assert!(matches_workflow_serde_import(
        "import { WORKFLOW_SERIALIZE } from '@workflow/serde';"
    ));
}

#[test]
fn serde_import_matches_double_quotes() {
    assert!(matches_workflow_serde_import(
        "import { WORKFLOW_SERIALIZE } from \"@workflow/serde\";"
    ));
}

#[test]
fn serde_import_matches_multiple_specifiers() {
    assert!(matches_workflow_serde_import(
        "import { WORKFLOW_SERIALIZE, WORKFLOW_DESERIALIZE } from '@workflow/serde';"
    ));
}

#[test]
fn serde_import_matches_type_imports() {
    assert!(matches_workflow_serde_import(
        "import type { SerializationSymbol } from '@workflow/serde';"
    ));
}

#[test]
fn serde_import_does_not_match_similar_packages() {
    assert!(!matches_workflow_serde_import(
        "import { x } from '@other/serde';"
    ));
    assert!(!matches_workflow_serde_import(
        "import { x } from '@workflow/serde-utils';"
    ));
}

#[test]
fn serde_symbol_matches_workflow_serialize() {
    assert!(matches_workflow_serde_symbol(
        "static [Symbol.for('workflow-serialize')](instance) {}"
    ));
}

#[test]
fn serde_symbol_matches_workflow_deserialize() {
    assert!(matches_workflow_serde_symbol(
        "static [Symbol.for('workflow-deserialize')](data) {}"
    ));
}

#[test]
fn serde_symbol_matches_double_quotes() {
    assert!(matches_workflow_serde_symbol(
        "static [Symbol.for(\"workflow-serialize\")](instance) {}"
    ));
}

#[test]
fn serde_symbol_matches_whitespace_variations() {
    assert!(matches_workflow_serde_symbol(
        "Symbol.for( 'workflow-serialize' )"
    ));
    assert!(matches_workflow_serde_symbol(
        "Symbol.for('workflow-deserialize')"
    ));
}

#[test]
fn serde_symbol_matches_inside_a_full_class() {
    let source = r#"
        export class Point {
          constructor(x, y) {
            this.x = x;
            this.y = y;
          }

          static [Symbol.for('workflow-serialize')](instance) {
            return { x: instance.x, y: instance.y };
          }

          static [Symbol.for('workflow-deserialize')](data) {
            return new Point(data.x, data.y);
          }
        }
    "#;
    assert!(matches_workflow_serde_symbol(source));
}

#[test]
fn serde_symbol_does_not_match_other_symbols() {
    assert!(!matches_workflow_serde_symbol("Symbol.for('other-symbol')"));
    assert!(!matches_workflow_serde_symbol(
        "Symbol.for('workflow-something-else')"
    ));
}

#[test]
fn serde_symbol_does_not_match_plain_strings_or_identifiers() {
    assert!(!matches_workflow_serde_symbol("'workflow-serialize'"));
    assert!(!matches_workflow_serde_symbol("workflow-deserialize"));
}

#[test]
fn combined_detection_recognizes_imported_symbols_only_as_imports() {
    let source = r#"
        import { WORKFLOW_SERIALIZE, WORKFLOW_DESERIALIZE } from '@workflow/serde';

        export class MyClass {
          static [WORKFLOW_SERIALIZE](instance) {
            return { value: instance.value };
          }

          static [WORKFLOW_DESERIALIZE](data) {
            return new MyClass(data.value);
          }
        }
    "#;
    assert!(matches_workflow_serde_import(source));
    assert!(!matches_workflow_serde_symbol(source));
}

#[test]
fn combined_detection_recognizes_direct_symbols_without_an_import() {
    let source = r#"
        export class Point {
          static [Symbol.for('workflow-serialize')](instance) {
            return { x: instance.x };
          }

          static [Symbol.for('workflow-deserialize')](data) {
            return new Point(data.x);
          }
        }
    "#;
    assert!(!matches_workflow_serde_import(source));
    assert!(matches_workflow_serde_symbol(source));
}

#[test]
fn combined_detection_recognizes_import_and_direct_symbol_patterns() {
    let source = r#"
        import { WORKFLOW_SERIALIZE } from '@workflow/serde';

        export class Point {
          static [WORKFLOW_SERIALIZE](instance) {
            return { x: instance.x };
          }

          static [Symbol.for('workflow-deserialize')](data) {
            return new Point(data.x);
          }
        }
    "#;
    assert!(matches_workflow_serde_import(source));
    assert!(matches_workflow_serde_symbol(source));
}

#[test]
fn computed_property_matches_workflow_serialize() {
    assert!(matches_workflow_serde_computed_property(
        "static [WORKFLOW_SERIALIZE](instance) {}"
    ));
}

#[test]
fn computed_property_matches_workflow_deserialize() {
    assert!(matches_workflow_serde_computed_property(
        "static [WORKFLOW_DESERIALIZE](data) {}"
    ));
}

#[test]
fn computed_property_matches_whitespace_inside_brackets() {
    assert!(matches_workflow_serde_computed_property(
        "[ WORKFLOW_SERIALIZE ]"
    ));
    assert!(matches_workflow_serde_computed_property(
        "[  WORKFLOW_DESERIALIZE  ]"
    ));
}

#[test]
fn computed_property_matches_bundled_chunk_imports() {
    let source = r#"
        import {
          WORKFLOW_DESERIALIZE,
          WORKFLOW_SERIALIZE
        } from "./chunks/chunk-453323QY.js";

        var Bash = class _Bash {
          static [WORKFLOW_SERIALIZE](instance) {
            return { fs: instance.fs };
          }
          static [WORKFLOW_DESERIALIZE](serialized) {
            return Object.create(_Bash.prototype, {
              fs: { value: serialized.fs }
            });
          }
        };
    "#;
    assert!(matches_workflow_serde_computed_property(source));
    assert!(!matches_workflow_serde_import(source));
}

#[test]
fn computed_property_does_not_match_partial_names() {
    assert!(!matches_workflow_serde_computed_property(
        "[WORKFLOW_SERIALIZE_EXTRA]"
    ));
    assert!(!matches_workflow_serde_computed_property(
        "[MY_WORKFLOW_SERIALIZE]"
    ));
}

#[test]
fn computed_property_does_not_match_string_literals() {
    assert!(!matches_workflow_serde_computed_property(
        "['WORKFLOW_SERIALIZE']"
    ));
    assert!(!matches_workflow_serde_computed_property(
        "[\"WORKFLOW_DESERIALIZE\"]"
    ));
}

#[test]
fn detect_patterns_marks_serde_imports() {
    let result = detect_workflow_patterns(
        "import { WORKFLOW_SERIALIZE } from '@workflow/serde';",
    );
    assert!(result.has_serde);
    assert!(result.has_serde_import);
}

#[test]
fn detect_patterns_marks_direct_serde_symbols() {
    let result = detect_workflow_patterns(
        "static [Symbol.for('workflow-serialize')](instance) {}",
    );
    assert!(result.has_serde);
    assert!(result.has_serde_symbol);
}

#[test]
fn detect_patterns_marks_serde_computed_properties() {
    let result = detect_workflow_patterns("static [WORKFLOW_SERIALIZE](instance) {}");
    assert!(result.has_serde);
}

#[test]
fn detect_patterns_marks_bundled_third_party_serde_classes() {
    let source = r#"
        import {
          WORKFLOW_DESERIALIZE,
          WORKFLOW_SERIALIZE
        } from "./chunks/chunk-ABC123.js";

        var MyClass = class {
          static [WORKFLOW_SERIALIZE](instance) {
            return { data: instance.data };
          }
          static [WORKFLOW_DESERIALIZE](serialized) {
            return new MyClass(serialized.data);
          }
        };
    "#;
    assert!(detect_workflow_patterns(source).has_serde);
}

#[test]
fn detect_patterns_does_not_mark_unrelated_code_as_serde() {
    let source = r#"
        export class RegularClass {
          constructor(value) {
            this.value = value;
          }
        }
    "#;
    assert!(!detect_workflow_patterns(source).has_serde);
}

#[test]
fn detect_patterns_marks_both_directive_and_serde_patterns() {
    let source = r#"
        'use step';
        import { WORKFLOW_SERIALIZE } from '@workflow/serde';

        export class Point {
          static [WORKFLOW_SERIALIZE](instance) {
            return { x: instance.x };
          }
        }
    "#;
    let result = detect_workflow_patterns(source);
    assert!(result.has_directive);
    assert!(result.has_use_step);
    assert!(result.has_serde);
}

#[test]
fn detect_patterns_ignores_workflow_directives_inside_template_literals() {
    let source = r#"'use client';
const CODE_SNIPPET = `import { sleep } from "workflow";

export async function handleUserSignup(email: string) {
  "use workflow";
  const user = await createUser(email);
}
`;
export default function Page() { return null; }
"#;
    let result = detect_workflow_patterns(source);
    assert!(!result.has_use_workflow);
    assert!(!result.has_directive);
}

#[test]
fn detect_patterns_ignores_step_directives_inside_template_literals() {
    let source = r#"const CODE_SNIPPET = `
export async function doThing() {
  'use step';
}
`;
"#;
    let result = detect_workflow_patterns(source);
    assert!(!result.has_use_step);
    assert!(!result.has_directive);
}

#[test]
fn detect_patterns_ignores_directives_inside_comments() {
    let source = r#"/*
export async function run() {
  "use workflow";
}
*/

// 'use step';
export const value = 1;
"#;
    let result = detect_workflow_patterns(source);
    assert!(!result.has_use_workflow);
    assert!(!result.has_use_step);
    assert!(!result.has_directive);
}

#[test]
fn detect_patterns_ignores_directive_strings_inside_multiline_calls() {
    let source = r#"console.log(
  "use step"
);

console.log(
  'use workflow'
);
"#;
    let result = detect_workflow_patterns(source);
    assert!(!result.has_use_workflow);
    assert!(!result.has_use_step);
    assert!(!result.has_directive);
}

#[test]
fn detect_patterns_ignores_quoted_directive_strings_inside_multiline_calls() {
    let source = r#"console.log(
  '"use step"'
);

console.log(
  "'use step'"
);
"#;
    let result = detect_workflow_patterns(source);
    assert!(!result.has_use_step);
    assert!(!result.has_directive);
}

#[test]
fn detect_patterns_finds_real_directives_after_template_literals() {
    let source = r#"const CODE_SNIPPET = `
  "use workflow";
`;

export async function run() {
  "use workflow";
}
"#;
    let result = detect_workflow_patterns(source);
    assert!(result.has_use_workflow);
    assert!(result.has_directive);
}

#[test]
fn detect_patterns_finds_directives_after_other_prologue_entries() {
    let source = r#"export async function run() {
  "use strict";
  "use workflow";
}
"#;
    let result = detect_workflow_patterns(source);
    assert!(result.has_use_workflow);
    assert!(result.has_directive);
}

#[test]
fn should_transform_excludes_generated_workflow_routes() {
    assert!(!should_transform_file(
        "/app/.well-known/workflow/v1/route.ts",
        WorkflowPatternDetection {
            has_use_workflow: true,
            has_directive: true,
            ..no_patterns()
        },
    ));
}

#[test]
fn should_transform_accepts_directive_files() {
    assert!(should_transform_file(
        "/app/workflows/my-workflow.ts",
        WorkflowPatternDetection {
            has_use_workflow: true,
            has_directive: true,
            ..no_patterns()
        },
    ));
}

#[test]
fn should_transform_accepts_serde_files() {
    assert!(should_transform_file(
        "/app/lib/my-class.ts",
        WorkflowPatternDetection {
            has_serde_import: true,
            has_serde: true,
            ..no_patterns()
        },
    ));
}

#[test]
fn should_transform_accepts_sdk_files_with_serde_patterns() {
    assert!(should_transform_file(
        "/app/node_modules/@workflow/core/dist/serialization.js",
        WorkflowPatternDetection {
            has_serde_symbol: true,
            has_serde: true,
            ..no_patterns()
        },
    ));
}

#[test]
fn should_transform_rejects_files_without_relevant_patterns() {
    assert!(!should_transform_file(
        "/app/lib/utils.ts",
        no_patterns()
    ));
}
