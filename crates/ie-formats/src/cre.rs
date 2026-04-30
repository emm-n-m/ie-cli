use ie_core::{ResRef, ResolvedStrRef, ResourceType, StrRef, StrRefResolver};
use serde::Serialize;
use thiserror::Error;

const CRE_HEADER_SIZE: usize = 0x2D4;
const CRE_KNOWN_SPELL_SIZE: usize = 12;
const CRE_MEMORIZATION_INFO_SIZE: usize = 16;
const CRE_MEMORIZED_SPELL_SIZE: usize = 12;
const CRE_EFFECT_V1_SIZE: usize = 48;
const CRE_EFFECT_V2_SIZE: usize = 0x108;
const CRE_ITEM_SIZE: usize = 20;
const CRE_SOUNDSET_COUNT: usize = 100;

#[derive(Debug, Clone, Serialize)]
pub struct CreatureJson {
    pub resource_type: String,
    pub resource_name: String,
    pub version: String,
    pub header: CreatureHeaderJson,
    pub known_spells: Vec<CreatureKnownSpellJson>,
    pub spell_memorization: Vec<CreatureSpellMemorizationJson>,
    pub memorized_spells: Vec<CreatureMemorizedSpellJson>,
    pub effects: Vec<CreatureEffectJson>,
    pub items: Vec<CreatureItemJson>,
    pub item_slots: Vec<CreatureItemSlotJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureHeaderJson {
    pub long_name: ResolvedStrRef,
    pub short_name: ResolvedStrRef,
    pub flags: RawDecodedFlags,
    pub xp_for_kill: u32,
    pub power_level_or_party_xp: u32,
    pub gold_carried: u32,
    pub status_flags: RawDecodedFlags,
    pub current_hit_points: u16,
    pub max_hit_points: u16,
    pub animation_id: u32,
    pub colors: CreatureColorsJson,
    pub effect_structure_version: RawDecoded<u8>,
    pub small_portrait: Option<ResRef>,
    pub large_portrait: Option<ResRef>,
    pub reputation: i8,
    pub hide_in_shadows: u8,
    pub armor_class: CreatureArmorClassJson,
    pub thac0: u8,
    pub attacks_per_round: u8,
    pub saving_throws: CreatureSavingThrowsJson,
    pub resistances: CreatureResistancesJson,
    pub thief_skills: CreatureThiefSkillsJson,
    pub fatigue: u8,
    pub intoxication: u8,
    pub luck: u8,
    pub proficiency_bytes: Vec<u8>,
    pub turn_undead_level: u8,
    pub tracking_skill: u8,
    pub tracking_target: String,
    pub shield_flags: RawDecodedFlags,
    pub field_of_vision: u8,
    pub attribute_flags: RawDecodedFlags,
    pub soundset: Vec<ResolvedStrRef>,
    pub class_levels: CreatureClassLevelsJson,
    pub base_attributes: CreatureBaseAttributesJson,
    pub morale: u8,
    pub morale_break: u8,
    pub racial_enemy: u8,
    pub morale_recovery_time: u16,
    pub kit: CreatureKitJson,
    pub scripts: CreatureScriptsJson,
    pub classification: CreatureClassificationJson,
    pub object_references: Vec<u8>,
    pub actor_ids: CreatureActorIdsJson,
    pub death_variable: String,
    pub known_spells_offset: u32,
    pub known_spells_count: u32,
    pub spell_memorization_offset: u32,
    pub spell_memorization_count: u32,
    pub memorized_spells_offset: u32,
    pub memorized_spells_count: u32,
    pub item_slots_offset: u32,
    pub items_offset: u32,
    pub items_count: u32,
    pub effects_offset: u32,
    pub effects_count: u32,
    pub dialog: Option<ResRef>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureColorsJson {
    pub metal: u8,
    pub minor: u8,
    pub major: u8,
    pub skin: u8,
    pub leather: u8,
    pub armor: u8,
    pub hair: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureArmorClassJson {
    pub natural: i16,
    pub effective: i16,
    pub crushing_modifier: i16,
    pub missile_modifier: i16,
    pub piercing_modifier: i16,
    pub slashing_modifier: i16,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureSavingThrowsJson {
    pub death: u8,
    pub wands: u8,
    pub polymorph: u8,
    pub breath: u8,
    pub spells: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureResistancesJson {
    pub fire: u8,
    pub cold: u8,
    pub electricity: u8,
    pub acid: u8,
    pub magic: u8,
    pub magic_fire: u8,
    pub magic_cold: u8,
    pub slashing: u8,
    pub crushing: u8,
    pub piercing: u8,
    pub missile: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureThiefSkillsJson {
    pub detect_illusion: u8,
    pub set_traps: u8,
    pub lore: u8,
    pub lockpicking: u8,
    pub move_silently: u8,
    pub find_traps: u8,
    pub pick_pockets: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureClassLevelsJson {
    pub first_class: u8,
    pub second_class: u8,
    pub third_class: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureBaseAttributesJson {
    pub strength: u8,
    pub strength_bonus: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub charisma: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureKitJson {
    pub raw_bytes: Vec<u8>,
    pub raw_big_endian: u32,
    pub decoded: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureScriptsJson {
    pub override_script: Option<ResRef>,
    pub class_script: Option<ResRef>,
    pub race_script: Option<ResRef>,
    pub general_script: Option<ResRef>,
    pub default_script: Option<ResRef>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureClassificationJson {
    pub enemy_ally: u8,
    pub general: u8,
    pub race: u8,
    pub class: u8,
    pub specific: u8,
    pub gender: u8,
    pub alignment: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureActorIdsJson {
    pub global_actor_id: u16,
    pub local_actor_id: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureKnownSpellJson {
    pub resource: Option<ResRef>,
    pub spell_level_minus_one: u16,
    pub spell_type: RawDecoded<u16>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureSpellMemorizationJson {
    pub spell_level_minus_one: u16,
    pub memorisable_count: u16,
    pub memorisable_count_after_effects: u16,
    pub spell_type: RawDecoded<u16>,
    pub first_memorized_spell_index: u32,
    pub memorized_spell_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureMemorizedSpellJson {
    pub resource: Option<ResRef>,
    pub flags: RawDecodedFlags,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureEffectJson {
    pub version: String,
    pub opcode: RawDecoded<u32>,
    pub target_type: RawDecoded<u32>,
    pub power: u32,
    pub parameter1: u32,
    pub parameter2: u32,
    pub timing: RawDecoded<u32>,
    pub duration: u32,
    pub probability_1: u16,
    pub probability_2: u16,
    pub resource: Option<ResRef>,
    pub dice_thrown_or_max_level: u32,
    pub dice_sides_or_min_level: u32,
    pub saving_throw_type: u32,
    pub saving_throw_bonus: i32,
    pub special: u32,
    pub dispel_resistance: u32,
    pub minimum_level: Option<u32>,
    pub maximum_level: Option<u32>,
    pub raw_prefix_bytes: Vec<u8>,
    pub raw_tail_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureItemJson {
    pub resource: Option<ResRef>,
    pub expiration_time_days: u16,
    pub charges_1: u16,
    pub charges_2: u16,
    pub charges_3: u16,
    pub flags: RawDecodedFlags,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatureItemSlotJson {
    pub index: usize,
    pub name: String,
    pub item_index: Option<u16>,
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
pub enum CreatureParseError {
    #[error("invalid CRE header: {0}")]
    InvalidHeader(String),
    #[error("unexpected end of CRE file: {0}")]
    UnexpectedEof(String),
    #[error("invalid CRE field: {0}")]
    InvalidField(String),
}

impl From<CreatureParseError> for crate::FormatError {
    fn from(err: CreatureParseError) -> Self {
        crate::FormatError::Parse(err.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureScalarPatch {
    pub field: String,
    pub value: String,
}

#[derive(Debug, Error)]
pub enum CreaturePatchError {
    #[error("invalid CRE patch: {0}")]
    InvalidPatch(String),
    #[error("unknown CRE scalar field: {0}")]
    UnknownField(String),
    #[error("invalid CRE scalar value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },
    #[error(transparent)]
    Parse(#[from] CreatureParseError),
}

impl From<CreaturePatchError> for crate::FormatError {
    fn from(err: CreaturePatchError) -> Self {
        crate::FormatError::Parse(err.to_string())
    }
}

struct ByteEdit {
    offset: usize,
    bytes: Vec<u8>,
}

pub fn patch_cre_scalars(
    bytes: &[u8],
    patches: &[CreatureScalarPatch],
) -> Result<Vec<u8>, CreaturePatchError> {
    validate_cre_header(bytes)?;

    let edits = patches
        .iter()
        .map(resolve_scalar_patch)
        .collect::<Result<Vec<_>, _>>()?;

    let mut output = bytes.to_vec();
    for edit in edits {
        let end = edit.offset + edit.bytes.len();
        output
            .get_mut(edit.offset..end)
            .ok_or_else(|| {
                CreaturePatchError::InvalidPatch("patch exceeds CRE length".to_string())
            })?
            .copy_from_slice(&edit.bytes);
    }

    Ok(output)
}

fn resolve_scalar_patch(patch: &CreatureScalarPatch) -> Result<ByteEdit, CreaturePatchError> {
    let field = patch.field.trim().to_ascii_lowercase();
    match field.as_str() {
        "long_name" | "long_name_strref" => u32_edit(0x08, &field, &patch.value),
        "short_name" | "short_name_strref" => u32_edit(0x0C, &field, &patch.value),
        "reputation" => i8_edit(0x44, &field, &patch.value),
        "morale" => u8_edit(0x23F, &field, &patch.value),
        "morale_break" => u8_edit(0x240, &field, &patch.value),
        "morale_recovery_time" => u16_edit(0x242, &field, &patch.value),
        "override_script" => resref_edit(0x248, &field, &patch.value),
        "class_script" => resref_edit(0x250, &field, &patch.value),
        "race_script" => resref_edit(0x258, &field, &patch.value),
        "general_script" => resref_edit(0x260, &field, &patch.value),
        "default_script" => resref_edit(0x268, &field, &patch.value),
        "dialog" | "dialog_resref" => resref_edit(0x2CC, &field, &patch.value),
        _ => Err(CreaturePatchError::UnknownField(patch.field.clone())),
    }
}

fn validate_cre_header(bytes: &[u8]) -> Result<(), CreaturePatchError> {
    if bytes.len() < CRE_HEADER_SIZE {
        return Err(CreaturePatchError::Parse(
            CreatureParseError::UnexpectedEof(format!(
                "CRE resource must contain at least {} bytes",
                CRE_HEADER_SIZE
            )),
        ));
    }

    if &bytes[0..4] != b"CRE " {
        return Err(CreaturePatchError::Parse(
            CreatureParseError::InvalidHeader("missing CRE signature".to_string()),
        ));
    }

    Ok(())
}

fn u8_edit(offset: usize, field: &str, value: &str) -> Result<ByteEdit, CreaturePatchError> {
    let value = parse_u64_value(field, value)?;
    let value = u8::try_from(value).map_err(|_| CreaturePatchError::InvalidValue {
        field: field.to_string(),
        reason: "expected integer in range 0..=255".to_string(),
    })?;
    Ok(ByteEdit {
        offset,
        bytes: vec![value],
    })
}

fn i8_edit(offset: usize, field: &str, value: &str) -> Result<ByteEdit, CreaturePatchError> {
    let value = value
        .trim()
        .parse::<i16>()
        .map_err(|err| CreaturePatchError::InvalidValue {
            field: field.to_string(),
            reason: err.to_string(),
        })?;
    let value = i8::try_from(value).map_err(|_| CreaturePatchError::InvalidValue {
        field: field.to_string(),
        reason: "expected integer in range -128..=127".to_string(),
    })?;
    Ok(ByteEdit {
        offset,
        bytes: vec![value as u8],
    })
}

fn u16_edit(offset: usize, field: &str, value: &str) -> Result<ByteEdit, CreaturePatchError> {
    let value = parse_u64_value(field, value)?;
    let value = u16::try_from(value).map_err(|_| CreaturePatchError::InvalidValue {
        field: field.to_string(),
        reason: "expected integer in range 0..=65535".to_string(),
    })?;
    Ok(ByteEdit {
        offset,
        bytes: value.to_le_bytes().to_vec(),
    })
}

fn u32_edit(offset: usize, field: &str, value: &str) -> Result<ByteEdit, CreaturePatchError> {
    let value = parse_u64_value(field, value)?;
    let value = u32::try_from(value).map_err(|_| CreaturePatchError::InvalidValue {
        field: field.to_string(),
        reason: "expected integer in range 0..=4294967295".to_string(),
    })?;
    Ok(ByteEdit {
        offset,
        bytes: value.to_le_bytes().to_vec(),
    })
}

fn resref_edit(offset: usize, field: &str, value: &str) -> Result<ByteEdit, CreaturePatchError> {
    let trimmed = value.trim();
    let mut bytes = vec![0u8; 8];
    if !trimmed.is_empty() {
        let resref = ResRef::new(trimmed).map_err(|err| CreaturePatchError::InvalidValue {
            field: field.to_string(),
            reason: err.to_string(),
        })?;
        bytes[..resref.as_str().len()].copy_from_slice(resref.as_str().as_bytes());
    }
    Ok(ByteEdit { offset, bytes })
}

fn parse_u64_value(field: &str, value: &str) -> Result<u64, CreaturePatchError> {
    value
        .trim()
        .parse::<u64>()
        .map_err(|err| CreaturePatchError::InvalidValue {
            field: field.to_string(),
            reason: err.to_string(),
        })
}

pub fn parse_cre(
    bytes: &[u8],
    resource_name: &str,
    resolver: Option<&dyn StrRefResolver>,
) -> Result<CreatureJson, crate::FormatError> {
    if bytes.len() < CRE_HEADER_SIZE {
        return Err(CreatureParseError::UnexpectedEof(format!(
            "CRE resource must contain at least {} bytes",
            CRE_HEADER_SIZE
        ))
        .into());
    }

    if &bytes[0..4] != b"CRE " {
        return Err(CreatureParseError::InvalidHeader("missing CRE signature".to_string()).into());
    }

    let version = parse_ascii_string(bytes, 4, 4)?;
    let effect_structure_version = parse_u8(bytes, 0x33)?;
    let known_spells_offset = parse_u32(bytes, 0x02A0)?;
    let known_spells_count = parse_u32(bytes, 0x02A4)?;
    let spell_memorization_offset = parse_u32(bytes, 0x02A8)?;
    let spell_memorization_count = parse_u32(bytes, 0x02AC)?;
    let memorized_spells_offset = parse_u32(bytes, 0x02B0)?;
    let memorized_spells_count = parse_u32(bytes, 0x02B4)?;
    let item_slots_offset = parse_u32(bytes, 0x02B8)?;
    let items_offset = parse_u32(bytes, 0x02BC)?;
    let items_count = parse_u32(bytes, 0x02C0)?;
    let effects_offset = parse_u32(bytes, 0x02C4)?;
    let effects_count = parse_u32(bytes, 0x02C8)?;

    let soundset = parse_soundset(bytes, resolver)?;

    let header = CreatureHeaderJson {
        long_name: parse_strref(bytes, 0x08, resolver)?,
        short_name: parse_strref(bytes, 0x0C, resolver)?,
        flags: RawDecodedFlags {
            raw: parse_u32(bytes, 0x10)?,
            decoded: decode_creature_flags(parse_u32(bytes, 0x10)?),
        },
        xp_for_kill: parse_u32(bytes, 0x14)?,
        power_level_or_party_xp: parse_u32(bytes, 0x18)?,
        gold_carried: parse_u32(bytes, 0x1C)?,
        status_flags: RawDecodedFlags {
            raw: parse_u32(bytes, 0x20)?,
            decoded: decode_status_flags(parse_u32(bytes, 0x20)?),
        },
        current_hit_points: parse_u16(bytes, 0x24)?,
        max_hit_points: parse_u16(bytes, 0x26)?,
        animation_id: parse_u32(bytes, 0x28)?,
        colors: CreatureColorsJson {
            metal: parse_u8(bytes, 0x2C)?,
            minor: parse_u8(bytes, 0x2D)?,
            major: parse_u8(bytes, 0x2E)?,
            skin: parse_u8(bytes, 0x2F)?,
            leather: parse_u8(bytes, 0x30)?,
            armor: parse_u8(bytes, 0x31)?,
            hair: parse_u8(bytes, 0x32)?,
        },
        effect_structure_version: RawDecoded {
            raw: effect_structure_version,
            decoded: decode_effect_structure_version(effect_structure_version).map(str::to_string),
        },
        small_portrait: parse_resref_option(bytes, 0x34, 8)?,
        large_portrait: parse_resref_option(bytes, 0x3C, 8)?,
        reputation: parse_i8(bytes, 0x44)?,
        hide_in_shadows: parse_u8(bytes, 0x45)?,
        armor_class: CreatureArmorClassJson {
            natural: parse_i16(bytes, 0x46)?,
            effective: parse_i16(bytes, 0x48)?,
            crushing_modifier: parse_i16(bytes, 0x4A)?,
            missile_modifier: parse_i16(bytes, 0x4C)?,
            piercing_modifier: parse_i16(bytes, 0x4E)?,
            slashing_modifier: parse_i16(bytes, 0x50)?,
        },
        thac0: parse_u8(bytes, 0x52)?,
        attacks_per_round: parse_u8(bytes, 0x53)?,
        saving_throws: CreatureSavingThrowsJson {
            death: parse_u8(bytes, 0x54)?,
            wands: parse_u8(bytes, 0x55)?,
            polymorph: parse_u8(bytes, 0x56)?,
            breath: parse_u8(bytes, 0x57)?,
            spells: parse_u8(bytes, 0x58)?,
        },
        resistances: CreatureResistancesJson {
            fire: parse_u8(bytes, 0x59)?,
            cold: parse_u8(bytes, 0x5A)?,
            electricity: parse_u8(bytes, 0x5B)?,
            acid: parse_u8(bytes, 0x5C)?,
            magic: parse_u8(bytes, 0x5D)?,
            magic_fire: parse_u8(bytes, 0x5E)?,
            magic_cold: parse_u8(bytes, 0x5F)?,
            slashing: parse_u8(bytes, 0x60)?,
            crushing: parse_u8(bytes, 0x61)?,
            piercing: parse_u8(bytes, 0x62)?,
            missile: parse_u8(bytes, 0x63)?,
        },
        thief_skills: CreatureThiefSkillsJson {
            detect_illusion: parse_u8(bytes, 0x64)?,
            set_traps: parse_u8(bytes, 0x65)?,
            lore: parse_u8(bytes, 0x66)?,
            lockpicking: parse_u8(bytes, 0x67)?,
            move_silently: parse_u8(bytes, 0x68)?,
            find_traps: parse_u8(bytes, 0x69)?,
            pick_pockets: parse_u8(bytes, 0x6A)?,
        },
        fatigue: parse_u8(bytes, 0x6B)?,
        intoxication: parse_u8(bytes, 0x6C)?,
        luck: parse_u8(bytes, 0x6D)?,
        proficiency_bytes: bytes
            .get(0x6E..0x82)
            .ok_or_else(|| {
                CreatureParseError::UnexpectedEof(
                    "missing proficiency byte range at 0x6E..0x82".to_string(),
                )
            })?
            .to_vec(),
        turn_undead_level: parse_u8(bytes, 0x82)?,
        tracking_skill: parse_u8(bytes, 0x83)?,
        tracking_target: parse_char_array(bytes, 0x84, 32)?,
        shield_flags: RawDecodedFlags {
            raw: parse_u8(bytes, 0x94)? as u32,
            decoded: decode_shield_flags(parse_u8(bytes, 0x94)?),
        },
        field_of_vision: parse_u8(bytes, 0x95)?,
        attribute_flags: RawDecodedFlags {
            raw: parse_u32(bytes, 0x96)?,
            decoded: decode_attribute_flags(parse_u32(bytes, 0x96)?),
        },
        soundset,
        class_levels: CreatureClassLevelsJson {
            first_class: parse_u8(bytes, 0x234)?,
            second_class: parse_u8(bytes, 0x235)?,
            third_class: parse_u8(bytes, 0x236)?,
        },
        base_attributes: CreatureBaseAttributesJson {
            strength: parse_u8(bytes, 0x238)?,
            strength_bonus: parse_u8(bytes, 0x239)?,
            intelligence: parse_u8(bytes, 0x23A)?,
            wisdom: parse_u8(bytes, 0x23B)?,
            dexterity: parse_u8(bytes, 0x23C)?,
            constitution: parse_u8(bytes, 0x23D)?,
            charisma: parse_u8(bytes, 0x23E)?,
        },
        morale: parse_u8(bytes, 0x23F)?,
        morale_break: parse_u8(bytes, 0x240)?,
        racial_enemy: parse_u8(bytes, 0x241)?,
        morale_recovery_time: parse_u16(bytes, 0x242)?,
        kit: parse_kit(bytes)?,
        scripts: CreatureScriptsJson {
            override_script: parse_resref_option(bytes, 0x248, 8)?,
            class_script: parse_resref_option(bytes, 0x250, 8)?,
            race_script: parse_resref_option(bytes, 0x258, 8)?,
            general_script: parse_resref_option(bytes, 0x260, 8)?,
            default_script: parse_resref_option(bytes, 0x268, 8)?,
        },
        classification: CreatureClassificationJson {
            enemy_ally: parse_u8(bytes, 0x270)?,
            general: parse_u8(bytes, 0x271)?,
            race: parse_u8(bytes, 0x272)?,
            class: parse_u8(bytes, 0x273)?,
            specific: parse_u8(bytes, 0x274)?,
            gender: parse_u8(bytes, 0x275)?,
            alignment: parse_u8(bytes, 0x27B)?,
        },
        object_references: bytes
            .get(0x276..0x27B)
            .ok_or_else(|| {
                CreatureParseError::UnexpectedEof(
                    "missing object references at 0x276..0x27B".to_string(),
                )
            })?
            .to_vec(),
        actor_ids: CreatureActorIdsJson {
            global_actor_id: parse_u16(bytes, 0x27C)?,
            local_actor_id: parse_u16(bytes, 0x27E)?,
        },
        death_variable: parse_char_array(bytes, 0x280, 32)?,
        known_spells_offset,
        known_spells_count,
        spell_memorization_offset,
        spell_memorization_count,
        memorized_spells_offset,
        memorized_spells_count,
        item_slots_offset,
        items_offset,
        items_count,
        effects_offset,
        effects_count,
        dialog: parse_resref_option(bytes, 0x2CC, 8)?,
    };

    let known_spells = parse_known_spells(bytes, known_spells_offset, known_spells_count)?;
    let spell_memorization =
        parse_spell_memorization(bytes, spell_memorization_offset, spell_memorization_count)?;
    let memorized_spells =
        parse_memorized_spells(bytes, memorized_spells_offset, memorized_spells_count)?;
    let effects = parse_effects(
        bytes,
        effects_offset,
        effects_count,
        effect_structure_version,
    )?;
    let items = parse_items(bytes, items_offset, items_count)?;
    let item_slots = parse_item_slots(bytes, item_slots_offset, items_offset)?;

    Ok(CreatureJson {
        resource_type: ResourceType::Cre.as_str().to_string(),
        resource_name: resource_name.to_string(),
        version,
        header,
        known_spells,
        spell_memorization,
        memorized_spells,
        effects,
        items,
        item_slots,
    })
}

fn parse_soundset(
    bytes: &[u8],
    resolver: Option<&dyn StrRefResolver>,
) -> Result<Vec<ResolvedStrRef>, CreatureParseError> {
    let mut soundset = Vec::with_capacity(CRE_SOUNDSET_COUNT);
    for index in 0..CRE_SOUNDSET_COUNT {
        let offset = 0x00A4 + index * 4;
        soundset.push(parse_strref(bytes, offset, resolver)?);
    }
    Ok(soundset)
}

fn parse_kit(bytes: &[u8]) -> Result<CreatureKitJson, CreatureParseError> {
    let raw = bytes.get(0x244..0x248).ok_or_else(|| {
        CreatureParseError::UnexpectedEof("missing kit info at 0x244".to_string())
    })?;

    let raw_big_endian = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);
    Ok(CreatureKitJson {
        raw_bytes: raw.to_vec(),
        raw_big_endian,
        decoded: decode_kit(raw_big_endian).map(str::to_string),
    })
}

fn parse_known_spells(
    bytes: &[u8],
    offset: u32,
    count: u32,
) -> Result<Vec<CreatureKnownSpellJson>, CreatureParseError> {
    parse_table(
        bytes,
        offset,
        count,
        CRE_KNOWN_SPELL_SIZE,
        |bytes, position| {
            let spell_type = parse_u16(bytes, position + 10)?;
            Ok(CreatureKnownSpellJson {
                resource: parse_resref_option(bytes, position, 8)?,
                spell_level_minus_one: parse_u16(bytes, position + 8)?,
                spell_type: RawDecoded {
                    raw: spell_type,
                    decoded: decode_creature_spell_type(spell_type).map(str::to_string),
                },
            })
        },
    )
}

fn parse_spell_memorization(
    bytes: &[u8],
    offset: u32,
    count: u32,
) -> Result<Vec<CreatureSpellMemorizationJson>, CreatureParseError> {
    parse_table(
        bytes,
        offset,
        count,
        CRE_MEMORIZATION_INFO_SIZE,
        |bytes, position| {
            let spell_type = parse_u16(bytes, position + 6)?;
            Ok(CreatureSpellMemorizationJson {
                spell_level_minus_one: parse_u16(bytes, position)?,
                memorisable_count: parse_u16(bytes, position + 2)?,
                memorisable_count_after_effects: parse_u16(bytes, position + 4)?,
                spell_type: RawDecoded {
                    raw: spell_type,
                    decoded: decode_creature_spell_type(spell_type).map(str::to_string),
                },
                first_memorized_spell_index: parse_u32(bytes, position + 8)?,
                memorized_spell_count: parse_u32(bytes, position + 12)?,
            })
        },
    )
}

fn parse_memorized_spells(
    bytes: &[u8],
    offset: u32,
    count: u32,
) -> Result<Vec<CreatureMemorizedSpellJson>, CreatureParseError> {
    parse_table(
        bytes,
        offset,
        count,
        CRE_MEMORIZED_SPELL_SIZE,
        |bytes, position| {
            let flags = parse_u32(bytes, position + 8)?;
            Ok(CreatureMemorizedSpellJson {
                resource: parse_resref_option(bytes, position, 8)?,
                flags: RawDecodedFlags {
                    raw: flags,
                    decoded: decode_memorized_spell_flags(flags),
                },
            })
        },
    )
}

fn parse_effects(
    bytes: &[u8],
    offset: u32,
    count: u32,
    effect_structure_version: u8,
) -> Result<Vec<CreatureEffectJson>, CreatureParseError> {
    let record_size = match effect_structure_version {
        0 => CRE_EFFECT_V1_SIZE,
        1 => CRE_EFFECT_V2_SIZE,
        other => {
            return Err(CreatureParseError::InvalidField(format!(
                "unsupported CRE effect structure version {other}"
            )));
        }
    };

    parse_table(
        bytes,
        offset,
        count,
        record_size,
        |bytes, position| match effect_structure_version {
            0 => parse_effect_v1(&bytes[position..position + CRE_EFFECT_V1_SIZE]),
            1 => parse_effect_v2(&bytes[position..position + CRE_EFFECT_V2_SIZE]),
            _ => unreachable!(),
        },
    )
}

fn parse_effect_v1(bytes: &[u8]) -> Result<CreatureEffectJson, CreatureParseError> {
    let opcode = parse_u16(bytes, 0)? as u32;
    let target_type = parse_u8(bytes, 2)? as u32;
    let timing = parse_u8(bytes, 12)? as u32;

    Ok(CreatureEffectJson {
        version: "V1".to_string(),
        opcode: RawDecoded {
            raw: opcode,
            decoded: decode_effect_opcode(opcode as u16).map(str::to_string),
        },
        target_type: RawDecoded {
            raw: target_type,
            decoded: decode_effect_target_type(target_type as u8).map(str::to_string),
        },
        power: parse_u8(bytes, 3)? as u32,
        parameter1: parse_u32(bytes, 4)?,
        parameter2: parse_u32(bytes, 8)?,
        timing: RawDecoded {
            raw: timing,
            decoded: decode_effect_timing(timing as u8).map(str::to_string),
        },
        duration: parse_u32(bytes, 14)?,
        probability_1: parse_u8(bytes, 18)? as u16,
        probability_2: parse_u8(bytes, 19)? as u16,
        resource: parse_resref_option(bytes, 20, 8)?,
        dice_thrown_or_max_level: parse_u32(bytes, 28)?,
        dice_sides_or_min_level: parse_u32(bytes, 32)?,
        saving_throw_type: parse_u32(bytes, 36)?,
        saving_throw_bonus: parse_i32(bytes, 40)?,
        special: parse_u32(bytes, 44)?,
        dispel_resistance: parse_u8(bytes, 13)? as u32,
        minimum_level: None,
        maximum_level: None,
        raw_prefix_bytes: Vec::new(),
        raw_tail_bytes: Vec::new(),
    })
}

fn parse_effect_v2(bytes: &[u8]) -> Result<CreatureEffectJson, CreatureParseError> {
    let opcode = parse_u32(bytes, 0x08)?;
    let target_type = parse_u32(bytes, 0x0C)?;
    let timing = parse_u32(bytes, 0x1C)?;

    Ok(CreatureEffectJson {
        version: "V2".to_string(),
        opcode: RawDecoded {
            raw: opcode,
            decoded: u16::try_from(opcode)
                .ok()
                .and_then(decode_effect_opcode)
                .map(str::to_string),
        },
        target_type: RawDecoded {
            raw: target_type,
            decoded: u8::try_from(target_type)
                .ok()
                .and_then(decode_effect_target_type)
                .map(str::to_string),
        },
        power: parse_u32(bytes, 0x10)?,
        parameter1: parse_u32(bytes, 0x14)?,
        parameter2: parse_u32(bytes, 0x18)?,
        timing: RawDecoded {
            raw: timing,
            decoded: u8::try_from(timing)
                .ok()
                .and_then(decode_effect_timing)
                .map(str::to_string),
        },
        duration: parse_u32(bytes, 0x20)?,
        probability_1: parse_u16(bytes, 0x24)?,
        probability_2: parse_u16(bytes, 0x26)?,
        resource: parse_resref_option(bytes, 0x28, 8)?,
        dice_thrown_or_max_level: parse_u32(bytes, 0x30)?,
        dice_sides_or_min_level: parse_u32(bytes, 0x34)?,
        saving_throw_type: parse_u32(bytes, 0x38)?,
        saving_throw_bonus: parse_i32(bytes, 0x3C)?,
        special: parse_u32(bytes, 0x40)?,
        dispel_resistance: parse_u32(bytes, 0x54)?,
        minimum_level: Some(parse_u32(bytes, 0x4C)?),
        maximum_level: Some(parse_u32(bytes, 0x50)?),
        raw_prefix_bytes: bytes[0..0x08].to_vec(),
        raw_tail_bytes: bytes[0x58..].to_vec(),
    })
}

fn parse_items(
    bytes: &[u8],
    offset: u32,
    count: u32,
) -> Result<Vec<CreatureItemJson>, CreatureParseError> {
    parse_table(bytes, offset, count, CRE_ITEM_SIZE, |bytes, position| {
        let flags = parse_u32(bytes, position + 16)?;
        Ok(CreatureItemJson {
            resource: parse_resref_option(bytes, position, 8)?,
            expiration_time_days: parse_u16(bytes, position + 8)?,
            charges_1: parse_u16(bytes, position + 10)?,
            charges_2: parse_u16(bytes, position + 12)?,
            charges_3: parse_u16(bytes, position + 14)?,
            flags: RawDecodedFlags {
                raw: flags,
                decoded: decode_item_flags(flags),
            },
        })
    })
}

fn parse_item_slots(
    bytes: &[u8],
    item_slots_offset: u32,
    items_offset: u32,
) -> Result<Vec<CreatureItemSlotJson>, CreatureParseError> {
    if item_slots_offset == 0 {
        return Ok(Vec::new());
    }

    let start = item_slots_offset as usize;
    if start >= bytes.len() {
        return Err(CreatureParseError::UnexpectedEof(format!(
            "item slots offset {} exceeds CRE length {}",
            start,
            bytes.len()
        )));
    }

    let available_bytes = if items_offset > item_slots_offset {
        (items_offset - item_slots_offset) as usize
    } else {
        bytes.len() - start
    };
    let slot_count = available_bytes / 2;
    let mut slots = Vec::with_capacity(slot_count);

    for index in 0..slot_count {
        let raw = parse_u16(bytes, start + index * 2)?;
        slots.push(CreatureItemSlotJson {
            index,
            name: slot_name(index, slot_count),
            item_index: if raw == u16::MAX { None } else { Some(raw) },
        });
    }

    Ok(slots)
}

fn slot_name(index: usize, total_slots: usize) -> String {
    if total_slots >= 2 && index + 2 == total_slots {
        "selected_weapon".to_string()
    } else if total_slots >= 1 && index + 1 == total_slots {
        "selected_weapon_ability".to_string()
    } else {
        format!("slot_{index}")
    }
}

fn parse_table<T, F>(
    bytes: &[u8],
    offset: u32,
    count: u32,
    record_size: usize,
    mut parser: F,
) -> Result<Vec<T>, CreatureParseError>
where
    F: FnMut(&[u8], usize) -> Result<T, CreatureParseError>,
{
    if count == 0 {
        return Ok(Vec::new());
    }

    let start = offset as usize;
    let total_size = (count as usize).checked_mul(record_size).ok_or_else(|| {
        CreatureParseError::InvalidField("table count and record size overflow".to_string())
    })?;
    let end = start.checked_add(total_size).ok_or_else(|| {
        CreatureParseError::InvalidField("table offset and size overflow".to_string())
    })?;

    if end > bytes.len() {
        return Err(CreatureParseError::UnexpectedEof(format!(
            "table at {} with {} records of {} bytes exceeds CRE length {}",
            offset,
            count,
            record_size,
            bytes.len()
        )));
    }

    let mut rows = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let position = start + index * record_size;
        rows.push(parser(bytes, position)?);
    }
    Ok(rows)
}

fn parse_resref_option(
    bytes: &[u8],
    offset: usize,
    length: usize,
) -> Result<Option<ResRef>, CreatureParseError> {
    let raw = bytes.get(offset..offset + length).ok_or_else(|| {
        CreatureParseError::UnexpectedEof(format!("missing resref at {}", offset))
    })?;
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    let string = String::from_utf8_lossy(&raw[..end])
        .trim()
        .to_ascii_uppercase();
    if string.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ResRef::new(string).map_err(|err| {
            CreatureParseError::InvalidField(err.to_string())
        })?))
    }
}

fn parse_strref(
    bytes: &[u8],
    offset: usize,
    resolver: Option<&dyn StrRefResolver>,
) -> Result<ResolvedStrRef, CreatureParseError> {
    let raw = parse_u32(bytes, offset)?;
    let text = if raw == u32::MAX {
        None
    } else {
        resolver.and_then(|resolver| resolver.resolve_strref(StrRef(raw)))
    };
    Ok(ResolvedStrRef {
        strref: StrRef(raw),
        text,
    })
}

fn parse_ascii_string(
    bytes: &[u8],
    offset: usize,
    length: usize,
) -> Result<String, CreatureParseError> {
    let raw = bytes.get(offset..offset + length).ok_or_else(|| {
        CreatureParseError::UnexpectedEof(format!("missing ascii string at {}", offset))
    })?;
    Ok(String::from_utf8_lossy(raw).to_string())
}

fn parse_char_array(
    bytes: &[u8],
    offset: usize,
    length: usize,
) -> Result<String, CreatureParseError> {
    let raw = bytes.get(offset..offset + length).ok_or_else(|| {
        CreatureParseError::UnexpectedEof(format!("missing char array at {}", offset))
    })?;
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    Ok(String::from_utf8_lossy(&raw[..end]).trim().to_string())
}

fn parse_u8(bytes: &[u8], offset: usize) -> Result<u8, CreatureParseError> {
    bytes.get(offset).copied().ok_or_else(|| {
        CreatureParseError::UnexpectedEof(format!("unable to read u8 at {}", offset))
    })
}

fn parse_i8(bytes: &[u8], offset: usize) -> Result<i8, CreatureParseError> {
    parse_u8(bytes, offset).map(|value| value as i8)
}

fn parse_u16(bytes: &[u8], offset: usize) -> Result<u16, CreatureParseError> {
    let raw = bytes.get(offset..offset + 2).ok_or_else(|| {
        CreatureParseError::UnexpectedEof(format!("unable to read u16 at {}", offset))
    })?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn parse_i16(bytes: &[u8], offset: usize) -> Result<i16, CreatureParseError> {
    let raw = bytes.get(offset..offset + 2).ok_or_else(|| {
        CreatureParseError::UnexpectedEof(format!("unable to read i16 at {}", offset))
    })?;
    Ok(i16::from_le_bytes([raw[0], raw[1]]))
}

fn parse_u32(bytes: &[u8], offset: usize) -> Result<u32, CreatureParseError> {
    let raw = bytes.get(offset..offset + 4).ok_or_else(|| {
        CreatureParseError::UnexpectedEof(format!("unable to read u32 at {}", offset))
    })?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn parse_i32(bytes: &[u8], offset: usize) -> Result<i32, CreatureParseError> {
    let raw = bytes.get(offset..offset + 4).ok_or_else(|| {
        CreatureParseError::UnexpectedEof(format!("unable to read i32 at {}", offset))
    })?;
    Ok(i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn decode_effect_structure_version(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("EFF V1"),
        1 => Some("EFF V2"),
        _ => None,
    }
}

fn decode_creature_spell_type(value: u16) -> Option<&'static str> {
    match value {
        0 => Some("Priest"),
        1 => Some("Wizard"),
        2 => Some("Innate"),
        _ => None,
    }
}

fn decode_creature_flags(value: u32) -> Vec<String> {
    let mapping = [
        (0x0000_0001, "ShowLongNameInTooltip"),
        (0x0000_0002, "NoCorpse"),
        (0x0000_0004, "KeepCorpse"),
        (0x0000_0800, "Exportable"),
        (0x0000_1000, "HideInjuryStatus"),
        (0x0000_2000, "QuestCritical"),
        (0x0000_4000, "MovingBetweenAreas"),
        (0x0000_8000, "BeenInParty"),
        (0x0010_0000, "PreventExplodingDeath"),
        (0x0040_0000, "IgnoreNightmareModeModifiers"),
        (0x0080_0000, "NoTooltip"),
    ];
    decode_bits(value, &mapping)
}

fn decode_status_flags(value: u32) -> Vec<String> {
    let mapping = [
        (0x0000_0001, "Sleeping"),
        (0x0000_0002, "Berserk"),
        (0x0000_0004, "Panic"),
        (0x0000_0008, "Stunned"),
        (0x0000_0010, "Invisible"),
        (0x0000_0020, "Helpless"),
        (0x0000_0800, "Dead"),
        (0x0000_1000, "Silenced"),
        (0x0000_2000, "Charmed"),
        (0x0000_4000, "Poisoned"),
        (0x0000_8000, "Hasted"),
        (0x0001_0000, "Slowed"),
        (0x0004_0000, "Blind"),
        (0x0020_0000, "Nondetection"),
        (0x0040_0000, "ImprovedInvisibility"),
        (0x8000_0000, "Confused"),
    ];
    decode_bits(value, &mapping)
}

fn decode_shield_flags(value: u8) -> Vec<String> {
    [(0x01, "NormalShield"), (0x02, "BlackBarbedShield")]
        .into_iter()
        .filter_map(|(bit, label)| (value & bit != 0).then_some(label.to_string()))
        .collect()
}

fn decode_attribute_flags(value: u32) -> Vec<String> {
    let mapping = [
        (0x0000_0001, "AutoCalculateMarkerRectangle"),
        (0x0000_0002, "Translucent"),
        (0x0000_0010, "IncrementDeathVariableCounter"),
        (0x0000_0020, "IncrementCharacterTypeDeathCounter"),
        (0x0000_0080, "IncrementFactionDeathCounter"),
        (0x0000_0100, "IncrementTeamDeathCounter"),
        (0x0000_0200, "Invulnerable"),
        (0x0000_0400, "IncrementGoodVariable"),
        (0x0000_0800, "IncrementLawVariable"),
        (0x0000_1000, "IncrementLadyVariable"),
        (0x0000_2000, "IncrementMurderVariable"),
        (0x0000_4000, "DoNotFaceTargetDuringDialogue"),
        (0x0000_8000, "HelpTurnsNearbyCreaturesHostile"),
        (0x4000_0000, "DoNotIncrementGlobals"),
        (0x8000_0000, "AreaSupplemental"),
    ];
    decode_bits(value, &mapping)
}

fn decode_memorized_spell_flags(value: u32) -> Vec<String> {
    decode_bits(
        value,
        &[(0x0000_0001, "Memorized"), (0x0000_0002, "Disabled")],
    )
}

fn decode_item_flags(value: u32) -> Vec<String> {
    decode_bits(
        value,
        &[
            (0x0000_0001, "Identified"),
            (0x0000_0002, "Unstealable"),
            (0x0000_0004, "Stolen"),
            (0x0000_0008, "Undroppable"),
        ],
    )
}

fn decode_kit(value: u32) -> Option<&'static str> {
    match value {
        0x0000_0000 => Some("None"),
        0x0000_4000 => Some("Barbarian"),
        0x0040_0000 => Some("Abjurer"),
        0x0080_0000 => Some("Conjurer"),
        0x0100_0000 => Some("Diviner"),
        0x0200_0000 => Some("Enchanter"),
        0x0400_0000 => Some("Illusionist"),
        0x0800_0000 => Some("Invoker"),
        0x1000_0000 => Some("Necromancer"),
        0x2000_0000 => Some("Transmuter"),
        0x4000_0000 => Some("TrueClass"),
        0x4001_0000 => Some("Berserker"),
        0x4002_0000 => Some("WizardSlayer"),
        0x4003_0000 => Some("Kensai"),
        0x4004_0000 => Some("Cavalier"),
        0x4005_0000 => Some("Inquisitor"),
        0x4006_0000 => Some("UndeadHunter"),
        0x4007_0000 => Some("Archer"),
        0x4008_0000 => Some("Stalker"),
        0x4009_0000 => Some("Beastmaster"),
        0x400A_0000 => Some("Assassin"),
        0x400B_0000 => Some("BountyHunter"),
        0x400C_0000 => Some("Swashbuckler"),
        0x400D_0000 => Some("Blade"),
        0x400E_0000 => Some("Jester"),
        0x400F_0000 => Some("Skald"),
        0x4010_0000 => Some("Totemic"),
        0x4011_0000 => Some("Shapeshifter"),
        0x4012_0000 => Some("Avenger"),
        _ => None,
    }
}

fn decode_effect_opcode(value: u16) -> Option<&'static str> {
    match value {
        0 => Some("Cure Condition"),
        1 => Some("Cure Poison"),
        2 => Some("Cure Disease"),
        3 => Some("Berserk"),
        12 => Some("Damage"),
        16 => Some("Poison"),
        25 => Some("Hold Monster"),
        38 => Some("Silence"),
        45 => Some("Strength"),
        46 => Some("Dexterity"),
        47 => Some("Constitution"),
        48 => Some("Intelligence"),
        49 => Some("Wisdom"),
        50 => Some("Charisma"),
        55 => Some("Slay"),
        128 => Some("Confusion"),
        139 => Some("Display String"),
        142 => Some("Display Special Effect Icon"),
        174 => Some("Play Sound Effect"),
        215 => Some("Play 3D Effect"),
        318 => Some("Protection from Resource"),
        _ => None,
    }
}

fn decode_effect_target_type(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("None"),
        1 => Some("Self"),
        2 => Some("ProjectileTarget"),
        3 => Some("Party"),
        4 => Some("Everyone"),
        5 => Some("EveryoneExceptParty"),
        6 => Some("CasterGroup"),
        7 => Some("TargetGroup"),
        8 => Some("EveryoneExceptSelf"),
        9 => Some("OriginalCaster"),
        _ => None,
    }
}

fn decode_effect_timing(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("InstantLimited"),
        1 => Some("InstantPermanent"),
        2 => Some("InstantWhileEquipped"),
        3 => Some("DelayLimited"),
        4 => Some("DelayPermanent"),
        5 => Some("DelayWhileEquipped"),
        6 => Some("LimitedAfterDuration"),
        7 => Some("PermanentAfterDuration"),
        8 => Some("EquippedAfterDuration"),
        9 => Some("InstantPermanentAfterDeath"),
        10 => Some("InstantLimitedTicks"),
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

    struct NullResolver;

    impl StrRefResolver for NullResolver {
        fn resolve_strref(&self, _strref: StrRef) -> Option<String> {
            None
        }
    }

    #[test]
    fn parse_minimal_cre_header() {
        let mut bytes = vec![0u8; CRE_HEADER_SIZE];
        bytes[0..4].copy_from_slice(b"CRE ");
        bytes[4..8].copy_from_slice(b"V1.0");
        bytes[0x08..0x0C].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x0C..0x10].copy_from_slice(&2u32.to_le_bytes());
        bytes[0x10..0x14].copy_from_slice(&0x0000_0801u32.to_le_bytes());
        bytes[0x20..0x24].copy_from_slice(&0x0000_1000u32.to_le_bytes());
        bytes[0x24..0x26].copy_from_slice(&12u16.to_le_bytes());
        bytes[0x26..0x28].copy_from_slice(&14u16.to_le_bytes());
        bytes[0x33] = 0;
        bytes[0x34..0x3C].copy_from_slice(b"SMALL\0\0\0");
        bytes[0x3C..0x44].copy_from_slice(b"LARGE\0\0\0");
        bytes[0x44] = 10;
        bytes[0x45] = 15;
        bytes[0x46..0x48].copy_from_slice(&(-2i16).to_le_bytes());
        bytes[0x48..0x4A].copy_from_slice(&(-1i16).to_le_bytes());
        bytes[0x52] = 20;
        bytes[0x53] = 1;
        bytes[0x59] = 25;
        bytes[0x64] = 40;
        bytes[0x6B] = 3;
        bytes[0x82] = 2;
        bytes[0x83] = 5;
        bytes[0x84..0x8A].copy_from_slice(b"GORION");
        bytes[0x94] = 1;
        bytes[0x96..0x9A].copy_from_slice(&0x0000_0201u32.to_le_bytes());
        bytes[0x0A4..0x0A8].copy_from_slice(&3u32.to_le_bytes());
        bytes[0x234] = 7;
        bytes[0x238] = 18;
        bytes[0x239] = 76;
        bytes[0x23A] = 10;
        bytes[0x23B] = 13;
        bytes[0x23C] = 18;
        bytes[0x23D] = 16;
        bytes[0x23E] = 15;
        bytes[0x23F] = 10;
        bytes[0x240] = 3;
        bytes[0x241] = 91;
        bytes[0x242..0x244].copy_from_slice(&5u16.to_le_bytes());
        bytes[0x244..0x248].copy_from_slice(&0x4001_0000u32.to_be_bytes());
        bytes[0x248..0x250].copy_from_slice(b"OVERRIDE");
        bytes[0x270] = 128;
        bytes[0x271] = 1;
        bytes[0x272] = 2;
        bytes[0x273] = 3;
        bytes[0x274] = 4;
        bytes[0x275] = 1;
        bytes[0x27B] = 0x11;
        bytes[0x27C..0x27E].copy_from_slice(&7u16.to_le_bytes());
        bytes[0x27E..0x280].copy_from_slice(&9u16.to_le_bytes());
        bytes[0x280..0x286].copy_from_slice(b"IMOEN\0");
        bytes[0x2CC..0x2D4].copy_from_slice(b"IMOENJ\0\0");

        let creature =
            parse_cre(&bytes, "IMOEN2.CRE", Some(&NullResolver)).expect("CRE should parse");
        assert_eq!(creature.resource_name, "IMOEN2.CRE");
        assert_eq!(creature.version, "V1.0");
        assert_eq!(creature.header.current_hit_points, 12);
        assert_eq!(creature.header.max_hit_points, 14);
        assert_eq!(
            creature.header.flags.decoded,
            vec!["ShowLongNameInTooltip", "Exportable"]
        );
        assert_eq!(creature.header.status_flags.decoded, vec!["Silenced"]);
        assert_eq!(
            creature.header.small_portrait.as_ref().map(|r| r.as_str()),
            Some("SMALL")
        );
        assert_eq!(creature.header.tracking_target, "GORION");
        assert_eq!(creature.header.soundset[0].strref.0, 3);
        assert_eq!(creature.header.kit.decoded.as_deref(), Some("Berserker"));
        assert_eq!(
            creature.header.dialog.as_ref().map(|r| r.as_str()),
            Some("IMOENJ")
        );
        assert!(creature.known_spells.is_empty());
        assert!(creature.effects.is_empty());
    }

    #[test]
    fn parse_cre_v2_effect_uses_common_near_infinity_offsets() {
        let mut bytes = vec![0u8; CRE_EFFECT_V2_SIZE];
        bytes[0x08..0x0C].copy_from_slice(&12u32.to_le_bytes());
        bytes[0x0C..0x10].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x10..0x14].copy_from_slice(&2u32.to_le_bytes());
        bytes[0x14..0x18].copy_from_slice(&3u32.to_le_bytes());
        bytes[0x18..0x1C].copy_from_slice(&4u32.to_le_bytes());
        bytes[0x1C..0x20].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x20..0x24].copy_from_slice(&90u32.to_le_bytes());
        bytes[0x24..0x26].copy_from_slice(&100u16.to_le_bytes());
        bytes[0x26..0x28].copy_from_slice(&0u16.to_le_bytes());
        bytes[0x28..0x30].copy_from_slice(b"SPWI112\0");
        bytes[0x30..0x34].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x34..0x38].copy_from_slice(&6u32.to_le_bytes());
        bytes[0x38..0x3C].copy_from_slice(&2u32.to_le_bytes());
        bytes[0x3C..0x40].copy_from_slice(&(-1i32).to_le_bytes());
        bytes[0x40..0x44].copy_from_slice(&77u32.to_le_bytes());
        bytes[0x4C..0x50].copy_from_slice(&5u32.to_le_bytes());
        bytes[0x50..0x54].copy_from_slice(&10u32.to_le_bytes());
        bytes[0x54..0x58].copy_from_slice(&3u32.to_le_bytes());

        let effect = parse_effect_v2(&bytes).expect("V2 effect should parse");
        assert_eq!(effect.opcode.raw, 12);
        assert_eq!(effect.opcode.decoded.as_deref(), Some("Damage"));
        assert_eq!(effect.target_type.raw, 1);
        assert_eq!(effect.power, 2);
        assert_eq!(effect.parameter1, 3);
        assert_eq!(effect.parameter2, 4);
        assert_eq!(effect.duration, 90);
        assert_eq!(effect.probability_1, 100);
        assert_eq!(effect.minimum_level, Some(5));
        assert_eq!(effect.maximum_level, Some(10));
        assert_eq!(effect.dispel_resistance, 3);
        assert_eq!(
            effect.resource.as_ref().map(|r| r.as_str()),
            Some("SPWI112")
        );
        assert_eq!(effect.raw_prefix_bytes.len(), 8);
        assert_eq!(effect.raw_tail_bytes.len(), CRE_EFFECT_V2_SIZE - 0x58);
    }

    #[test]
    fn patch_cre_scalars_copy_only_is_byte_exact() {
        let bytes = patchable_cre();

        let patched = patch_cre_scalars(&bytes, &[]).expect("copy-only patch should work");

        assert_eq!(patched, bytes);
    }

    #[test]
    fn patch_cre_scalars_edits_only_requested_fixed_offsets() {
        let bytes = patchable_cre();
        let patched = patch_cre_scalars(
            &bytes,
            &[
                CreatureScalarPatch {
                    field: "morale".to_string(),
                    value: "9".to_string(),
                },
                CreatureScalarPatch {
                    field: "morale_break".to_string(),
                    value: "4".to_string(),
                },
                CreatureScalarPatch {
                    field: "dialog".to_string(),
                    value: "NEWDLG".to_string(),
                },
                CreatureScalarPatch {
                    field: "short_name".to_string(),
                    value: "12345".to_string(),
                },
            ],
        )
        .expect("scalar patch should work");

        assert_eq!(patched[0x23F], 9);
        assert_eq!(patched[0x240], 4);
        assert_eq!(
            &patched[0x2CC..0x2D4],
            &[b'N', b'E', b'W', b'D', b'L', b'G', 0, 0]
        );
        assert_eq!(&patched[0x0C..0x10], &12345u32.to_le_bytes());

        let changed = changed_offsets(&bytes, &patched);
        assert_eq!(changed, vec![0x0C, 0x0D, 0x23F, 0x240, 0x2CC, 0x2CD, 0x2CE]);
    }

    #[test]
    fn patch_cre_scalars_rejects_unknown_field_before_editing() {
        let bytes = patchable_cre();

        let error = patch_cre_scalars(
            &bytes,
            &[CreatureScalarPatch {
                field: "not_a_cre_field".to_string(),
                value: "1".to_string(),
            }],
        )
        .expect_err("unknown field should fail");

        assert!(error.to_string().contains("unknown CRE scalar field"));
    }

    #[test]
    fn patch_cre_scalars_rejects_out_of_range_values() {
        let bytes = patchable_cre();

        let error = patch_cre_scalars(
            &bytes,
            &[CreatureScalarPatch {
                field: "morale".to_string(),
                value: "256".to_string(),
            }],
        )
        .expect_err("out-of-range value should fail");

        assert!(error.to_string().contains("0..=255"));
    }

    #[test]
    fn patch_cre_scalars_rejects_invalid_resrefs() {
        let bytes = patchable_cre();

        let error = patch_cre_scalars(
            &bytes,
            &[CreatureScalarPatch {
                field: "override_script".to_string(),
                value: "TOO_LONG_RESREF".to_string(),
            }],
        )
        .expect_err("invalid resref should fail");

        assert!(error.to_string().contains("exceeds 8 characters"));
    }

    fn patchable_cre() -> Vec<u8> {
        let mut bytes = vec![0u8; CRE_HEADER_SIZE];
        bytes[0..4].copy_from_slice(b"CRE ");
        bytes[4..8].copy_from_slice(b"V1.0");
        bytes[0x0C..0x10].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x23F] = 10;
        bytes[0x240] = 3;
        bytes[0x2CC..0x2D4].copy_from_slice(b"OLDDLG\0\0");
        bytes
    }

    fn changed_offsets(before: &[u8], after: &[u8]) -> Vec<usize> {
        before
            .iter()
            .zip(after)
            .enumerate()
            .filter_map(|(index, (before, after))| (before != after).then_some(index))
            .collect()
    }
}
