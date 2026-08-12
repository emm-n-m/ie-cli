use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

// The detected game variant selects the effect-opcode table, so a misdetected install decodes
// PST effects against the BG table and silently produces wrong opcode names. `locate` is where
// that detection becomes visible, so these tests pin it to the JSON contract.

#[test]
fn locate_reports_standard_variant_for_a_plain_install() {
    let installation = TestInstallation::new("locate-standard");
    installation.write_override_resource("ACIDBL.ITM", b"ITM V1  ");

    let located = run_locate(installation.root(), "ACIDBL.ITM");

    assert_eq!(located["resource_name"], "ACIDBL.ITM");
    assert_eq!(located["source_kind"], "override");
    assert_eq!(located["game_variant"], "standard");
}

#[test]
fn locate_reports_pst_variant_from_root_marker_file() {
    let installation = TestInstallation::new("locate-pst");
    installation.write_override_resource("ACIDBL.ITM", b"ITM V1  ");
    // Detection keys off root marker files rather than the folder name, because installs are
    // routinely renamed (the local PSTEE install is a directory called "Project P").
    installation.write_root_file("torment.lua", b"-- marker");

    let located = run_locate(installation.root(), "ACIDBL.ITM");

    assert_eq!(located["game_variant"], "pst");
}

#[test]
fn locate_reports_iwd_variant_from_root_marker_file() {
    let installation = TestInstallation::new("locate-iwd");
    installation.write_override_resource("ACIDBL.ITM", b"ITM V1  ");
    installation.write_root_file("icewind.lua", b"-- marker");

    let located = run_locate(installation.root(), "ACIDBL.ITM");

    assert_eq!(located["game_variant"], "iwd");
}

fn run_locate(game: &Path, resource: &str) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("locate")
        .arg("--game")
        .arg(game)
        .arg("--resource")
        .arg(resource)
        .output()
        .expect("iecli should run");

    assert!(
        output.status.success(),
        "iecli failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("locate should emit JSON")
}

struct TestInstallation {
    root: PathBuf,
}

impl TestInstallation {
    fn new(label: &str) -> Self {
        let mut root = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        root.push(format!("iecli-{label}-{unique}-{}", std::process::id()));

        fs::create_dir_all(&root).expect("temporary installation root should be creatable");
        fs::write(root.join("chitin.key"), build_empty_key())
            .expect("chitin.key should be writable");

        Self { root }
    }

    fn write_override_resource(&self, file_name: &str, bytes: &[u8]) {
        let override_dir = self.root.join("override");
        fs::create_dir_all(&override_dir).expect("override directory should be creatable");
        fs::write(override_dir.join(file_name), bytes)
            .expect("override resource should be writable");
    }

    fn write_root_file(&self, file_name: &str, bytes: &[u8]) {
        fs::write(self.root.join(file_name), bytes).expect("root marker file should be writable");
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
