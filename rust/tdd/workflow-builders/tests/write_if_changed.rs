use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::write_if_changed::{
    GeneratedFileWriter, has_same_content, write_file_if_changed,
};

struct TestRoot(PathBuf);

impl TestRoot {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "write-if-changed-rust-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }

    fn join(&self, relative: &str) -> PathBuf {
        self.0.join(relative)
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(unix)]
fn file_identity(path: &Path) -> String {
    use std::os::unix::fs::MetadataExt;

    let metadata = fs::metadata(path).unwrap();
    let modified = metadata
        .modified()
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{modified}:{}", metadata.ino())
}

#[cfg(windows)]
fn file_identity(path: &Path) -> String {
    use std::os::windows::fs::MetadataExt;

    let metadata = fs::metadata(path).unwrap();
    format!("{}:{}", metadata.last_write_time(), metadata.file_index())
}

#[test]
fn has_same_content_is_false_when_the_file_does_not_exist() {
    let root = TestRoot::new();
    assert!(!has_same_content(&root.join("missing.js"), "hello"));
}

#[test]
fn has_same_content_is_true_for_byte_identical_content() {
    let root = TestRoot::new();
    let target = root.join("a.js");
    fs::write(&target, "export const a = 1;\n").unwrap();

    assert!(has_same_content(&target, "export const a = 1;\n"));
}

#[test]
fn has_same_content_is_false_for_same_length_different_bytes() {
    let root = TestRoot::new();
    let target = root.join("a.js");
    fs::write(&target, "aaaa").unwrap();

    assert!(!has_same_content(&target, "aaab"));
}

#[test]
fn has_same_content_is_false_when_only_the_length_differs() {
    let root = TestRoot::new();
    let target = root.join("a.js");
    fs::write(&target, "aaaa").unwrap();

    assert!(!has_same_content(&target, "aaaaa"));
}

#[test]
fn has_same_content_compares_utf8_byte_length_not_code_units() {
    let root = TestRoot::new();
    let target = root.join("a.js");
    fs::write(&target, "// é😀\n").unwrap();

    assert!(has_same_content(&target, "// é😀\n"));
}

#[test]
fn has_same_content_rejects_different_bytes_that_decode_to_the_same_text() {
    let root = TestRoot::new();
    let target = root.join("a.js");
    fs::write(&target, [0x61, 0xf0, 0x90, 0x80]).unwrap();

    assert!(!has_same_content(&target, "a�"));
}

#[test]
fn has_same_content_is_false_when_the_path_is_a_directory() {
    let root = TestRoot::new();
    let same_size_as_directory = "x".repeat(fs::metadata(&root.0).unwrap().len() as usize);

    assert!(!has_same_content(&root.0, &same_size_as_directory));
}

#[cfg(unix)]
#[test]
fn has_same_content_is_false_when_the_file_cannot_be_read() {
    use std::os::unix::fs::PermissionsExt;

    let root = TestRoot::new();
    let target = root.join("locked.js");
    fs::write(&target, "secret").unwrap();
    let original_mode = fs::metadata(&target).unwrap().permissions().mode();
    fs::set_permissions(&target, fs::Permissions::from_mode(0)).unwrap();

    if fs::read(&target).is_ok() {
        fs::set_permissions(&target, fs::Permissions::from_mode(original_mode)).unwrap();
        return;
    }

    let same = has_same_content(&target, "secret");
    fs::set_permissions(&target, fs::Permissions::from_mode(original_mode)).unwrap();
    assert!(!same);
}

#[cfg(not(unix))]
#[test]
fn has_same_content_is_false_when_the_file_cannot_be_read() {
    let root = TestRoot::new();
    let target = root.join("locked.js");
    fs::write(&target, "secret").unwrap();

    if fs::read(&target).is_ok() {
        return;
    }
    assert!(!has_same_content(&target, "secret"));
}

#[test]
fn write_file_if_changed_creates_a_missing_file() {
    let root = TestRoot::new();
    let target = root.join("a.js");

    assert!(write_file_if_changed(&target, "hello").unwrap());
    assert_eq!(fs::read_to_string(target).unwrap(), "hello");
}

#[test]
fn write_file_if_changed_does_not_touch_unchanged_content() {
    let root = TestRoot::new();
    let target = root.join("a.js");
    write_file_if_changed(&target, "hello").unwrap();
    let before = file_identity(&target);

    assert!(!write_file_if_changed(&target, "hello").unwrap());
    assert_eq!(file_identity(&target), before);
}

#[test]
fn write_file_if_changed_rewrites_changed_content() {
    let root = TestRoot::new();
    let target = root.join("a.js");
    write_file_if_changed(&target, "hello").unwrap();

    assert!(write_file_if_changed(&target, "goodbye").unwrap());
    assert_eq!(fs::read_to_string(target).unwrap(), "goodbye");
}

#[test]
fn generated_file_writer_leaves_unchanged_output_untouched() {
    let root = TestRoot::new();
    let target = root.join("route.js");
    let writer = GeneratedFileWriter::new(true);
    writer
        .write_generated_file(&target, "export const GET = () => {};")
        .unwrap();
    let before = file_identity(&target);

    writer
        .write_generated_file(&target, "export const GET = () => {};")
        .unwrap();
    assert_eq!(file_identity(&target), before);
}

#[test]
fn generated_file_writer_leaves_no_temp_files_when_skipping() {
    let root = TestRoot::new();
    let target = root.join("route.js");
    let writer = GeneratedFileWriter::new(true);
    writer.write_generated_file(&target, "same").unwrap();
    writer.write_generated_file(&target, "same").unwrap();

    let temp_files = fs::read_dir(&root.0)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
        .count();
    assert_eq!(temp_files, 0);
}

#[test]
fn generated_file_writer_writes_changed_output() {
    let root = TestRoot::new();
    let target = root.join("route.js");
    let writer = GeneratedFileWriter::new(true);
    writer.write_generated_file(&target, "v1").unwrap();
    writer.write_generated_file(&target, "v2").unwrap();

    assert_eq!(fs::read_to_string(target).unwrap(), "v2");
}

#[test]
fn generated_file_writer_creates_the_file_on_first_write() {
    let root = TestRoot::new();
    let target = root.join("route.js");
    let writer = GeneratedFileWriter::new(true);
    writer.write_generated_file(&target, "first").unwrap();

    assert_eq!(fs::read_to_string(target).unwrap(), "first");
}

#[test]
fn generated_file_writer_rewrites_unchanged_output_when_opted_out() {
    let root = TestRoot::new();
    let target = root.join("workflows.mjs");
    let writer = GeneratedFileWriter::new(false);
    writer
        .write_generated_file(&target, "export const a = 1;")
        .unwrap();
    let before = file_identity(&target);

    writer
        .write_generated_file(&target, "export const a = 1;")
        .unwrap();
    assert_ne!(file_identity(&target), before);
    assert_eq!(fs::read_to_string(target).unwrap(), "export const a = 1;");
}
