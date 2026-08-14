use ie_core::{ResourceName, SourceKind};
use ie_io::{GameInstallation, IoError, ResourceLocator, ResourceReader, TlkResolver};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

const ITM_TYPE_CODE: u16 = 0x03ED;
const RESOURCE_LOCATOR: u32 = 1;

#[test]
fn reads_keyed_biff_from_dlc_zip_case_insensitively() {
    let fixture = TestInstallation::new("dlc-biff");
    fixture.write_keyed_archive("data/items.bif", b"ITM FROM DLC");
    fixture.write_zip(
        "test-dlc.zip",
        &[("DATA/ITEMS.BIF", fixture.biff_bytes(b"ITM FROM DLC"))],
    );
    fs::remove_file(fixture.root.join("data/items.bif")).expect("archive should only be in DLC");

    let installation = GameInstallation::discover(fixture.root()).expect("DLC should mount");
    let locator = ResourceLocator::new(installation).expect("KEY should parse");
    let resource = ResourceName::parse("foo.itm").expect("resource should parse");
    let bytes = ResourceReader
        .read(&locator, &resource)
        .expect("DLC-backed BIF should load");

    assert_eq!(bytes.bytes, b"ITM FROM DLC");
    assert_eq!(bytes.metadata.source_kind, SourceKind::Dlc);
    assert!(
        bytes
            .metadata
            .source_path
            .to_string_lossy()
            .contains("!DATA/ITEMS.BIF")
    );
}

#[test]
fn disk_biff_beats_dlc_biff() {
    let fixture = TestInstallation::new("dlc-disk-precedence");
    fixture.write_keyed_archive("data/items.bif", b"ITM ON DISK");
    fixture.write_zip(
        "test-dlc.zip",
        &[("data/items.bif", fixture.biff_bytes(b"ITM IN DLC"))],
    );

    let installation = GameInstallation::discover(fixture.root()).expect("DLC should mount");
    let locator = ResourceLocator::new(installation).expect("KEY should parse");
    let resource = ResourceName::parse("FOO.ITM").expect("resource should parse");
    let bytes = ResourceReader
        .read(&locator, &resource)
        .expect("resource should load");

    assert_eq!(bytes.bytes, b"ITM ON DISK");
    assert_eq!(bytes.metadata.source_kind, SourceKind::Bif);
}

#[test]
fn game_override_beats_dlc_override() {
    let fixture = TestInstallation::new("dlc-override-precedence");
    fixture.write_keyed_archive("data/items.bif", b"ITM BASE");
    fixture.write_override("FOO.ITM", b"ITM GAME OVERRIDE");
    fixture.write_zip(
        "test-dlc.zip",
        &[("override/foo.itm", b"ITM DLC OVERRIDE".to_vec())],
    );

    let installation = GameInstallation::discover(fixture.root()).expect("DLC should mount");
    let locator = ResourceLocator::new(installation).expect("KEY should parse");
    let resource = ResourceName::parse("FOO.ITM").expect("resource should parse");
    let bytes = ResourceReader
        .read(&locator, &resource)
        .expect("resource should load");

    assert_eq!(bytes.bytes, b"ITM GAME OVERRIDE");
    assert_eq!(bytes.metadata.source_kind, SourceKind::Override);
}

#[test]
fn later_sorted_dlc_override_wins_deterministically_and_is_listed() {
    let fixture = TestInstallation::new("dlc-order");
    fixture.write_keyed_archive("data/items.bif", b"ITM BASE");
    fixture.write_zip("a-first.zip", &[("override/foo.itm", b"ITM A".to_vec())]);
    fixture.write_zip("b-second.zip", &[("override/foo.itm", b"ITM B".to_vec())]);

    let installation = GameInstallation::discover(fixture.root()).expect("DLCs should mount");
    let locator = ResourceLocator::new(installation).expect("KEY should parse");
    let resource = ResourceName::parse("FOO.ITM").expect("resource should parse");
    let bytes = ResourceReader
        .read(&locator, &resource)
        .expect("resource should load");
    assert_eq!(bytes.bytes, b"ITM B");
    assert!(
        bytes
            .metadata
            .source_path
            .to_string_lossy()
            .contains("b-second.zip!")
    );

    let entries = locator
        .list(Default::default())
        .expect("listing should work");
    let foo = entries
        .into_iter()
        .find(|entry| entry.resource_name == "FOO.ITM")
        .expect("DLC override should be listed");
    assert_eq!(foo.source_kind, SourceKind::Dlc);
    assert!(foo.source_path.to_string_lossy().contains("b-second.zip!"));
}

#[test]
fn disabled_files_are_not_mounted() {
    let fixture = TestInstallation::new("dlc-disabled");
    fixture.write_keyed_archive("data/items.bif", b"ITM BASE");
    fixture.write_zip_path(
        &fixture.root.join("ignored.disabled"),
        &[("override/foo.itm", b"ITM DISABLED".to_vec())],
    );

    let installation = GameInstallation::discover(fixture.root()).expect("install should load");
    assert!(installation.dlc_archives.is_empty());
    let locator = ResourceLocator::new(installation).expect("KEY should parse");
    let resource = ResourceName::parse("FOO.ITM").expect("resource should parse");
    let bytes = ResourceReader
        .read(&locator, &resource)
        .expect("base resource should load");
    assert_eq!(bytes.bytes, b"ITM BASE");
    assert_eq!(bytes.metadata.source_kind, SourceKind::Bif);
}

