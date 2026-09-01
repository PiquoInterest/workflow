use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::node_module_plugin::{
    escape_reg_exp, get_imported_identifier, get_package_name, get_violation_location,
};

struct TestRoot(PathBuf);

impl TestRoot {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::current_dir()
            .unwrap()
            .join("target/tdd-fixtures")
            .join(format!(
                "node-module-helper-{label}-{}-{nonce}",
                std::process::id()
            ));
        fs::create_dir_all(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, contents).unwrap();
        path
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn gets_package_name_from_simple_node_modules_paths() {
    assert_eq!(
        get_package_name("/Users/adrianlam/GitHub/workflow/node_modules/node-fetch/src/index.js")
            .as_deref(),
        Some("node-fetch")
    );
}

#[test]
fn gets_package_name_from_pnpm_nested_paths() {
    assert_eq!(
        get_package_name(
            "/Users/adrianlam/GitHub/workflow/node_modules/.pnpm/node-fetch@3.3.2/node_modules/node-fetch/src/index.js"
        )
        .as_deref(),
        Some("node-fetch")
    );
}

#[test]
fn gets_scoped_package_names() {
    assert_eq!(
        get_package_name("/project/node_modules/@supabase/supabase-js/dist/index.js").as_deref(),
        Some("@supabase/supabase-js")
    );
}

#[test]
fn returns_no_package_name_for_paths_without_node_modules() {
    assert_eq!(
        get_package_name("/Users/adrianlam/GitHub/workflow/src/index.js"),
        None
    );
}

#[test]
fn escapes_regular_expression_special_characters() {
    assert_eq!(escape_reg_exp("test.file"), "test\\.file");
    assert_eq!(escape_reg_exp("test*file"), "test\\*file");
    assert_eq!(escape_reg_exp("test+file"), "test\\+file");
    assert_eq!(escape_reg_exp("test?file"), "test\\?file");
    assert_eq!(escape_reg_exp("test^file"), "test\\^file");
    assert_eq!(escape_reg_exp("test$file"), "test\\$file");
}

#[test]
fn escapes_brackets_braces_and_parentheses() {
    assert_eq!(escape_reg_exp("test{file}"), "test\\{file\\}");
    assert_eq!(escape_reg_exp("test[file]"), "test\\[file\\]");
    assert_eq!(escape_reg_exp("test(file)"), "test\\(file\\)");
}

#[test]
fn escapes_pipes_and_backslashes() {
    assert_eq!(escape_reg_exp("test|file"), "test\\|file");
    assert_eq!(escape_reg_exp("test\\file"), "test\\\\file");
}

#[test]
fn leaves_strings_without_special_characters_unchanged() {
    assert_eq!(escape_reg_exp("testfile"), "testfile");
    assert_eq!(escape_reg_exp("test-file"), "test-file");
}

#[test]
fn preserves_package_separators_while_escaping_package_punctuation() {
    assert_eq!(
        escape_reg_exp("@supabase/supabase-js"),
        "@supabase/supabase-js"
    );
    assert_eq!(escape_reg_exp("package.name"), "package\\.name");
}

#[test]
fn extracts_namespace_import_identifiers() {
    assert_eq!(get_imported_identifier("* as fs").as_deref(), Some("fs"));
    assert_eq!(
        get_imported_identifier("*   as   path").as_deref(),
        Some("path")
    );
}

#[test]
fn extracts_the_first_named_import() {
    assert_eq!(
        get_imported_identifier("{ readFile }").as_deref(),
        Some("readFile")
    );
    assert_eq!(
        get_imported_identifier("{ readFile, writeFile }").as_deref(),
        Some("readFile")
    );
}

#[test]
fn extracts_aliased_named_imports() {
    assert_eq!(
        get_imported_identifier("{ readFile as read }").as_deref(),
        Some("read")
    );
    assert_eq!(
        get_imported_identifier("{ readFile as read, writeFile }").as_deref(),
        Some("read")
    );
}

#[test]
fn extracts_default_import_identifiers() {
    assert_eq!(get_imported_identifier("fs").as_deref(), Some("fs"));
    assert_eq!(
        get_imported_identifier("myDefault").as_deref(),
        Some("myDefault")
    );
}

#[test]
fn named_imports_take_precedence_in_mixed_import_clauses() {
    assert_eq!(
        get_imported_identifier("fs, { readFile }").as_deref(),
        Some("readFile")
    );
    assert_eq!(
        get_imported_identifier("defaultExport, { named }").as_deref(),
        Some("named")
    );
}

#[test]
fn handles_import_clause_whitespace_variations() {
    assert_eq!(
        get_imported_identifier("  { readFile }  ").as_deref(),
        Some("readFile")
    );
    assert_eq!(
        get_imported_identifier("{readFile}").as_deref(),
        Some("readFile")
    );
    assert_eq!(
        get_imported_identifier("{ readFile , writeFile }").as_deref(),
        Some("readFile")
    );
}

#[test]
fn handles_type_and_default_as_named_imports() {
    assert_eq!(
        get_imported_identifier("type { ReadStream }").as_deref(),
        Some("ReadStream")
    );
    assert_eq!(
        get_imported_identifier("{ default as fs }").as_deref(),
        Some("fs")
    );
}

#[test]
fn returns_no_imported_identifier_for_empty_or_incomplete_clauses() {
    assert_eq!(get_imported_identifier("*"), None);
    assert_eq!(get_imported_identifier(""), None);
    assert_eq!(get_imported_identifier("{}"), None);
}

#[test]
fn finds_the_first_real_identifier_usage_for_an_imported_package() {
    let root = TestRoot::new("location");
    let source = r#"import { describe, expect, it } from 'vitest';
import http from 'node:http';

describe('fixture', () => {
  it('works', () => expect(true).toBe(true));
});
"#;
    let file = root.write("src/fixture.test.ts", source);
    let relative = file.strip_prefix(&root.0).unwrap();

    let location = get_violation_location(&root.0, relative, "vitest").unwrap();
    assert_eq!(location.file, "src/fixture.test.ts");
    assert_eq!(location.line, 4);
    assert_eq!(location.column, 0);
    assert!(location.line_text.contains("describe("));
    assert_eq!(location.length, 8);
}

#[test]
fn returns_no_location_for_nonexistent_files() {
    let root = TestRoot::new("missing-file");
    assert_eq!(
        get_violation_location(&root.0, Path::new("non-existent-file.ts"), "some-package"),
        None
    );
}

#[test]
fn returns_no_location_when_the_package_is_not_imported() {
    let root = TestRoot::new("missing-package");
    let file = root.write(
        "src/fixture.test.ts",
        "import { describe } from 'vitest';\ndescribe('fixture', () => {});\n",
    );
    let relative = file.strip_prefix(&root.0).unwrap();

    assert_eq!(
        get_violation_location(&root.0, relative, "non-existent-package"),
        None
    );
}

#[test]
fn returns_no_location_for_an_unused_but_parseable_import() {
    let root = TestRoot::new("unused-import");
    let file = root.write(
        "src/fixture.test.ts",
        "import http from 'node:http';\nexport const value = 1;\n",
    );
    let relative = file.strip_prefix(&root.0).unwrap();

    assert_eq!(get_violation_location(&root.0, relative, "node:http"), None);
}
