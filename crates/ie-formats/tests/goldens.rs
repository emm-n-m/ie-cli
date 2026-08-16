//! Value goldens for decoded JSON, pinned against synthetic fixtures.
//!
//! These are the half of the golden story that runs in CI. The fixtures are
//! built here byte by byte, so no game data is committed and no install is
//! needed -- which also means the expected JSON can be pinned exactly, values
//! included, rather than only by shape.
//!
//! That exactness is the point. The real-install goldens in `ie-cli`'s
//! `shape.rs` assert only that an install produces no *unknown* path, because a
//! modded install and a sampled sweep can legitimately omit paths. Omission is
//! therefore invisible there: delete a field and that test still passes. Here a
//! deleted, renamed, re-nested, or reordered field changes the file and fails.
//! The two together cover both directions.
//!
//! Fixtures are deliberately sparse. A golden's job is to pin the structure and
//! the encoding of each field, not to re-test parsing -- the format modules'
//! own unit tests do that against richer inputs. Zeroed fields still serialize,
//! so a sparse fixture pins just as many names as a dense one.
//!
//! Regenerate with `UPDATE_GOLDENS=1`, then read the diff before committing it:
//! a golden that is updated without being read is worse than no golden at all.

use ie_core::{
    GameVariant, ResolverBundle, ResourceBytes, ResourceMetadata, ResourceType, SourceKind, StrRef,
    StrRefResolver,
};
use ie_formats::decode_to_json;
use std::path::{Path, PathBuf};

/// Resolves every strref to a deterministic marker.
///
/// A real TLK would make the golden depend on the install's language and patch
/// level. This keeps the resolved-string path exercised -- so a change in how
/// resolved text is nested still fails the golden -- while staying reproducible
/// on any machine.
struct MarkerResolver;

impl StrRefResolver for MarkerResolver {
    fn resolve_strref(&self, strref: StrRef) -> Option<String> {
        Some(format!("<string {}>", strref.0))
    }
}

#[test]
fn itm_json_matches_golden() {
    assert_golden("itm", ResourceType::Itm, "SW1H01.ITM", &minimal_itm());
}

#[test]
fn spl_json_matches_golden() {
    assert_golden("spl", ResourceType::Spl, "SPWI112.SPL", &minimal_spl());
}

#[test]
fn cre_json_matches_golden() {
    assert_golden("cre", ResourceType::Cre, "GORION.CRE", &minimal_cre());
}

#[test]
fn chr_json_matches_golden() {
    // CHR nests a whole CRE under a wrapper. That nesting is exactly the kind of
    // structure a refactor can flatten by accident, and it is new enough to have
    // no other shape coverage: `list` does not enumerate `characters/`, so the
    // real-install shape goldens never see a CHR.
    assert_golden("chr", ResourceType::Chr, "01FIGHT.CHR", &minimal_chr());
}

fn assert_golden(name: &str, resource_type: ResourceType, resource_name: &str, bytes: &[u8]) {
    let resource = ResourceBytes {
        metadata: ResourceMetadata {
            // A fixed placeholder: the real path is machine-specific, and the
            // decoders do not read it.
            source_path: PathBuf::from("<fixture>"),
            source_kind: SourceKind::Override,
            resource_type,
            resource_name: resource_name.to_string(),
            game_variant: GameVariant::Standard,
        },
        bytes: bytes.to_vec(),
    };

    let value = decode_to_json(
        &resource,
        ResolverBundle {
            strref: Some(&MarkerResolver),
            ids: None,
            links: None,
        },
    )
    .unwrap_or_else(|error| panic!("{name} fixture should decode: {error}"));

    let actual = format!(
        "{}\n",
        serde_json::to_string_pretty(&value).expect("decoded JSON should serialize")
    );
    let path = golden_path(name);

    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        std::fs::create_dir_all(path.parent().expect("golden path should have a parent"))
            .expect("golden directory should be creatable");
        std::fs::write(&path, &actual).expect("golden should be writable");
        return;
    }

    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "no golden at {}; regenerate with UPDATE_GOLDENS=1",
            path.display()
        )
    });
    // `.gitattributes` pins these files to LF, but a checkout that ignores it
    // would otherwise fail all four goldens on Windows and nowhere else -- a
    // whole-file comparison sees CRLF as a difference on every single line. The
    // line ending is not part of what a golden is pinning.
    let expected = expected.replace("\r\n", "\n");

    assert_eq!(
        actual,
        expected,
        "decoded {name} JSON no longer matches {}. \
         Anything reading this output by field name breaks on this change. \
         If it is intended, regenerate with UPDATE_GOLDENS=1 and review the diff.",
        path.display()
    );
}

