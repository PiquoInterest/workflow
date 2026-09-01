use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SerializationFormat {
    DevalueV1,
    Encrypted,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunCapabilities {
    pub supported_formats: BTreeSet<SerializationFormat>,
    pub framed_byte_streams: bool,
}

impl RunCapabilities {
    pub fn supports(&self, format: SerializationFormat) -> bool {
        self.supported_formats.contains(&format)
    }
}

/// Derives runtime capabilities from the producer's semantic version.
pub fn get_run_capabilities(version: Option<&str>) -> RunCapabilities {
    let _ = version;
    panic!("TDD RED: packages/core/src/capabilities.test.ts implementation pending")
}
