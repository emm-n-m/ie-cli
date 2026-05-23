use flate2::read::ZlibDecoder;
use ie_core::{ResRef, ResolvedStrRef, StrRef, StrRefResolver};
use serde::Serialize;
use std::io::Read;
use thiserror::Error;

const GAM_V2_HEADER_SIZE: usize = 0xB4;
const GAM_V2_NPC_SIZE: usize = 0x160;
const GAM_V2_VARIABLE_SIZE: usize = 0x54;
const SAV_HEADER_SIZE: usize = 0x08;

#[derive(Debug, Clone, Serialize)]
pub struct GameStateJson {
    pub resource_type: String,
    pub resource_name: String,
    pub version: String,
    pub header: GameStateHeaderJson,
    pub party_members: Vec<GameNpcJson>,
    pub globals: Vec<GameVariableJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameStateHeaderJson {
    pub game_time: u32,
    pub game_time_real_seconds: u32,
    pub party_gold: u32,
    pub party_reputation_raw: u32,
    pub party_reputation: f32,
    pub active_area_party_member: i16,
    pub main_area: Option<ResRef>,
    pub current_area: Option<ResRef>,
    pub party_members_offset: u32,
    pub party_members_count: u32,
    pub non_party_members_offset: u32,
    pub non_party_members_count: u32,
    pub globals_offset: u32,
    pub globals_count: u32,
    pub journal_entries_offset: u32,
    pub journal_entries_count: u32,
    pub chapter: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameNpcJson {
    pub index: usize,
    pub character_selection: RawDecoded<u16>,
    pub party_order: RawDecoded<u16>,
    pub embedded_cre_offset: u32,
    pub embedded_cre_size: u32,
    pub character_name: String,
    pub character_name_long: String,
    pub orientation: u32,
    pub current_area: Option<ResRef>,
    pub position: GamePointJson,
    pub viewing_position: GamePointJson,
    pub modal_action: u16,
    pub happiness: u16,
    pub talk_count: u32,
    pub stats: GameNpcStatsJson,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameNpcStatsJson {
    pub most_powerful_vanquished_name: ResolvedStrRef,
    pub most_powerful_vanquished_xp: u32,
    pub time_in_party_ticks: u32,
    pub time_joined_ticks: u32,
    pub in_party: bool,
    pub first_cre_resref_letter: u8,
    pub kills_chapter_xp: u32,
    pub kills_chapter_count: u32,
    pub kills_game_xp: u32,
    pub kills_game_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GamePointJson {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameVariableJson {
    pub index: usize,
    pub name: String,
    pub type_flags: RawDecodedFlags,
    pub ref_value: u16,
    pub dword_value: u32,
    pub int_value: i32,
    pub double_value: f64,
    pub script_name_value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveArchiveJson {
    pub resource_type: String,
    pub resource_name: String,
    pub version: String,
    pub entries: Vec<SaveArchiveEntryJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveArchiveEntryJson {
    pub index: usize,
    pub filename: String,
    pub resource_name: Option<String>,
    pub resource_type: Option<String>,
    pub uncompressed_size: u32,
    pub compressed_size: u32,
    pub inflated_size: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RawDecoded<T>
where
    T: Serialize,
{
    pub raw: T,
    pub decoded: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RawDecodedFlags {
    pub raw: u16,
    pub decoded: Vec<String>,
}

#[derive(Debug, Error)]
pub enum SaveParseError {
    #[error("invalid save header: {0}")]
    InvalidHeader(String),
    #[error("unexpected end of save file: {0}")]
    UnexpectedEof(String),
    #[error("invalid save field: {0}")]
    InvalidField(String),
    #[error("SAV decompression failed for {filename}: {message}")]
    Decompression { filename: String, message: String },
}

impl From<SaveParseError> for crate::FormatError {
    fn from(err: SaveParseError) -> Self {
        crate::FormatError::Parse(err.to_string())
    }
}

pub fn parse_gam(
    bytes: &[u8],
    resource_name: &str,
    strrefs: Option<&dyn StrRefResolver>,
) -> Result<GameStateJson, crate::FormatError> {
    if bytes.len() < GAM_V2_HEADER_SIZE {
        return Err(SaveParseError::UnexpectedEof(format!(
            "GAM resource must contain at least {} bytes",
            GAM_V2_HEADER_SIZE
        ))
        .into());
    }

    if &bytes[0..4] != b"GAME" {
        return Err(SaveParseError::InvalidHeader("missing GAME signature".to_string()).into());
    }

    let version = parse_ascii_string(bytes, 4, 4)?;
    if !matches!(version.as_str(), "V2.0" | "V2.1" | "V2.2") {
        return Err(SaveParseError::InvalidHeader(format!(
            "unsupported GAM version {version}"
        ))
        .into());
    }

    let party_members_offset = parse_u32(bytes, 0x20)?;
    let party_members_count = parse_u32(bytes, 0x24)?;
    let globals_offset = parse_u32(bytes, 0x38)?;
    let globals_count = parse_u32(bytes, 0x3C)?;
    let journal_entries_count = parse_u32(bytes, 0x4C)?;
    let journal_entries_offset = parse_u32(bytes, 0x50)?;
    let globals = parse_globals(bytes, globals_offset, globals_count)?;
    let chapter = globals
        .iter()
        .find(|global| global.name.eq_ignore_ascii_case("CHAPTER"))
        .map(|global| global.int_value);

    Ok(GameStateJson {
        resource_type: "GAM".to_string(),
        resource_name: resource_name.to_string(),
        version,
        header: GameStateHeaderJson {
            game_time: parse_u32(bytes, 0x08)?,
            game_time_real_seconds: parse_u32(bytes, 0x74)?,
            party_gold: parse_u32(bytes, 0x18)?,
            party_reputation_raw: parse_u32(bytes, 0x54)?,
            party_reputation: parse_u32(bytes, 0x54)? as f32 / 10.0,
            active_area_party_member: parse_i16(bytes, 0x1C)?,
            main_area: parse_resref_option(bytes, 0x40, 8)?,
            current_area: parse_resref_option(bytes, 0x58, 8)?,
            party_members_offset,
            party_members_count,
            non_party_members_offset: parse_u32(bytes, 0x30)?,
            non_party_members_count: parse_u32(bytes, 0x34)?,
            globals_offset,
            globals_count,
            journal_entries_offset,
            journal_entries_count,
            chapter,
        },
        party_members: parse_npcs(bytes, party_members_offset, party_members_count, strrefs)?,
        globals,
    })
}

pub fn parse_sav(bytes: &[u8], resource_name: &str) -> Result<SaveArchiveJson, crate::FormatError> {
    if bytes.len() < SAV_HEADER_SIZE {
        return Err(SaveParseError::UnexpectedEof(
            "SAV resource must contain at least 8 bytes".to_string(),
        )
        .into());
    }

    if &bytes[0..4] != b"SAV " {
        return Err(SaveParseError::InvalidHeader("missing SAV signature".to_string()).into());
    }

    let version = parse_ascii_string(bytes, 4, 4)?;
    if version != "V1.0" {
        return Err(SaveParseError::InvalidHeader(format!(
            "unsupported SAV version {version}"
        ))
        .into());
    }

    let mut offset = SAV_HEADER_SIZE;
    let mut entries = Vec::new();
    while offset < bytes.len() {
        let filename_len = parse_u32(bytes, offset)? as usize;
        offset = offset
            .checked_add(4)
            .ok_or_else(|| SaveParseError::InvalidField("SAV offset overflow".to_string()))?;
        if filename_len == 0 {
            return Err(SaveParseError::InvalidField(
                "SAV entry has zero-length filename".to_string(),
            )
            .into());
        }
        let filename_end = checked_end(offset, filename_len, bytes.len(), "SAV filename")?;
        let filename_bytes = &bytes[offset..filename_end];
        let filename = parse_nul_terminated_string(filename_bytes);
        offset = filename_end;

        let uncompressed_size = parse_u32(bytes, offset)?;
        let compressed_size = parse_u32(bytes, offset + 4)?;
        offset = offset
            .checked_add(8)
            .ok_or_else(|| SaveParseError::InvalidField("SAV offset overflow".to_string()))?;
        let compressed_end =
            checked_end(offset, compressed_size as usize, bytes.len(), "SAV compressed data")?;
        let inflated = inflate_sav_entry(&filename, &bytes[offset..compressed_end])?;
        if inflated.len() != uncompressed_size as usize {
            return Err(SaveParseError::InvalidField(format!(
                "SAV entry {filename} inflated to {} bytes, expected {uncompressed_size}",
                inflated.len()
            ))
            .into());
        }
        offset = compressed_end;

        let (resource_name, resource_type) = filename
            .rsplit_once('.')
            .map(|(_, ext)| (Some(filename.to_ascii_uppercase()), Some(ext.to_ascii_uppercase())))
            .unwrap_or((None, None));

        entries.push(SaveArchiveEntryJson {
            index: entries.len(),
            filename,
            resource_name,
            resource_type,
            uncompressed_size,
            compressed_size,
            inflated_size: inflated.len(),
        });
    }

    Ok(SaveArchiveJson {
        resource_type: "SAV".to_string(),
        resource_name: resource_name.to_string(),
        version,
        entries,
    })
}

fn parse_npcs(
    bytes: &[u8],
    offset: u32,
    count: u32,
    strrefs: Option<&dyn StrRefResolver>,
) -> Result<Vec<GameNpcJson>, SaveParseError> {
    let offset = offset as usize;
    let count = count as usize;
    let table_end = checked_table_end(offset, count, GAM_V2_NPC_SIZE, bytes.len(), "GAM NPCs")?;
    let mut npcs = Vec::with_capacity(count);

    for index in 0..count {
        let base = offset + index * GAM_V2_NPC_SIZE;
        debug_assert!(base + GAM_V2_NPC_SIZE <= table_end);
        let stats = base + 0xE4;

        npcs.push(GameNpcJson {
            index,
            character_selection: RawDecoded {
                raw: parse_u16(bytes, base)?,
                decoded: decode_character_selection(parse_u16(bytes, base)?),
            },
            party_order: RawDecoded {
                raw: parse_u16(bytes, base + 0x02)?,
                decoded: decode_party_order(parse_u16(bytes, base + 0x02)?),
            },
            embedded_cre_offset: parse_u32(bytes, base + 0x04)?,
            embedded_cre_size: parse_u32(bytes, base + 0x08)?,
            character_name: parse_ascii_string(bytes, base + 0x0C, 8)?,
            character_name_long: parse_ascii_string(bytes, base + 0xC0, 32)?,
            orientation: parse_u32(bytes, base + 0x14)?,
            current_area: parse_resref_option(bytes, base + 0x18, 8)?,
            position: GamePointJson {
                x: parse_u16(bytes, base + 0x20)?,
                y: parse_u16(bytes, base + 0x22)?,
            },
            viewing_position: GamePointJson {
                x: parse_u16(bytes, base + 0x24)?,
                y: parse_u16(bytes, base + 0x26)?,
            },
            modal_action: parse_u16(bytes, base + 0x28)?,
            happiness: parse_u16(bytes, base + 0x2A)?,
            talk_count: parse_u32(bytes, base + 0xE0)?,
            stats: GameNpcStatsJson {
                most_powerful_vanquished_name: resolve_strref(strrefs, parse_u32(bytes, stats)?),
                most_powerful_vanquished_xp: parse_u32(bytes, stats + 0x04)?,
                time_in_party_ticks: parse_u32(bytes, stats + 0x08)?,
                time_joined_ticks: parse_u32(bytes, stats + 0x0C)?,
                in_party: bytes.get(stats + 0x10).copied().unwrap_or_default() != 0,
                first_cre_resref_letter: bytes.get(stats + 0x13).copied().unwrap_or_default(),
                kills_chapter_xp: parse_u32(bytes, stats + 0x14)?,
                kills_chapter_count: parse_u32(bytes, stats + 0x18)?,
                kills_game_xp: parse_u32(bytes, stats + 0x1C)?,
                kills_game_count: parse_u32(bytes, stats + 0x20)?,
            },
        });
    }

    Ok(npcs)
}

fn parse_globals(
    bytes: &[u8],
    offset: u32,
    count: u32,
) -> Result<Vec<GameVariableJson>, SaveParseError> {
    let offset = offset as usize;
    let count = count as usize;
    let table_end = checked_table_end(
        offset,
        count,
        GAM_V2_VARIABLE_SIZE,
        bytes.len(),
        "GAM globals",
    )?;
    let mut globals = Vec::with_capacity(count);

    for index in 0..count {
        let base = offset + index * GAM_V2_VARIABLE_SIZE;
        debug_assert!(base + GAM_V2_VARIABLE_SIZE <= table_end);
        let raw_type = parse_u16(bytes, base + 0x20)?;

        globals.push(GameVariableJson {
            index,
            name: parse_ascii_string(bytes, base, 32)?,
            type_flags: RawDecodedFlags {
                raw: raw_type,
                decoded: decode_variable_type_flags(raw_type),
            },
            ref_value: parse_u16(bytes, base + 0x22)?,
            dword_value: parse_u32(bytes, base + 0x24)?,
            int_value: parse_i32(bytes, base + 0x28)?,
            double_value: parse_f64(bytes, base + 0x2C)?,
            script_name_value: parse_ascii_string(bytes, base + 0x34, 32)?,
        });
    }

    Ok(globals)
}

fn inflate_sav_entry(filename: &str, bytes: &[u8]) -> Result<Vec<u8>, SaveParseError> {
    let mut decoder = ZlibDecoder::new(bytes);
    let mut inflated = Vec::new();
    decoder
        .read_to_end(&mut inflated)
        .map_err(|err| SaveParseError::Decompression {
            filename: filename.to_string(),
            message: err.to_string(),
        })?;
    Ok(inflated)
}

fn resolve_strref(strrefs: Option<&dyn StrRefResolver>, raw: u32) -> ResolvedStrRef {
    ResolvedStrRef {
        strref: StrRef(raw),
        text: strrefs.and_then(|resolver| resolver.resolve_strref(StrRef(raw))),
    }
}

fn decode_character_selection(value: u16) -> Option<String> {
    match value {
        0 => Some("not_selected".to_string()),
        1 => Some("selected".to_string()),
        0x8000 => Some("dead".to_string()),
        _ => None,
    }
}

fn decode_party_order(value: u16) -> Option<String> {
    match value {
        0..=5 => Some(format!("player_{}", value + 1)),
        0xFFFF => Some("not_in_party".to_string()),
        _ => None,
    }
}

fn decode_variable_type_flags(value: u16) -> Vec<String> {
    [
        (0x0001, "int"),
        (0x0002, "float"),
        (0x0004, "script_name"),
        (0x0008, "resref"),
        (0x0010, "strref"),
        (0x0020, "dword"),
    ]
    .into_iter()
    .filter(|(bit, _)| value & bit != 0)
    .map(|(_, label)| label.to_string())
    .collect()
}

fn parse_ascii_string(bytes: &[u8], offset: usize, len: usize) -> Result<String, SaveParseError> {
    let end = checked_end(offset, len, bytes.len(), "ASCII string")?;
    Ok(parse_nul_terminated_string(&bytes[offset..end]))
}

fn parse_nul_terminated_string(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim().to_string()
}

fn parse_resref_option(
    bytes: &[u8],
    offset: usize,
    len: usize,
) -> Result<Option<ResRef>, SaveParseError> {
    let value = parse_ascii_string(bytes, offset, len)?;
    if value.is_empty() {
        return Ok(None);
    }
    ResRef::new(value)
        .map(Some)
        .map_err(|err| SaveParseError::InvalidField(err.to_string()))
}

fn parse_u16(bytes: &[u8], offset: usize) -> Result<u16, SaveParseError> {
    let end = checked_end(offset, 2, bytes.len(), "u16")?;
    Ok(u16::from_le_bytes([bytes[offset], bytes[end - 1]]))
}

fn parse_i16(bytes: &[u8], offset: usize) -> Result<i16, SaveParseError> {
    Ok(parse_u16(bytes, offset)? as i16)
}

fn parse_u32(bytes: &[u8], offset: usize) -> Result<u32, SaveParseError> {
    checked_end(offset, 4, bytes.len(), "u32")?;
    Ok(u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]))
}

fn parse_i32(bytes: &[u8], offset: usize) -> Result<i32, SaveParseError> {
    Ok(parse_u32(bytes, offset)? as i32)
}

fn parse_f64(bytes: &[u8], offset: usize) -> Result<f64, SaveParseError> {
    checked_end(offset, 8, bytes.len(), "f64")?;
    Ok(f64::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ]))
}

fn checked_table_end(
    offset: usize,
    count: usize,
    entry_size: usize,
    file_len: usize,
    label: &str,
) -> Result<usize, SaveParseError> {
    let size = count
        .checked_mul(entry_size)
        .ok_or_else(|| SaveParseError::InvalidField(format!("{label} table size overflow")))?;
    checked_end(offset, size, file_len, label)
}

fn checked_end(
    offset: usize,
    len: usize,
    file_len: usize,
    label: &str,
) -> Result<usize, SaveParseError> {
    let end = offset
        .checked_add(len)
        .ok_or_else(|| SaveParseError::InvalidField(format!("{label} offset overflow")))?;
    if end > file_len {
        return Err(SaveParseError::UnexpectedEof(format!(
            "{label} at 0x{offset:X} extends past end of file"
        )));
    }
    Ok(end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::ZlibEncoder;
    use std::io::Write;

    #[test]
    fn parses_gam_v2_header_party_and_globals() {
        let mut bytes = vec![0u8; 0xB4 + GAM_V2_NPC_SIZE + GAM_V2_VARIABLE_SIZE];
        bytes[0..8].copy_from_slice(b"GAMEV2.0");
        write_u32(&mut bytes, 0x08, 2181);
        write_u32(&mut bytes, 0x18, 1234);
        write_u16(&mut bytes, 0x1C, 0xFFFF);
        write_u32(&mut bytes, 0x20, 0xB4);
        write_u32(&mut bytes, 0x24, 1);
        write_u32(&mut bytes, 0x38, (0xB4 + GAM_V2_NPC_SIZE) as u32);
        write_u32(&mut bytes, 0x3C, 1);
        bytes[0x40..0x46].copy_from_slice(b"AR0602");
        write_u32(&mut bytes, 0x54, 125);
        bytes[0x58..0x5E].copy_from_slice(b"AR0602");
        write_u32(&mut bytes, 0x74, 9876);

        let npc = 0xB4;
        write_u16(&mut bytes, npc, 1);
        write_u16(&mut bytes, npc + 0x02, 0);
        write_u32(&mut bytes, npc + 0x04, 0x500);
        write_u32(&mut bytes, npc + 0x08, 0x300);
        bytes[npc + 0x0C..npc + 0x11].copy_from_slice(b"IMOEN");
        bytes[npc + 0x18..npc + 0x1E].copy_from_slice(b"AR0602");
        write_u16(&mut bytes, npc + 0x20, 100);
        write_u16(&mut bytes, npc + 0x22, 200);
        bytes[npc + 0xC0..npc + 0xCA].copy_from_slice(b"Imoen Long");
        write_u32(&mut bytes, npc + 0xE0, 7);
        write_u32(&mut bytes, npc + 0xE4, 42);
        bytes[npc + 0xF4] = 1;

        let global = 0xB4 + GAM_V2_NPC_SIZE;
        bytes[global..global + 7].copy_from_slice(b"CHAPTER");
        write_u16(&mut bytes, global + 0x20, 1);
        write_u32(&mut bytes, global + 0x28, 3);

        let parsed = parse_gam(&bytes, "BALDUR.GAM", None).expect("GAM should parse");
        assert_eq!(parsed.header.game_time, 2181);
        assert_eq!(parsed.header.party_gold, 1234);
        assert_eq!(parsed.header.chapter, Some(3));
        assert_eq!(parsed.party_members[0].character_name, "IMOEN");
        assert_eq!(parsed.party_members[0].position.x, 100);
        assert_eq!(parsed.globals[0].name, "CHAPTER");
    }

    #[test]
    fn parses_sav_manifest_and_inflates_entries() {
        let payload = b"AREA bytes";
        let compressed = zlib(payload);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"SAV V1.0");
        bytes.extend_from_slice(&11u32.to_le_bytes());
        bytes.extend_from_slice(b"AR0602.ARE\0");
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&compressed);

        let parsed = parse_sav(&bytes, "BALDUR.SAV").expect("SAV should parse");
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].filename, "AR0602.ARE");
        assert_eq!(parsed.entries[0].resource_type.as_deref(), Some("ARE"));
        assert_eq!(parsed.entries[0].inflated_size, payload.len());
    }

    fn zlib(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(bytes).expect("test payload should compress");
        encoder.finish().expect("test payload should finish")
    }

    fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}
