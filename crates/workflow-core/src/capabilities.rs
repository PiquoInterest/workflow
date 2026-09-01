use std::collections::BTreeSet;
use std::sync::LazyLock;

use semver::Version;

/// npm `semver` rejects raw version strings longer than this before parsing.
pub const NODE_SEMVER_MAX_LENGTH: usize = 256;

/// JavaScript's largest exactly representable integer.
///
/// npm `semver` rejects major, minor, and patch components above this value.
/// Matching that bound prevents Rust from accepting metadata the TypeScript
/// producer would treat as invalid.
pub const JAVASCRIPT_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

const ENCRYPTED_MIN_VERSION_TEXT: &str = "4.2.0-beta.64";
const FRAMED_BYTE_STREAMS_MIN_VERSION_TEXT: &str = "5.0.0-beta.15";
const COMPRESSION_MIN_VERSION_TEXT: &str = "5.0.0-beta.18";
const SEALED_MIN_VERSION_TEXT: &str = "5.0.0-beta.37";

static ENCRYPTED_MIN_VERSION: LazyLock<Version> = LazyLock::new(|| {
    Version::parse(ENCRYPTED_MIN_VERSION_TEXT)
        .expect("the encrypted capability cutoff must be valid semver")
});
static FRAMED_BYTE_STREAMS_MIN_VERSION: LazyLock<Version> = LazyLock::new(|| {
    Version::parse(FRAMED_BYTE_STREAMS_MIN_VERSION_TEXT)
        .expect("the framed-byte-stream capability cutoff must be valid semver")
});
static COMPRESSION_MIN_VERSION: LazyLock<Version> = LazyLock::new(|| {
    Version::parse(COMPRESSION_MIN_VERSION_TEXT)
        .expect("the compression capability cutoff must be valid semver")
});
static SEALED_MIN_VERSION: LazyLock<Version> = LazyLock::new(|| {
    Version::parse(SEALED_MIN_VERSION_TEXT)
        .expect("the sealed capability cutoff must be valid semver")
});

/// Serialization formats a target workflow run may decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SerializationFormat {
    DevalueV1,
    Encrypted,
    Gzip,
    Zstd,
    Sealed,
}

impl SerializationFormat {
    /// Four-byte wire prefix used by the TypeScript protocol.
    pub const fn as_prefix(self) -> &'static str {
        match self {
            Self::DevalueV1 => "devl",
            Self::Encrypted => "encr",
            Self::Gzip => "gzip",
            Self::Zstd => "zstd",
            Self::Sealed => "encp",
        }
    }
}

/// Capabilities advertised by the deployment that created a workflow run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunCapabilities {
    pub supported_formats: BTreeSet<SerializationFormat>,
    pub framed_byte_streams: bool,
}

impl Default for RunCapabilities {
    fn default() -> Self {
        Self {
            supported_formats: BTreeSet::from([SerializationFormat::DevalueV1]),
            framed_byte_streams: false,
        }
    }
}

impl RunCapabilities {
    pub fn supports(&self, format: SerializationFormat) -> bool {
        self.supported_formats.contains(&format)
    }
}

/// Parse the npm-semver-compatible subset used by persisted run metadata.
///
/// Invalid, malformed, overlong, or numerically inexact versions return
/// `None`. Callers must fail closed to baseline capabilities.
fn parse_workflow_core_version(raw: &str) -> Option<Version> {
    if raw.encode_utf16().count() > NODE_SEMVER_MAX_LENGTH {
        return None;
    }

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // npm semver accepts a single lower-case `v` prefix in strict mode.
    let normalized = trimmed.strip_prefix('v').unwrap_or(trimmed);
    let version = Version::parse(normalized).ok()?;

    if [version.major, version.minor, version.patch]
        .into_iter()
        .any(|component| component > JAVASCRIPT_MAX_SAFE_INTEGER)
    {
        return None;
    }

    Some(version)
}

/// Derive target-run capabilities from its persisted `@workflow/core` version.
///
/// Capability negotiation is deliberately conservative. A malformed or absent
/// producer version returns only the baseline `devl` format and raw byte
/// streams, preventing a new writer from sending a representation the target
/// run cannot decode.
pub fn get_run_capabilities(version: Option<&str>) -> RunCapabilities {
    let Some(version) = version.and_then(parse_workflow_core_version) else {
        return RunCapabilities::default();
    };

    let mut capabilities = RunCapabilities::default();

    if &version >= &*ENCRYPTED_MIN_VERSION {
        capabilities
            .supported_formats
            .insert(SerializationFormat::Encrypted);
    }
    if &version >= &*COMPRESSION_MIN_VERSION {
        capabilities
            .supported_formats
            .insert(SerializationFormat::Gzip);
        capabilities
            .supported_formats
            .insert(SerializationFormat::Zstd);
    }
    if &version >= &*SEALED_MIN_VERSION {
        capabilities
            .supported_formats
            .insert(SerializationFormat::Sealed);
    }
    if &version >= &*FRAMED_BYTE_STREAMS_MIN_VERSION {
        capabilities.framed_byte_streams = true;
    }

    capabilities
}
