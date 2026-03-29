use flate2::Compression;
use flate2::write::ZlibEncoder;
use ie_core::{ResourceName, SourceKind};
use ie_io::{GameInstallation, IoError, ResourceLocator, ResourceReader};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const ITM_TYPE_CODE: u16 = 0x03ED;
const TIS_TYPE_CODE: u16 = 0x03EB;
const TEST_RESOURCE: &str = "FOO.ITM";
const RESOURCE_LOCATOR: u32 = 0x0000_0001;

#[test]
fn reads_resource_from_uncompressed_biff_archive() {
    let fixture = TestInstallation::new("uncompressed-biff");
    fixture.write_archive_install("data/items.bif", ArchiveKind::Biff, b"ITM BASE");

    let bytes = read_resource(fixture.root(), TEST_RESOURCE);
    assert_eq!(bytes.bytes, b"ITM BASE");
    assert_eq!(bytes.metadata.source_kind, SourceKind::Bif);
}

#[test]
fn reads_resource_from_compressed_bif_archive() {
    let fixture = TestInstallation::new("compressed-bif");
    fixture.write_archive_install("data/items.bif", ArchiveKind::Bif, b"ITM BIF");

    let bytes = read_resource(fixture.root(), TEST_RESOURCE);
    assert_eq!(bytes.bytes, b"ITM BIF");
    assert_eq!(bytes.metadata.source_kind, SourceKind::Bif);
}

#[test]
fn reads_resource_from_block_compressed_bifc_archive() {
    let fixture = TestInstallation::new("block-compressed-bifc");
    fixture.write_archive_install("data/items.cbf", ArchiveKind::Bifc, b"ITM BIFC");

    let bytes = read_resource(fixture.root(), TEST_RESOURCE);
    assert_eq!(bytes.bytes, b"ITM BIFC");
    assert_eq!(bytes.metadata.source_kind, SourceKind::Bif);
}

#[test]
fn override_resource_takes_precedence_over_archive_resource() {
    let fixture = TestInstallation::new("override-precedence");
    fixture.write_archive_install("data/items.bif", ArchiveKind::Biff, b"ITM BASE");
    fixture.write_override(TEST_RESOURCE, b"ITM OVERRIDE");

    let bytes = read_resource(fixture.root(), TEST_RESOURCE);
    assert_eq!(bytes.bytes, b"ITM OVERRIDE");
    assert_eq!(bytes.metadata.source_kind, SourceKind::Override);
}

#[test]
fn leaves_file_table_tis_entries_as_raw_bytes() {
    let fixture = TestInstallation::new("file-table-tis");
    fixture.write_custom_archive_install(
        "data/tiles.bif",
        "FOO.TIS",
        ArchiveKind::Biff,
        RESOURCE_LOCATOR,
        build_biff_with_file_entry("FOO", TIS_TYPE_CODE, RESOURCE_LOCATOR, b"TIS RAW"),
    );

    let bytes = read_resource(fixture.root(), "FOO.TIS");
    assert_eq!(bytes.bytes, b"TIS RAW");
    assert_eq!(bytes.metadata.source_kind, SourceKind::Bif);
}

#[test]
fn reads_tileset_table_entries_with_synthesized_tis_header() {
    let fixture = TestInstallation::new("tileset-table");
    fixture.write_custom_archive_install(
        "data/tiles.bif",
        "BAR.TIS",
        ArchiveKind::Biff,
        0x0000_0002,
        build_biff_with_file_and_tileset_entries(
            (ITM_TYPE_CODE, RESOURCE_LOCATOR, b"ITM BASE"),
            (TIS_TYPE_CODE, 0x0000_0002, 2, 4, b"ABCDEFGH"),
        ),
    );

    let bytes = read_resource(fixture.root(), "BAR.TIS");
    assert_eq!(&bytes.bytes[..8], b"TIS V1  ");
    assert_eq!(&bytes.bytes[8..12], &2u32.to_le_bytes());
    assert_eq!(&bytes.bytes[12..16], &4u32.to_le_bytes());
    assert_eq!(&bytes.bytes[24..], b"ABCDEFGH");
}

#[test]
fn returns_resource_not_found_when_locator_is_missing_from_biff() {
    let fixture = TestInstallation::new("missing-locator");
    fixture.write_custom_archive_install(
        "data/items.bif",
        TEST_RESOURCE,
        ArchiveKind::Biff,
        RESOURCE_LOCATOR,
        build_biff_with_file_entry("FOO", ITM_TYPE_CODE, 0x0000_0002, b"ITM BASE"),
    );

    let error = read_resource_error(fixture.root(), TEST_RESOURCE);
    assert!(matches!(
        error,
        IoError::ResourceNotFound(message) if message == "BIFF locator 0x00001"
    ));
}

