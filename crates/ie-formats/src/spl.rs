use ie_core::{ResolvedStrRef, ResRef, ResourceType, StrRef, StrRefResolver};
use serde::Serialize;
use thiserror::Error;

const SPL_HEADER_SIZE: usize = 0x74;
const SPL_EXTENDED_HEADER_SIZE: usize = 40;
const SPL_FEATURE_BLOCK_SIZE: usize = 48;

#[derive(Debug, Clone, Serialize)]
pub struct SpellJson {
    pub resource_type: String,
    pub resource_name: String,
    pub version: String,
    pub header: SpellHeaderJson,
    pub extended_headers: Vec<SpellExtendedHeaderJson>,
    pub feature_blocks: Vec<SpellFeatureBlockJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpellHeaderJson {
    pub spell_name: ResolvedStrRef,
    pub completion_sound: Option<ResRef>,
    pub flags: RawDecodedFlags,
    pub spell_type: RawDecoded<u16>,
    pub exclusion_flags: u32,
    pub casting_graphics: u16,
    pub school: RawDecoded<u8>,
    pub secondary_type: u8,
    pub spell_level: u8,
    pub spellbook_icon: Option<ResRef>,
    pub description: ResolvedStrRef,
    pub extended_headers_offset: u32,
    pub extended_headers_count: u16,
    pub feature_block_offset: u32,
    pub casting_feature_block_offset: u32,
    pub casting_feature_block_count: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpellExtendedHeaderJson {
    pub spell_form: RawDecoded<u8>,
    pub type_flags: u8,
    pub location: RawDecoded<u8>,
    pub memorized_icon: Option<ResRef>,
    pub target_type: RawDecoded<u8>,
    pub target_count: u8,
    pub range: i16,
    pub level_required: u8,
    pub casting_time: u8,
    pub times_per_day: u16,
    pub feature_block_count: u16,
    pub feature_block_index: u16,
    pub projectile: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpellFeatureBlockJson {
    pub opcode: RawDecoded<u16>,
    pub target_type: RawDecoded<u8>,
    pub power: u8,
    pub parameter1: u32,
    pub parameter2: u32,
    pub resource: Option<ResRef>,
    pub timing: RawDecoded<u8>,
    pub timing_dispel: u8,
    pub duration: u32,
    pub probability_1: u8,
    pub probability_2: u8,
    pub dice: Option<String>,
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
    pub raw: u32,
    pub decoded: Vec<String>,
}

#[derive(Debug, Error)]
pub enum SpellParseError {
    #[error("invalid SPL header: {0}")]
    InvalidHeader(String),
    #[error("unexpected end of SPL file: {0}")]
    UnexpectedEof(String),
    #[error("invalid resource field: {0}")]
    InvalidField(String),
}

impl From<SpellParseError> for crate::FormatError {
    fn from(err: SpellParseError) -> Self {
        crate::FormatError::Parse(err.to_string())
    }
}

pub fn parse_spl(
    bytes: &[u8],
    resource_name: &str,
    resolver: Option<&dyn StrRefResolver>,
) -> Result<SpellJson, crate::FormatError> {
    if bytes.len() < SPL_HEADER_SIZE {
        return Err(SpellParseError::UnexpectedEof(format!(
            "SPL resource must contain at least {} bytes",
            SPL_HEADER_SIZE
        )))?;
    }

    if &bytes[0..4] != b"SPL " {
        return Err(SpellParseError::InvalidHeader("missing SPL signature".to_string()))?;
    }

    let version = parse_ascii_string(bytes, 4, 4)?;
    let spell_name = parse_strref(bytes, 0x08, resolver)?;
    let completion_sound = parse_resref_option(bytes, 0x0C, 8)?;
    let flags = parse_u32(bytes, 0x18)?;
    let spell_type = parse_u16(bytes, 0x1C)?;
    let exclusion_flags = parse_u32(bytes, 0x1E)?;
    let casting_graphics = parse_u16(bytes, 0x22)?;
    let school = parse_u8(bytes, 0x25)?;
    let secondary_type = parse_u8(bytes, 0x26)?;
    let spell_level = parse_u8(bytes, 0x34)?;
    let spellbook_icon = parse_resref_option(bytes, 0x3A, 8)?;
    let description = parse_strref(bytes, 0x50, resolver)?;
    let extended_headers_offset = parse_u32(bytes, 0x64)?;
    let extended_headers_count = parse_u16(bytes, 0x68)?;
    let feature_block_offset = parse_u32(bytes, 0x6A)?;
    let casting_feature_block_offset = parse_u32(bytes, 0x6E)?;
    let casting_feature_block_count = parse_u16(bytes, 0x72)?;

    let header = SpellHeaderJson {
        spell_name,
        completion_sound,
        flags: RawDecodedFlags {
            raw: flags,
            decoded: decode_spell_flags(flags),
        },
        spell_type: RawDecoded {
            raw: spell_type,
            decoded: decode_spell_type(spell_type).map(str::to_string),
        },
        exclusion_flags,
        casting_graphics,
        school: RawDecoded {
            raw: school,
            decoded: decode_school(school).map(str::to_string),
        },
        secondary_type,
        spell_level,
        spellbook_icon,
        description,
        extended_headers_offset,
        extended_headers_count,
        feature_block_offset,
        casting_feature_block_offset,
        casting_feature_block_count,
    };

    let extended_headers = parse_extended_headers(bytes, extended_headers_offset, extended_headers_count)?;
    let feature_blocks = parse_feature_blocks(bytes, feature_block_offset, extended_headers_count, extended_headers_offset)?;

    Ok(SpellJson {
        resource_type: ResourceType::Spl.as_str().to_string(),
        resource_name: resource_name.to_string(),
        version,
        header,
        extended_headers,
        feature_blocks,
    })
}

fn parse_extended_headers(
    bytes: &[u8],
    offset: u32,
    count: u16,
) -> Result<Vec<SpellExtendedHeaderJson>, SpellParseError> {
    if count == 0 {
        return Ok(Vec::new());
    }

    let start = offset as usize;
    let total_size = count as usize * SPL_EXTENDED_HEADER_SIZE;
    let end = start.checked_add(total_size).ok_or_else(|| {
        SpellParseError::InvalidField("extended headers offset and count overflow".to_string())
    })?;

    if end > bytes.len() {
        return Err(SpellParseError::UnexpectedEof(format!(
            "extended header table exceeds SPL length: {} > {}",
            end,
            bytes.len()
        )));
    }

    let mut headers = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let position = start + index * SPL_EXTENDED_HEADER_SIZE;
        let header = parse_extended_header(bytes, position)?;
        headers.push(header);
    }
    Ok(headers)
}

fn parse_extended_header(
    bytes: &[u8],
    offset: usize,
) -> Result<SpellExtendedHeaderJson, SpellParseError> {
    if offset + SPL_EXTENDED_HEADER_SIZE > bytes.len() {
        return Err(SpellParseError::UnexpectedEof(format!(
            "extended header record at {} is truncated",
            offset
        )));
    }

    let spell_form = parse_u8(bytes, offset)?;
    let type_flags = parse_u8(bytes, offset + 1)?;
    let location = parse_u8(bytes, offset + 2)?;
    let memorized_icon = parse_resref_option(bytes, offset + 4, 8)?;
    let target_type = parse_u8(bytes, offset + 0x0C)?;
    let target_count = parse_u8(bytes, offset + 0x0D)?;
    let range = parse_i16(bytes, offset + 0x0E)?;
    let level_required = parse_u8(bytes, offset + 0x10)?;
    let casting_time = parse_u8(bytes, offset + 0x12)?;
    let times_per_day = parse_u16(bytes, offset + 0x14)?;
    let feature_block_count = parse_u16(bytes, offset + 0x1E)?;
    let feature_block_index = parse_u16(bytes, offset + 0x20)?;
    let projectile = parse_u16(bytes, offset + 0x26)?;

    Ok(SpellExtendedHeaderJson {
        spell_form: RawDecoded {
            raw: spell_form,
            decoded: decode_spell_form(spell_form).map(str::to_string),
        },
        type_flags,
        location: RawDecoded {
            raw: location,
            decoded: decode_spell_location(location).map(str::to_string),
        },
        memorized_icon,
        target_type: RawDecoded {
            raw: target_type,
            decoded: decode_spell_target_type(target_type).map(str::to_string),
        },
        target_count,
        range,
        level_required,
        casting_time,
        times_per_day,
        feature_block_count,
        feature_block_index,
        projectile,
    })
}

fn parse_feature_blocks(
    bytes: &[u8],
    offset: u32,
    header_count: u16,
    header_offset: u32,
) -> Result<Vec<SpellFeatureBlockJson>, SpellParseError> {
    // Calculate total feature blocks from all headers
    let mut total_blocks = 0u16;
    let header_start = header_offset as usize;
    for index in 0..header_count as usize {
        let position = header_start + index * SPL_EXTENDED_HEADER_SIZE;
        if position + 0x22 <= bytes.len() {
            let count = parse_u16(bytes, position + 0x1E).unwrap_or(0);
            total_blocks = total_blocks.saturating_add(count);
        }
    }

    if total_blocks == 0 {
        return Ok(Vec::new());
    }

    let start = offset as usize;
    let byte_end = start.checked_add(total_blocks as usize * SPL_FEATURE_BLOCK_SIZE).ok_or_else(|| {
        SpellParseError::InvalidField("feature block offset and count overflow".to_string())
    })?;

    if byte_end > bytes.len() {
        return Err(SpellParseError::UnexpectedEof(format!(
            "feature block table exceeds SPL length: {} > {}",
            byte_end,
            bytes.len()
        )));
    }

    let mut blocks = Vec::with_capacity(total_blocks as usize);
    for index in 0..total_blocks as usize {
        let position = start + index * SPL_FEATURE_BLOCK_SIZE;
        let block = parse_feature_block(bytes, position)?;
        blocks.push(block);
    }
    Ok(blocks)
}

fn parse_feature_block(
    bytes: &[u8],
    offset: usize,
) -> Result<SpellFeatureBlockJson, SpellParseError> {
    if offset + SPL_FEATURE_BLOCK_SIZE > bytes.len() {
        return Err(SpellParseError::UnexpectedEof(format!(
            "feature block record at {} is truncated",
            offset
        )));
    }

    let opcode = parse_u16(bytes, offset)?;
    let target_type = parse_u8(bytes, offset + 2)?;
    let power = parse_u8(bytes, offset + 3)?;
    let parameter1 = parse_u32(bytes, offset + 4)?;
    let parameter2 = parse_u32(bytes, offset + 8)?;
    let resource = parse_resref_option(bytes, offset + 12, 8)?;
    let timing = parse_u8(bytes, offset + 20)?;
    let timing_dispel = parse_u8(bytes, offset + 21)?;
    let duration = parse_u32(bytes, offset + 22)?;
    let probability_1 = parse_u8(bytes, offset + 26)?;
    let probability_2 = parse_u8(bytes, offset + 27)?;

    let dice = if probability_2 > 0 {
        Some(format!("{}d{}", probability_1, probability_2))
    } else {
        None
    };

    Ok(SpellFeatureBlockJson {
        opcode: RawDecoded {
            raw: opcode,
            decoded: decode_effect_opcode(opcode).map(str::to_string),
        },
        target_type: RawDecoded {
            raw: target_type,
            decoded: decode_effect_target_type(target_type).map(str::to_string),
        },
        power,
        parameter1,
        parameter2,
        resource,
        timing: RawDecoded {
            raw: timing,
            decoded: decode_effect_timing(timing).map(str::to_string),
        },
        timing_dispel,
        duration,
        probability_1,
        probability_2,
        dice,
    })
}

fn parse_resref_option(bytes: &[u8], offset: usize, length: usize) -> Result<Option<ResRef>, SpellParseError> {
    let raw = bytes
        .get(offset..offset + length)
        .ok_or_else(|| SpellParseError::UnexpectedEof(format!("missing resref at {}", offset)))?;

    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    let string = String::from_utf8_lossy(&raw[..end]).trim().to_ascii_uppercase();
    if string.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ResRef::new(string).map_err(|err| SpellParseError::InvalidField(err.to_string()))?))
    }
}

