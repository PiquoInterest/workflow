use std::collections::BTreeMap;

use workflow_utils_tdd::{ModuleScopeFinding, scan_module_scope_sources};

fn one_source(source: &str) -> BTreeMap<String, String> {
    BTreeMap::from([("state.ts".to_owned(), source.to_owned())])
}

fn has_finding(findings: &[ModuleScopeFinding], name: &str) -> bool {
    findings.iter().any(|finding| finding.name == name)
}

#[test]
fn rejects_an_untrusted_method_named_global_singleton() {
    let findings = scan_module_scope_sources(&one_source(
        "const localFactory = { globalSingleton: () => ({ count: 0 }) };\nconst state = localFactory.globalSingleton();\nexport function bump() { state.count++; }\n",
    ));
    assert!(has_finding(&findings, "state"));
}

#[test]
fn rejects_a_local_binary_result_that_only_mentions_global_this() {
    let findings = scan_module_scope_sources(&one_source(
        "const state = globalThis && { count: 0 };\nexport function bump() { state.count++; }\n",
    ));
    assert!(has_finding(&findings, "state"));
}

#[test]
fn resolves_literal_computed_static_fields() {
    let findings = scan_module_scope_sources(&one_source(
        "export class Registry {\n  static transports = new Map<string, number>();\n  static open(id: string) { Registry['transports'].set(id, 1); }\n}\n",
    ));
    assert!(has_finding(&findings, "Registry.transports"));
}