#[test]
fn larger_dlc_tlk_extends_base_tlk() {
    let fixture = TestInstallation::new("dlc-tlk");
    fixture.write_language_tlk(&["Base"]);
    fixture.write_zip(
        "test-dlc.zip",
        &[(
            "lang/en_US/dialog.tlk",
            build_tlk(&["Base", "DLC addition"]),
        )],
    );

    let installation = GameInstallation::discover(fixture.root()).expect("DLC should mount");
    assert_eq!(installation.language.as_deref(), Some("en_US"));
    assert!(
        installation
            .dialog_tlk
            .as_ref()
            .expect("TLK should be found")
            .to_string_lossy()
            .contains("test-dlc.zip!lang/en_US/dialog.tlk")
    );
    let resolver = TlkResolver::new(&installation).expect("DLC TLK should load");
    assert_eq!(
        resolver.resolve(1).expect("DLC strref should resolve").text,
        "DLC addition"
    );
}

#[test]
fn mounts_zip64_archive() {
    let fixture = TestInstallation::new("dlc-zip64");
    fixture.write_keyed_archive("data/items.bif", b"ITM BASE");
    let archive_path = fixture.root.join("dlc/zip64.zip");
    fs::create_dir_all(archive_path.parent().expect("DLC parent should exist"))
        .expect("DLC directory should exist");
    fs::write(
        &archive_path,
        build_zip64_stored("override/zip64.itm", b"ITM ZIP64"),
    )
    .expect("Zip64 fixture should be writable");

    let installation = GameInstallation::discover(fixture.root()).expect("Zip64 should mount");
    let locator = ResourceLocator::new(installation).expect("KEY should parse");
    let resource = ResourceName::parse("ZIP64.ITM").expect("resource should parse");
    let bytes = ResourceReader
        .read(&locator, &resource)
        .expect("Zip64 entry should load");
    assert_eq!(bytes.bytes, b"ITM ZIP64");
    assert_eq!(bytes.metadata.source_kind, SourceKind::Dlc);
}

#[test]
fn invalid_dlc_is_reported_during_discovery() {
    let fixture = TestInstallation::new("dlc-invalid");
    fs::create_dir_all(fixture.root.join("dlc")).expect("DLC directory should exist");
    fs::write(fixture.root.join("dlc/broken.zip"), b"not a zip").expect("fixture should write");

    let error = GameInstallation::discover(fixture.root()).expect_err("invalid DLC must fail");
    assert!(matches!(error, IoError::DlcArchive { .. }));
}

struct TestInstallation {
    root: PathBuf,
}

impl TestInstallation {
    fn new(label: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("iecli-dlc-{label}-{unique}"));
        fs::create_dir_all(&root).expect("fixture root should be creatable");
        fs::write(root.join("chitin.key"), build_empty_key()).expect("KEY should be writable");
        fs::write(root.join("dialog.tlk"), build_tlk(&[])).expect("TLK should be writable");
        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn write_keyed_archive(&self, relative_path: &str, bytes: &[u8]) {
        let archive_path = self.root.join(relative_path);
        fs::create_dir_all(archive_path.parent().expect("archive parent should exist"))
            .expect("archive parent should exist");
        fs::write(&archive_path, self.biff_bytes(bytes)).expect("BIF should be writable");
        fs::write(
            self.root.join("chitin.key"),
            build_key(relative_path, "FOO", "ITM", ITM_TYPE_CODE, RESOURCE_LOCATOR),
        )
        .expect("KEY should be writable");
    }

    fn biff_bytes(&self, bytes: &[u8]) -> Vec<u8> {
        build_biff(bytes)
    }

    fn write_override(&self, name: &str, bytes: &[u8]) {
        fs::create_dir_all(self.root.join("override")).expect("override should exist");
        fs::write(self.root.join("override").join(name), bytes).expect("override should write");
    }

    fn write_language_tlk(&self, entries: &[&str]) {
        fs::create_dir_all(self.root.join("lang/en_US")).expect("language should exist");
        fs::write(self.root.join("lang/en_US/dialog.tlk"), build_tlk(entries))
            .expect("language TLK should write");
        fs::remove_file(self.root.join("dialog.tlk")).expect("direct TLK should be removed");
    }

    fn write_zip(&self, name: &str, entries: &[(&str, Vec<u8>)]) {
        self.write_zip_path(&self.root.join("dlc").join(name), entries);
    }

