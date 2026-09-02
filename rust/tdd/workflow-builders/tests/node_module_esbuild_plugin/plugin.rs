use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::node_module_plugin::{
    ModuleFormat, NodeModuleBoundaryObservation, NodeModuleBoundaryOptions,
    inspect_node_module_boundary,
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
                "node-module-{label}-{}-{nonce}",
                std::process::id()
            ));
        fs::create_dir_all(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }

    fn entry(&self, source: &str) -> PathBuf {
        let path = self.0.join("workflow.ts");
        write_file(&path, source);
        path
    }

    fn relative(&self, path: &Path) -> String {
        path.strip_prefix(std::env::current_dir().unwrap())
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/")
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

fn inspect_source(
    root: &TestRoot,
    source: &str,
    format: ModuleFormat,
) -> NodeModuleBoundaryObservation {
    let entry_file = root.entry(source);
    inspect_node_module_boundary(&NodeModuleBoundaryOptions {
        entry_file,
        format,
        main_fields: Vec::new(),
    })
    .unwrap()
}

#[test]
fn errors_on_fs_imports() {
    let root = TestRoot::new("fs");
    let entry = root.entry(
        r#"
      import { readFile } from "fs";
      export function workflow() {
        return readFile("test.txt");
      }
    "#,
    );
    let observation =
        inspect_node_module_boundary(&NodeModuleBoundaryOptions::new(entry.clone())).unwrap();

    assert_eq!(observation.errors.len(), 1);
    let violation = &observation.errors[0];
    assert!(
        violation
            .text
            .contains("You are attempting to use \"fs\" which is a Node.js module")
    );
    let location = violation.location.as_ref().unwrap();
    assert_eq!(location.file, root.relative(&entry));
    assert_eq!(
        location.suggestion.as_deref(),
        Some("Move this function into a step function.")
    );
    assert!(location.line > 0);
    assert!(location.line_text.contains("readFile"));
}

#[test]
fn errors_on_node_prefixed_imports() {
    let root = TestRoot::new("node-prefix");
    let entry = root.entry(
        r#"
      import { readFile } from "node:fs";
      export function workflow() {
        return readFile;
      }
    "#,
    );
    let observation = inspect_node_module_boundary(&NodeModuleBoundaryOptions {
        entry_file: entry.clone(),
        format: ModuleFormat::Cjs,
        main_fields: Vec::new(),
    })
    .unwrap();

    assert_eq!(observation.errors.len(), 1);
    let violation = &observation.errors[0];
    assert!(
        violation
            .text
            .contains("You are attempting to use \"node:fs\" which is a Node.js module")
    );
    let location = violation.location.as_ref().unwrap();
    assert_eq!(location.file, root.relative(&entry));
    assert_eq!(
        location.suggestion.as_deref(),
        Some("Move this function into a step function.")
    );
    assert!(location.line_text.contains("readFile"));
}

#[test]
fn reports_each_used_node_import() {
    let root = TestRoot::new("multiple");
    let entry = root.entry(
        r#"
      import { readFile } from "fs";
      import { join } from "path";
      export function workflow() {
        return readFile(join("a", "b"));
      }
    "#,
    );
    let observation = inspect_node_module_boundary(&NodeModuleBoundaryOptions {
        entry_file: entry.clone(),
        format: ModuleFormat::Cjs,
        main_fields: Vec::new(),
    })
    .unwrap();

    assert_eq!(observation.errors.len(), 2);
    let fs_violation = observation
        .errors
        .iter()
        .find(|error| error.text.contains("\"fs\""))
        .unwrap();
    let path_violation = observation
        .errors
        .iter()
        .find(|error| error.text.contains("\"path\""))
        .unwrap();
    assert!(fs_violation.text.contains("which is a Node.js module"));
    assert!(path_violation.text.contains("which is a Node.js module"));
    let relative_entry = root.relative(&entry);
    assert_eq!(fs_violation.location.as_ref().unwrap().file, relative_entry);
    assert!(
        fs_violation
            .location
            .as_ref()
            .unwrap()
            .line_text
            .contains("readFile")
    );
    assert_eq!(
        path_violation.location.as_ref().unwrap().file,
        relative_entry
    );
    assert!(
        path_violation
            .location
            .as_ref()
            .unwrap()
            .line_text
            .contains("join")
    );
}

#[test]
fn attributes_nested_builtin_usage_to_the_top_level_package() {
    let root = TestRoot::new("nested-package");
    let package_dir = root.0.join("node_modules/fake-package");
    write_file(
        &package_dir.join("index.js"),
        r#"
      import { Stream } from "stream";
      export function fakePackage() {
        return new Stream();
      }
    "#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"fake-package","main":"index.js"}"#,
    );
    let entry = root.entry(
        r#"
      import { fakePackage } from "fake-package";
      export function workflow() {
        return fakePackage();
      }
    "#,
    );

    let observation =
        inspect_node_module_boundary(&NodeModuleBoundaryOptions::new(entry.clone())).unwrap();
    assert_eq!(observation.errors.len(), 1);
    let violation = &observation.errors[0];
    assert!(violation.text.contains("\"fake-package\""));
    assert!(violation.text.contains("which depends on Node.js modules"));
    assert!(!violation.text.contains("\"stream\""));
    assert_eq!(
        violation.location.as_ref().unwrap().file,
        root.relative(&entry)
    );
}

#[test]
fn follows_the_same_package_entry_fields_as_the_bundle() {
    let root = TestRoot::new("dual-entry");
    let package_dir = root.0.join("node_modules/dual-entry-package");
    write_file(
        &package_dir.join("esm/index.js"),
        r#"
      import os from "os";
      export function getPlatform() {
        return os.platform();
      }
    "#,
    );
    write_file(
        &package_dir.join("cjs/index.cjs"),
        r#"
      module.exports = {
        getPlatform() {
          return "cjs";
        }
      };
    "#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"dual-entry-package","main":"cjs/index.cjs","module":"esm/index.js"}"#,
    );
    let entry = root.entry(
        r#"
      import { getPlatform } from "dual-entry-package";
      export function workflow() {
        return getPlatform();
      }
    "#,
    );

    let observation = inspect_node_module_boundary(&NodeModuleBoundaryOptions {
        entry_file: entry.clone(),
        format: ModuleFormat::Cjs,
        main_fields: vec!["module".to_owned(), "main".to_owned()],
    })
    .unwrap();
    assert_eq!(observation.errors.len(), 1);
    let violation = &observation.errors[0];
    assert!(violation.text.contains("\"dual-entry-package\""));
    assert!(violation.text.contains("depends on Node.js modules"));
    assert_eq!(
        violation.location.as_ref().unwrap().file,
        root.relative(&entry)
    );
}

#[test]
fn locates_namespace_import_usage() {
    let root = TestRoot::new("namespace");
    let entry = root.entry(
        r#"
      import * as fs from "fs";
      export function workflow() {
        return fs.readFile("test.txt");
      }
    "#,
    );
    let observation =
        inspect_node_module_boundary(&NodeModuleBoundaryOptions::new(entry.clone())).unwrap();

    assert_eq!(observation.errors.len(), 1);
    let violation = &observation.errors[0];
    assert!(violation.text.contains("\"fs\" which is a Node.js module"));
    let location = violation.location.as_ref().unwrap();
    assert_eq!(location.file, root.relative(&entry));
    assert!(location.line_text.contains("fs.readFile"));
}

#[test]
fn locates_default_import_usage() {
    let root = TestRoot::new("default");
    let entry = root.entry(
        r#"
      import fs from "fs";
      export function workflow() {
        return fs.readFile("test.txt");
      }
    "#,
    );
    let observation =
        inspect_node_module_boundary(&NodeModuleBoundaryOptions::new(entry.clone())).unwrap();

    assert_eq!(observation.errors.len(), 1);
    let violation = &observation.errors[0];
    assert!(violation.text.contains("\"fs\" which is a Node.js module"));
    let location = violation.location.as_ref().unwrap();
    assert_eq!(location.file, root.relative(&entry));
    assert!(location.line_text.contains("fs.readFile"));
}

#[test]
fn locates_aliased_import_usage() {
    let root = TestRoot::new("alias");
    let entry = root.entry(
        r#"
      import { readFile as read } from "fs";
      export function workflow() {
        return read("test.txt");
      }
    "#,
    );
    let observation =
        inspect_node_module_boundary(&NodeModuleBoundaryOptions::new(entry.clone())).unwrap();

    assert_eq!(observation.errors.len(), 1);
    let violation = &observation.errors[0];
    assert!(violation.text.contains("\"fs\" which is a Node.js module"));
    let location = violation.location.as_ref().unwrap();
    assert_eq!(location.file, root.relative(&entry));
    assert!(location.line_text.contains("read("));
}

#[test]
fn unused_node_imports_are_tree_shaken_without_an_error() {
    let root = TestRoot::new("unused");
    let observation = inspect_source(
        &root,
        r#"
      import { readFile } from "fs";
      export function workflow() {
        return "no fs usage";
      }
    "#,
        ModuleFormat::Esm,
    );

    assert!(observation.errors.is_empty());
}

#[test]
fn unused_node_exports_in_shared_modules_are_tree_shaken() {
    let root = TestRoot::new("shared-safe");
    write_file(
        &root.0.join("shared.ts"),
        r#"
      import { readFile } from "node:fs/promises";
      import path from "node:path";

      export const isSupportedValue = (value) => value === "ok";

      export async function readFixtureFile() {
        return readFile(path.join(process.cwd(), "fixture.txt"), "utf8");
      }
    "#,
    );
    let entry = root.entry(
        r#"
      import { isSupportedValue } from "./shared.js";
      export function workflow() {
        return isSupportedValue("ok");
      }
    "#,
    );

    let observation = inspect_node_module_boundary(&NodeModuleBoundaryOptions::new(entry)).unwrap();
    assert!(observation.errors.is_empty());
}

#[test]
fn referenced_node_exports_in_shared_modules_still_fail() {
    let root = TestRoot::new("shared-used");
    let shared = root.0.join("shared.ts");
    write_file(
        &shared,
        r#"
      import { readFile } from "node:fs/promises";
      import path from "node:path";

      export const isSupportedValue = (value) => value === "ok";

      export async function readFixtureFile() {
        return readFile(path.join(process.cwd(), "fixture.txt"), "utf8");
      }
    "#,
    );
    let entry = root.entry(
        r#"
      import { isSupportedValue, readFixtureFile } from "./shared.js";
      export async function workflow() {
        if (!isSupportedValue("ok")) throw new Error("nope");
        return readFixtureFile();
      }
    "#,
    );

    let observation = inspect_node_module_boundary(&NodeModuleBoundaryOptions::new(entry)).unwrap();
    let texts = observation
        .errors
        .iter()
        .map(|error| error.text.as_str())
        .collect::<Vec<_>>();
    assert!(
        texts
            .iter()
            .any(|text| text.contains("\"node:fs/promises\""))
    );
    assert!(texts.iter().any(|text| text.contains("\"node:path\"")));
    let relative_shared = root.relative(&shared);
    assert!(observation.errors.iter().all(|error| {
        error
            .location
            .as_ref()
            .map(|location| location.file.as_str())
            == Some(relative_shared.as_str())
    }));
}

#[test]
fn source_locations_skip_jsdoc_mentions() {
    let root = TestRoot::new("jsdoc");
    let observation = inspect_source(
        &root,
        r#"
      import { Writable } from "stream";
      /**
       * Convert a Web WritableStream<string> into a Node.js Writable stream
       */
      export function workflow() {
        return new Writable();
      }
    "#,
        ModuleFormat::Esm,
    );

    assert_eq!(observation.errors.len(), 1);
    let violation = &observation.errors[0];
    assert!(
        violation
            .text
            .contains("\"stream\" which is a Node.js module")
    );
    let line_text = &violation.location.as_ref().unwrap().line_text;
    assert!(line_text.contains("new Writable()"));
    assert!(!line_text.contains('*'));
}

#[test]
fn source_locations_skip_single_line_block_comments() {
    let root = TestRoot::new("block-comment");
    let observation = inspect_source(
        &root,
        r#"
      import { Writable } from "stream";
      /* Writable is used below */
      export function workflow() {
        return new Writable();
      }
    "#,
        ModuleFormat::Esm,
    );

    assert_eq!(observation.errors.len(), 1);
    assert!(
        observation.errors[0]
            .location
            .as_ref()
            .unwrap()
            .line_text
            .contains("new Writable()")
    );
}

#[test]
fn comment_delimiters_inside_strings_do_not_hide_real_usage() {
    let root = TestRoot::new("comment-string");
    let observation = inspect_source(
        &root,
        r#"
      import { Writable } from "stream";
      const pattern = "/* Writable */";
      export function workflow() {
        return new Writable();
      }
    "#,
        ModuleFormat::Esm,
    );

    assert_eq!(observation.errors.len(), 1);
    let violation = &observation.errors[0];
    assert!(
        violation
            .text
            .contains("\"stream\" which is a Node.js module")
    );
    assert!(
        violation
            .location
            .as_ref()
            .unwrap()
            .line_text
            .contains("new Writable()")
    );
}

#[test]
fn errors_on_bun_module_imports() {
    let root = TestRoot::new("bun");
    let entry = root.entry(
        r#"
      import { serve } from "bun";
      export function workflow() {
        return serve({ port: 3000 });
      }
    "#,
    );
    let observation =
        inspect_node_module_boundary(&NodeModuleBoundaryOptions::new(entry.clone())).unwrap();

    assert_eq!(observation.errors.len(), 1);
    let violation = &observation.errors[0];
    assert!(violation.text.contains("\"bun\" which is a Bun module"));
    assert!(violation.text.contains("Bun modules are not available"));
    assert_eq!(
        violation.location.as_ref().unwrap().file,
        root.relative(&entry)
    );
}

#[test]
fn errors_on_bun_subpath_imports() {
    let root = TestRoot::new("bun-subpath");
    let entry = root.entry(
        r#"
      import { Database } from "bun:sqlite";
      export function workflow() {
        return new Database("test.db");
      }
    "#,
    );
    let observation =
        inspect_node_module_boundary(&NodeModuleBoundaryOptions::new(entry.clone())).unwrap();

    assert_eq!(observation.errors.len(), 1);
    let violation = &observation.errors[0];
    assert!(
        violation
            .text
            .contains("\"bun:sqlite\" which is a Bun module")
    );
    assert!(violation.text.contains("Bun modules are not available"));
    assert_eq!(
        violation.location.as_ref().unwrap().file,
        root.relative(&entry)
    );
}