fn parse_strref(
    bytes: &[u8],
    offset: usize,
    resolver: Option<&dyn StrRefResolver>,
) -> Result<ResolvedStrRef, SpellParseError> {
    let raw = parse_u32(bytes, offset)?;
    let text = if raw == u32::MAX {
        None
    } else {
        resolver.and_then(|resolver| resolver.resolve_strref(StrRef(raw)))
    };
    Ok(ResolvedStrRef { strref: StrRef(raw), text })
}

fn parse_ascii_string(bytes: &[u8], offset: usize, length: usize) -> Result<String, SpellParseError> {
    let raw = bytes
        .get(offset..offset + length)
        .ok_or_else(|| SpellParseError::UnexpectedEof(format!("missing ascii string at {}", offset)))?;
    Ok(String::from_utf8_lossy(raw).to_string())
}

fn parse_u8(bytes: &[u8], offset: usize) -> Result<u8, SpellParseError> {
    bytes
        .get(offset)
        .copied()
        .ok_or_else(|| SpellParseError::UnexpectedEof(format!("unable to read u8 at {}", offset)))
}

fn parse_u16(bytes: &[u8], offset: usize) -> Result<u16, SpellParseError> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| SpellParseError::UnexpectedEof(format!("unable to read u16 at {}", offset)))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn parse_u32(bytes: &[u8], offset: usize) -> Result<u32, SpellParseError> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| SpellParseError::UnexpectedEof(format!("unable to read u32 at {}", offset)))?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn parse_i16(bytes: &[u8], offset: usize) -> Result<i16, SpellParseError> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| SpellParseError::UnexpectedEof(format!("unable to read i16 at {}", offset)))?;
    Ok(i16::from_le_bytes([raw[0], raw[1]]))
}

