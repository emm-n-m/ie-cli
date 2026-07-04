use flate2::read::ZlibDecoder;
use ie_core::{GameVariant, ResRef, ResolvedStrRef, StrRef, StrRefResolver};
use serde::Serialize;
use std::io::Read;
use thiserror::Error;

const GAM_V2_HEADER_SIZE: usize = 0xB4;
const GAM_V2_NPC_SIZE: usize = 0x160;
const GAM_V2_VARIABLE_SIZE: usize = 0x54;
const GAM_V2_HEADER_OFFSET_FIELDS: [usize; 8] = [0x20, 0x28, 0x30, 0x38, 0x50, 0x68, 0x6C, 0x78];
const SAV_HEADER_SIZE: usize = 0x08;
const CRE_HEADER_SIZE: usize = 0x2D4;
const CRE_ITEM_SIZE: usize = 20;
const CRE_ITEMS_COUNT_OFFSET: usize = 0x02C0;
const CRE_ITEM_IDENTIFIED_FLAG: u32 = 0x0000_0001;
const PST_INVENTORY_SLOT_START: usize = 20;
const PST_INVENTORY_SLOT_END_EXCLUSIVE: usize = 40;

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
    pub party_inventory_offset: u32,
    pub party_inventory_count: u32,
    pub non_party_members_offset: u32,
    pub non_party_members_count: u32,
    pub globals_offset: u32,
    pub globals_count: u32,
    pub journal_entries_offset: u32,
    pub journal_entries_count: u32,
    pub unknown_section_68_offset: u32,
    pub unknown_section_6c_offset: u32,
    pub unknown_section_78_offset: u32,
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

#[derive(Debug, Clone)]
pub struct NewItem {
    pub resref: ResRef,
    pub expiration_time_days: u16,
    pub charges_1: u16,
    pub charges_2: u16,
    pub charges_3: u16,
    pub flags: u32,
}

