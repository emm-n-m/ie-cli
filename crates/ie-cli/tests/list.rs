use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const CRE_TYPE_CODE: u16 = 0x03F1;
const ITM_TYPE_CODE: u16 = 0x03ED;

#[test]
fn dump_raw_source_bif_reads_archive_when_override_exists() {
    let fixture = TestInstallation::new("dump-raw-source-bif");
    fixture.write_archive_install(
        "data/items.bif",
        &[TestResourceSpec::new("KIRINH.CRE", CRE_TYPE_CODE, b"CRE BASE")],
    );
    fixture.write_override("KIRINH.CRE", b"CRE OVERRIDE");

    let output_path = fixture.root().join("kirinh.cre");
    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("dump-raw")
        .arg("--game")
        .arg(fixture.root())
        .arg("--resource")
        .arg("KIRINH.CRE")
        .arg("--source")
        .arg("bif")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("iecli should run");

    assert!(
        output.status.success(),
        "iecli failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read(&output_path).expect("dump-raw should write output"),
        b"CRE BASE"
    );
}

#[test]
fn list_json_prefers_override_and_filters_by_glob() {
    let fixture = TestInstallation::new("list-json");
    fixture.write_archive_install(
        "data/items.bif",
        &[
            TestResourceSpec::new("KIRINH.CRE", CRE_TYPE_CODE, b"CRE BASE"),
            TestResourceSpec::new("MISC51.ITM", ITM_TYPE_CODE, b"ITM BASE"),
        ],
    );
    fixture.write_override("KIRINH.CRE", b"CRE OVERRIDE");

    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("list")
        .arg("--game")
        .arg(fixture.root())
        .arg("--type")
        .arg("CRE")
        .arg("--name")
        .arg("kir*")
        .arg("--format")
        .arg("json")
        .output()
        .expect("iecli should run");

    assert!(
        output.status.success(),
        "iecli failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout: Value = serde_json::from_slice(&output.stdout).expect("list should emit JSON");
    let entries = stdout.as_array().expect("list output should be a JSON array");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["resref"], "KIRINH");
    assert_eq!(entries[0]["type"], "CRE");
    assert_eq!(entries[0]["source_kind"], "Override");
}

#[test]
fn locate_source_override_errors_cleanly_when_missing() {
    let fixture = TestInstallation::new("locate-override-missing");
    fixture.write_archive_install(
        "data/items.bif",
        &[TestResourceSpec::new("KIRINH.CRE", CRE_TYPE_CODE, b"CRE BASE")],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("locate")
        .arg("--game")
        .arg(fixture.root())
        .arg("--resource")
        .arg("KIRINH.CRE")
        .arg("--source")
        .arg("override")
        .output()
        .expect("iecli should run");

    assert!(!output.status.success(), "locate should fail");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("resource not found in override"),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[derive(Clone, Copy)]
struct TestResourceSpec<'a> {
    resource_name: &'a str,
    type_code: u16,
    bytes: &'a [u8],
}

impl<'a> TestResourceSpec<'a> {
    fn new(resource_name: &'a str, type_code: u16, bytes: &'a [u8]) -> Self {
        Self {
            resource_name,
            type_code,
            bytes,
        }
    }
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
        fs::write(root.join("dialog.tlk"), build_minimal_tlk()).expect("dialog.tlk should exist");
        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn write_archive_install(&self, relative_archive_path: &str, resources: &[TestResourceSpec<'_>]) {
        let archive_path = self.root.join(relative_archive_path);
        if let Some(parent) = archive_path.parent() {
            fs::create_dir_all(parent).expect("archive parent directory should exist");
        }

        let archive_entries = resources
            .iter()
            .enumerate()
            .map(|(index, resource)| BiffFileEntry {
                type_code: resource.type_code,
                locator: (index as u32) + 1,
                bytes: resource.bytes,
            })
            .collect::<Vec<_>>();
        let key_entries = resources
            .iter()
            .enumerate()
            .map(|(index, resource)| {
                let (resref, extension) = resource
                    .resource_name
                    .split_once('.')
                    .expect("resource_name should include an extension");
                KeyResourceSpec {
                    resref,
                    extension,
                    type_code: resource.type_code,
                    locator: (index as u32) + 1,
                }
            })
            .collect::<Vec<_>>();

        fs::write(&archive_path, build_biff_archive(&archive_entries))
            .expect("archive should be writable");
        fs::write(
            self.root.join("chitin.key"),
            build_key_file(relative_archive_path, &key_entries),
        )
        .expect("chitin.key should be writable");
    }

    fn write_override(&self, resource_name: &str, bytes: &[u8]) {
        let override_dir = self.root.join("override");
        fs::create_dir_all(&override_dir).expect("override directory should be creatable");
        fs::write(override_dir.join(resource_name), bytes)
            .expect("override resource should be writable");
    }
}

impl Drop for TestInstallation {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[derive(Clone, Copy)]
struct BiffFileEntry<'a> {
    type_code: u16,
    locator: u32,
    bytes: &'a [u8],
}

fn build_biff_archive(entries: &[BiffFileEntry<'_>]) -> Vec<u8> {
    let file_entry_offset = 20u32;
    let resource_offset = file_entry_offset + (entries.len() as u32 * 16);
    let mut next_resource_offset = resource_offset;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BIFFV1  ");
    bytes.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&file_entry_offset.to_le_bytes());

    for entry in entries {
        bytes.extend_from_slice(&(entry.locator & 0x000F_FFFF).to_le_bytes());
        bytes.extend_from_slice(&next_resource_offset.to_le_bytes());
        bytes.extend_from_slice(&(entry.bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&entry.type_code.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        next_resource_offset += entry.bytes.len() as u32;
    }

    for entry in entries {
        bytes.extend_from_slice(entry.bytes);
    }

    bytes
}

#[derive(Clone, Copy)]
struct KeyResourceSpec<'a> {
    resref: &'a str,
    extension: &'a str,
    type_code: u16,
    locator: u32,
}

fn build_key_file(relative_archive_path: &str, resources: &[KeyResourceSpec<'_>]) -> Vec<u8> {
    let mut archive_name_bytes = relative_archive_path.replace('/', "\\").into_bytes();
    archive_name_bytes.push(0);

    let bif_count = 1u32;
    let resource_count = resources.len() as u32;
    let bif_offset = 24u32;
    let resource_offset = bif_offset + 12;
    let string_offset = resource_offset + (14 * resource_count);

    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"KEY V1  ");
    bytes.extend_from_slice(&bif_count.to_le_bytes());
    bytes.extend_from_slice(&resource_count.to_le_bytes());
    bytes.extend_from_slice(&bif_offset.to_le_bytes());
    bytes.extend_from_slice(&resource_offset.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&string_offset.to_le_bytes());
    bytes.extend_from_slice(&(archive_name_bytes.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());

    for resource in resources {
        bytes.extend_from_slice(&padded_resref(resource.resref));
        bytes.extend_from_slice(&resource.type_code.to_le_bytes());
        bytes.extend_from_slice(&resource.locator.to_le_bytes());
        assert_eq!(
            resource.extension.len(),
            3,
            "test helper expects 3-char extensions"
        );
    }

    bytes.extend_from_slice(&archive_name_bytes);
    bytes
}

fn build_minimal_tlk() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"TLK V1  ");
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&18u32.to_le_bytes());
    bytes
}

fn padded_resref(resref: &str) -> [u8; 8] {
    let mut bytes = [0u8; 8];
    bytes[..resref.len()].copy_from_slice(resref.as_bytes());
    bytes
}