fn decode_spell_type(value: u16) -> Option<&'static str> {
    match value {
        1 => Some("Wizard"),
        2 => Some("Priest"),
        3 => Some("Psionic"),
        4 => Some("Innate"),
        5 => Some("Bard"),
        _ => None,
    }
}

fn decode_school(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("Abjuration"),
        1 => Some("Conjuration"),
        2 => Some("Divination"),
        3 => Some("Enchantment"),
        4 => Some("Evocation"),
        5 => Some("Illusion"),
        6 => Some("Necromancy"),
        7 => Some("Transmutation"),
        8 => Some("Universalist"),
        _ => None,
    }
}

fn decode_spell_flags(value: u32) -> Vec<String> {
    let mut labels = Vec::new();
    let mapping = [
        (0x00000001, "AllowSpotting"),
        (0x00000002, "BreakSanctuary"),
        (0x00000004, "Hostile"),
        (0x00000008, "NoLineOfSight"),
        (0x00000010, "OutdoorsOnly"),
        (0x00000020, "IgnoreDeadMagic"),
        (0x00000040, "IgnoreWildSurge"),
    ];

    for (bit, label) in mapping {
        if value & bit != 0 {
            labels.push(label.to_string());
        }
    }
    labels
}

fn decode_spell_form(value: u8) -> Option<&'static str> {
    match value {
        1 => Some("Standard"),
        2 => Some("Projectile"),
        _ => None,
    }
}

