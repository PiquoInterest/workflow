use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

pub const RESULT_MARKER: &str = "__ROUTE_BUNDLE_ISOLATION_RESULT__";
pub const ROUTE_BUNDLE_PATH: &str = ".next/server/app/api/resume-plain-hook/route.js";
pub const NONEXISTENT_TOKEN: &str = "route-bundle-isolation-nonexistent-token";
pub const BUILD_TIMEOUT_MS: u64 = 300_000;
pub const HARNESS_TIMEOUT_MS: u64 = 60_000;
pub const OUTPUT_LIMIT_BYTES: usize = 4 * 1024 * 1024;
pub const TDD_RED_MARKER: &str =
    "TDD RED: packages/core/e2e/route-bundle-isolation.test.ts implementation pending";

pub const HARNESS_SCRIPT: &str = r#"
const m = require(process.argv[1]);
const report = (result) =>
  console.log('__ROUTE_BUNDLE_ISOLATION_RESULT__' + JSON.stringify(result));
Promise.resolve(m)
  .then(async (mod) => {
    const POST = mod.routeModule?.userland?.POST;
    if (typeof POST !== 'function') {
      report({
        harnessError: 'route bundle did not expose routeModule.userland.POST',
        exportKeys: Object.keys(mod),
      });
      return;
    }
    const res = await POST(
      new Request('http://localhost/api/resume-plain-hook', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
          token: 'route-bundle-isolation-nonexistent-token',
          ok: true,
        }),
      })
    );
    report({ status: res.status, body: await res.text() });
  })
  .catch((err) => {
    report({
      harnessError: err instanceof Error ? err.message : String(err),
    });
  });
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsolatedCommandSpec {
    pub program: &'static str,
    pub args: Vec<String>,
    pub timeout_ms: u64,
    pub output_limit_bytes: usize,
    pub kill_process_group_on_timeout: bool,
    pub use_shell: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteBundlePlan {
    pub project: &'static str,
    pub build: IsolatedCommandSpec,
    pub harness: IsolatedCommandSpec,
    pub route_bundle_path: &'static str,
    pub result_marker: &'static str,
    pub nonexistent_token: &'static str,
    pub expected_status: u16,
    pub expected_body_fragment: &'static str,
    pub forbidden_body_fragments: Vec<&'static str>,
    pub reject_duplicate_result_records: bool,
    pub require_exact_marker_prefix: bool,
}

pub fn route_bundle_plan() -> RouteBundlePlan {
    RouteBundlePlan {
        project: "nextjs-turbopack",
        build: IsolatedCommandSpec {
            program: "pnpm",
            args: vec!["build".to_owned()],
            timeout_ms: BUILD_TIMEOUT_MS,
            output_limit_bytes: OUTPUT_LIMIT_BYTES,
            kill_process_group_on_timeout: true,
            use_shell: false,
        },
        harness: IsolatedCommandSpec {
            program: "current-node",
            args: vec![
                "-e".to_owned(),
                HARNESS_SCRIPT.to_owned(),
                ROUTE_BUNDLE_PATH.to_owned(),
            ],
            timeout_ms: HARNESS_TIMEOUT_MS,
            output_limit_bytes: OUTPUT_LIMIT_BYTES,
            kill_process_group_on_timeout: true,
            use_shell: false,
        },
        route_bundle_path: ROUTE_BUNDLE_PATH,
        result_marker: RESULT_MARKER,
        nonexistent_token: NONEXISTENT_TOKEN,
        expected_status: 500,
        expected_body_fragment: "Hook not found",
        forbidden_body_fragments: vec![
            "Cannot find module as expression is too dynamic",
            "world runtime was not initialized",
        ],
        reject_duplicate_result_records: true,
        require_exact_marker_prefix: true,
    }
}

pub fn sanitized_isolation_environment(
    environment: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    const REMOVED_KEYS: [&str; 6] = [
        "VERCEL",
        "VERCEL_ENV",
        "VERCEL_DEPLOYMENT_ID",
        "VERCEL_PROJECT_ID",
        "NODE_OPTIONS",
        "NODE_PATH",
    ];

    let mut sanitized = environment.clone();
    for key in REMOVED_KEYS {
        sanitized.remove(key);
    }
    sanitized.insert("FORCE_COLOR".to_owned(), "0".to_owned());
    sanitized
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessParseError {
    MissingResult { stdout_preview: String },
    DuplicateResults,
    EmptyPayload,
}

impl Display for HarnessParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingResult { stdout_preview } => write!(
                formatter,
                "route bundle harness produced no {RESULT_MARKER} line; bounded stdout preview:\n{stdout_preview}"
            ),
            Self::DuplicateResults => {
                formatter.write_str("route bundle harness produced duplicate result records")
            }
            Self::EmptyPayload => {
                formatter.write_str("route bundle harness produced an empty result payload")
            }
        }
    }
}

