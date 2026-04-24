use ie_io::{GameInstallation, IoError, TlkResolver};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn resolves_valid_tlk_string() {
    let fixture = TestInstallation::new("tlk-valid", build_tlk(&["Flail of Ages"]));
    let installation =
        GameInstallation::discover(fixture.root()).expect("synthetic installation should be valid");
    let resolver = TlkResolver::new(&installation).expect("dialog.tlk should be discoverable");

    let entry = resolver.resolve(0).expect("strref 0 should resolve");
    assert_eq!(entry.strref, 0);
    assert_eq!(entry.text, "Flail of Ages");
}

#[test]
fn returns_out_of_range_for_invalid_tlk_string_ref() {
    let fixture = TestInstallation::new("tlk-out-of-range", build_tlk(&["Hello there"]));
    let installation =
        GameInstallation::discover(fixture.root()).expect("synthetic installation should be valid");
    let resolver = TlkResolver::new(&installation).expect("dialog.tlk should be discoverable");

    let error = resolver
        .resolve(1)
        .expect_err("strref 1 should be out of range for a 1-entry TLK");
    assert!(matches!(
        error,
        IoError::StrRefOutOfRange {
            requested: 1,
            entry_count: 1
        }
    ));
}

#[test]
fn resolves_empty_tlk_strings_without_error() {
    let fixture = TestInstallation::new("tlk-empty", build_tlk(&[""]));
    let installation =
        GameInstallation::discover(fixture.root()).expect("synthetic installation should be valid");
    let resolver = TlkResolver::new(&installation).expect("dialog.tlk should be discoverable");

    let entry = resolver.resolve(0).expect("empty TLK entry should resolve");
    assert_eq!(entry.text, "");
}

struct TestInstallation {
    root: PathBuf,
}

impl TestInstallation {
    fn new(label: &str, tlk_bytes: Vec<u8>) -> Self {
        let mut root = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        root.push(format!(
            "nearinfinity-tlk-{label}-{unique}-{}",
            std::process::id()
        ));

        fs::create_dir_all(&root).expect("temporary installation root should be creatable");
        fs::write(root.join("chitin.key"), build_empty_key())
            .expect("chitin.key should be writable");
        fs::write(root.join("dialog.tlk"), tlk_bytes).expect("dialog.tlk should be writable");

        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for TestInstallation {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn build_empty_key() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"KEY V1  ");
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&24u32.to_le_bytes());
    bytes.extend_from_slice(&24u32.to_le_bytes());
    bytes
}

fn build_tlk(entries: &[&str]) -> Vec<u8> {
    let strings_offset = 18u32 + (entries.len() as u32 * 26u32);
    let mut bytes = Vec::new();
    let mut text_bytes = Vec::new();

    bytes.extend_from_slice(b"TLK V1  ");
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&strings_offset.to_le_bytes());

    for entry in entries {
        bytes.extend_from_slice(&[0u8; 18]);
        bytes.extend_from_slice(&(text_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(entry.len() as u32).to_le_bytes());
        text_bytes.extend_from_slice(entry.as_bytes());
    }

    bytes.extend_from_slice(&text_bytes);
    bytes
}