fn golden_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens")
        .join(format!("{name}.json"))
}

/// Header sizes are the parsers' own constants, restated here so a fixture stays
/// readable; a mismatch surfaces immediately as a parse failure.
const ITM_HEADER_SIZE: usize = 0x72;
const SPL_HEADER_SIZE: usize = 0x72;
const CRE_HEADER_SIZE: usize = 0x2D4;

fn minimal_itm() -> Vec<u8> {
    let mut bytes = vec![0u8; ITM_HEADER_SIZE];
    bytes[0..4].copy_from_slice(b"ITM ");
    bytes[4..8].copy_from_slice(b"V1  ");
    bytes[0x08..0x0C].copy_from_slice(&1u32.to_le_bytes()); // unidentified name
    bytes[0x0C..0x10].copy_from_slice(&2u32.to_le_bytes()); // identified name
    bytes[0x18..0x1C].copy_from_slice(&0x0000_0040u32.to_le_bytes()); // flags: Magical
    bytes[0x1C..0x1E].copy_from_slice(&0x0010u16.to_le_bytes()); // category
    bytes[0x34..0x38].copy_from_slice(&123u32.to_le_bytes()); // price
    bytes[0x3A..0x42].copy_from_slice(b"ICON\0\0\0\0");
    bytes[0x44..0x4C].copy_from_slice(b"GRND\0\0\0\0");
    bytes[0x4C..0x50].copy_from_slice(&456u32.to_le_bytes()); // weight
    bytes[0x64..0x68].copy_from_slice(&(ITM_HEADER_SIZE as u32).to_le_bytes());
    bytes
}

fn minimal_spl() -> Vec<u8> {
    let mut bytes = vec![0u8; SPL_HEADER_SIZE];
    bytes[0..4].copy_from_slice(b"SPL ");
    bytes[4..8].copy_from_slice(b"V1  ");
    bytes[0x08..0x0C].copy_from_slice(&1u32.to_le_bytes()); // name
    bytes[0x10..0x18].copy_from_slice(b"SOUND\0\0\0");
    bytes[0x18..0x1C].copy_from_slice(&0x0000_0600u32.to_le_bytes()); // flags
    bytes[0x1C..0x1E].copy_from_slice(&1u16.to_le_bytes()); // spell type
    bytes[0x1E..0x22].copy_from_slice(&0x0000_0800u32.to_le_bytes()); // exclusion
    bytes[0x25] = 6; // school
    bytes[0x27] = 10; // secondary type
    bytes[0x34..0x38].copy_from_slice(&1u32.to_le_bytes()); // spell level
    bytes[0x3A..0x42].copy_from_slice(b"ICON\0\0\0\0");
    bytes[0x64..0x68].copy_from_slice(&(SPL_HEADER_SIZE as u32).to_le_bytes());
    bytes
}

fn minimal_cre() -> Vec<u8> {
    let mut bytes = vec![0u8; CRE_HEADER_SIZE];
    bytes[0..4].copy_from_slice(b"CRE ");
    bytes[4..8].copy_from_slice(b"V1.0");
    bytes[0x08..0x0C].copy_from_slice(&1u32.to_le_bytes()); // long name
    bytes[0x0C..0x10].copy_from_slice(&2u32.to_le_bytes()); // short name
    bytes[0x34..0x3C].copy_from_slice(b"SMALL\0\0\0");
    bytes[0x3C..0x44].copy_from_slice(b"LARGE\0\0\0");
    bytes[0x44] = 10; // morale
    bytes[0x45] = 15; // morale break
    bytes[0x84..0x8A].copy_from_slice(b"GORION");
    bytes[0x238] = 18; // strength
    bytes[0x23A] = 10; // intelligence
    bytes[0x248..0x250].copy_from_slice(b"OVERRIDE");
    bytes
}

fn minimal_chr() -> Vec<u8> {
    const CRE_OFFSET: usize = 0x64;
    let cre = minimal_cre();
    let mut bytes = vec![0u8; CRE_OFFSET];
    bytes[0..4].copy_from_slice(b"CHR ");
    bytes[4..8].copy_from_slice(b"V2.0");
    bytes[0x08..0x0D].copy_from_slice(b"Abdel");
    bytes[0x28..0x2C].copy_from_slice(&(CRE_OFFSET as u32).to_le_bytes());
    bytes[0x2C..0x30].copy_from_slice(&(cre.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&cre);
    bytes
}
