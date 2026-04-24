use ie_core::{ResRef, ResolvedStrRef, ResourceType, StrRef, StrRefResolver};
use serde::Serialize;
use thiserror::Error;

const ITM_HEADER_SIZE: usize = 0x72;
const ITEM_ABILITY_SIZE: usize = 56;

#[derive(Debug, Clone, Serialize)]
pub struct ItemJson {
    pub resource_type: String,
    pub resource_name: String,
    pub version: String,
    pub header: ItemHeaderJson,
    pub abilities: Vec<ItemAbilityJson>,
    pub global_effects: Vec<ItemEffectJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemHeaderJson {
    pub identified_name: ResolvedStrRef,
    pub general_name: ResolvedStrRef,
    pub used_up_item: Option<ResRef>,
    pub drop_sound: Option<ResRef>,
    pub flags: RawDecodedFlags,
    pub category: RawDecoded<u16>,
    pub unusable_by: u32,
    pub equipped_appearance: u16,
    pub min_level: u16,
    pub min_strength: u16,
    pub min_strength_bonus: u16,
    pub min_intelligence: u16,
    pub min_dexterity: u16,
    pub min_wisdom: u16,
    pub min_constitution: u16,
    pub min_charisma: u16,
    pub price: u32,
    pub max_in_stack: u16,
    pub icon: Option<ResRef>,
    pub lore_to_identify: u16,
    pub ground_icon: Option<ResRef>,
    pub weight: u32,
    pub general_description: ResolvedStrRef,
    pub identified_description: ResolvedStrRef,
    pub description_image: Option<ResRef>,
    pub enchantment: u32,
    pub abilities_offset: u32,
    pub ability_count: u16,
    pub effects_offset: u32,
    pub first_effect_index: u16,
    pub global_effect_count: u16,
    pub dialogue: Option<ResRef>,
    pub speaker_name: Option<ResolvedStrRef>,
    pub weapon_color: Option<u16>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemAbilityJson {
    pub ability_type: RawDecoded<u8>,
    pub type_flags: u8,
    pub location: RawDecoded<u8>,
    pub dice_size_alt: u8,
    pub icon: Option<ResRef>,
    pub target_type: RawDecoded<u8>,
    pub target_count: u8,
    pub range: i16,
    pub launcher_required: RawDecoded<u8>,
    pub dice_count_alt: u8,
    pub speed: u8,
    pub damage_bonus_alt: u8,
    pub hit_bonus: i16,
    pub dice_size: u8,
    pub primary_type: RawDecoded<u8>,
    pub dice_count: u8,
    pub secondary_type: RawDecoded<u8>,
    pub damage_bonus: i16,
    pub damage_type: RawDecoded<u16>,
    pub num_effects: u16,
    pub first_effect_index: u16,
    pub charges: u16,
    pub drain_on: RawDecoded<u16>,
    pub flags: RawDecodedFlags,
    pub projectile: u16,
    pub animation_overhand: i16,
    pub animation_backhand: i16,
    pub animation_thrust: i16,
    pub is_arrow: u16,
    pub is_bolt: u16,
    pub is_bullet: u16,
    pub damage_dice: Option<String>,
    pub effects: Vec<ItemEffectJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemEffectJson {
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
}

const ITEM_EFFECT_SIZE: usize = 48;

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
pub enum ItemParseError {
    #[error("invalid ITM header: {0}")]
    InvalidHeader(String),
    #[error("unexpected end of ITM file: {0}")]
    UnexpectedEof(String),
    #[error("invalid resource field: {0}")]
    InvalidField(String),
}

impl From<ItemParseError> for crate::FormatError {
    fn from(err: ItemParseError) -> Self {
        crate::FormatError::Parse(err.to_string())
    }
}

pub fn parse_itm(
    bytes: &[u8],
    resource_name: &str,
    resolver: Option<&dyn StrRefResolver>,
) -> Result<ItemJson, crate::FormatError> {
    if bytes.len() < ITM_HEADER_SIZE {
        return Err(ItemParseError::UnexpectedEof(format!(
            "ITM resource must contain at least {} bytes",
            ITM_HEADER_SIZE
        ))
        .into());
    }

    if &bytes[0..4] != b"ITM " {
        return Err(ItemParseError::InvalidHeader("missing ITM signature".to_string()).into());
    }

    let version = parse_ascii_string(bytes, 4, 4)?;
    let general_name = parse_strref(bytes, 0x08, resolver)?;
    let identified_name = parse_strref(bytes, 0x0C, resolver)?;

    let used_up_item = if version == "V1.1" {
        None
    } else {
        parse_resref_option(bytes, 0x10, 8)?
    };

    let drop_sound = if version == "V1.1" {
        parse_resref_option(bytes, 0x10, 8)?
    } else {
        None
    };

    let flags = parse_u32(bytes, 0x18)?;
    let category = parse_u16(bytes, 0x1C)?;
    let unusable_by = parse_u32(bytes, 0x1E)?;
    let equipped_appearance = parse_u16(bytes, 0x22)?;
    let min_level = parse_u16(bytes, 0x24)?;
    let min_strength = parse_u16(bytes, 0x26)?;
    let min_strength_bonus = parse_u16(bytes, 0x28)?;
    let min_intelligence = parse_u16(bytes, 0x2A)?;
    let min_dexterity = parse_u16(bytes, 0x2C)?;
    let min_wisdom = parse_u16(bytes, 0x2E)?;
    let min_constitution = parse_u16(bytes, 0x30)?;
    let min_charisma = parse_u16(bytes, 0x32)?;
    let price = parse_u32(bytes, 0x34)?;
    let max_in_stack = parse_u16(bytes, 0x38)?;
    let icon = parse_resref_option(bytes, 0x3A, 8)?;
    let lore_to_identify = parse_u16(bytes, 0x42)?;
    let ground_icon = parse_resref_option(bytes, 0x44, 8)?;
    let weight = parse_u32(bytes, 0x4C)?;
    let general_description = parse_strref(bytes, 0x50, resolver)?;
    let identified_description = parse_strref(bytes, 0x54, resolver)?;
    let description_image = parse_resref_option(bytes, 0x58, 8)?;
    let enchantment = parse_u32(bytes, 0x60)?;
    let abilities_offset = parse_u32(bytes, 0x64)?;
    let ability_count = parse_u16(bytes, 0x68)?;
    let effects_offset = parse_u32(bytes, 0x6A)?;
    let first_effect_index = parse_u16(bytes, 0x6E)?;
    let global_effect_count = parse_u16(bytes, 0x70)?;

    let (dialogue, speaker_name, weapon_color) = if bytes.len() >= 0x80 && version == "V1.1" {
        let dialogue = parse_resref_option(bytes, 0x72, 8)?;
        let speaker_name = parse_strref(bytes, 0x7A, resolver).ok();
        let weapon_color = parse_u16(bytes, 0x7E).ok();
        (dialogue, speaker_name, weapon_color)
    } else {
        (None, None, None)
    };

    let header = ItemHeaderJson {
        identified_name,
        general_name,
        used_up_item,
        drop_sound,
        flags: RawDecodedFlags {
            raw: flags,
            decoded: decode_item_flags(flags),
        },
        category: RawDecoded {
            raw: category,
            decoded: decode_item_category(category).map(str::to_string),
        },
        unusable_by,
        equipped_appearance,
        min_level,
        min_strength,
        min_strength_bonus,
        min_intelligence,
        min_dexterity,
        min_wisdom,
        min_constitution,
        min_charisma,
        price,
        max_in_stack,
        icon,
        lore_to_identify,
        ground_icon,
        weight,
        general_description,
        identified_description,
        description_image,
        enchantment,
        abilities_offset,
        ability_count,
        effects_offset,
        first_effect_index,
        global_effect_count,
        dialogue,
        speaker_name,
        weapon_color,
    };

    let mut abilities = parse_abilities(bytes, abilities_offset, ability_count, resolver)?;
    for ability in &mut abilities {
        ability.effects = parse_effects(
            bytes,
            effects_offset,
            ability.first_effect_index,
            ability.num_effects,
        )?;
    }

    let global_effects = parse_effects(
        bytes,
        effects_offset,
        first_effect_index,
        global_effect_count,
    )?;

    Ok(ItemJson {
        resource_type: ResourceType::Itm.as_str().to_string(),
        resource_name: resource_name.to_string(),
        version,
        header,
        abilities,
        global_effects,
    })
}

fn parse_effects(
    bytes: &[u8],
    offset: u32,
    first_index: u16,
    count: u16,
) -> Result<Vec<ItemEffectJson>, ItemParseError> {
    if count == 0 {
        return Ok(Vec::new());
    }

    let start = offset as usize;
    let first = first_index as usize;
    let range_end = first + count as usize;
    let byte_end = start
        .checked_add(range_end * ITEM_EFFECT_SIZE)
        .ok_or_else(|| {
            ItemParseError::InvalidField("effect offset and count overflow".to_string())
        })?;

    if byte_end > bytes.len() {
        return Err(ItemParseError::UnexpectedEof(format!(
            "effect table exceeds ITM length: {} > {}",
            byte_end,
            bytes.len()
        )));
    }

    let mut effects = Vec::with_capacity(count as usize);
    for index in first..range_end {
        let position = start + index * ITEM_EFFECT_SIZE;
        let effect = parse_effect(bytes, position)?;
        effects.push(effect);
    }
    Ok(effects)
}

fn parse_effect(bytes: &[u8], offset: usize) -> Result<ItemEffectJson, ItemParseError> {
    if offset + ITEM_EFFECT_SIZE > bytes.len() {
        return Err(ItemParseError::UnexpectedEof(format!(
            "effect record at {} is truncated",
            offset
        )));
    }

    let opcode = parse_u16(bytes, offset)?;
    let target_type = parse_u8(bytes, offset + 2)?;
    let power = parse_u8(bytes, offset + 3)?;
    let parameter1 = parse_u32(bytes, offset + 4)?;
    let parameter2 = parse_u32(bytes, offset + 8)?;
    let timing = parse_u8(bytes, offset + 0x0C)?;
    let timing_dispel = parse_u8(bytes, offset + 0x0D)?;
    let duration = parse_u32(bytes, offset + 0x0E)?;
    let probability_1 = parse_u8(bytes, offset + 0x12)?;
    let probability_2 = parse_u8(bytes, offset + 0x13)?;
    let resource = parse_resref_option(bytes, offset + 0x14, 8)?;

    Ok(ItemEffectJson {
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
    })
}

fn parse_abilities(
    bytes: &[u8],
    offset: u32,
    count: u16,
    resolver: Option<&dyn StrRefResolver>,
) -> Result<Vec<ItemAbilityJson>, ItemParseError> {
    let start = offset as usize;
    let total_size = count as usize * ITEM_ABILITY_SIZE;
    let end = start.checked_add(total_size).ok_or_else(|| {
        ItemParseError::InvalidField("abilities offset and count overflow".to_string())
    })?;

    if total_size > 0 && end > bytes.len() {
        return Err(ItemParseError::UnexpectedEof(format!(
            "ability table exceeds ITM length: {} > {}",
            end,
            bytes.len()
        )));
    }

    let mut abilities = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let position = start + index * ITEM_ABILITY_SIZE;
        let ability = parse_ability(bytes, position, resolver)?;
        abilities.push(ability);
    }
    Ok(abilities)
}

fn parse_ability(
    bytes: &[u8],
    offset: usize,
    _resolver: Option<&dyn StrRefResolver>,
) -> Result<ItemAbilityJson, ItemParseError> {
    if offset + ITEM_ABILITY_SIZE > bytes.len() {
        return Err(ItemParseError::UnexpectedEof(format!(
            "ability record at {} is truncated",
            offset
        )));
    }

    let ability_type = parse_u8(bytes, offset)?;
    let type_flags = parse_u8(bytes, offset + 1)?;
    let location = parse_u8(bytes, offset + 2)?;
    let dice_size_alt = parse_u8(bytes, offset + 3)?;
    let icon = parse_resref_option(bytes, offset + 4, 8)?;
    let target_type = parse_u8(bytes, offset + 0x0C)?;
    let target_count = parse_u8(bytes, offset + 0x0D)?;
    let range = parse_i16(bytes, offset + 0x0E)?;
    let launcher_required = parse_u8(bytes, offset + 0x10)?;
    let dice_count_alt = parse_u8(bytes, offset + 0x11)?;
    let speed = parse_u8(bytes, offset + 0x12)?;
    let damage_bonus_alt = parse_u8(bytes, offset + 0x13)?;
    let hit_bonus = parse_i16(bytes, offset + 0x14)?;
    let dice_size = parse_u8(bytes, offset + 0x16)?;
    let primary_type = parse_u8(bytes, offset + 0x17)?;
    let dice_count = parse_u8(bytes, offset + 0x18)?;
    let secondary_type = parse_u8(bytes, offset + 0x19)?;
    let damage_bonus = parse_i16(bytes, offset + 0x1A)?;
    let damage_type = parse_u16(bytes, offset + 0x1C)?;
    let num_effects = parse_u16(bytes, offset + 0x1E)?;
    let first_effect_index = parse_u16(bytes, offset + 0x20)?;
    let charges = parse_u16(bytes, offset + 0x22)?;
    let drain_on = parse_u16(bytes, offset + 0x24)?;
    let flags = parse_u32(bytes, offset + 0x26)?;
    let projectile = parse_u16(bytes, offset + 0x2A)?;
    let animation_overhand = parse_i16(bytes, offset + 0x2C)?;
    let animation_backhand = parse_i16(bytes, offset + 0x2E)?;
    let animation_thrust = parse_i16(bytes, offset + 0x30)?;
    let is_arrow = parse_u16(bytes, offset + 0x32)?;
    let is_bolt = parse_u16(bytes, offset + 0x34)?;
    let is_bullet = parse_u16(bytes, offset + 0x36)?;

    let damage_dice = if dice_size > 0 && dice_count > 0 {
        Some(format!(
            "{}d{}{}",
            dice_count,
            dice_size,
            if damage_bonus >= 0 {
                format!("+{}", damage_bonus)
            } else {
                damage_bonus.to_string()
            }
        ))
    } else {
        None
    };

    Ok(ItemAbilityJson {
        ability_type: RawDecoded {
            raw: ability_type,
            decoded: decode_ability_type(ability_type).map(str::to_string),
        },
        type_flags,
        location: RawDecoded {
            raw: location,
            decoded: decode_ability_location(location).map(str::to_string),
        },
        dice_size_alt,
        icon,
        target_type: RawDecoded {
            raw: target_type,
            decoded: decode_target_type(target_type).map(str::to_string),
        },
        target_count,
        range,
        launcher_required: RawDecoded {
            raw: launcher_required,
            decoded: decode_launcher_required(launcher_required).map(str::to_string),
        },
        dice_count_alt,
        speed,
        damage_bonus_alt,
        hit_bonus,
        dice_size,
        primary_type: RawDecoded {
            raw: primary_type,
            decoded: decode_spell_type(primary_type).map(str::to_string),
        },
        dice_count,
        secondary_type: RawDecoded {
            raw: secondary_type,
            decoded: decode_spell_type(secondary_type).map(str::to_string),
        },
        damage_bonus,
        damage_type: RawDecoded {
            raw: damage_type,
            decoded: decode_damage_type(damage_type).map(str::to_string),
        },
        num_effects,
        first_effect_index,
        charges,
        drain_on: RawDecoded {
            raw: drain_on,
            decoded: decode_drain_on(drain_on).map(str::to_string),
        },
        flags: RawDecodedFlags {
            raw: flags,
            decoded: decode_ability_flags(flags),
        },
        projectile,
        animation_overhand,
        animation_backhand,
        animation_thrust,
        is_arrow,
        is_bolt,
        is_bullet,
        damage_dice,
        effects: Vec::new(),
    })
}

fn parse_resref_option(
    bytes: &[u8],
    offset: usize,
    length: usize,
) -> Result<Option<ResRef>, ItemParseError> {
    let raw = bytes
        .get(offset..offset + length)
        .ok_or_else(|| ItemParseError::UnexpectedEof(format!("missing resref at {}", offset)))?;

    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    let string = String::from_utf8_lossy(&raw[..end])
        .trim()
        .to_ascii_uppercase();
    if string.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ResRef::new(string).map_err(|err| {
            ItemParseError::InvalidField(err.to_string())
        })?))
    }
}

