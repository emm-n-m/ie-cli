//! CHR (saved party member) decoding.
//!
//! A CHR is a small header wrapping a complete CRE. The header records where
//! that CRE begins and how long it is, so the CRE is decoded by the existing
//! CRE parser rather than duplicated here.
//!
//! Header layout, measured against real files (see `docs/formats/chr.md`):
//!
//! | offset | size | field |
//! | --- | --- | --- |
//! | 0x00 | 4 | signature `CHR ` |
//! | 0x04 | 4 | version, e.g. `V2.0` |
//! | 0x08 | 32 | character name, NUL-padded |
//! | 0x28 | 4 | offset to the embedded CRE |
//! | 0x2C | 4 | length of the embedded CRE |
//! | 0x30 | .. | quick weapon/spell/item slots, version-specific |
//!
//! Only the fields above are decoded. The quick-slot region differs between
//! `V1.0`, `V2.0`, `V2.2` and `V9.0`, and this crate has no real sample for
//! the versions it cannot reach, so those bytes are preserved raw instead of
//! being given invented names.

use crate::common::signature_mismatch;
use crate::cre::{CreatureJson, parse_cre_with_variant};
use ie_core::{GameVariant, StrRefResolver};
use serde::Serialize;
use thiserror::Error;

/// Through the end of the embedded-CRE length field; the quick-slot region that
/// follows is version-specific and is not required to locate the CRE.
const CHR_MIN_HEADER_SIZE: usize = 0x30;
const CHR_NAME_OFFSET: usize = 0x08;
const CHR_NAME_LENGTH: usize = 32;
const CHR_CRE_OFFSET_FIELD: usize = 0x28;
const CHR_CRE_LENGTH_FIELD: usize = 0x2C;

#[derive(Debug, Clone, Serialize)]
pub struct CharacterJson {
    pub resource_type: String,
    pub resource_name: String,
    pub version: String,
    pub header: CharacterHeaderJson,
    pub creature: CreatureJson,
}

#[derive(Debug, Clone, Serialize)]
pub struct CharacterHeaderJson {
    /// The literal name stored in the CHR. This is a plain string, not a
    /// strref: it is whatever the player typed, so it has no `dialog.tlk`
    /// entry. The embedded CRE carries the strref-backed names.
    pub name: String,
    pub creature_offset: u32,
    pub creature_length: u32,
    /// Quick weapon, spell and item slots. Version-specific and undecoded --
    /// preserved so nothing is silently dropped.
    pub unknown_header_bytes_0x30: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum CharacterParseError {
    #[error("invalid CHR header: {0}")]
    InvalidHeader(String),
    #[error("unexpected end of CHR resource: {0}")]
    UnexpectedEof(String),
}

impl From<CharacterParseError> for crate::FormatError {
    fn from(err: CharacterParseError) -> Self {
        crate::FormatError::Parse(err.to_string())
    }
}

pub(crate) fn parse_chr(
    bytes: &[u8],
    resource_name: &str,
    resolver: Option<&dyn StrRefResolver>,
    game_variant: GameVariant,
) -> Result<CharacterJson, crate::FormatError> {
    if bytes.len() < CHR_MIN_HEADER_SIZE {
        return Err(CharacterParseError::UnexpectedEof(format!(
            "CHR resource must contain at least {CHR_MIN_HEADER_SIZE} bytes, found {}",
            bytes.len()
        ))
        .into());
    }

    if &bytes[0..4] != b"CHR " {
        return Err(CharacterParseError::InvalidHeader(signature_mismatch(
            "CHR",
            b"CHR ",
            &bytes[0..4],
        ))
        .into());
    }

    let version = ascii_string(&bytes[4..8]);
    let name = ascii_string(&bytes[CHR_NAME_OFFSET..CHR_NAME_OFFSET + CHR_NAME_LENGTH]);
    let creature_offset = u32_at(bytes, CHR_CRE_OFFSET_FIELD);
    let creature_length = u32_at(bytes, CHR_CRE_LENGTH_FIELD);

    let (start, end) = embedded_cre_range(bytes)?;
    let creature =
        parse_cre_with_variant(&bytes[start..end], resource_name, resolver, game_variant)?;

    Ok(CharacterJson {
        resource_type: "CHR".to_string(),
        resource_name: resource_name.to_string(),
        version,
        header: CharacterHeaderJson {
            name,
            creature_offset,
            creature_length,
            unknown_header_bytes_0x30: bytes[CHR_MIN_HEADER_SIZE..start].to_vec(),
        },
        creature,
    })
}

/// Applies CRE scalar patches to the CRE embedded in a CHR.
///
/// The CLI has always advertised `CRE/CHR` patching, but a CHR was routed to
/// the CRE patcher whole and rejected on the signature check. Scalar edits are
/// fixed-width and in-place, so the patched CRE splices back at the same offset
/// and every header offset in the CHR stays valid.
pub fn patch_chr_scalars(
    bytes: &[u8],
    patches: &[crate::cre::CreatureScalarPatch],
) -> Result<Vec<u8>, crate::FormatError> {
    let (start, end) = embedded_cre_range(bytes)?;
    let patched = crate::cre::patch_cre_scalars(&bytes[start..end], patches)
        .map_err(|err| crate::FormatError::Parse(err.to_string()))?;

    if patched.len() != end - start {
        return Err(CharacterParseError::InvalidHeader(format!(
            "patched CRE changed length from {} to {}; CHR offsets would be invalidated",
            end - start,
            patched.len()
        ))
        .into());
    }

    let mut output = bytes.to_vec();
    output[start..end].copy_from_slice(&patched);
    Ok(output)
}

/// Validated byte range of the CRE embedded in `bytes`.
fn embedded_cre_range(bytes: &[u8]) -> Result<(usize, usize), CharacterParseError> {
    if bytes.len() < CHR_MIN_HEADER_SIZE {
        return Err(CharacterParseError::UnexpectedEof(format!(
            "CHR resource must contain at least {CHR_MIN_HEADER_SIZE} bytes, found {}",
            bytes.len()
        )));
    }

    if &bytes[0..4] != b"CHR " {
        return Err(CharacterParseError::InvalidHeader(signature_mismatch(
            "CHR",
            b"CHR ",
            &bytes[0..4],
        )));
    }

    let creature_offset = u32_at(bytes, CHR_CRE_OFFSET_FIELD);
    let creature_length = u32_at(bytes, CHR_CRE_LENGTH_FIELD);

    let start = creature_offset as usize;
    let end = start.checked_add(creature_length as usize).ok_or_else(|| {
        CharacterParseError::InvalidHeader(format!(
            "embedded CRE range overflows: offset {creature_offset} length {creature_length}"
        ))
    })?;

    if start < CHR_MIN_HEADER_SIZE {
        return Err(CharacterParseError::InvalidHeader(format!(
            "embedded CRE offset {creature_offset} overlaps the CHR header"
        )));
    }

    if end > bytes.len() {
        return Err(CharacterParseError::UnexpectedEof(format!(
            "embedded CRE range {start}..{end} exceeds the {} byte CHR resource",
            bytes.len()
        )));
    }

    Ok((start, end))
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

/// Trailing NUL padding is stripped; the engine writes fixed-width fields.
fn ascii_string(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end])
        .trim_end()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cre::CreatureScalarPatch;
    use ie_core::StrRef;

