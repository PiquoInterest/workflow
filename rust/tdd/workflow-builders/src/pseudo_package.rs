use std::collections::BTreeSet;
use std::path::PathBuf;

pub const PSEUDO_PACKAGES: [&str; 4] = [
    "server-only",
    "client-only",
    "next/dist/compiled/server-only",
    "next/dist/compiled/client-only",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleOptions {
    pub entry_file: PathBuf,
    pub use_pseudo_package_plugin: bool,
    pub external_packages: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BundleObservation {
    pub errors: Vec<String>,
    pub output: String,
}

pub fn bundle_with_pseudo_package_support(
    options: &BundleOptions,
) -> Result<BundleObservation, String> {
    let _ = options;
    panic!(
        "TDD RED: packages/builders/src/pseudo-package-esbuild-plugin.test.ts implementation pending"
    )
}