fn parse_strref(
    bytes: &[u8],
    offset: usize,
    resolver: Option<&dyn StrRefResolver>,
) -> Result<ResolvedStrRef, ItemParseError> {
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
) -> Result<String, ItemParseError> {
    let raw = bytes.get(offset..offset + length).ok_or_else(|| {
        ItemParseError::UnexpectedEof(format!("missing ascii string at {}", offset))
    })?;
    Ok(String::from_utf8_lossy(raw).to_string())
}

fn parse_u8(bytes: &[u8], offset: usize) -> Result<u8, ItemParseError> {
    bytes
        .get(offset)
        .copied()
        .ok_or_else(|| ItemParseError::UnexpectedEof(format!("unable to read u8 at {}", offset)))
}

fn parse_u16(bytes: &[u8], offset: usize) -> Result<u16, ItemParseError> {
    let raw = bytes.get(offset..offset + 2).ok_or_else(|| {
        ItemParseError::UnexpectedEof(format!("unable to read u16 at {}", offset))
    })?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn parse_u32(bytes: &[u8], offset: usize) -> Result<u32, ItemParseError> {
    let raw = bytes.get(offset..offset + 4).ok_or_else(|| {
        ItemParseError::UnexpectedEof(format!("unable to read u32 at {}", offset))
    })?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn parse_i16(bytes: &[u8], offset: usize) -> Result<i16, ItemParseError> {
    let raw = bytes.get(offset..offset + 2).ok_or_else(|| {
        ItemParseError::UnexpectedEof(format!("unable to read i16 at {}", offset))
    })?;
    Ok(i16::from_le_bytes([raw[0], raw[1]]))
}

fn decode_item_category(value: u16) -> Option<&'static str> {
    match value {
        1 => Some("Misc"),
        2 => Some("Amulet"),
        3 => Some("Armor"),
        7 => Some("Ring"),
        8 => Some("Cloak"),
        0x0B => Some("Scroll"),
        0x0C => Some("Wand"),
        0x10 => Some("Weapon"),
        _ => None,
    }
}

fn decode_item_flags(value: u32) -> Vec<String> {
    let mut labels = Vec::new();
    let mapping = [
        (0x00000001, "Critical"),
        (0x00000002, "TwoHanded"),
        (0x00000004, "Droppable"),
        (0x00000008, "Displayable"),
        (0x00000010, "Cursed"),
        (0x00000020, "NoCopy"),
        (0x00000040, "Magical"),
        (0x00000080, "LeftHanded"),
        (0x00000100, "Silver"),
        (0x00000200, "ColdIron"),
        (0x00000400, "OffHand"),
        (0x00000800, "Conversable"),
        (0x00001000, "FakeTwoHanded"),
        (0x00002000, "ForbidOffHand"),
        (0x00010000, "Adamantine"),
        (0x02000000, "Undispellable"),
        (0x04000000, "ToggleCriticalHits"),
    ];

    for (bit, label) in mapping {
        if value & bit != 0 {
            labels.push(label.to_string());
        }
    }
    labels
}

fn decode_ability_type(value: u8) -> Option<&'static str> {
    match value {
        1 => Some("Attack"),
        2 => Some("Ability"),
        3 => Some("Innate"),
        _ => None,
    }
}