    struct NullResolver;

    impl StrRefResolver for NullResolver {
        fn resolve_strref(&self, _strref: StrRef) -> Option<String> {
            None
        }
    }

    /// Header shape taken from real BGEE files: name at 0x08, CRE at 0x64.
    fn build_chr(version: &[u8; 4], name: &str, cre: &[u8]) -> Vec<u8> {
        build_chr_at(version, name, cre, 0x64)
    }

    /// Same, with the embedded CRE placed wherever the caller asks.
    ///
    /// The quick-slot region between 0x30 and the CRE is version-sized, so the
    /// offset is not a constant across CHR versions. Only V2.0 files were
    /// available to sample; this lets a test state an offset the sample does not
    /// have.
    fn build_chr_at(version: &[u8; 4], name: &str, cre: &[u8], cre_offset: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; cre_offset];
        bytes[0..4].copy_from_slice(b"CHR ");
        bytes[4..8].copy_from_slice(version);
        bytes[CHR_NAME_OFFSET..CHR_NAME_OFFSET + name.len()].copy_from_slice(name.as_bytes());
        bytes[CHR_CRE_OFFSET_FIELD..CHR_CRE_OFFSET_FIELD + 4]
            .copy_from_slice(&(cre_offset as u32).to_le_bytes());
        bytes[CHR_CRE_LENGTH_FIELD..CHR_CRE_LENGTH_FIELD + 4]
            .copy_from_slice(&(cre.len() as u32).to_le_bytes());
        bytes.extend_from_slice(cre);
        bytes
    }

    fn minimal_cre() -> Vec<u8> {
        let mut bytes = vec![0u8; 0x2D4];
        bytes[0..4].copy_from_slice(b"CRE ");
        bytes[4..8].copy_from_slice(b"V1.0");
        bytes[0x44] = 10;
        bytes
    }

    #[test]
    fn parses_header_and_embedded_creature() {
        let bytes = build_chr(b"V2.0", "Abdel", &minimal_cre());
        let character = parse_chr(
            &bytes,
            "01FIGHT.CHR",
            Some(&NullResolver),
            GameVariant::Standard,
        )
        .expect("CHR should parse");

        assert_eq!(character.resource_type, "CHR");
        assert_eq!(character.version, "V2.0");
        assert_eq!(character.header.name, "Abdel");
        assert_eq!(character.header.creature_offset, 0x64);
        assert_eq!(character.header.creature_length, 0x2D4);
        assert_eq!(character.creature.resource_type, "CRE");
        // The quick-slot region is preserved rather than dropped.
        assert_eq!(
            character.header.unknown_header_bytes_0x30.len(),
            0x64 - CHR_MIN_HEADER_SIZE
        );
    }

    /// The bug this module exists to fix: a CHR routed to the CRE parser failed
    /// on the signature check, so every CHR in every install was undecodable.
    #[test]
    fn embedded_creature_is_read_at_the_recorded_offset_not_from_byte_zero() {
        let bytes = build_chr(b"V2.0", "Abdel", &minimal_cre());
        assert_eq!(&bytes[0..4], b"CHR ");
        assert!(parse_cre_with_variant(&bytes, "X.CHR", None, GameVariant::Standard).is_err());
        assert!(parse_chr(&bytes, "X.CHR", None, GameVariant::Standard).is_ok());
    }

    /// Every other fixture here puts the CRE at 0x64, the offset the sampled
    /// V2.0 files happen to use, so they cannot tell reading the field apart
    /// from assuming that constant. V2.2 grows the quick-slot region and moves
    /// the CRE to 0x80; this fails if the offset is ever hardcoded.
    #[test]
    fn embedded_creature_offset_is_read_rather_than_assumed() {
        let bytes = build_chr_at(b"V2.2", "Abdel", &minimal_cre(), 0x80);
        let character = parse_chr(&bytes, "X.CHR", None, GameVariant::Standard)
            .expect("a CHR whose CRE sits at 0x80 should parse");

        assert_eq!(character.header.creature_offset, 0x80);
        assert_eq!(character.creature.resource_type, "CRE");
        assert_eq!(
            character.header.unknown_header_bytes_0x30.len(),
            0x80 - CHR_MIN_HEADER_SIZE
        );
    }

    #[test]
    fn rejects_wrong_signature() {
        let mut bytes = build_chr(b"V2.0", "Abdel", &minimal_cre());
        bytes[0..4].copy_from_slice(b"CRE ");
        assert!(parse_chr(&bytes, "X.CHR", None, GameVariant::Standard).is_err());
    }

    #[test]
    fn rejects_creature_offset_inside_the_header() {
        let mut bytes = build_chr(b"V2.0", "Abdel", &minimal_cre());
        bytes[CHR_CRE_OFFSET_FIELD..CHR_CRE_OFFSET_FIELD + 4].copy_from_slice(&4u32.to_le_bytes());
        assert!(parse_chr(&bytes, "X.CHR", None, GameVariant::Standard).is_err());
    }

    #[test]
    fn rejects_creature_range_past_end_of_resource() {
        let mut bytes = build_chr(b"V2.0", "Abdel", &minimal_cre());
        bytes[CHR_CRE_LENGTH_FIELD..CHR_CRE_LENGTH_FIELD + 4]
            .copy_from_slice(&0xFFFF_u32.to_le_bytes());
        assert!(parse_chr(&bytes, "X.CHR", None, GameVariant::Standard).is_err());
    }

    #[test]
    fn rejects_creature_range_that_overflows() {
        let mut bytes = build_chr(b"V2.0", "Abdel", &minimal_cre());
        bytes[CHR_CRE_OFFSET_FIELD..CHR_CRE_OFFSET_FIELD + 4]
            .copy_from_slice(&u32::MAX.to_le_bytes());
        bytes[CHR_CRE_LENGTH_FIELD..CHR_CRE_LENGTH_FIELD + 4]
            .copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(parse_chr(&bytes, "X.CHR", None, GameVariant::Standard).is_err());
    }

    /// A scalar patch must land inside the embedded CRE and leave every CHR
    /// header offset valid, so the file length may not change.
    #[test]
    fn patches_the_embedded_creature_in_place() {
        let bytes = build_chr(b"V2.0", "Abdel", &minimal_cre());
        let patched = patch_chr_scalars(
            &bytes,
            &[CreatureScalarPatch {
                field: "reputation".to_string(),
                value: "15".to_string(),
            }],
        )
        .expect("CHR patch should apply");

        assert_eq!(patched.len(), bytes.len());
        assert_eq!(
            &patched[0..CHR_MIN_HEADER_SIZE],
            &bytes[0..CHR_MIN_HEADER_SIZE]
        );
        assert_eq!(patched[0x64 + 0x44], 15);
        assert_eq!(bytes[0x64 + 0x44], 10);
    }
}