fn bounded_preview(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

pub fn extract_harness_payload(stdout: &str) -> Result<String, HarnessParseError> {
    let mut payloads = stdout
        .lines()
        .filter_map(|line| line.strip_prefix(RESULT_MARKER));
    let Some(first) = payloads.next() else {
        return Err(HarnessParseError::MissingResult {
            stdout_preview: bounded_preview(stdout, 512),
        });
    };
    if first.is_empty() {
        return Err(HarnessParseError::EmptyPayload);
    }
    if payloads.next().is_some() {
        return Err(HarnessParseError::DuplicateResults);
    }
    Ok(first.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HarnessResult {
    pub status: Option<u16>,
    pub body: Option<String>,
    pub harness_error: Option<String>,
    pub export_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessValidationError {
    HarnessError(String),
    MissingBody,
    DynamicRequireFallback,
    WorldRuntimeNotInitialized,
    UnexpectedStatus(Option<u16>),
    MissingHookNotFound,
}

impl Display for HarnessValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::HarnessError(message) => write!(formatter, "isolated harness failed: {message}"),
            Self::MissingBody => formatter.write_str("isolated route returned no response body"),
            Self::DynamicRequireFallback => formatter.write_str(
                "isolated route used the Turbopack dynamic-require fallback",
            ),
            Self::WorldRuntimeNotInitialized => formatter.write_str(
                "isolated route bundle did not initialize the workflow world runtime",
            ),
            Self::UnexpectedStatus(status) => {
                write!(formatter, "isolated route returned unexpected status {status:?}")
            }
            Self::MissingHookNotFound => {
                formatter.write_str("isolated route did not return the expected Hook not found body")
            }
        }
    }
}

pub fn validate_harness_result(result: &HarnessResult) -> Result<(), HarnessValidationError> {
    if let Some(error) = &result.harness_error {
        return Err(HarnessValidationError::HarnessError(error.clone()));
    }
    let body = result
        .body
        .as_deref()
        .ok_or(HarnessValidationError::MissingBody)?;
    if body.contains("Cannot find module as expression is too dynamic") {
        return Err(HarnessValidationError::DynamicRequireFallback);
    }
    if body.contains("world runtime was not initialized") {
        return Err(HarnessValidationError::WorldRuntimeNotInitialized);
    }
    if result.status != Some(500) {
        return Err(HarnessValidationError::UnexpectedStatus(result.status));
    }
    if !body.contains("Hook not found") {
        return Err(HarnessValidationError::MissingHookNotFound);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteBundleObservation {
    pub plan: RouteBundlePlan,
    pub build_environment: BTreeMap<String, String>,
    pub harness_environment: BTreeMap<String, String>,
    pub stdout: String,
    pub result: HarnessResult,
}

/// Builds and invokes the route bundle through the future Rust E2E harness.
///
/// Final GREEN must create a fresh process with sanitized preload state, parse
/// exactly one marker-prefixed record, and validate the real isolated handler.
pub fn run_route_bundle_isolation(
    environment: &BTreeMap<String, String>,
) -> RouteBundleObservation {
    let _ = environment;
    panic!("{TDD_RED_MARKER}")
}