fn decode_ability_location(value: u8) -> Option<&'static str> {
    match value {
        1 => Some("Weapon"),
        2 => Some("Spell"),
        3 => Some("Item"),
        4 => Some("Ability"),
        _ => None,
    }
}

fn decode_target_type(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("None"),
        1 => Some("Single"),
        2 => Some("Group"),
        3 => Some("Area"),
        _ => None,
    }
}

fn decode_launcher_required(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("None"),
        1 => Some("Bow"),
        2 => Some("Crossbow"),
        3 => Some("Sling"),
        _ => None,
    }
}

fn decode_spell_type(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("Unknown"),
        1 => Some("Piercing"),
        2 => Some("Slashing"),
        3 => Some("Crushing"),
        _ => None,
    }
}

fn decode_damage_type(value: u16) -> Option<&'static str> {
    match value {
        0 => Some("Unknown"),
        1 => Some("Piercing"),
        2 => Some("Slashing"),
        3 => Some("Crushing"),
        4 => Some("Magic"),
        _ => None,
    }
}

fn decode_drain_on(value: u16) -> Option<&'static str> {
    match value {
        0 => Some("Never"),
        1 => Some("Always"),
        2 => Some("AfterRest"),
        _ => None,
    }
}

fn decode_ability_flags(value: u32) -> Vec<String> {
    let mut labels = Vec::new();
    let mapping = [
        (0x00000001, "AddStrBonus"),
        (0x00000002, "Breakable"),
        (0x00000400, "BreakSanctuary"),
        (0x00000800, "Hostile"),
        (0x00001000, "RechargeAfterRest"),
        (0x01000000, "ToggleBackstab"),
        (0x02000000, "CannotTargetInvisible"),
    ];

    for (bit, label) in mapping {
        if value & bit != 0 {
            labels.push(label.to_string());
        }
    }
    labels
}

