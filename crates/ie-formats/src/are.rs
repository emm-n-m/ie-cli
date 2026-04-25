use ie_core::{
    CreatureResourceLink, ResRef, ResourceLink, ResourceLinkResolver, ResourceType, ScriptSlots,
    SourceKind,
};
use serde::Serialize;
use std::path::PathBuf;
use thiserror::Error;

const ARE_HEADER_SIZE: usize = 0x11C;
const ARE_ACTOR_SIZE: usize = 0x110;

#[derive(Debug, Clone, Serialize)]
pub struct AreaJson {
    pub resource_type: String,
    pub resource_name: String,
    pub version: String,
    pub header: AreaHeaderJson,
    pub actors: Vec<AreaActorJson>,
    pub deferred_sections: AreaDeferredSectionsJson,
}

#[derive(Debug, Clone, Serialize)]
pub struct AreaHeaderJson {
    pub wed_resource: Option<ResRef>,
    pub last_saved_seconds: u32,
    pub area_flags: RawDecodedFlags,
    pub area_type_flags: RawDecodedFlags,
    pub rain_probability: u16,
    pub snow_probability: u16,
    pub fog_probability: u16,
    pub lightning_probability: u16,
    pub actors_offset: u32,
    pub actors_count: u16,
    pub area_script: Option<ResourceLink>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AreaDeferredSectionsJson {
    pub regions_offset: u32,
    pub regions_count: u16,
    pub spawn_points_offset: u32,
    pub spawn_points_count: u32,
    pub entrances_offset: u32,
    pub entrances_count: u32,
    pub containers_offset: u32,
    pub containers_count: u16,
    pub items_offset: u32,
    pub items_count: u16,
    pub vertices_offset: u32,
    pub vertices_count: u16,
    pub ambients_offset: u32,
    pub ambients_count: u16,
    pub variables_offset: u32,
    pub variables_count: u32,
    pub doors_offset: u32,
    pub doors_count: u32,
    pub animations_offset: u32,
    pub animations_count: u32,
    pub tiled_objects_offset: u32,
    pub tiled_objects_count: u32,
    pub song_entries_offset: u32,
    pub rest_interruptions_offset: u32,
    pub automap_notes_offset: u32,
    pub automap_notes_count: u32,
    pub projectile_traps_offset: u32,
    pub projectile_traps_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AreaActorJson {
    pub index: usize,
    pub name: String,
    pub position: AreaPointJson,
    pub destination_position: AreaPointJson,
    pub flags: RawDecodedFlags,
    pub is_random_monster: RawDecoded<u16>,
    pub first_cre_resref_letter: u8,
    pub animation: u32,
    pub orientation: u16,
    pub removal_timer_seconds: i32,
    pub movement_restriction_distance: u16,
    pub movement_restriction_distance_to_object: u16,
    pub appearance_schedule_raw: u32,
    pub num_times_talked_to: u32,
    pub dialog: Option<ResourceLink>,
    pub scripts: ScriptSlots<ResourceLink>,
    pub cre_file: Option<CreatureResourceLink>,
    pub embedded_cre_offset: u32,
    pub embedded_cre_size: u32,
    pub unknown_tail_bytes_0x90: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AreaPointJson {
    pub x: u16,
    pub y: u16,
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
pub enum AreaParseError {
    #[error("invalid ARE header: {0}")]
    InvalidHeader(String),
    #[error("unexpected end of ARE file: {0}")]
    UnexpectedEof(String),
    #[error("invalid ARE field: {0}")]
    InvalidField(String),
}

impl From<AreaParseError> for crate::FormatError {
    fn from(err: AreaParseError) -> Self {
        crate::FormatError::Parse(err.to_string())
    }
}

pub fn parse_are(
    bytes: &[u8],
    resource_name: &str,
    links: Option<&dyn ResourceLinkResolver>,
) -> Result<AreaJson, crate::FormatError> {
    if bytes.len() < ARE_HEADER_SIZE {
        return Err(AreaParseError::UnexpectedEof(format!(
            "ARE resource must contain at least {} bytes",
            ARE_HEADER_SIZE
        ))
        .into());
    }

    if &bytes[0..4] != b"AREA" {
        return Err(AreaParseError::InvalidHeader("missing AREA signature".to_string()).into());
    }

    let version = parse_ascii_string(bytes, 4, 4)?;
    if version != "V1.0" {
        return Err(
            AreaParseError::InvalidHeader(format!("unsupported ARE version {version}")).into(),
        );
    }

    let actors_offset = parse_u32(bytes, 0x54)?;
    let actors_count = parse_u16(bytes, 0x58)?;

    let header = AreaHeaderJson {
        wed_resource: parse_resref_option(bytes, 0x08, 8)?,
        last_saved_seconds: parse_u32(bytes, 0x10)?,
        area_flags: RawDecodedFlags {
            raw: parse_u32(bytes, 0x14)?,
            decoded: decode_area_flags(parse_u32(bytes, 0x14)?),
        },
        area_type_flags: RawDecodedFlags {
            raw: parse_u16(bytes, 0x48)? as u32,
            decoded: decode_area_type_flags(parse_u16(bytes, 0x48)?),
        },
        rain_probability: parse_u16(bytes, 0x4A)?,
        snow_probability: parse_u16(bytes, 0x4C)?,
        fog_probability: parse_u16(bytes, 0x4E)?,
        lightning_probability: parse_u16(bytes, 0x50)?,
        actors_offset,
        actors_count,
        area_script: parse_resref_option(bytes, 0x94, 8)?
            .map(|resref| resolve_resource_link(links, &resref, ResourceType::Bcs)),
    };

    let actors = parse_actors(bytes, actors_offset, actors_count, links)?;

    Ok(AreaJson {
        resource_type: ResourceType::Are.as_str().to_string(),
        resource_name: resource_name.to_string(),
        version,
        header,
        actors,
        deferred_sections: parse_deferred_sections(bytes)?,
    })
}

fn parse_deferred_sections(bytes: &[u8]) -> Result<AreaDeferredSectionsJson, AreaParseError> {
    Ok(AreaDeferredSectionsJson {
        regions_count: parse_u16(bytes, 0x5A)?,
        regions_offset: parse_u32(bytes, 0x5C)?,
        spawn_points_offset: parse_u32(bytes, 0x60)?,
        spawn_points_count: parse_u32(bytes, 0x64)?,
        entrances_offset: parse_u32(bytes, 0x68)?,
        entrances_count: parse_u32(bytes, 0x6C)?,
        containers_offset: parse_u32(bytes, 0x70)?,
        containers_count: parse_u16(bytes, 0x74)?,
        items_count: parse_u16(bytes, 0x76)?,
        items_offset: parse_u32(bytes, 0x78)?,
        vertices_offset: parse_u32(bytes, 0x7C)?,
        vertices_count: parse_u16(bytes, 0x80)?,
        ambients_count: parse_u16(bytes, 0x82)?,
        ambients_offset: parse_u32(bytes, 0x84)?,
        variables_offset: parse_u32(bytes, 0x88)?,
        variables_count: parse_u32(bytes, 0x8C)?,
        doors_count: parse_u32(bytes, 0xA4)?,
        doors_offset: parse_u32(bytes, 0xA8)?,
        animations_count: parse_u32(bytes, 0xAC)?,
        animations_offset: parse_u32(bytes, 0xB0)?,
        tiled_objects_count: parse_u32(bytes, 0xB4)?,
        tiled_objects_offset: parse_u32(bytes, 0xB8)?,
        song_entries_offset: parse_u32(bytes, 0xBC)?,
        rest_interruptions_offset: parse_u32(bytes, 0xC0)?,
        automap_notes_offset: parse_u32(bytes, 0xC4)?,
        automap_notes_count: parse_u32(bytes, 0xC8)?,
        projectile_traps_offset: parse_u32(bytes, 0xCC)?,
        projectile_traps_count: parse_u32(bytes, 0xD0)?,
    })
}

fn parse_actors(
    bytes: &[u8],
    offset: u32,
    count: u16,
    links: Option<&dyn ResourceLinkResolver>,
) -> Result<Vec<AreaActorJson>, AreaParseError> {
    parse_table(
        bytes,
        offset,
        count as u32,
        ARE_ACTOR_SIZE,
        |bytes, position, index| {
            let flags = parse_u32(bytes, position + 0x28)?;
            let is_random_monster = parse_u16(bytes, position + 0x2C)?;

            Ok(AreaActorJson {
                index,
                name: parse_char_array(bytes, position, 32)?,
                position: AreaPointJson {
                    x: parse_u16(bytes, position + 0x20)?,
                    y: parse_u16(bytes, position + 0x22)?,
                },
                destination_position: AreaPointJson {
                    x: parse_u16(bytes, position + 0x24)?,
                    y: parse_u16(bytes, position + 0x26)?,
                },
                flags: RawDecodedFlags {
                    raw: flags,
                    decoded: decode_actor_flags(flags),
                },
                is_random_monster: RawDecoded {
                    raw: is_random_monster,
                    decoded: decode_random_monster(is_random_monster).map(str::to_string),
                },
                first_cre_resref_letter: parse_u8(bytes, position + 0x2E)?,
                animation: parse_u32(bytes, position + 0x30)?,
                orientation: parse_u16(bytes, position + 0x34)?,
                removal_timer_seconds: parse_i32(bytes, position + 0x38)?,
                movement_restriction_distance: parse_u16(bytes, position + 0x3C)?,
                movement_restriction_distance_to_object: parse_u16(bytes, position + 0x3E)?,
                appearance_schedule_raw: parse_u32(bytes, position + 0x40)?,
                num_times_talked_to: parse_u32(bytes, position + 0x44)?,
                dialog: parse_resref_option(bytes, position + 0x48, 8)?
                    .map(|resref| resolve_resource_link(links, &resref, ResourceType::Dlg)),
                scripts: ScriptSlots {
                    override_script: parse_script_link(bytes, position + 0x50, links)?,
                    general_script: parse_script_link(bytes, position + 0x58, links)?,
                    class_script: parse_script_link(bytes, position + 0x60, links)?,
                    race_script: parse_script_link(bytes, position + 0x68, links)?,
                    default_script: parse_script_link(bytes, position + 0x70, links)?,
                    specific_script: parse_script_link(bytes, position + 0x78, links)?,
                },
                cre_file: parse_resref_option(bytes, position + 0x80, 8)?
                    .map(|resref| resolve_creature_link(links, &resref)),
                embedded_cre_offset: parse_u32(bytes, position + 0x88)?,
                embedded_cre_size: parse_u32(bytes, position + 0x8C)?,
                unknown_tail_bytes_0x90: bytes
                    .get(position + 0x90..position + ARE_ACTOR_SIZE)
                    .ok_or_else(|| {
                        AreaParseError::UnexpectedEof(format!(
                            "missing actor unknown tail bytes at {}",
                            position + 0x90
                        ))
                    })?
                    .to_vec(),
            })
        },
    )
}

fn parse_script_link(
    bytes: &[u8],
    offset: usize,
    links: Option<&dyn ResourceLinkResolver>,
) -> Result<Option<ResourceLink>, AreaParseError> {
    Ok(parse_resref_option(bytes, offset, 8)?
        .map(|resref| resolve_resource_link(links, &resref, ResourceType::Bcs)))
}

fn resolve_resource_link(
    links: Option<&dyn ResourceLinkResolver>,
    resref: &ResRef,
    resource_type: ResourceType,
) -> ResourceLink {
    links.map_or_else(
        || missing_resource_link(resref, resource_type),
        |resolver| resolver.resolve_resource_link(resref, resource_type),
    )
}

fn resolve_creature_link(
    links: Option<&dyn ResourceLinkResolver>,
    resref: &ResRef,
) -> CreatureResourceLink {
    links.map_or_else(
        || CreatureResourceLink {
            link: missing_resource_link(resref, ResourceType::Cre),
            short_name: None,
            long_name: None,
        },
        |resolver| resolver.resolve_creature_link(resref),
    )
}

fn missing_resource_link(resref: &ResRef, resource_type: ResourceType) -> ResourceLink {
    ResourceLink {
        resref: resref.clone(),
        resource_name: format!("{}.{}", resref.as_str(), resource_type.as_str()),
        resource_type: resource_type.as_str().to_string(),
        exists: false,
        source_kind: None::<SourceKind>,
        source_path: None::<PathBuf>,
    }
}

fn parse_table<T, F>(
    bytes: &[u8],
    offset: u32,
    count: u32,
    record_size: usize,
    mut parser: F,
) -> Result<Vec<T>, AreaParseError>
where
    F: FnMut(&[u8], usize, usize) -> Result<T, AreaParseError>,
{
    if count == 0 {
        return Ok(Vec::new());
    }

    let start = offset as usize;
    let total_size = (count as usize).checked_mul(record_size).ok_or_else(|| {
        AreaParseError::InvalidField("table count and record size overflow".to_string())
    })?;
    let end = start.checked_add(total_size).ok_or_else(|| {
        AreaParseError::InvalidField("table offset and size overflow".to_string())
    })?;

    if end > bytes.len() {
        return Err(AreaParseError::UnexpectedEof(format!(
            "table at {} with {} records of {} bytes exceeds ARE length {}",
            offset,
            count,
            record_size,
            bytes.len()
        )));
    }

    let mut rows = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let position = start + index * record_size;
        rows.push(parser(bytes, position, index)?);
    }
    Ok(rows)
}

fn parse_resref_option(
    bytes: &[u8],
    offset: usize,
    length: usize,
) -> Result<Option<ResRef>, AreaParseError> {
    let raw = bytes
        .get(offset..offset + length)
        .ok_or_else(|| AreaParseError::UnexpectedEof(format!("missing resref at {}", offset)))?;
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    let string = String::from_utf8_lossy(&raw[..end])
        .trim()
        .to_ascii_uppercase();
    if string.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ResRef::new(string).map_err(|err| {
            AreaParseError::InvalidField(err.to_string())
        })?))
    }
}

