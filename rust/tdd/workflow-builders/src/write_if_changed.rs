use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneratedFileWriter {
    pub skips_unchanged_generated_writes: bool,
}

impl GeneratedFileWriter {
    pub const fn new(skips_unchanged_generated_writes: bool) -> Self {
        Self {
            skips_unchanged_generated_writes,
        }
    }

    pub fn write_generated_file(&self, target_path: &Path, content: &str) -> Result<(), String> {
        let _ = (self.skips_unchanged_generated_writes, target_path, content);
        panic!("TDD RED: packages/builders/src/write-if-changed.test.ts implementation pending")
    }
}

pub fn has_same_content(target_path: &Path, content: &str) -> bool {
    let _ = (target_path, content);
    panic!("TDD RED: packages/builders/src/write-if-changed.test.ts implementation pending")
}

pub fn write_file_if_changed(target_path: &Path, content: &str) -> Result<bool, String> {
    let _ = (target_path, content);
    panic!("TDD RED: packages/builders/src/write-if-changed.test.ts implementation pending")
}