fn decode_effect_opcode(value: u16) -> Option<&'static str> {
    match value {
        0 => Some("Cure Condition"),
        1 => Some("Cure Poison"),
        2 => Some("Cure Disease"),
        3 => Some("Heal"),
        4 => Some("Drain Level"),
        5 => Some("Drain HP"),
        6 => Some("Drain Magic"),
        7 => Some("Drain Item Charges"),
        8 => Some("Cure Fatigue"),
        9 => Some("Cure Intoxication"),
        16 => Some("Poison"),
        17 => Some("Disease"),
        18 => Some("Fatigue"),
        19 => Some("Intoxication"),
        20 => Some("Sleep"),
        21 => Some("Confusion"),
        22 => Some("Charm"),
        23 => Some("Fear"),
        24 => Some("Power Word: Stun"),
        25 => Some("Hold Monster"),
        26 => Some("Paralyze"),
        27 => Some("Haste"),
        28 => Some("Slow"),
        29 => Some("Protection: Fire"),
        30 => Some("Protection: Cold"),
        31 => Some("Protection: Lightning"),
        32 => Some("Protection: Acid"),
        33 => Some("Protection: Magic"),
        34 => Some("Protection: Magic Fire"),
        35 => Some("Protection: Magic Cold"),
        36 => Some("Invisibility 10' Radius"),
        37 => Some("Invisibility"),
        38 => Some("Detect Invisible"),
        39 => Some("Invisibility Purge"),
        40 => Some("Protection: Cold (Lesser)"),
        41 => Some("Protection: Fire (Lesser)"),
        42 => Some("Protection: Lightning (Lesser)"),
        43 => Some("Protection: Magic Damage (Lesser)"),
        44 => Some("Protection: Acid (Lesser)"),
        45 => Some("Strength"),
        46 => Some("Dexterity"),
        47 => Some("Constitution"),
        48 => Some("Intelligence"),
        49 => Some("Wisdom"),
        50 => Some("Charisma"),
        51 => Some("Damage"),
        52 => Some("Thac0 Bonus"),
        53 => Some("Saving Throw: Death"),
        54 => Some("Saving Throw: Wands"),
        55 => Some("Saving Throw: Polymorph"),
        56 => Some("Saving Throw: Breath"),
        57 => Some("Saving Throw: Spells"),
        _ => None,
    }
}

