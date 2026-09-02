#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourcemapMode {
    Disabled,
    Enabled,
    Inline,
    Linked,
    External,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourcemapConfig {
    pub explicit: Option<SourcemapMode>,
    pub environment: Option<String>,
    pub node_environment: Option<String>,
    pub watch: bool,
}

pub fn resolve_sourcemap(config: &SourcemapConfig, default_mode: SourcemapMode) -> SourcemapMode {
    let _ = (config, default_mode);
    panic!("TDD RED: packages/builders/src/resolve-sourcemap.test.ts implementation pending")
}

pub fn is_development_build(config: &SourcemapConfig) -> bool {
    let _ = config;
    panic!("TDD RED: packages/builders/src/resolve-sourcemap.test.ts implementation pending")
}

pub fn default_sourcemap_mode(config: &SourcemapConfig) -> SourcemapMode {
    let _ = config;
    panic!("TDD RED: packages/builders/src/resolve-sourcemap.test.ts implementation pending")
}

pub fn sourcemaps_enabled(config: &SourcemapConfig) -> bool {
    let _ = config;
    panic!("TDD RED: packages/builders/src/resolve-sourcemap.test.ts implementation pending")
}
