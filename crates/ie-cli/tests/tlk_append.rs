use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn tlk_append_writes_copy_and_strref_file() {
    let fixture = TestInstallation::new("cli-tlk-append", build_tlk(&["Existing"]));
    let output = fixture.root().join("dialog-patched.tlk");
    let strref_output = fixture.root().join("new-strref.txt");

    let output_json = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("tlk-append")
        .arg("--game")
        .arg(fixture.root())
        .arg("--text")
        .arg("New line")
        .arg("--tlk-out")
        .arg(&output)
        .arg("--output-strref-to")
        .arg(&strref_output)
        .output()
        .expect("iecli should run");

    assert!(
        output_json.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output_json.stderr)
    );

    let stdout: Value =
        serde_json::from_slice(&output_json.stdout).expect("stdout should be JSON");
    assert_eq!(stdout["strref"], 1);
    assert_eq!(stdout["text"], "New line");
    assert_eq!(fs::read_to_string(strref_output).expect("strref file should exist"), "1");
    assert!(output.exists());

    fs::copy(output, fixture.root().join("dialog.tlk")).expect("patched TLK should be installed");
    let lookup = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("tlk")
        .arg("--game")
        .arg(fixture.root())
        .arg("--strref")
        .arg("1")
        .output()
        .expect("iecli should run");

    assert!(
        lookup.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    let stdout: Value = serde_json::from_slice(&lookup.stdout).expect("stdout should be JSON");
    assert_eq!(stdout["text"], "New line");
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
        root.push(format!("iecli-{label}-{unique}-{}", std::process::id()));

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
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 16]);
        bytes.extend_from_slice(&(text_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(entry.len() as u32).to_le_bytes());
        text_bytes.extend_from_slice(entry.as_bytes());
    }

    bytes.extend_from_slice(&text_bytes);
    bytes
}