impl NewItem {
    pub fn identified(resref: ResRef) -> Self {
        Self {
            resref,
            expiration_time_days: 0,
            charges_1: 0,
            charges_2: 0,
            charges_3: 0,
            flags: CRE_ITEM_IDENTIFIED_FLAG,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotChoice {
    AutoInventory,
    Index(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberSelector {
    Index(usize),
    CreResRef(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct AddItemResult {
    pub item_resref: String,
    pub member_index: Option<usize>,
    pub member_name: Option<String>,
    pub slot_index: usize,
    pub new_item_index: u32,
    pub old_items_count: u32,
    pub new_items_count: u32,
    pub byte_delta: usize,
    #[serde(skip)]
    pub bytes: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum SaveWriteError {
    #[error("invalid save item write: {0}")]
    InvalidWrite(String),
    #[error(transparent)]
    Parse(#[from] SaveParseError),
    #[error("resource parsing failure: {0}")]
    Format(String),
}

impl From<crate::FormatError> for SaveWriteError {
    fn from(err: crate::FormatError) -> Self {
        Self::Format(err.to_string())
    }
}

impl From<SaveWriteError> for crate::FormatError {
    fn from(err: SaveWriteError) -> Self {
        crate::FormatError::Parse(err.to_string())
    }
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
        return Err(
            SaveParseError::InvalidHeader(format!("unsupported GAM version {version}")).into(),
        );
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
            party_inventory_offset: parse_u32(bytes, 0x28)?,
            party_inventory_count: parse_u32(bytes, 0x2C)?,
            non_party_members_offset: parse_u32(bytes, 0x30)?,
            non_party_members_count: parse_u32(bytes, 0x34)?,
            globals_offset,
            globals_count,
            journal_entries_offset,
            journal_entries_count,
            unknown_section_68_offset: parse_u32(bytes, 0x68)?,
            unknown_section_6c_offset: parse_u32(bytes, 0x6C)?,
            unknown_section_78_offset: parse_u32(bytes, 0x78)?,
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
        return Err(
            SaveParseError::InvalidHeader(format!("unsupported SAV version {version}")).into(),
        );
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
        let compressed_end = checked_end(
            offset,
            compressed_size as usize,
            bytes.len(),
            "SAV compressed data",
        )?;
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
            .map(|(_, ext)| {
                (
                    Some(filename.to_ascii_uppercase()),
                    Some(ext.to_ascii_uppercase()),
                )
            })
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

pub fn add_item_to_cre(
    cre: &[u8],
    game_variant: GameVariant,
    item: &NewItem,
    slot: SlotChoice,
) -> Result<AddItemResult, SaveWriteError> {
    validate_cre_for_item_write(cre)?;

    let before = crate::cre::parse_cre_with_variant(cre, "EMBEDDED.CRE", None, game_variant)?;
    let items_offset = parse_u32(cre, 0x02BC)? as usize;
    let old_items_count = parse_u32(cre, CRE_ITEMS_COUNT_OFFSET)?;
    let insertion = items_offset
        .checked_add(old_items_count as usize * CRE_ITEM_SIZE)
        .ok_or_else(|| SaveWriteError::InvalidWrite("CRE item insertion offset overflow".into()))?;
    checked_end(insertion, 0, cre.len(), "CRE item insertion")?;

    let slot_index = resolve_slot(&before.item_slots, game_variant, slot)?;
    let new_item_index = old_items_count;
    let mut output = Vec::with_capacity(cre.len() + CRE_ITEM_SIZE);
    output.extend_from_slice(&cre[..insertion]);
    output.extend_from_slice(&encode_cre_item(item));
    output.extend_from_slice(&cre[insertion..]);

    write_u32(&mut output, CRE_ITEMS_COUNT_OFFSET, old_items_count + 1)?;
    shift_cre_offsets(&mut output, insertion, CRE_ITEM_SIZE as u32)?;

    let new_item_slots_offset = parse_u32(&output, 0x02B8)? as usize;
    let slot_offset = new_item_slots_offset
        .checked_add(slot_index * 2)
        .ok_or_else(|| SaveWriteError::InvalidWrite("CRE slot offset overflow".into()))?;
    write_u16(&mut output, slot_offset, new_item_index as u16)?;

    let after = crate::cre::parse_cre_with_variant(&output, "EMBEDDED.CRE", None, game_variant)?;
    assert_added_item(&after, item, slot_index, new_item_index, old_items_count)?;

    Ok(AddItemResult {
        item_resref: item.resref.as_str().to_string(),
        member_index: None,
        member_name: None,
        slot_index,
        new_item_index,
        old_items_count,
        new_items_count: old_items_count + 1,
        byte_delta: CRE_ITEM_SIZE,
        bytes: output,
    })
}

pub fn add_item_to_save_gam(
    gam: &[u8],
    game_variant: GameVariant,
    member: MemberSelector,
    item: &NewItem,
    slot: SlotChoice,
) -> Result<AddItemResult, SaveWriteError> {
    let before = parse_gam(gam, "BALDUR.GAM", None)?;
    let member_index = resolve_member_index(&before, &member)?;
    let npc_base = (before.header.party_members_offset as usize)
        .checked_add(member_index * GAM_V2_NPC_SIZE)
        .ok_or_else(|| SaveWriteError::InvalidWrite("GAM NPC offset overflow".into()))?;
    checked_end(npc_base, GAM_V2_NPC_SIZE, gam.len(), "GAM NPC")?;

    let embedded_cre_offset = parse_u32(gam, npc_base + 0x04)? as usize;
    let embedded_cre_size = parse_u32(gam, npc_base + 0x08)? as usize;
    let embedded_cre_end = checked_end(
        embedded_cre_offset,
        embedded_cre_size,
        gam.len(),
        "embedded CRE",
    )?;
    let cre_before = &gam[embedded_cre_offset..embedded_cre_end];
    let mut cre_result = add_item_to_cre(cre_before, game_variant, item, slot)?;

    let mut output = Vec::with_capacity(gam.len() + CRE_ITEM_SIZE);
    output.extend_from_slice(&gam[..embedded_cre_offset]);
    output.extend_from_slice(&cre_result.bytes);
    output.extend_from_slice(&gam[embedded_cre_end..]);

    write_u32(
        &mut output,
        npc_base + 0x08,
        (embedded_cre_size + CRE_ITEM_SIZE) as u32,
    )?;
    shift_gam_offsets(&mut output, embedded_cre_end, CRE_ITEM_SIZE as u32)?;

    let after = parse_gam(&output, "BALDUR.GAM", None)?;
    let edited_after = &after.party_members[member_index];
    let edited_cre_start = edited_after.embedded_cre_offset as usize;
    let edited_cre_size = edited_after.embedded_cre_size as usize;
    let edited_cre_end = checked_end(
        edited_cre_start,
        edited_cre_size,
        output.len(),
        "edited embedded CRE",
    )?;
    let parsed_cre = crate::cre::parse_cre_with_variant(
        &output[edited_cre_start..edited_cre_end],
        "EMBEDDED.CRE",
        None,
        game_variant,
    )?;
    assert_added_item(
        &parsed_cre,
        item,
        cre_result.slot_index,
        cre_result.new_item_index,
        cre_result.old_items_count,
    )?;
    assert_eq!(
        before.header.globals_count, after.header.globals_count,
        "globals count changed during save item write"
    );
    assert_eq!(
        before.header.journal_entries_count, after.header.journal_entries_count,
        "journal count changed during save item write"
    );
    // Every embedded CRE offset (party AND non-party) must still point at parseable CRE
    // data after the edit. A stale/unshifted offset lands on non-CRE bytes and fails here,
    // catching offset-fixup mistakes that a header-only re-parse would miss.
    assert_embedded_cres_parse(&output, game_variant)?;

    cre_result.member_index = Some(member_index);
    cre_result.member_name = Some(before.party_members[member_index].character_name.clone());
    cre_result.bytes = output;
    Ok(cre_result)
}

fn assert_embedded_cres_parse(
    gam: &[u8],
    game_variant: GameVariant,
) -> Result<(), SaveWriteError> {
    for (table_field, count_field, label) in [(0x20usize, 0x24usize, "party"), (0x30, 0x34, "non-party")]
    {
        let table = parse_u32(gam, table_field)? as usize;
        let count = parse_u32(gam, count_field)? as usize;
        if count == 0 {
            continue;
        }
        checked_table_end(table, count, GAM_V2_NPC_SIZE, gam.len(), "GAM NPCs")?;
        for index in 0..count {
            let base = table + index * GAM_V2_NPC_SIZE;
            let cre_offset = parse_u32(gam, base + 0x04)? as usize;
            let cre_size = parse_u32(gam, base + 0x08)? as usize;
            let cre_end = checked_end(cre_offset, cre_size, gam.len(), "embedded CRE")?;
            crate::cre::parse_cre_with_variant(
                &gam[cre_offset..cre_end],
                "EMBEDDED.CRE",
                None,
                game_variant,
            )
            .map_err(|err| {
                SaveWriteError::InvalidWrite(format!(
                    "post-write validation: {label} member {index} embedded CRE at offset \
                     {cre_offset} failed to parse: {err}"
                ))
            })?;
        }
    }
    Ok(())
}

fn validate_cre_for_item_write(cre: &[u8]) -> Result<(), SaveWriteError> {
    if cre.len() < CRE_HEADER_SIZE {
        return Err(SaveWriteError::InvalidWrite(format!(
            "CRE resource must contain at least {CRE_HEADER_SIZE} bytes"
        )));
    }
    if &cre[0..4] != b"CRE " {
        return Err(SaveWriteError::InvalidWrite(
            "embedded creature is missing CRE signature".to_string(),
        ));
    }
    let version = parse_ascii_string(cre, 4, 4)?;
    if version != "V1.0" {
        return Err(SaveWriteError::InvalidWrite(format!(
            "save item write supports embedded CRE V1.0 only, got {version}"
        )));
    }
    Ok(())
}

fn encode_cre_item(item: &NewItem) -> [u8; CRE_ITEM_SIZE] {
    let mut bytes = [0u8; CRE_ITEM_SIZE];
    let resref = item.resref.as_str().as_bytes();
    bytes[..resref.len()].copy_from_slice(resref);
    bytes[8..10].copy_from_slice(&item.expiration_time_days.to_le_bytes());
    bytes[10..12].copy_from_slice(&item.charges_1.to_le_bytes());
    bytes[12..14].copy_from_slice(&item.charges_2.to_le_bytes());
    bytes[14..16].copy_from_slice(&item.charges_3.to_le_bytes());
    bytes[16..20].copy_from_slice(&item.flags.to_le_bytes());
    bytes
}

fn resolve_slot(
    slots: &[crate::cre::CreatureItemSlotJson],
    game_variant: GameVariant,
    choice: SlotChoice,
) -> Result<usize, SaveWriteError> {
    match choice {
        SlotChoice::Index(index) => {
            let slot = slots.get(index).ok_or_else(|| {
                SaveWriteError::InvalidWrite(format!("slot index {index} is outside slot table"))
            })?;
            if slot.item_index.is_some() {
                return Err(SaveWriteError::InvalidWrite(format!(
                    "slot index {index} is not empty"
                )));
            }
            Ok(index)
        }
        SlotChoice::AutoInventory => {
            let range = inventory_slot_range(game_variant)?;
            let end = range.end.min(slots.len().saturating_sub(2));
            (range.start..end)
                .find(|index| {
                    slots
                        .get(*index)
                        .is_some_and(|slot| slot.item_index.is_none())
                })
                .ok_or_else(|| {
                    SaveWriteError::InvalidWrite(
                        "no empty general-inventory slot is available".to_string(),
                    )
                })
        }
    }
}

fn inventory_slot_range(
    game_variant: GameVariant,
) -> Result<std::ops::Range<usize>, SaveWriteError> {
    match game_variant {
        // PST/PSTEE CREs use slots 20..39 for backpack inventory cells; trailing
        // selected_weapon markers are never auto-targeted by resolve_slot.
        GameVariant::Pst => Ok(PST_INVENTORY_SLOT_START..PST_INVENTORY_SLOT_END_EXCLUSIVE),
        GameVariant::Standard => Err(SaveWriteError::InvalidWrite(
            "auto inventory slot selection is verified for PST saves only; pass --slot INDEX"
                .to_string(),
        )),
    }
}

fn shift_cre_offsets(bytes: &mut [u8], insertion: usize, delta: u32) -> Result<(), SaveWriteError> {
    for offset_field in [0x02A0, 0x02A8, 0x02B0, 0x02B8, 0x02C4] {
        add_to_offset_if_at_or_after(bytes, offset_field, insertion, delta)?;
    }
    Ok(())
}

fn shift_gam_offsets(output: &mut [u8], insertion: usize, delta: u32) -> Result<(), SaveWriteError> {
    for offset_field in GAM_V2_HEADER_OFFSET_FIELDS {
        add_to_offset_if_at_or_after(output, offset_field, insertion, delta)?;
    }

    // Read each NPC table's location from `output`, not the pre-edit buffer: the header
    // loop above has already advanced any table whose own offset sits past the insertion
    // point (e.g. the non-party table, which physically moved when the 20 bytes were
    // spliced in). Using the stale pre-edit offset here would edit the wrong bytes —
    // failing to shift the real embedded_cre_offsets while corrupting unrelated data.
    let party_offset = parse_u32(output, 0x20)? as usize;
    let party_count = parse_u32(output, 0x24)? as usize;
    shift_npc_embedded_cre_offsets(output, party_offset, party_count, insertion, delta)?;

    let non_party_offset = parse_u32(output, 0x30)? as usize;
    let non_party_count = parse_u32(output, 0x34)? as usize;
    shift_npc_embedded_cre_offsets(output, non_party_offset, non_party_count, insertion, delta)?;
    Ok(())
}

fn shift_npc_embedded_cre_offsets(
    bytes: &mut [u8],
    table_offset: usize,
    count: usize,
    insertion: usize,
    delta: u32,
) -> Result<(), SaveWriteError> {
    if count == 0 {
        return Ok(());
    }
    checked_table_end(
        table_offset,
        count,
        GAM_V2_NPC_SIZE,
        bytes.len(),
        "GAM NPCs",
    )?;
    for index in 0..count {
        add_to_offset_if_at_or_after(
            bytes,
            table_offset + index * GAM_V2_NPC_SIZE + 0x04,
            insertion,
            delta,
        )?;
    }
    Ok(())
}

fn add_to_offset_if_at_or_after(
    bytes: &mut [u8],
    field_offset: usize,
    insertion: usize,
    delta: u32,
) -> Result<(), SaveWriteError> {
    let value = parse_u32(bytes, field_offset)?;
    if value as usize >= insertion {
        write_u32(
            bytes,
            field_offset,
            value
                .checked_add(delta)
                .ok_or_else(|| SaveWriteError::InvalidWrite("offset overflow".into()))?,
        )?;
    }
    Ok(())
}

fn resolve_member_index(
    gam: &GameStateJson,
    member: &MemberSelector,
) -> Result<usize, SaveWriteError> {
    match member {
        MemberSelector::Index(index) => {
            if *index < gam.party_members.len() {
                Ok(*index)
            } else {
                Err(SaveWriteError::InvalidWrite(format!(
                    "party member index {index} is outside party size {}",
                    gam.party_members.len()
                )))
            }
        }
        MemberSelector::CreResRef(resref) => gam
            .party_members
            .iter()
            .position(|npc| npc.character_name.eq_ignore_ascii_case(resref))
            .ok_or_else(|| {
                SaveWriteError::InvalidWrite(format!("party member {resref} was not found"))
            }),
    }
}

fn assert_added_item(
    creature: &crate::cre::CreatureJson,
    item: &NewItem,
    slot_index: usize,
    item_index: u32,
    old_items_count: u32,
) -> Result<(), SaveWriteError> {
    if creature.header.items_count != old_items_count + 1 {
        return Err(SaveWriteError::InvalidWrite(
            "round-trip check failed: items_count did not increment by one".to_string(),
        ));
    }
    let parsed_item = creature.items.get(item_index as usize).ok_or_else(|| {
        SaveWriteError::InvalidWrite(
            "round-trip check failed: inserted item index is missing".to_string(),
        )
    })?;
    if parsed_item.resource.as_ref().map(ResRef::as_str) != Some(item.resref.as_str())
        || parsed_item.expiration_time_days != item.expiration_time_days
        || parsed_item.charges_1 != item.charges_1
        || parsed_item.charges_2 != item.charges_2
        || parsed_item.charges_3 != item.charges_3
        || parsed_item.flags.raw != item.flags
    {
        return Err(SaveWriteError::InvalidWrite(
            "round-trip check failed: inserted item fields differ".to_string(),
        ));
    }
    let slot = creature.item_slots.get(slot_index).ok_or_else(|| {
        SaveWriteError::InvalidWrite("round-trip check failed: chosen slot is missing".to_string())
    })?;
    if slot.item_index != Some(item_index as u16) {
        return Err(SaveWriteError::InvalidWrite(
            "round-trip check failed: chosen slot does not point at inserted item".to_string(),
        ));
    }
    Ok(())
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

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) -> Result<(), SaveWriteError> {
    let end = checked_end(offset, 2, bytes.len(), "u16")?;
    bytes[offset..end].copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) -> Result<(), SaveWriteError> {
    let end = checked_end(offset, 4, bytes.len(), "u32")?;
    bytes[offset..end].copy_from_slice(&value.to_le_bytes());
    Ok(())
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
    use std::path::PathBuf;

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

    #[test]
    fn add_item_to_cre_appends_item_shifts_slots_and_sets_slot() {
        let cre = build_test_cre(1, 42);
        let item = NewItem {
            resref: ResRef::new("CUBE").expect("resref should be valid"),
            expiration_time_days: 0,
            charges_1: 3,
            charges_2: 4,
            charges_3: 5,
            flags: 1,
        };

        let result = add_item_to_cre(&cre, GameVariant::Pst, &item, SlotChoice::AutoInventory)
            .expect("CRE item insert should work");

        assert_eq!(result.byte_delta, 20);
        assert_eq!(result.slot_index, 20);
        assert_eq!(result.old_items_count, 1);
        assert_eq!(result.new_items_count, 2);
        assert_eq!(parse_u32(&result.bytes, 0x02BC).unwrap(), 0x2D4);
        assert_eq!(parse_u32(&result.bytes, 0x02C0).unwrap(), 2);
        assert_eq!(parse_u32(&result.bytes, 0x02B8).unwrap(), 0x2D4 + 40);
        assert_eq!(parse_u16(&result.bytes, 0x2D4 + 40 + 20 * 2).unwrap(), 1);
        assert_eq!(&result.bytes[0x2D4 + 20..0x2D4 + 24], b"CUBE");
        assert_byte_exact_after_insert(
            &cre,
            &result.bytes,
            0x2D4 + 20,
            &[
                0x02A0..0x02A4,
                0x02A8..0x02AC,
                0x02B0..0x02B4,
                0x02B8..0x02BC,
                0x02C0..0x02C4,
                0x02C4..0x02C8,
                (0x2D4 + 20 + 20 * 2)..(0x2D4 + 20 + 20 * 2 + 2),
            ],
        );
    }

    #[test]
    fn add_item_to_cre_shifts_equal_boundary_offsets_but_keeps_items_offset() {
        let cre = build_test_cre(0, 42);
        let item = NewItem::identified(ResRef::new("CUBE").expect("resref should be valid"));

        let result = add_item_to_cre(&cre, GameVariant::Pst, &item, SlotChoice::AutoInventory)
            .expect("CRE item insert should work");

        assert_eq!(parse_u32(&result.bytes, 0x02BC).unwrap(), 0x2D4);
        assert_eq!(parse_u32(&result.bytes, 0x02B8).unwrap(), 0x2D4 + 20);
        assert_eq!(parse_u16(&result.bytes, 0x2D4 + 20 + 20 * 2).unwrap(), 0);
    }

    #[test]
    fn add_item_to_save_gam_updates_embedded_cre_size_and_later_offsets() {
        let cre1 = build_test_cre(0, 42);
        let cre2 = build_test_cre(0, 42);
        let cre3 = build_test_cre(0, 42);
        let gam = build_test_gam_with_creatures(&cre1, &cre2, &cre3);
        let item = NewItem::identified(ResRef::new("CUBE").expect("resref should be valid"));
        let party_offset = GAM_V2_HEADER_SIZE;
        let cre1_offset = party_offset + 2 * GAM_V2_NPC_SIZE;
        let cre2_offset = cre1_offset + cre1.len();
        let globals_offset = cre2_offset + cre2.len();
        let nonparty_offset = globals_offset + GAM_V2_VARIABLE_SIZE;
        let cre3_offset = nonparty_offset + GAM_V2_NPC_SIZE;
        let tail_offset = cre3_offset + cre3.len();
        let insertion = cre1_offset + 0x2D4;

        let result = add_item_to_save_gam(
            &gam,
            GameVariant::Pst,
            MemberSelector::Index(0),
            &item,
            SlotChoice::AutoInventory,
        )
        .expect("GAM item insert should work");

        let npc0 = party_offset;
        let npc1 = party_offset + GAM_V2_NPC_SIZE;
        // edited member 0: CRE starts before the insertion so its offset is unchanged; only
        // its size grows by 20.
        assert_eq!(
            parse_u32(&result.bytes, npc0 + 0x04).unwrap(),
            cre1_offset as u32
        );
        assert_eq!(
            parse_u32(&result.bytes, npc0 + 0x08).unwrap(),
            (cre1.len() + 20) as u32
        );
        // member 1 CRE sits after the insertion -> shifted.
        assert_eq!(
            parse_u32(&result.bytes, npc1 + 0x04).unwrap(),
            (cre2_offset + 20) as u32
        );
        // header section offsets after the insertion shift by 20.
        assert_eq!(
            parse_u32(&result.bytes, 0x38).unwrap(),
            (globals_offset + 20) as u32
        );
        assert_eq!(
            parse_u32(&result.bytes, 0x30).unwrap(),
            (nonparty_offset + 20) as u32
        );
        assert_eq!(
            parse_u32(&result.bytes, 0x28).unwrap(),
            (tail_offset + 20) as u32
        );
        assert_eq!(
            parse_u32(&result.bytes, 0x68).unwrap(),
            (tail_offset + 20) as u32
        );
        assert_eq!(
            parse_u32(&result.bytes, 0x6C).unwrap(),
            (tail_offset + 36) as u32
        );
        assert_eq!(
            parse_u32(&result.bytes, 0x78).unwrap(),
            (tail_offset + 72) as u32
        );
        // Regression guard for the crash: the NON-PARTY creature's table itself moved, so
        // its embedded_cre_offset must be read+shifted at the NEW table location.
        let new_nonparty = nonparty_offset + 20;
        assert_eq!(
            parse_u32(&result.bytes, new_nonparty + 0x04).unwrap(),
            (cre3_offset + 20) as u32
        );

        assert_byte_exact_after_insert(
            &gam,
            &result.bytes,
            insertion,
            &[
                0x28..0x2C,
                0x30..0x34,
                0x38..0x3C,
                0x50..0x54,
                0x68..0x6C,
                0x6C..0x70,
                0x78..0x7C,
                (npc0 + 0x08)..(npc0 + 0x0C),
                (npc1 + 0x04)..(npc1 + 0x08),
                // non-party embedded_cre_offset, in pre-edit (folded) coordinates.
                (nonparty_offset + 0x04)..(nonparty_offset + 0x08),
                (cre1_offset + 0x02A0)..(cre1_offset + 0x02A4),
                (cre1_offset + 0x02A8)..(cre1_offset + 0x02AC),
                (cre1_offset + 0x02B0)..(cre1_offset + 0x02B4),
                (cre1_offset + 0x02B8)..(cre1_offset + 0x02BC),
                (cre1_offset + 0x02C0)..(cre1_offset + 0x02C4),
                (cre1_offset + 0x02C4)..(cre1_offset + 0x02C8),
                (cre1_offset + 0x2D4 + 20 * 2)..(cre1_offset + 0x2D4 + 20 * 2 + 2),
            ],
        );

        let parsed = parse_gam(&result.bytes, "BALDUR.GAM", None).expect("GAM should reparse");
        assert_eq!(parsed.party_members.len(), 2);
        assert_eq!(parsed.globals.len(), 1);
    }

    #[test]
    fn add_item_to_real_pstee_quick_save_uses_observed_inventory_cell_when_paths_set() {
        let Some(saves_dir) = pstee_saves_dir_from_env() else {
            return;
        };
        let gam_path = saves_dir.join("000000001-Quick-Save-4").join("BALDUR.gam");
        if !gam_path.is_file() {
            return;
        }

        let gam = std::fs::read(&gam_path).expect("real PSTEE GAM should be readable");
        let parsed = parse_gam(&gam, "BALDUR.GAM", None).expect("real PSTEE GAM should parse");
        let protagonist = parsed
            .party_members
            .first()
            .expect("real PSTEE save should have a protagonist");
        let cre_start = protagonist.embedded_cre_offset as usize;
        let cre_end = cre_start + protagonist.embedded_cre_size as usize;
        let cre = gam
            .get(cre_start..cre_end)
            .expect("embedded protagonist CRE should be in bounds");
        let creature = crate::cre::parse_cre_with_variant(cre, "TNO.CRE", None, GameVariant::Pst)
            .expect("embedded protagonist CRE should parse");

        assert!(
            creature.item_slots.len() >= PST_INVENTORY_SLOT_END_EXCLUSIVE + 2,
            "real PSTEE slot table should contain the documented inventory range plus markers"
        );
        assert!(
            creature.item_slots[PST_INVENTORY_SLOT_START..33]
                .iter()
                .all(|slot| slot.item_index.is_some()),
            "Quick-Save-4 documents slots 20..32 as occupied inventory cells"
        );
        assert_eq!(
            creature.item_slots[33].item_index, None,
            "Quick-Save-4 documents slot 33 as the first empty inventory cell"
        );

        let result = add_item_to_cre(
            cre,
            GameVariant::Pst,
            &NewItem::identified(ResRef::new("CUBE").expect("resref should be valid")),
            SlotChoice::AutoInventory,
        )
        .expect("real PSTEE auto inventory insert should work");
        assert_eq!(result.slot_index, 33);

        let save_result = add_item_to_save_gam(
            &gam,
            GameVariant::Pst,
            MemberSelector::Index(0),
            &NewItem::identified(ResRef::new("CUBE").expect("resref should be valid")),
            SlotChoice::AutoInventory,
        )
        .expect("real PSTEE save insert should repair tail offsets");
        for offset_field in [0x68, 0x6C, 0x78] {
            assert_eq!(
                parse_u32(&save_result.bytes, offset_field).unwrap(),
                parse_u32(&gam, offset_field).unwrap() + CRE_ITEM_SIZE as u32,
                "real PSTEE GAM header offset 0x{offset_field:X} should shift"
            );
        }
    }

    fn zlib(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(bytes)
            .expect("test payload should compress");
        encoder.finish().expect("test payload should finish")
    }

    fn build_test_cre(items_count: u32, slot_count: usize) -> Vec<u8> {
        let items_offset = CRE_HEADER_SIZE;
        let item_slots_offset = items_offset + items_count as usize * CRE_ITEM_SIZE;
        let mut bytes = vec![0u8; item_slots_offset + slot_count * 2];
        bytes[0..8].copy_from_slice(b"CRE V1.0");
        bytes[0x33] = 1;
        write_u32(&mut bytes, 0x02A0, item_slots_offset as u32);
        write_u32(&mut bytes, 0x02A8, item_slots_offset as u32);
        write_u32(&mut bytes, 0x02B0, item_slots_offset as u32);
        write_u32(&mut bytes, 0x02B8, item_slots_offset as u32);
        write_u32(&mut bytes, 0x02BC, items_offset as u32);
        write_u32(&mut bytes, 0x02C0, items_count);
        write_u32(&mut bytes, 0x02C4, item_slots_offset as u32);

        for index in 0..items_count as usize {
            let item = items_offset + index * CRE_ITEM_SIZE;
            bytes[item..item + 3].copy_from_slice(b"OLD");
        }
        for slot in 0..slot_count {
            write_u16(&mut bytes, item_slots_offset + slot * 2, u16::MAX);
        }
        if items_count > 0 {
            write_u16(&mut bytes, item_slots_offset, 0);
        }
        bytes
    }

    // Layout mirrors a real PSTEE GAM closely enough to exercise the offset fix-up:
    // two party members (their CREs early in the file), then globals, then a *non-party*
    // creature whose table AND embedded CRE both sit after the edited member's CRE — so
    // both must shift when 20 bytes are spliced in. The non-party section is the case
    // that a party-only fixture silently skipped.
    fn build_test_gam_with_creatures(cre1: &[u8], cre2: &[u8], cre3: &[u8]) -> Vec<u8> {
        let party_offset = GAM_V2_HEADER_SIZE;
        let cre1_offset = party_offset + 2 * GAM_V2_NPC_SIZE;
        let cre2_offset = cre1_offset + cre1.len();
        let globals_offset = cre2_offset + cre2.len();
        let nonparty_offset = globals_offset + GAM_V2_VARIABLE_SIZE;
        let cre3_offset = nonparty_offset + GAM_V2_NPC_SIZE;
        let tail_offset = cre3_offset + cre3.len();
        let mut bytes = vec![0u8; tail_offset + 52];
        bytes[0..8].copy_from_slice(b"GAMEV2.0");
        write_u32(&mut bytes, 0x20, party_offset as u32);
        write_u32(&mut bytes, 0x24, 2);
        write_u32(&mut bytes, 0x28, tail_offset as u32);
        write_u32(&mut bytes, 0x2C, 1);
        write_u32(&mut bytes, 0x30, nonparty_offset as u32);
        write_u32(&mut bytes, 0x34, 1);
        write_u32(&mut bytes, 0x38, globals_offset as u32);
        write_u32(&mut bytes, 0x3C, 1);
        write_u32(&mut bytes, 0x50, globals_offset as u32);
        write_u32(&mut bytes, 0x4C, 0);
        write_u32(&mut bytes, 0x68, tail_offset as u32);
        write_u32(&mut bytes, 0x6C, (tail_offset + 16) as u32);
        write_u32(&mut bytes, 0x78, (tail_offset + 52) as u32);

        write_u32(&mut bytes, party_offset + 0x04, cre1_offset as u32);
        write_u32(&mut bytes, party_offset + 0x08, cre1.len() as u32);
        bytes[party_offset + 0x0C..party_offset + 0x0F].copy_from_slice(b"TNO");
        write_u32(
            &mut bytes,
            party_offset + GAM_V2_NPC_SIZE + 0x04,
            cre2_offset as u32,
        );
        write_u32(
            &mut bytes,
            party_offset + GAM_V2_NPC_SIZE + 0x08,
            cre2.len() as u32,
        );
        bytes[party_offset + GAM_V2_NPC_SIZE + 0x0C..party_offset + GAM_V2_NPC_SIZE + 0x12]
            .copy_from_slice(b"DAKKON");
        // non-party creature (e.g. a world actor) — table + CRE both after the insertion
        write_u32(&mut bytes, nonparty_offset + 0x04, cre3_offset as u32);
        write_u32(&mut bytes, nonparty_offset + 0x08, cre3.len() as u32);
        bytes[nonparty_offset + 0x0C..nonparty_offset + 0x13].copy_from_slice(b"VHAILOR");
        bytes[cre1_offset..cre1_offset + cre1.len()].copy_from_slice(cre1);
        bytes[cre2_offset..cre2_offset + cre2.len()].copy_from_slice(cre2);
        bytes[cre3_offset..cre3_offset + cre3.len()].copy_from_slice(cre3);
        bytes[globals_offset..globals_offset + 7].copy_from_slice(b"CHAPTER");
        write_u16(&mut bytes, globals_offset + 0x20, 1);
        write_u32(&mut bytes, globals_offset + 0x28, 1);
        bytes[tail_offset..tail_offset + 16].copy_from_slice(b"PARTY-INVENTORY.");
        bytes[tail_offset + 16..tail_offset + 32].copy_from_slice(b"PST-TAIL-ONE....");
        bytes[tail_offset + 32..tail_offset + 52].copy_from_slice(b"PST-TAIL-TWO........");
        bytes
    }

    fn assert_byte_exact_after_insert(
        original: &[u8],
        output: &[u8],
        insertion: usize,
        changed_ranges: &[std::ops::Range<usize>],
    ) {
        let mut folded = output.to_vec();
        folded.drain(insertion..insertion + CRE_ITEM_SIZE);
        for range in changed_ranges {
            folded[range.clone()].copy_from_slice(&original[range.clone()]);
        }
        assert_eq!(
            folded, original,
            "output differs outside inserted item bytes and declared adjusted fields"
        );
    }

    fn pstee_saves_dir_from_env() -> Option<PathBuf> {
        std::env::var_os("IE_PSTEE_SAVES")
            .or_else(|| std::env::var_os("IE_PSTEE_SAVES_PATH"))
            .map(PathBuf::from)
    }

    fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}