#[test]
fn returns_unexpected_eof_for_truncated_archive_bytes() {
    let fixture = TestInstallation::new("truncated-biff");
    fixture.write_custom_archive_install(
        "data/items.bif",
        TEST_RESOURCE,
        ArchiveKind::Biff,
        RESOURCE_LOCATOR,
        b"BIFFV1  \x01\x00".to_vec(),
    );

    let error = read_resource_error(fixture.root(), TEST_RESOURCE);
    assert!(matches!(error, IoError::UnexpectedEof(_)));
}

#[test]
fn reads_known_bg2ee_resource_when_ie_game_path_is_set() {
    let Some(game_path) = std::env::var_os("IE_GAME_PATH") else {
        return;
    };

    let installation =
        GameInstallation::discover(game_path).expect("real installation should exist");
    let locator = ResourceLocator::new(installation).expect("KEY parsing should succeed");
    let resource = ResourceName::parse("ACIDBL.ITM").expect("resource name should parse");
    let reader = ResourceReader;
    let bytes = reader
        .read(&locator, &resource)
        .expect("resource should be readable from the real installation");

    assert!(bytes.bytes.starts_with(b"ITM "));
}

fn read_resource(root: &Path, resource_name: &str) -> ie_core::ResourceBytes {
    let installation =
        GameInstallation::discover(root).expect("synthetic installation should be valid");
    let locator = ResourceLocator::new(installation).expect("KEY parsing should succeed");
    let resource = ResourceName::parse(resource_name).expect("resource name should parse");
    let reader = ResourceReader;
    reader
        .read(&locator, &resource)
        .expect("resource should load")
}

