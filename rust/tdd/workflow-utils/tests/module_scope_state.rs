use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use workflow_utils_tdd::{
    ModuleScopeFinding, discover_bundled_runtime_packages, format_module_scope_findings,
    scan_module_scope_sources, scan_package,
};

fn sources(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
    entries
        .iter()
        .map(|(name, source)| ((*name).to_owned(), (*source).to_owned()))
        .collect()
}

fn one_source(source: &str) -> BTreeMap<String, String> {
    sources(&[("state.ts", source)])
}

fn has_finding(
    findings: &[ModuleScopeFinding],
    name: &str,
    keyword: Option<&str>,
    reason: Option<&str>,
) -> bool {
    findings.iter().any(|finding| {
        finding.name == name
            && keyword.is_none_or(|value| finding.keyword.as_deref() == Some(value))
            && reason.is_none_or(|value| finding.reason.as_deref() == Some(value))
    })
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn discovers_the_bundled_runtime_packages_to_check() {
    let packages = discover_bundled_runtime_packages(&repository_root());
    let names: Vec<_> = packages
        .iter()
        .filter_map(|path| path.file_name()?.to_str())
        .collect();
    for required in ["world-local", "world-vercel", "core", "utils"] {
        assert!(names.contains(&required), "missing bundled package {required}");
    }
}

#[test]
fn reports_nothing_for_the_real_bundled_runtime_packages() {
    let root = repository_root();
    let packages = discover_bundled_runtime_packages(&root);
    assert!(!packages.is_empty());
    for package in packages {
        let findings = scan_package(&package, &root);
        assert!(
            findings.is_empty(),
            "{}",
            format_module_scope_findings(&findings)
        );
    }
}

#[test]
fn flags_a_module_scope_map_that_is_written_to() {
    let findings = scan_module_scope_sources(&one_source(
        "const transports = new Map<string, number>();\nexport function open(id: string) {\n  transports.set(id, 1);\n}\n",
    ));
    assert!(has_finding(
        &findings,
        "transports",
        Some("const"),
        Some("`.set()`")
    ));
}

#[test]
fn flags_a_module_scope_let_that_is_reassigned() {
    let findings = scan_module_scope_sources(&one_source(
        "let started = false;\nexport function start() {\n  started = true;\n}\n",
    ));
    assert!(has_finding(
        &findings,
        "started",
        Some("let"),
        Some("reassigned")
    ));
}

#[test]
fn flags_a_field_written_through_a_member_chain() {
    let findings = scan_module_scope_sources(&one_source(
        "const state = { count: 0 };\nexport function bump() {\n  state.count += 1;\n}\n",
    ));
    assert!(has_finding(
        &findings,
        "state",
        None,
        Some("field written")
    ));
}

#[test]
fn ignores_module_scope_state_that_never_changes() {
    let findings = scan_module_scope_sources(&one_source(
        "const LIMIT = 10;\nconst NAMES = new Set([\"a\"]);\nexport const total = () => LIMIT + NAMES.size;\n",
    ));
    assert!(findings.is_empty());
}

#[test]
fn accepts_state_parked_on_global_singleton() {
    let findings = scan_module_scope_sources(&one_source(
        "import { globalSingleton } from '@workflow/utils';\nconst state = globalSingleton('pkg//transports', 1, () => ({\n  transports: new Map<string, number>(),\n}));\nexport function open(id: string) {\n  state.transports.set(id, 1);\n}\n",
    ));
    assert!(findings.is_empty());
}

#[test]
fn accepts_a_per_copy_ok_declaration_with_a_reason() {
    let findings = scan_module_scope_sources(&one_source(
        "// per-copy-ok: reports what THIS copy sees, so once-per-copy is the point.\nlet logged = false;\nexport function warnOnce() {\n  logged = true;\n}\n",
    ));
    assert!(findings.is_empty());
}

#[test]
fn ignores_a_table_filled_once_at_module_evaluation() {
    let findings = scan_module_scope_sources(&one_source(
        "const BASE64_LOOKUP = new Uint8Array(256);\nfor (let i = 0; i < 64; i++) BASE64_LOOKUP[i] = i;\nexport function decode(i: number) {\n  return BASE64_LOOKUP[i];\n}\n",
    ));
    assert!(findings.is_empty());
}

#[test]
fn accepts_state_hand_rolled_onto_global_this_through_an_alias() {
    let findings = scan_module_scope_sources(&one_source(
        "type WorldState = { locks: Map<string, Promise<void>> };\nconst StateKey = Symbol.for('@your-org/world-foo//locks/v1');\nconst store = globalThis as typeof globalThis &\n  Record<symbol, WorldState | undefined>;\nconst state: WorldState = (store[StateKey] ??= { locks: new Map() });\nexport function open(id: string) {\n  state.locks.set(id, Promise.resolve());\n}\n",
    ));
    assert!(findings.is_empty());
}

#[test]
fn flags_a_static_class_field() {
    let findings = scan_module_scope_sources(&one_source(
        "export class Registry {\n  static transports = new Map<string, number>();\n  static open(id: string) {\n    Registry.transports.set(id, 1);\n  }\n}\n",
    ));
    assert!(has_finding(
        &findings,
        "Registry.transports",
        Some("static"),
        None
    ));
}

#[test]
fn reports_each_static_field_on_a_class_separately() {
    let findings = scan_module_scope_sources(&one_source(
        "export class Registry {\n  static transports = new Map<string, number>();\n  static latch = false;\n  static open(id: string) {\n    Registry.transports.set(id, 1);\n  }\n  static mark() {\n    Registry.latch = true;\n  }\n}\n",
    ));
    assert!(has_finding(
        &findings,
        "Registry.transports",
        Some("static"),
        Some("`.set()`")
    ));
    assert!(has_finding(
        &findings,
        "Registry.latch",
        Some("static"),
        Some("field written")
    ));
}

#[test]
fn resolves_this_to_the_class_inside_a_static_member() {
    let findings = scan_module_scope_sources(&one_source(
        "export class Counters {\n  static hits = new Map<string, number>();\n  static bump(id: string) {\n    this.hits.set(id, 1);\n  }\n}\n",
    ));
    assert!(has_finding(
        &findings,
        "Counters.hits",
        Some("static"),
        None
    ));
}

#[test]
fn ignores_an_instance_field() {
    let findings = scan_module_scope_sources(&one_source(
        "export class Session {\n  seen = new Map<string, number>();\n  mark(id: string) {\n    this.seen.set(id, 1);\n  }\n}\n",
    ));
    assert!(findings.is_empty());
}

#[test]
fn flags_a_field_incremented_with_postfix_increment() {
    let findings = scan_module_scope_sources(&one_source(
        "const state = { count: 0 };\nexport function bump() {\n  state.count++;\n}\n",
    ));
    assert!(has_finding(
        &findings,
        "state",
        None,
        Some("field written")
    ));
}

#[test]
fn flags_an_exported_empty_collection_filled_from_another_file() {
    let findings = scan_module_scope_sources(&sources(&[
        (
            "registry.ts",
            "export const transports = new Map<string, number>();\n",
        ),
        (
            "consumer.ts",
            "import { transports } from './registry.js';\nexport function open(id: string) {\n  transports.set(id, 1);\n}\n",
        ),
    ]));
    assert!(has_finding(
        &findings,
        "transports",
        None,
        Some("exported empty collection")
    ));
}

#[test]
fn leaves_a_non_empty_exported_lookup_table_alone() {
    let findings = scan_module_scope_sources(&one_source(
        "export const LIMITS = new Map([['a', 1]]);\n",
    ));
    assert!(findings.is_empty());
}

#[test]
fn scans_mts_sources() {
    let findings = scan_module_scope_sources(&sources(&[(
        "state.mts",
        "const counts = new Map<string, number>();\nexport function bump(id: string) {\n  counts.set(id, 1);\n}\n",
    )]));
    assert!(has_finding(&findings, "counts", None, None));
}

#[test]
fn does_not_accept_a_bare_per_copy_ok_annotation_without_a_reason() {
    let findings = scan_module_scope_sources(&one_source(
        "// per-copy-ok:\nlet logged = false;\nexport function warnOnce() {\n  logged = true;\n}\n",
    ));
    assert!(has_finding(&findings, "logged", None, None));
}