fn decode_effect_target_type(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("Self"),
        1 => Some("Single Target"),
        2 => Some("Area (friendly)"),
        3 => Some("Area (hostile)"),
        4 => Some("Area (all)"),
        5 => Some("Caster Group"),
        6 => Some("Single Target (dispellable)"),
        7 => Some("Party"),
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
        5 => Some("Instant (permanent, unless removed)"),
        6 => Some("Delayed (until end of combat)"),
        7 => Some("Instant (permanently until death)"),
        8 => Some("Instant (semi-permanent)"),
        9 => Some("Instant (permanent after resting)"),
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
    fn parse_minimal_itm_header() {
        let mut bytes = vec![0u8; ITM_HEADER_SIZE];
        bytes[0..4].copy_from_slice(b"ITM ");
        bytes[4..8].copy_from_slice(b"V1  ");
        bytes[0x08..0x0C].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x0C..0x10].copy_from_slice(&2u32.to_le_bytes());
        bytes[0x18..0x1C].copy_from_slice(&0x00000040u32.to_le_bytes());
        bytes[0x1C..0x1E].copy_from_slice(&0x0010u16.to_le_bytes());
        bytes[0x34..0x38].copy_from_slice(&123u32.to_le_bytes());
        bytes[0x3A..0x42].copy_from_slice(b"ICON\0\0\0\0");
        bytes[0x44..0x4C].copy_from_slice(b"GRND\0\0\0\0");
        bytes[0x4C..0x50].copy_from_slice(&456u32.to_le_bytes());
        bytes[0x50..0x54].copy_from_slice(&3u32.to_le_bytes());
        bytes[0x54..0x58].copy_from_slice(&4u32.to_le_bytes());
        bytes[0x58..0x60].copy_from_slice(b"IMAGE\0\0\0");
        bytes[0x60..0x64].copy_from_slice(&2u32.to_le_bytes());
        bytes[0x64..0x68].copy_from_slice(&(ITM_HEADER_SIZE as u32).to_le_bytes());
        bytes[0x68..0x6A].copy_from_slice(&0u16.to_le_bytes());
        bytes[0x6A..0x6E].copy_from_slice(&0u32.to_le_bytes());
        bytes[0x6E..0x70].copy_from_slice(&0u16.to_le_bytes());
        bytes[0x70..0x72].copy_from_slice(&0u16.to_le_bytes());

        let item = parse_itm(&bytes, "FOO.ITM", Some(&NullResolver)).expect("should parse ITM");
        assert_eq!(item.resource_name, "FOO.ITM");
        assert_eq!(item.version, "V1  ");
        assert_eq!(item.header.price, 123);
        assert_eq!(item.header.weight, 456);
        assert_eq!(item.header.icon.unwrap().as_str(), "ICON");
        assert_eq!(item.header.ground_icon.unwrap().as_str(), "GRND");
        assert_eq!(item.header.category.decoded.as_deref(), Some("Weapon"));
        assert_eq!(item.header.flags.decoded, vec!["Magical"]);
        assert!(item.abilities.is_empty());
        assert!(item.global_effects.is_empty());
    }

    #[test]
    fn parse_ability_local_effects() {
        let ability_offset = ITM_HEADER_SIZE;
        let effects_offset = ability_offset + ITEM_ABILITY_SIZE;
        let mut bytes = vec![0u8; effects_offset + ITEM_EFFECT_SIZE * 2];
        bytes[0..4].copy_from_slice(b"ITM ");
        bytes[4..8].copy_from_slice(b"V1  ");
        bytes[0x64..0x68].copy_from_slice(&(ability_offset as u32).to_le_bytes());
        bytes[0x68..0x6A].copy_from_slice(&1u16.to_le_bytes());
        bytes[0x6A..0x6E].copy_from_slice(&(effects_offset as u32).to_le_bytes());
        bytes[0x6E..0x70].copy_from_slice(&0u16.to_le_bytes());
        bytes[0x70..0x72].copy_from_slice(&0u16.to_le_bytes());

        bytes[ability_offset + 0x1E..ability_offset + 0x20].copy_from_slice(&2u16.to_le_bytes());
        bytes[ability_offset + 0x20..ability_offset + 0x22].copy_from_slice(&0u16.to_le_bytes());

        let first_effect = effects_offset;
        bytes[first_effect..first_effect + 2].copy_from_slice(&171u16.to_le_bytes());
        bytes[first_effect + 2] = 1;
        bytes[first_effect + 0x14..first_effect + 0x1C].copy_from_slice(b"DLIGHT\0\0");

        let second_effect = effects_offset + ITEM_EFFECT_SIZE;
        bytes[second_effect..second_effect + 2].copy_from_slice(&123u16.to_le_bytes());
        bytes[second_effect + 2] = 2;
        bytes[second_effect + 0x14..second_effect + 0x1C].copy_from_slice(b"DISCIPLE");

        let item = parse_itm(&bytes, "FOO.ITM", Some(&NullResolver)).expect("should parse ITM");

        assert_eq!(item.abilities.len(), 1);
        assert_eq!(item.global_effects.len(), 0);

        let effects = &item.abilities[0].effects;
        assert_eq!(effects.len(), 2);
        assert_eq!(effects[0].opcode.raw, 171);
        assert_eq!(effects[0].resource.as_ref().unwrap().as_str(), "DLIGHT");
        assert_eq!(effects[1].opcode.raw, 123);
        assert_eq!(effects[1].resource.as_ref().unwrap().as_str(), "DISCIPLE");
    }
}