fn decode_spell_location(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("None"),
        2 => Some("Spell"),
        4 => Some("Innate"),
        _ => None,
    }
}

fn decode_spell_target_type(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("Invalid"),
        1 => Some("Living"),
        2 => Some("Inventory"),
        3 => Some("Dead"),
        4 => Some("Any Point"),
        5 => Some("Caster"),
        6 => Some("Crash"),
        7 => Some("Instant Self"),
        _ => None,
    }
}

fn decode_effect_opcode(value: u16) -> Option<&'static str> {
    match value {
        0 => Some("Cure Condition"),
        1 => Some("Cure Poison"),
        2 => Some("Cure Disease"),
        3 => Some("Heal"),
        4 => Some("Drain Level"),
        5 => Some("Drain HP"),
        70 => Some("Projectile"),
        12 => Some("Heal"),
        _ => None,
    }
}

fn decode_effect_target_type(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("None"),
        1 => Some("Self"),
        2 => Some("Projectile"),
        3 => Some("Party"),
        4 => Some("Everyone"),
        5 => Some("Not Party"),
        6 => Some("Caster Group"),
        7 => Some("Target Group"),
        _ => None,
    }
}

fn decode_effect_timing(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("Instant (duration)"),
        1 => Some("Instant (permanent)"),
        2 => Some("Delayed (duration)"),
        3 => Some("Delayed (permanent)"),
        4 => Some("Instant (while equipped)"),
        5 => Some("Instant (permanent unless removed)"),
        6 => Some("Delayed (until end of combat)"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NullResolver;
    impl StrRefResolver for NullResolver {
        fn resolve_strref(&self, _strref: StrRef) -> Option<String> {
            None
        }
    }

    #[test]
    fn parse_minimal_spl_header() {
        let mut bytes = vec![0u8; SPL_HEADER_SIZE];
        bytes[0..4].copy_from_slice(b"SPL ");
        bytes[4..8].copy_from_slice(b"V1  ");
        bytes[0x08..0x0C].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x0C..0x14].copy_from_slice(b"SOUND\0\0\0");
        bytes[0x18..0x1C].copy_from_slice(&0x00000004u32.to_le_bytes());
        bytes[0x1C..0x1E].copy_from_slice(&1u16.to_le_bytes());
        bytes[0x25..0x26].copy_from_slice(&4u8.to_le_bytes());
        bytes[0x34..0x35].copy_from_slice(&1u8.to_le_bytes());
        bytes[0x3A..0x42].copy_from_slice(b"ICON\0\0\0\0");
        bytes[0x50..0x54].copy_from_slice(&2u32.to_le_bytes());
        bytes[0x64..0x68].copy_from_slice(&0u32.to_le_bytes());
        bytes[0x68..0x6A].copy_from_slice(&0u16.to_le_bytes());
        bytes[0x6A..0x6E].copy_from_slice(&0u32.to_le_bytes());
        bytes[0x6E..0x72].copy_from_slice(&0u32.to_le_bytes());
        bytes[0x72..0x74].copy_from_slice(&0u16.to_le_bytes());

        let spell = parse_spl(&bytes, "MAGMSSL.SPL", Some(&NullResolver)).expect("should parse SPL");
        assert_eq!(spell.resource_name, "MAGMSSL.SPL");
        assert_eq!(spell.version, "V1  ");
        assert_eq!(spell.header.spell_type.decoded.as_deref(), Some("Wizard"));
        assert_eq!(spell.header.school.decoded.as_deref(), Some("Evocation"));
        assert_eq!(spell.header.spell_level, 1);
        assert_eq!(spell.header.spellbook_icon.as_ref().map(|r| r.as_str()), Some("ICON"));
        assert_eq!(spell.header.flags.decoded, vec!["Hostile"]);
        assert!(spell.extended_headers.is_empty());
        assert!(spell.feature_blocks.is_empty());
    }
}