    fn write_zip_path(&self, path: &Path, entries: &[(&str, Vec<u8>)]) {
        fs::create_dir_all(path.parent().expect("zip parent should exist"))
            .expect("zip parent should exist");
        let file = File::create(path).expect("zip should be creatable");
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for (name, bytes) in entries {
            writer
                .start_file(name, options)
                .expect("zip entry should start");
            writer.write_all(bytes).expect("zip entry should write");
        }
        writer.finish().expect("zip should finish");
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

fn build_key(path: &str, resref: &str, extension: &str, type_code: u16, locator: u32) -> Vec<u8> {
    let path = path.replace('/', "\\");
    let mut path_bytes = path.into_bytes();
    path_bytes.push(0);
    let resource_offset = 36u32;
    let string_offset = resource_offset + 14;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"KEY V1  ");
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&24u32.to_le_bytes());
    bytes.extend_from_slice(&resource_offset.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&string_offset.to_le_bytes());
    bytes.extend_from_slice(&(path_bytes.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    let mut resref_bytes = [0u8; 8];
    resref_bytes[..resref.len()].copy_from_slice(resref.as_bytes());
    bytes.extend_from_slice(&resref_bytes);
    assert_eq!(extension, "ITM");
    bytes.extend_from_slice(&type_code.to_le_bytes());
    bytes.extend_from_slice(&locator.to_le_bytes());
    bytes.extend_from_slice(&path_bytes);
    bytes
}

fn build_biff(bytes: &[u8]) -> Vec<u8> {
    let data_offset = 36u32;
    let mut archive = Vec::new();
    archive.extend_from_slice(b"BIFFV1  ");
    archive.extend_from_slice(&1u32.to_le_bytes());
    archive.extend_from_slice(&0u32.to_le_bytes());
    archive.extend_from_slice(&20u32.to_le_bytes());
    archive.extend_from_slice(&RESOURCE_LOCATOR.to_le_bytes());
    archive.extend_from_slice(&data_offset.to_le_bytes());
    archive.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    archive.extend_from_slice(&ITM_TYPE_CODE.to_le_bytes());
    archive.extend_from_slice(&0u16.to_le_bytes());
    archive.extend_from_slice(bytes);
    archive
}

fn build_tlk(entries: &[&str]) -> Vec<u8> {
    let strings_offset = 18u32 + entries.len() as u32 * 26;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"TLK V1  ");
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&strings_offset.to_le_bytes());
    let mut text = Vec::new();
    for entry in entries {
        bytes.extend_from_slice(&[0u8; 18]);
        bytes.extend_from_slice(&(text.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(entry.len() as u32).to_le_bytes());
        text.extend_from_slice(entry.as_bytes());
    }
    bytes.extend_from_slice(&text);
    bytes
}

fn build_zip64_stored(name: &str, data: &[u8]) -> Vec<u8> {
    let name = name.as_bytes();
    let crc = crc32(data);
    let local_extra = zip64_extra(&[data.len() as u64, data.len() as u64]);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&0x04034b50u32.to_le_bytes());
    bytes.extend_from_slice(&45u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&crc.to_le_bytes());
    bytes.extend_from_slice(&0xffff_ffffu32.to_le_bytes());
    bytes.extend_from_slice(&0xffff_ffffu32.to_le_bytes());
    bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&(local_extra.len() as u16).to_le_bytes());
    bytes.extend_from_slice(name);
    bytes.extend_from_slice(&local_extra);
    bytes.extend_from_slice(data);
    let central_offset = bytes.len() as u64;
    let central_extra = zip64_extra(&[data.len() as u64, data.len() as u64, 0]);
    bytes.extend_from_slice(&0x02014b50u32.to_le_bytes());
    bytes.extend_from_slice(&45u16.to_le_bytes());
    bytes.extend_from_slice(&45u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&crc.to_le_bytes());
    bytes.extend_from_slice(&0xffff_ffffu32.to_le_bytes());
    bytes.extend_from_slice(&0xffff_ffffu32.to_le_bytes());
    bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&(central_extra.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0xffff_ffffu32.to_le_bytes());
    bytes.extend_from_slice(name);
    bytes.extend_from_slice(&central_extra);
    let central_size = bytes.len() as u64 - central_offset;
    let zip64_eocd_offset = bytes.len() as u64;
    bytes.extend_from_slice(&0x06064b50u32.to_le_bytes());
    bytes.extend_from_slice(&44u64.to_le_bytes());
    bytes.extend_from_slice(&45u16.to_le_bytes());
    bytes.extend_from_slice(&45u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&1u64.to_le_bytes());
    bytes.extend_from_slice(&1u64.to_le_bytes());
    bytes.extend_from_slice(&central_size.to_le_bytes());
    bytes.extend_from_slice(&central_offset.to_le_bytes());
    bytes.extend_from_slice(&0x07064b50u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&zip64_eocd_offset.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&0x06054b50u32.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0xffffu16.to_le_bytes());
    bytes.extend_from_slice(&0xffffu16.to_le_bytes());
    bytes.extend_from_slice(&0xffff_ffffu32.to_le_bytes());
    bytes.extend_from_slice(&0xffff_ffffu32.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes
}

fn zip64_extra(values: &[u64]) -> Vec<u8> {
    let mut extra = Vec::new();
    extra.extend_from_slice(&1u16.to_le_bytes());
    extra.extend_from_slice(&((values.len() * 8) as u16).to_le_bytes());
    for value in values {
        extra.extend_from_slice(&value.to_le_bytes());
    }
    extra
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}
