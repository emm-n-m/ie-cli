use flate2::Compression;
use flate2::write::ZlibEncoder;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn save_list_and_info_read_folder_save() {
    let fixture = TestInstallation::new("save-list-info");
    let save_dir = fixture.root().join("save");
    let save = save_dir.join("000000007-Chapter 1 Start");
    fs::create_dir_all(&save).expect("save folder should be creatable");
    fs::write(save.join("BALDUR.gam"), build_minimal_gam()).expect("GAM should be writable");
    fs::write(save.join("BALDUR.SAV"), build_minimal_sav()).expect("SAV should be writable");

    let list_output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("save-list")
        .arg("--game")
        .arg(fixture.root())
        .arg("--format")
        .arg("json")
        .output()
        .expect("iecli should run");

    assert!(
        list_output.status.success(),
        "save-list failed: {}",
        String::from_utf8_lossy(&list_output.stderr)
    );
    let saves: Value = serde_json::from_slice(&list_output.stdout).expect("JSON should parse");
    assert_eq!(saves[0]["folder_name"], "000000007-Chapter 1 Start");
    assert_eq!(saves[0]["display_name"], "Chapter 1 Start");
    assert_eq!(saves[0]["has_gam"], true);
    assert_eq!(saves[0]["has_sav"], true);

    let info_output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("save-info")
        .arg("--game")
        .arg(fixture.root())
        .arg("--save")
        .arg("Chapter 1 Start")
        .arg("--part")
        .arg("all")
        .output()
        .expect("iecli should run");

    assert!(
        info_output.status.success(),
        "save-info failed: {}",
        String::from_utf8_lossy(&info_output.stderr)
    );
    let info: Value = serde_json::from_slice(&info_output.stdout).expect("JSON should parse");
    assert_eq!(info["save"]["display_name"], "Chapter 1 Start");
    assert_eq!(info["gam"]["header"]["game_time"], 2181);
    assert_eq!(info["gam"]["header"]["party_gold"], 2500);
    assert_eq!(info["sav"]["entries"][0]["filename"], "AR0602.ARE");
    assert_eq!(info["sav"]["entries"][0]["resource_type"], "ARE");
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
        fs::write(root.join("chitin.key"), build_empty_key()).expect("chitin.key should exist");
        fs::write(root.join("dialog.tlk"), build_minimal_tlk()).expect("dialog.tlk should exist");
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

fn build_minimal_tlk() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"TLK V1  ");
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&18u32.to_le_bytes());
    bytes
}

fn build_minimal_gam() -> Vec<u8> {
    let mut bytes = vec![0u8; 0xB4];
    bytes[0..8].copy_from_slice(b"GAMEV2.0");
    write_u32(&mut bytes, 0x08, 2181);
    write_u32(&mut bytes, 0x18, 2500);
    write_u16(&mut bytes, 0x1C, 0xFFFF);
    write_u32(&mut bytes, 0x20, 0xB4);
    write_u32(&mut bytes, 0x24, 0);
    write_u32(&mut bytes, 0x38, 0xB4);
    write_u32(&mut bytes, 0x3C, 0);
    bytes[0x40..0x46].copy_from_slice(b"AR0602");
    write_u32(&mut bytes, 0x54, 125);
    bytes[0x58..0x5E].copy_from_slice(b"AR0602");
    write_u32(&mut bytes, 0x74, 9876);
    bytes
}

fn build_minimal_sav() -> Vec<u8> {
    let payload = b"AREA bytes";
    let compressed = zlib(payload);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"SAV V1.0");
    bytes.extend_from_slice(&11u32.to_le_bytes());
    bytes.extend_from_slice(b"AR0602.ARE\0");
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&compressed);
    bytes
}

fn zlib(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(bytes)
        .expect("test payload should compress");
    encoder.finish().expect("test payload should finish")
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
