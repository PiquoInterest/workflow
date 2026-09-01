use std::fmt::{self, Display, Formatter};

pub const LOCAL_BUILD_TIMEOUT_MS: u64 = 180_000;
pub const COMMAND_OUTPUT_LIMIT_BYTES: usize = 8 * 1024 * 1024;
pub const SOURCE_MAP_WARNING: &str = "failed to read input source map";
pub const SOURCE_MAP_FIXTURE_PACKAGE: &str = "workflow-sourcemap-warning-fixture";
pub const TDD_RED_MARKER: &str =
    "TDD RED: packages/core/e2e/local-build.test.ts implementation pending";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalBuildProject {
    Example,
    NextWebpack,
    NextTurbopack,
    Nitro,
    Vite,
    SvelteKit,
    Nuxt,
    Hono,
    Express,
    Fastify,
    Nest,
    Astro,
    TanstackStart,
}

pub const ALL_LOCAL_BUILD_PROJECTS: [LocalBuildProject; 13] = [
    LocalBuildProject::Example,
    LocalBuildProject::NextWebpack,
    LocalBuildProject::NextTurbopack,
    LocalBuildProject::Nitro,
    LocalBuildProject::Vite,
    LocalBuildProject::SvelteKit,
    LocalBuildProject::Nuxt,
    LocalBuildProject::Hono,
    LocalBuildProject::Express,
    LocalBuildProject::Fastify,
    LocalBuildProject::Nest,
    LocalBuildProject::Astro,
    LocalBuildProject::TanstackStart,
];

