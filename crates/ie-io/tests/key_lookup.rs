use ie_core::{ResourceName, ResourceType, SourceKind};
use ie_io::{GameInstallation, KeyFile, ResourceLocator, ResourceReader};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const ITM_TYPE_CODE: u16 = 0x03ED;
const SPL_TYPE_CODE: u16 = 0x03EE;
const CRE_TYPE_CODE: u16 = 0x03F1;

#[test]
fn parses_key_file_into_typed_model() {
    let fixture = TestInstallation::new("typed-key");
    fixture.write_archive(
        "data/mixed.bif",
        &[
            ("FOO", ITM_TYPE_CODE, 0x0000_0001, b"ITM DATA".as_slice()),
            ("SPWI112", SPL_TYPE_CODE, 0x0000_0002, b"SPL DATA".as_slice()),
            ("IMOEN2", CRE_TYPE_CODE, 0x0000_0003, b"CRE DATA".as_slice()),
        ],
    );

    let installation =
        GameInstallation::discover(fixture.root()).expect("synthetic installation should be valid");
    let key = KeyFile::parse(&installation).expect("KEY parsing should succeed");

    assert_eq!(key.header.signature, "KEY ");
    assert_eq!(key.header.version, "V1  ");
    assert_eq!(key.header.bif_count, 1);
    assert_eq!(key.header.resource_count, 3);
    assert_eq!(key.biffs.len(), 1);
    assert_eq!(key.biffs[0].relative_path, "data/mixed.bif");
    assert_eq!(key.resources.len(), 3);

    let spell = key
        .resource(&ResourceName::parse("spwi112.spl").expect("resource name should parse"))
        .expect("SPL entry should be indexed");
    assert_eq!(spell.resource_name, "SPWI112.SPL");
    assert_eq!(spell.type_code, SPL_TYPE_CODE);
    assert_eq!(spell.resource_type, ResourceType::Spl);
    assert_eq!(spell.biff_index, 0);
    assert_eq!(spell.resource_index, 0x0000_0002);
}

#[test]
fn locates_and_reads_multiple_resource_families_from_key() {
    let fixture = TestInstallation::new("multi-family-key");
    fixture.write_archive(
        "data/mixed.bif",
        &[
            ("FOO", ITM_TYPE_CODE, 0x0000_0001, b"ITM DATA".as_slice()),
            ("SPWI112", SPL_TYPE_CODE, 0x0000_0002, b"SPL DATA".as_slice()),
            ("IMOEN2", CRE_TYPE_CODE, 0x0000_0003, b"CRE DATA".as_slice()),
        ],
    );

    let installation =
        GameInstallation::discover(fixture.root()).expect("synthetic installation should be valid");
    let locator = ResourceLocator::new(installation).expect("KEY parsing should succeed");
    let reader = ResourceReader;

    let item = reader
        .read(
            &locator,
            &ResourceName::parse("foo.itm").expect("resource name should parse"),
        )
        .expect("ITM should be readable");
    assert_eq!(item.metadata.source_kind, SourceKind::Bif);
    assert_eq!(item.metadata.resource_type, ResourceType::Itm);
    assert_eq!(item.bytes, b"ITM DATA");

    let spell = reader
        .read(
            &locator,
            &ResourceName::parse("spwi112.spl").expect("resource name should parse"),
        )
        .expect("SPL should be readable");
    assert_eq!(spell.metadata.source_kind, SourceKind::Bif);
    assert_eq!(spell.metadata.resource_type, ResourceType::Spl);
    assert_eq!(spell.bytes, b"SPL DATA");

    let creature = reader
        .read(
            &locator,
            &ResourceName::parse("imoen2.cre").expect("resource name should parse"),
        )
        .expect("CRE should be readable");
    assert_eq!(creature.metadata.source_kind, SourceKind::Bif);
    assert_eq!(creature.metadata.resource_type, ResourceType::Cre);
    assert_eq!(creature.bytes, b"CRE DATA");
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
            "nearinfinity-key-{label}-{unique}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temporary installation root should be creatable");

        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn write_archive(&self, relative_archive_path: &str, entries: &[ArchiveResource<'_>]) {
        let archive_path = self.root.join(relative_archive_path);
        if let Some(parent) = archive_path.parent() {
            fs::create_dir_all(parent).expect("archive parent directory should exist");
        }

        fs::write(&archive_path, build_biff_archive(entries)).expect("archive should be writable");
        fs::write(
            self.root.join("chitin.key"),
            build_key_file(relative_archive_path, entries),
        )
        .expect("chitin.key should be writable");
    }
}

impl Drop for TestInstallation {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

type ArchiveResource<'a> = (&'a str, u16, u32, &'a [u8]);

fn build_key_file(relative_archive_path: &str, entries: &[ArchiveResource<'_>]) -> Vec<u8> {
    let mut archive_name_bytes = relative_archive_path.replace('/', "\\").into_bytes();
    archive_name_bytes.push(0);

    let bif_count = 1u32;
    let resource_count = entries.len() as u32;
    let bif_offset = 24u32;
    let resource_offset = bif_offset + 12;
    let string_offset = resource_offset + (resource_count * 14);

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

    for (resref, type_code, locator, _) in entries {
        bytes.extend_from_slice(&padded_resref(resref));
        bytes.extend_from_slice(&type_code.to_le_bytes());
        bytes.extend_from_slice(&locator.to_le_bytes());
    }

    bytes.extend_from_slice(&archive_name_bytes);
    bytes
}

fn build_biff_archive(entries: &[ArchiveResource<'_>]) -> Vec<u8> {
    let entry_offset = 20u32;
    let data_start = entry_offset + (entries.len() as u32 * 16);
    let mut data_offset = data_start;

    let mut directory = Vec::new();
    let mut payload = Vec::new();

    for (_, type_code, locator, bytes) in entries {
        directory.extend_from_slice(&(locator & 0x000F_FFFF).to_le_bytes());
        directory.extend_from_slice(&data_offset.to_le_bytes());
        directory.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        directory.extend_from_slice(&type_code.to_le_bytes());
        directory.extend_from_slice(&0u16.to_le_bytes());

        payload.extend_from_slice(bytes);
        data_offset += bytes.len() as u32;
    }

    let mut archive = Vec::new();
    archive.extend_from_slice(b"BIFFV1  ");
    archive.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    archive.extend_from_slice(&0u32.to_le_bytes());
    archive.extend_from_slice(&entry_offset.to_le_bytes());
    archive.extend_from_slice(&directory);
    archive.extend_from_slice(&payload);
    archive
}

fn padded_resref(resref: &str) -> [u8; 8] {
    let mut bytes = [0u8; 8];
    bytes[..resref.len()].copy_from_slice(resref.as_bytes());
    bytes
}