fn read_resource_error(root: &Path, resource_name: &str) -> IoError {
    let installation =
        GameInstallation::discover(root).expect("synthetic installation should be valid");
    let locator = ResourceLocator::new(installation).expect("KEY parsing should succeed");
    let resource = ResourceName::parse(resource_name).expect("resource name should parse");
    let reader = ResourceReader;
    reader
        .read(&locator, &resource)
        .expect_err("resource load should fail")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArchiveKind {
    Biff,
    Bif,
    Bifc,
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
        root.push(format!(
            "nearinfinity-{label}-{unique}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temporary installation root should be creatable");
        fs::write(root.join("dialog.tlk"), build_minimal_tlk())
            .expect("dialog.tlk should be writable");

        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn write_archive_install(
        &self,
        relative_archive_path: &str,
        kind: ArchiveKind,
        resource_bytes: &[u8],
    ) {
        let archive_path = self.root.join(relative_archive_path);
        if let Some(parent) = archive_path.parent() {
            fs::create_dir_all(parent).expect("archive parent directory should exist");
        }

        let decompressed_biff = build_biff_archive(resource_bytes);
        let archive_bytes = match kind {
            ArchiveKind::Biff => decompressed_biff,
            ArchiveKind::Bif => build_bif_archive(relative_archive_path, &decompressed_biff),
            ArchiveKind::Bifc => build_bifc_archive(&decompressed_biff),
        };

        fs::write(&archive_path, archive_bytes).expect("archive should be writable");
        fs::write(
            self.root.join("chitin.key"),
            build_key_file(
                "FOO",
                "ITM",
                relative_archive_path,
                relative_archive_path.ends_with(".cbf"),
                ITM_TYPE_CODE,
                RESOURCE_LOCATOR,
            ),
        )
        .expect("chitin.key should be writable");
    }

    fn write_custom_archive_install(
        &self,
        relative_archive_path: &str,
        resource_name: &str,
        kind: ArchiveKind,
        locator: u32,
        decompressed_biff: Vec<u8>,
    ) {
        let archive_path = self.root.join(relative_archive_path);
        if let Some(parent) = archive_path.parent() {
            fs::create_dir_all(parent).expect("archive parent directory should exist");
        }

        let archive_bytes = match kind {
            ArchiveKind::Biff => decompressed_biff,
            ArchiveKind::Bif => build_bif_archive(relative_archive_path, &decompressed_biff),
            ArchiveKind::Bifc => build_bifc_archive(&decompressed_biff),
        };

        let (resref, extension) = resource_name
            .split_once('.')
            .expect("resource_name should include an extension");
        let type_code = match extension {
            "ITM" => ITM_TYPE_CODE,
            "TIS" => TIS_TYPE_CODE,
            other => panic!("unsupported test resource type: {other}"),
        };

        fs::write(&archive_path, archive_bytes).expect("archive should be writable");
        fs::write(
            self.root.join("chitin.key"),
            build_key_file(
                resref,
                extension,
                relative_archive_path,
                relative_archive_path.ends_with(".cbf"),
                type_code,
                locator,
            ),
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

fn build_minimal_tlk() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"TLK V1  ");
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&18u32.to_le_bytes());
    bytes
}

fn build_key_file(
    resref: &str,
    extension: &str,
    relative_archive_path: &str,
    use_cbf_extension: bool,
    type_code: u16,
    locator: u32,
) -> Vec<u8> {
    let archive_name = if use_cbf_extension {
        replace_extension(relative_archive_path, "bif")
    } else {
        relative_archive_path.replace('/', "\\")
    };
    let mut archive_name_bytes = archive_name.into_bytes();
    archive_name_bytes.push(0);

    let bif_count = 1u32;
    let resource_count = 1u32;
    let bif_offset = 24u32;
    let resource_offset = bif_offset + 12;
    let string_offset = resource_offset + 14;

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

    assert_eq!(extension.len(), 3, "test helper expects 3-char extension");
    bytes.extend_from_slice(&padded_resref(resref));
    bytes.extend_from_slice(&type_code.to_le_bytes());
    bytes.extend_from_slice(&locator.to_le_bytes());

    bytes.extend_from_slice(&archive_name_bytes);
    bytes
}

fn build_biff_archive(resource_bytes: &[u8]) -> Vec<u8> {
    build_biff_with_file_entry("FOO", ITM_TYPE_CODE, RESOURCE_LOCATOR, resource_bytes)
}

fn build_biff_with_file_entry(
    _resref: &str,
    type_code: u16,
    locator: u32,
    resource_bytes: &[u8],
) -> Vec<u8> {
    let file_entry_offset = 20u32;
    let resource_offset = 36u32;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BIFFV1  ");
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&file_entry_offset.to_le_bytes());

    bytes.extend_from_slice(&(locator & 0x000F_FFFF).to_le_bytes());
    bytes.extend_from_slice(&resource_offset.to_le_bytes());
    bytes.extend_from_slice(&(resource_bytes.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&type_code.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());

    bytes.extend_from_slice(resource_bytes);
    bytes
}

fn build_biff_with_file_and_tileset_entries(
    file_entry: (u16, u32, &[u8]),
    tileset_entry: (u16, u32, u32, u32, &[u8]),
) -> Vec<u8> {
    let (file_type_code, file_locator, file_bytes) = file_entry;
    let (tileset_type_code, tileset_locator, tile_count, tile_size, tileset_bytes) = tileset_entry;
    let file_entry_offset = 20u32;
    let file_data_offset = 20u32 + 16u32 + 20u32;
    let tileset_data_offset = file_data_offset + file_bytes.len() as u32;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BIFFV1  ");
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&file_entry_offset.to_le_bytes());

    bytes.extend_from_slice(&(file_locator & 0x000F_FFFF).to_le_bytes());
    bytes.extend_from_slice(&file_data_offset.to_le_bytes());
    bytes.extend_from_slice(&(file_bytes.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&file_type_code.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());

    bytes.extend_from_slice(&(tileset_locator & 0x000F_FFFF).to_le_bytes());
    bytes.extend_from_slice(&tileset_data_offset.to_le_bytes());
    bytes.extend_from_slice(&tile_count.to_le_bytes());
    bytes.extend_from_slice(&tile_size.to_le_bytes());
    bytes.extend_from_slice(&tileset_type_code.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());

    bytes.extend_from_slice(file_bytes);
    bytes.extend_from_slice(tileset_bytes);
    bytes
}

fn build_bif_archive(relative_archive_path: &str, decompressed_biff: &[u8]) -> Vec<u8> {
    let mut archive_name = relative_archive_path.replace('/', "\\").into_bytes();
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(decompressed_biff)
        .expect("BIF encoder should accept the archive");
    let compressed = encoder.finish().expect("BIF encoder should finish");

    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BIF V1.0");
    bytes.extend_from_slice(&(archive_name.len() as u32).to_le_bytes());
    bytes.append(&mut archive_name);
    bytes.extend_from_slice(&(decompressed_biff.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&compressed);
    bytes
}

fn build_bifc_archive(decompressed_biff: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BIFCV1.0");
    bytes.extend_from_slice(&(decompressed_biff.len() as u32).to_le_bytes());

    for chunk in decompressed_biff.chunks(12) {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(chunk)
            .expect("BIFC encoder should accept the block");
        let compressed = encoder.finish().expect("BIFC encoder should finish");
        bytes.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&compressed);
    }

    bytes
}

fn padded_resref(resref: &str) -> [u8; 8] {
    let mut bytes = [0u8; 8];
    bytes[..resref.len()].copy_from_slice(resref.as_bytes());
    bytes
}

fn replace_extension(path: &str, extension: &str) -> String {
    match path.rsplit_once('.') {
        Some((base, _)) => format!("{base}.{extension}"),
        None => path.to_string(),
    }
}