impl LocalBuildProject {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Example => "example",
            Self::NextWebpack => "nextjs-webpack",
            Self::NextTurbopack => "nextjs-turbopack",
            Self::Nitro => "nitro",
            Self::Vite => "vite",
            Self::SvelteKit => "sveltekit",
            Self::Nuxt => "nuxt",
            Self::Hono => "hono",
            Self::Express => "express",
            Self::Fastify => "fastify",
            Self::Nest => "nest",
            Self::Astro => "astro",
            Self::TanstackStart => "tanstack-start",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldTarget {
    Local,
    Vercel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalBuildOptions {
    pub world_target: WorldTarget,
    pub ci: bool,
}

impl Default for LocalBuildOptions {
    fn default() -> Self {
        Self {
            world_target: WorldTarget::Local,
            ci: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: &'static str,
    pub args: Vec<String>,
    pub timeout_ms: u64,
    pub output_limit_bytes: usize,
    pub kill_process_group_on_timeout: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixturePolicy {
    pub setup_before_build: bool,
    pub preserve_after_build: bool,
    pub cleanup_in_finally: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalBuildPlan {
    pub project: LocalBuildProject,
    pub preflight: Option<CommandSpec>,
    pub build: CommandSpec,
    pub diagnostics_manifest_path: Option<&'static str>,
    pub esm_step_registration_path: Option<&'static str>,
    pub forbidden_legacy_step_route_path: Option<&'static str>,
    pub source_map_fixture: FixturePolicy,
    pub forbidden_output_fragments: Vec<&'static str>,
    pub file_reads_fail_closed: bool,
}

fn command(program: &'static str, args: &[&str]) -> CommandSpec {
    CommandSpec {
        program,
        args: args.iter().map(|value| (*value).to_owned()).collect(),
        timeout_ms: LOCAL_BUILD_TIMEOUT_MS,
        output_limit_bytes: COMMAND_OUTPUT_LIMIT_BYTES,
        kill_process_group_on_timeout: true,
    }
}

pub fn local_build_plan(project: LocalBuildProject, options: LocalBuildOptions) -> LocalBuildPlan {
    let preflight = (project == LocalBuildProject::SvelteKit).then(|| {
        command(
            "current-node",
            &[
                "-e",
                "import('workflow/sveltekit').then(() => console.log('workflow/sveltekit import ok')).catch((error) => { console.error(error); process.exit(1); })",
            ],
        )
    });

    let diagnostics_manifest_path = if options.world_target == WorldTarget::Vercel {
        Some(".vercel/output/diagnostics/workflows-manifest.json")
    } else {
        match project {
            LocalBuildProject::Example => {
                Some(".vercel/output/diagnostics/workflows-manifest.json")
            }
            LocalBuildProject::NextWebpack | LocalBuildProject::NextTurbopack => {
                Some(".next/diagnostics/workflows-manifest.json")
            }
            _ => None,
        }
    };

    let setup_source_map_fixture = project == LocalBuildProject::NextTurbopack;
    let preserve_source_map_fixture = setup_source_map_fixture && options.ci;

    LocalBuildPlan {
        project,
        preflight,
        build: command("pnpm", &["build"]),
        diagnostics_manifest_path,
        esm_step_registration_path: (project == LocalBuildProject::Example).then_some(
            ".vercel/output/functions/.well-known/workflow/v1/flow.func/__step_registrations.mjs",
        ),
        forbidden_legacy_step_route_path: match project {
            LocalBuildProject::Example => {
                Some(".vercel/output/functions/.well-known/workflow/v1/step.func")
            }
            LocalBuildProject::NextWebpack | LocalBuildProject::NextTurbopack => {
                Some("app/.well-known/workflow/v1/step")
            }
            _ => None,
        },
        source_map_fixture: FixturePolicy {
            setup_before_build: setup_source_map_fixture,
            preserve_after_build: preserve_source_map_fixture,
            cleanup_in_finally: setup_source_map_fixture && !preserve_source_map_fixture,
        },
        forbidden_output_fragments: vec!["Error:", SOURCE_MAP_WARNING],
        file_reads_fail_closed: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedCommandOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub combined: Vec<u8>,
    pub seen_bytes: usize,
    pub accepted_bytes: usize,
    pub limit_bytes: usize,
    pub truncated: bool,
}

impl BoundedCommandOutput {
    pub fn new(limit_bytes: usize) -> Self {
        Self {
            stdout: Vec::new(),
            stderr: Vec::new(),
            combined: Vec::new(),
            seen_bytes: 0,
            accepted_bytes: 0,
            limit_bytes,
            truncated: false,
        }
    }

    pub fn append(&mut self, stream: OutputStream, chunk: &[u8]) {
        self.seen_bytes = self.seen_bytes.saturating_add(chunk.len());
        let remaining = self.limit_bytes.saturating_sub(self.accepted_bytes);
        let accepted = chunk.len().min(remaining);
        let accepted_chunk = &chunk[..accepted];

        match stream {
            OutputStream::Stdout => self.stdout.extend_from_slice(accepted_chunk),
            OutputStream::Stderr => self.stderr.extend_from_slice(accepted_chunk),
        }
        self.combined.extend_from_slice(accepted_chunk);
        self.accepted_bytes += accepted;
        self.truncated |= accepted < chunk.len();
    }

    pub fn stdout_text(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    pub fn stderr_text(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }

    pub fn combined_text(&self) -> String {
        String::from_utf8_lossy(&self.combined).into_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandExit {
    Code(i32),
    Signal(String),
}

impl Display for CommandExit {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Code(code) => write!(formatter, "exit code {code}"),
            Self::Signal(signal) => write!(formatter, "signal {signal}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFailure {
    pub program: String,
    pub args: Vec<String>,
    pub exit: CommandExit,
    pub output: BoundedCommandOutput,
}

impl Display for CommandFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let command_line = std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ");
        write!(
            formatter,
            "Command \"{command_line}\" failed with {}\n{}",
            self.exit,
            self.output.combined_text()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileReadError {
    NotFound,
    Io(String),
}

pub fn gate_optional_file<T>(result: Result<T, FileReadError>) -> Result<Option<T>, FileReadError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(FileReadError::NotFound) => Ok(None),
        Err(error @ FileReadError::Io(_)) => Err(error),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalBuildObservation {
    pub plan: LocalBuildPlan,
    pub sveltekit_preflight_succeeded: bool,
    pub build_succeeded: bool,
    pub diagnostics_manifest_found: bool,
    pub esm_bundle_uses_native_import_meta: bool,
    pub legacy_step_route_absent: bool,
    pub source_map_fixture_cleaned_up: bool,
    pub output: BoundedCommandOutput,
}

/// Runs the future Rust local-build E2E boundary for one workbench project.
///
/// Final GREEN must spawn a direct argv command in a killable process group,
/// bound captured output, inspect real generated artifacts, and run cleanup on
/// every success and failure path.
pub fn run_local_build_case(
    project: LocalBuildProject,
    options: LocalBuildOptions,
) -> LocalBuildObservation {
    let _ = (project, options);
    panic!("{TDD_RED_MARKER}")
}