fn parse_ascii_string(
    bytes: &[u8],
    offset: usize,
    length: usize,
) -> Result<String, AreaParseError> {
    let raw = bytes.get(offset..offset + length).ok_or_else(|| {
        AreaParseError::UnexpectedEof(format!("missing ascii string at {}", offset))
    })?;
    Ok(String::from_utf8_lossy(raw).to_string())
}

fn parse_char_array(bytes: &[u8], offset: usize, length: usize) -> Result<String, AreaParseError> {
    let raw = bytes.get(offset..offset + length).ok_or_else(|| {
        AreaParseError::UnexpectedEof(format!("missing char array at {}", offset))
    })?;
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    Ok(String::from_utf8_lossy(&raw[..end]).trim().to_string())
}

fn parse_u8(bytes: &[u8], offset: usize) -> Result<u8, AreaParseError> {
    bytes
        .get(offset)
        .copied()
        .ok_or_else(|| AreaParseError::UnexpectedEof(format!("unable to read u8 at {}", offset)))
}

fn parse_u16(bytes: &[u8], offset: usize) -> Result<u16, AreaParseError> {
    let raw = bytes.get(offset..offset + 2).ok_or_else(|| {
        AreaParseError::UnexpectedEof(format!("unable to read u16 at {}", offset))
    })?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn parse_u32(bytes: &[u8], offset: usize) -> Result<u32, AreaParseError> {
    let raw = bytes.get(offset..offset + 4).ok_or_else(|| {
        AreaParseError::UnexpectedEof(format!("unable to read u32 at {}", offset))
    })?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn parse_i32(bytes: &[u8], offset: usize) -> Result<i32, AreaParseError> {
    let raw = bytes.get(offset..offset + 4).ok_or_else(|| {
        AreaParseError::UnexpectedEof(format!("unable to read i32 at {}", offset))
    })?;
    Ok(i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn decode_area_flags(value: u32) -> Vec<String> {
    decode_bits(
        value,
        &[
            (0x0000_0001, "SaveNotAllowed"),
            (0x0000_0002, "TutorialOrRestRestricted"),
            (0x0000_0004, "DeadMagicOrDangerousRest"),
            (0x0000_0008, "Dream"),
            (0x0000_0010, "Player1DeathDoesNotEndGame"),
            (0x0000_0020, "RestingNotAllowed"),
            (0x0000_0040, "TravelNotAllowed"),
            (0x0000_0080, "CannotRestHere"),
            (0x0000_0100, "TooDangerousToRest"),
        ],
    )
}

fn decode_area_type_flags(value: u16) -> Vec<String> {
    decode_bits(
        value as u32,
        &[
            (0x0001, "OutdoorOrHive"),
            (0x0002, "DayNightOrHiveNight"),
            (0x0004, "WeatherOrClerksWard"),
            (0x0008, "CityOrLowerWard"),
            (0x0010, "ForestOrRavelsMaze"),
            (0x0020, "DungeonOrBaator"),
            (0x0040, "ExtendedNightOrRubikon"),
            (0x0080, "CanRestIndoorsOrFortressOfRegrets"),
            (0x0100, "Curst"),
            (0x0200, "Carceri"),
            (0x0400, "OutdoorsPst"),
        ],
    )
}

fn decode_actor_flags(value: u32) -> Vec<String> {
    decode_bits(
        value,
        &[
            (0x0000_0001, "CreNotAttached"),
            (0x0000_0002, "HasSeenParty"),
            (0x0000_0004, "Invulnerable"),
            (0x0000_0008, "OverrideScriptName"),
        ],
    )
}

fn decode_random_monster(value: u16) -> Option<&'static str> {
    match value {
        0 => Some("No"),
        1 => Some("Yes"),
        _ => None,
    }
}

fn decode_bits(value: u32, mapping: &[(u32, &str)]) -> Vec<String> {
    mapping
        .iter()
        .filter_map(|(bit, label)| (value & bit != 0).then_some((*label).to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ie_core::{ResolvedStrRef, StrRef};

    struct TestLinks;

    impl ResourceLinkResolver for TestLinks {
        fn resolve_resource_link(
            &self,
            resref: &ResRef,
            resource_type: ResourceType,
        ) -> ResourceLink {
            ResourceLink {
                resref: resref.clone(),
                resource_name: format!("{}.{}", resref.as_str(), resource_type.as_str()),
                resource_type: resource_type.as_str().to_string(),
                exists: true,
                source_kind: Some(SourceKind::Bif),
                source_path: Some(PathBuf::from("data/test.bif")),
            }
        }

        fn resolve_creature_link(&self, resref: &ResRef) -> CreatureResourceLink {
            CreatureResourceLink {
                link: self.resolve_resource_link(resref, ResourceType::Cre),
                short_name: Some(ResolvedStrRef {
                    strref: StrRef(10),
                    text: Some("Short".to_string()),
                }),
                long_name: Some(ResolvedStrRef {
                    strref: StrRef(11),
                    text: Some("Long".to_string()),
                }),
            }
        }
    }

    #[test]
    fn parse_minimal_are_header_with_zero_actors() {
        let bytes = minimal_are();
        let area = parse_are(&bytes, "AR0202.ARE", None).expect("ARE should parse");

        assert_eq!(area.resource_type, "ARE");
        assert_eq!(area.resource_name, "AR0202.ARE");
        assert_eq!(area.version, "V1.0");
        assert_eq!(
            area.header.wed_resource.as_ref().map(|r| r.as_str()),
            Some("AR0202")
        );
        assert_eq!(area.header.actors_count, 0);
        assert!(area.actors.is_empty());
        assert_eq!(area.deferred_sections.regions_count, 2);
    }

    #[test]
    fn parse_are_actor_links_and_coordinates() {
        let mut bytes = minimal_are();
        let actor_offset = ARE_HEADER_SIZE;
        bytes.resize(ARE_HEADER_SIZE + ARE_ACTOR_SIZE, 0);
        bytes[0x54..0x58].copy_from_slice(&(actor_offset as u32).to_le_bytes());
        bytes[0x58..0x5A].copy_from_slice(&1u16.to_le_bytes());

        bytes[actor_offset..actor_offset + 5].copy_from_slice(b"Grace");
        bytes[actor_offset + 0x20..actor_offset + 0x22].copy_from_slice(&100u16.to_le_bytes());
        bytes[actor_offset + 0x22..actor_offset + 0x24].copy_from_slice(&120u16.to_le_bytes());
        bytes[actor_offset + 0x24..actor_offset + 0x26].copy_from_slice(&140u16.to_le_bytes());
        bytes[actor_offset + 0x26..actor_offset + 0x28].copy_from_slice(&160u16.to_le_bytes());
        bytes[actor_offset + 0x28..actor_offset + 0x2C].copy_from_slice(&0x0006u32.to_le_bytes());
        bytes[actor_offset + 0x2C..actor_offset + 0x2E].copy_from_slice(&1u16.to_le_bytes());
        bytes[actor_offset + 0x34..actor_offset + 0x36].copy_from_slice(&12u16.to_le_bytes());
        bytes[actor_offset + 0x38..actor_offset + 0x3C].copy_from_slice(&(-1i32).to_le_bytes());
        bytes[actor_offset + 0x40..actor_offset + 0x44]
            .copy_from_slice(&0x00FF_FFFFu32.to_le_bytes());
        bytes[actor_offset + 0x48..actor_offset + 0x50].copy_from_slice(b"DGRACE\0\0");
        bytes[actor_offset + 0x50..actor_offset + 0x58].copy_from_slice(b"OVR\0\0\0\0\0");
        bytes[actor_offset + 0x70..actor_offset + 0x78].copy_from_slice(b"DEF\0\0\0\0\0");
        bytes[actor_offset + 0x80..actor_offset + 0x88].copy_from_slice(b"DGRACE\0\0");
        bytes[actor_offset + 0x90] = 0xAA;

        let area = parse_are(&bytes, "AR0202.ARE", Some(&TestLinks)).expect("ARE should parse");
        let actor = &area.actors[0];

        assert_eq!(actor.name, "Grace");
        assert_eq!(actor.position.x, 100);
        assert_eq!(actor.position.y, 120);
        assert_eq!(actor.destination_position.x, 140);
        assert_eq!(actor.flags.decoded, vec!["HasSeenParty", "Invulnerable"]);
        assert_eq!(actor.is_random_monster.decoded.as_deref(), Some("Yes"));
        assert_eq!(actor.orientation, 12);
        assert_eq!(actor.removal_timer_seconds, -1);
        assert_eq!(
            actor
                .dialog
                .as_ref()
                .map(|link| link.resource_name.as_str()),
            Some("DGRACE.DLG")
        );
        assert_eq!(
            actor
                .scripts
                .override_script
                .as_ref()
                .map(|link| link.resource_name.as_str()),
            Some("OVR.BCS")
        );
        assert_eq!(
            actor
                .scripts
                .default_script
                .as_ref()
                .map(|link| link.resource_name.as_str()),
            Some("DEF.BCS")
        );
        assert_eq!(
            actor
                .cre_file
                .as_ref()
                .and_then(|link| link.short_name.as_ref())
                .and_then(|name| name.text.as_deref()),
            Some("Short")
        );
        assert_eq!(actor.unknown_tail_bytes_0x90[0], 0xAA);
    }

    #[test]
    fn reject_invalid_signature() {
        let mut bytes = minimal_are();
        bytes[0..4].copy_from_slice(b"NOPE");

        let error = parse_are(&bytes, "AR0202.ARE", None).expect_err("signature should fail");
        assert!(error.to_string().contains("missing AREA signature"));
    }

    #[test]
    fn reject_unsupported_version() {
        let mut bytes = minimal_are();
        bytes[4..8].copy_from_slice(b"V2.0");

        let error = parse_are(&bytes, "AR0202.ARE", None).expect_err("version should fail");
        assert!(error.to_string().contains("unsupported ARE version"));
    }

    #[test]
    fn reject_truncated_actor_table() {
        let mut bytes = minimal_are();
        bytes[0x54..0x58].copy_from_slice(&(ARE_HEADER_SIZE as u32).to_le_bytes());
        bytes[0x58..0x5A].copy_from_slice(&1u16.to_le_bytes());

        let error = parse_are(&bytes, "AR0202.ARE", None).expect_err("actor table should fail");
        assert!(error.to_string().contains("exceeds ARE length"));
    }

    #[test]
    fn parse_are_with_two_actors_checks_index_ordering_and_positions() {
        let mut bytes = minimal_are();
        let actor_offset = ARE_HEADER_SIZE;
        bytes.resize(ARE_HEADER_SIZE + 2 * ARE_ACTOR_SIZE, 0);
        bytes[0x54..0x58].copy_from_slice(&(actor_offset as u32).to_le_bytes());
        bytes[0x58..0x5A].copy_from_slice(&2u16.to_le_bytes());

        // Actor 0 — "Morte" at (100, 200)
        bytes[actor_offset..actor_offset + 5].copy_from_slice(b"Morte");
        bytes[actor_offset + 0x20..actor_offset + 0x22].copy_from_slice(&100u16.to_le_bytes());
        bytes[actor_offset + 0x22..actor_offset + 0x24].copy_from_slice(&200u16.to_le_bytes());

        // Actor 1 — "Nordom" at (300, 400)
        let second = actor_offset + ARE_ACTOR_SIZE;
        bytes[second..second + 6].copy_from_slice(b"Nordom");
        bytes[second + 0x20..second + 0x22].copy_from_slice(&300u16.to_le_bytes());
        bytes[second + 0x22..second + 0x24].copy_from_slice(&400u16.to_le_bytes());

        let area = parse_are(&bytes, "TESTAREA.ARE", None).expect("ARE should parse");
        assert_eq!(area.actors.len(), 2);
        assert_eq!(area.actors[0].index, 0);
        assert_eq!(area.actors[0].name, "Morte");
        assert_eq!(area.actors[0].position.x, 100);
        assert_eq!(area.actors[0].position.y, 200);
        assert_eq!(area.actors[1].index, 1);
        assert_eq!(area.actors[1].name, "Nordom");
        assert_eq!(area.actors[1].position.x, 300);
        assert_eq!(area.actors[1].position.y, 400);
    }

    #[test]
    fn area_flags_and_type_flags_decode_from_minimal_fixture() {
        // minimal_are encodes area_flags=0x21 (0x01|0x20) and area_type_flags=0x0400.
        // Expected decoded names are derived from IESDP ARE V1.0 actor/header field tables
        // (https://gibberlings3.github.io/iesdp/file_formats/ie_formats/are_v1.htm — returned
        // 403 during this run; offsets were validated against existing implementation).
        let bytes = minimal_are();
        let area = parse_are(&bytes, "AR0202.ARE", None).expect("ARE should parse");

        assert!(
            area.header
                .area_flags
                .decoded
                .contains(&"SaveNotAllowed".to_string()),
            "expected SaveNotAllowed bit (0x01) in area_flags 0x21"
        );
        assert!(
            area.header
                .area_flags
                .decoded
                .contains(&"RestingNotAllowed".to_string()),
            "expected RestingNotAllowed bit (0x20) in area_flags 0x21"
        );
        assert!(
            area.header
                .area_type_flags
                .decoded
                .contains(&"OutdoorsPst".to_string()),
            "expected OutdoorsPst bit (0x0400) in area_type_flags 0x0400"
        );
        // Raw values must round-trip unchanged.
        assert_eq!(area.header.area_flags.raw, 0x21);
        assert_eq!(area.header.area_type_flags.raw, 0x0400);
    }

    fn minimal_are() -> Vec<u8> {
        let mut bytes = vec![0u8; ARE_HEADER_SIZE];
        bytes[0..4].copy_from_slice(b"AREA");
        bytes[4..8].copy_from_slice(b"V1.0");
        bytes[0x08..0x10].copy_from_slice(b"AR0202\0\0");
        bytes[0x14..0x18].copy_from_slice(&0x21u32.to_le_bytes());
        bytes[0x48..0x4A].copy_from_slice(&0x0400u16.to_le_bytes());
        bytes[0x54..0x58].copy_from_slice(&(ARE_HEADER_SIZE as u32).to_le_bytes());
        bytes[0x5A..0x5C].copy_from_slice(&2u16.to_le_bytes());
        bytes[0x94..0x9C].copy_from_slice(b"AR0202\0\0");
        bytes
    }
}
