use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// Build mode passed to the SWC transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SwcMode {
    #[default]
    Step,
    Workflow,
}

/// A manifest identifier emitted by one transformed source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestEntry {
    Step {
        source_file: String,
        export_name: String,
        step_id: String,
    },
    Workflow {
        source_file: String,
        export_name: String,
        workflow_id: String,
    },
}

impl ManifestEntry {
    pub fn step(source_file: &str, export_name: &str, step_id: &str) -> Self {
        Self::Step {
            source_file: source_file.to_owned(),
            export_name: export_name.to_owned(),
            step_id: step_id.to_owned(),
        }
    }

    pub fn workflow(source_file: &str, export_name: &str, workflow_id: &str) -> Self {
        Self::Workflow {
            source_file: source_file.to_owned(),
            export_name: export_name.to_owned(),
            workflow_id: workflow_id.to_owned(),
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Step { step_id, .. } => step_id,
            Self::Workflow { workflow_id, .. } => workflow_id,
        }
    }
}

/// Result returned by the future Rust SWC transform implementation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TransformOutput {
    pub code: String,
    pub manifest_entries: Vec<ManifestEntry>,
}

impl TransformOutput {
    pub fn passthrough(source: &str) -> Self {
        Self {
            code: source.to_owned(),
            manifest_entries: Vec::new(),
        }
    }
}

/// Synthetic stdin entry used by the side-effect-entry fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualEntry {
    pub source_file: String,
    pub resolve_dir: PathBuf,
    pub source: String,
}

/// How the translated fixture asks the transform observer to behave.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ObserverPolicy {
    #[default]
    None,
    Record,
    Deferred,
    Fail(String),
}

/// Inputs required to exercise the future Rust SWC/esbuild integration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwcPluginBuildOptions {
    pub mode: SwcMode,
    pub abs_working_dir: PathBuf,
    pub entry_points: Vec<PathBuf>,
    pub virtual_entry: Option<VirtualEntry>,
    pub outdir: Option<PathBuf>,
    pub entries_to_bundle: BTreeSet<PathBuf>,
    pub side_effect_entries: BTreeSet<PathBuf>,
    pub aliases: BTreeMap<String, PathBuf>,
    pub rewrite_ts_extensions: bool,
    pub bundle_transitive_local_step_dependencies: bool,
    pub project_root: Option<PathBuf>,
    pub module_specifier_root: Option<PathBuf>,
    pub transform_outputs: BTreeMap<PathBuf, TransformOutput>,
    pub observer: ObserverPolicy,
}

impl SwcPluginBuildOptions {
    pub fn new(mode: SwcMode, abs_working_dir: PathBuf, entry_points: Vec<PathBuf>) -> Self {
        Self {
            mode,
            abs_working_dir,
            entry_points,
            virtual_entry: None,
            outdir: None,
            entries_to_bundle: BTreeSet::new(),
            side_effect_entries: BTreeSet::new(),
            aliases: BTreeMap::new(),
            rewrite_ts_extensions: false,
            bundle_transitive_local_step_dependencies: false,
            project_root: None,
            module_specifier_root: None,
            transform_outputs: BTreeMap::new(),
            observer: ObserverPolicy::None,
        }
    }
}

/// One invocation of the transformed-source boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformCall {
    pub filename: String,
    pub source: String,
    pub mode: SwcMode,
    pub absolute_path: PathBuf,
    pub project_root: PathBuf,
    pub module_specifier_root: PathBuf,
}

/// Authoritative transform data delivered to the optional observer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformObserverCall {
    pub mode: SwcMode,
    pub filename: String,
    pub absolute_path: PathBuf,
    pub source: String,
    pub code: String,
    pub manifest_entries: Vec<ManifestEntry>,
}

/// A warning emitted by the future bundle implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildWarning {
    pub id: String,
    pub text: String,
}

/// Observable result of one translated bundle fixture.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SwcPluginBuildObservation {
    pub errors: Vec<String>,
    pub warnings: Vec<BuildWarning>,
    pub output: String,
    pub transform_calls: Vec<TransformCall>,
    pub observer_calls: Vec<TransformObserverCall>,
    pub observer_awaited_before_completion: bool,
    pub manifest_entries: Vec<ManifestEntry>,
}

/// A failed build together with the observations made before it stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwcPluginBuildFailure {
    pub message: String,
    pub observation: SwcPluginBuildObservation,
}

impl SwcPluginBuildFailure {
    pub fn contains(&self, needle: &str) -> bool {
        self.message.contains(needle)
    }
}

/// Executes the future Rust equivalent of `createSwcPlugin`.
pub fn build_with_swc_plugin(
    options: &SwcPluginBuildOptions,
) -> Result<SwcPluginBuildObservation, Box<SwcPluginBuildFailure>> {
    let _ = options;
    panic!("TDD RED: packages/builders/src/swc-esbuild-plugin.test.ts implementation pending")
}
