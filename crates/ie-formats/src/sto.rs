use ie_core::{ResRef, ResolvedStrRef, ResourceType, StrRef, StrRefResolver};
use serde::Serialize;
use thiserror::Error;

const STO_HEADER_SIZE: usize = 0x9C;
const STO_ITEM_SIZE: usize = 0x1C;
const STO_DRINK_SIZE: usize = 0x14;
const STO_CURE_SIZE: usize = 0x0C;
const STO_PURCHASED_ITEM_SIZE: usize = 4;

#[derive(Debug, Clone, Serialize)]
pub struct StoreJson {
    pub resource_type: String,
    pub resource_name: String,
    pub version: String,
    pub header: StoreHeaderJson,
    pub items_for_sale: Vec<StoreItemJson>,
    pub drinks: Vec<StoreDrinkJson>,
    pub cures: Vec<StoreCureJson>,
    pub items_purchased: Vec<StorePurchasedCategoryJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreHeaderJson {
    pub store_type: RawDecoded<u32>,
    pub name: ResolvedStrRef,
    pub flags: RawDecodedFlags,
    pub sell_price_markup: u32,
    pub buy_price_markup: u32,
    pub depreciation_rate: u32,
    pub steal_failure_chance: u16,
    pub capacity: u16,
    pub unknown_header_bytes_0x24: Vec<u8>,
    pub items_purchased_offset: u32,
    pub items_purchased_count: u32,
    pub items_for_sale_offset: u32,
    pub items_for_sale_count: u32,
    pub lore: u32,
    pub id_price: u32,
    pub tavern_rumours: Option<ResRef>,
    pub drinks_offset: u32,
    pub drinks_count: u32,
    pub temple_rumours: Option<ResRef>,
    pub room_flags: RawDecodedFlags,
    pub room_prices: StoreRoomPricesJson,
    pub cures_offset: u32,
    pub cures_count: u32,
    pub unknown_header_bytes_0x78: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreRoomPricesJson {
    pub peasant: u32,
    pub merchant: u32,
    pub noble: u32,
    pub royal: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreItemJson {
    pub resource: Option<ResRef>,
    pub expiration_time: u16,
    pub charges_1: u16,
    pub charges_2: u16,
    pub charges_3: u16,
    pub flags: RawDecodedFlags,
    pub stock_count: u32,
    pub infinite_supply: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreDrinkJson {
    pub rumour_resource: Option<ResRef>,
    pub name: ResolvedStrRef,
    pub price: u32,
    pub alcoholic_strength: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreCureJson {
    pub spell_resource: Option<ResRef>,
    pub price: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct StorePurchasedCategoryJson {
    pub raw: u32,
    pub decoded: Option<String>,
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
pub enum StoreParseError {
    #[error("invalid STO header: {0}")]
    InvalidHeader(String),
    #[error("unexpected end of STO file: {0}")]
    UnexpectedEof(String),
    #[error("invalid STO field: {0}")]
    InvalidField(String),
}

impl From<StoreParseError> for crate::FormatError {
    fn from(err: StoreParseError) -> Self {
        crate::FormatError::Parse(err.to_string())
    }
}

pub fn parse_sto(
    bytes: &[u8],
    resource_name: &str,
    resolver: Option<&dyn StrRefResolver>,
) -> Result<StoreJson, crate::FormatError> {
    if bytes.len() < STO_HEADER_SIZE {
        return Err(StoreParseError::UnexpectedEof(format!(
            "STO resource must contain at least {} bytes",
            STO_HEADER_SIZE
        ))
        .into());
    }

    if &bytes[0..4] != b"STOR" {
        return Err(StoreParseError::InvalidHeader("missing STOR signature".to_string()).into());
    }

    let version = parse_ascii_string(bytes, 4, 4)?;
    let store_type = parse_u32(bytes, 0x08)?;
    let flags = parse_u32(bytes, 0x10)?;
    let room_flags = parse_u32(bytes, 0x5C)?;
    let items_purchased_offset = parse_u32(bytes, 0x2C)?;
    let items_purchased_count = parse_u32(bytes, 0x30)?;
    let items_for_sale_offset = parse_u32(bytes, 0x34)?;
    let items_for_sale_count = parse_u32(bytes, 0x38)?;
    let drinks_offset = parse_u32(bytes, 0x4C)?;
    let drinks_count = parse_u32(bytes, 0x50)?;
    let cures_offset = parse_u32(bytes, 0x70)?;
    let cures_count = parse_u32(bytes, 0x74)?;

    let header = StoreHeaderJson {
        store_type: RawDecoded {
            raw: store_type,
            decoded: decode_store_type(store_type).map(str::to_string),
        },
        name: parse_strref(bytes, 0x0C, resolver)?,
        flags: RawDecodedFlags {
            raw: flags,
            decoded: decode_store_flags(flags),
        },
        sell_price_markup: parse_u32(bytes, 0x14)?,
        buy_price_markup: parse_u32(bytes, 0x18)?,
        depreciation_rate: parse_u32(bytes, 0x1C)?,
        steal_failure_chance: parse_u16(bytes, 0x20)?,
        capacity: parse_u16(bytes, 0x22)?,
        unknown_header_bytes_0x24: bytes
            .get(0x24..0x2C)
            .ok_or_else(|| {
                StoreParseError::UnexpectedEof("missing header bytes at 0x24".to_string())
            })?
            .to_vec(),
        items_purchased_offset,
        items_purchased_count,
        items_for_sale_offset,
        items_for_sale_count,
        lore: parse_u32(bytes, 0x3C)?,
        id_price: parse_u32(bytes, 0x40)?,
        tavern_rumours: parse_resref_option(bytes, 0x44, 8)?,
        drinks_offset,
        drinks_count,
        temple_rumours: parse_resref_option(bytes, 0x54, 8)?,
        room_flags: RawDecodedFlags {
            raw: room_flags,
            decoded: decode_room_flags(room_flags),
        },
        room_prices: StoreRoomPricesJson {
            peasant: parse_u32(bytes, 0x60)?,
            merchant: parse_u32(bytes, 0x64)?,
            noble: parse_u32(bytes, 0x68)?,
            royal: parse_u32(bytes, 0x6C)?,
        },
        cures_offset,
        cures_count,
        unknown_header_bytes_0x78: bytes
            .get(0x78..0x9C)
            .ok_or_else(|| {
                StoreParseError::UnexpectedEof("missing header bytes at 0x78".to_string())
            })?
            .to_vec(),
    };

    let items_for_sale = parse_items_for_sale(bytes, items_for_sale_offset, items_for_sale_count)?;
    let drinks = parse_drinks(bytes, drinks_offset, drinks_count, resolver)?;
    let cures = parse_cures(bytes, cures_offset, cures_count)?;
    let items_purchased =
        parse_purchased_categories(bytes, items_purchased_offset, items_purchased_count)?;

    Ok(StoreJson {
        resource_type: ResourceType::Sto.as_str().to_string(),
        resource_name: resource_name.to_string(),
        version,
        header,
        items_for_sale,
        drinks,
        cures,
        items_purchased,
    })
}

fn parse_items_for_sale(
    bytes: &[u8],
    offset: u32,
    count: u32,
) -> Result<Vec<StoreItemJson>, StoreParseError> {
    parse_table(bytes, offset, count, STO_ITEM_SIZE, |bytes, position| {
        let flags = parse_u32(bytes, position + 0x10)?;
        Ok(StoreItemJson {
            resource: parse_resref_option(bytes, position, 8)?,
            expiration_time: parse_u16(bytes, position + 0x08)?,
            charges_1: parse_u16(bytes, position + 0x0A)?,
            charges_2: parse_u16(bytes, position + 0x0C)?,
            charges_3: parse_u16(bytes, position + 0x0E)?,
            flags: RawDecodedFlags {
                raw: flags,
                decoded: decode_item_flags(flags),
            },
            stock_count: parse_u32(bytes, position + 0x14)?,
            infinite_supply: parse_u32(bytes, position + 0x18)? != 0,
        })
    })
}

fn parse_drinks(
    bytes: &[u8],
    offset: u32,
    count: u32,
    resolver: Option<&dyn StrRefResolver>,
) -> Result<Vec<StoreDrinkJson>, StoreParseError> {
    parse_table(bytes, offset, count, STO_DRINK_SIZE, |bytes, position| {
        Ok(StoreDrinkJson {
            rumour_resource: parse_resref_option(bytes, position, 8)?,
            name: parse_strref(bytes, position + 8, resolver)?,
            price: parse_u32(bytes, position + 0x0C)?,
            alcoholic_strength: parse_u32(bytes, position + 0x10)?,
        })
    })
}

fn parse_cures(
    bytes: &[u8],
    offset: u32,
    count: u32,
) -> Result<Vec<StoreCureJson>, StoreParseError> {
    parse_table(bytes, offset, count, STO_CURE_SIZE, |bytes, position| {
        Ok(StoreCureJson {
            spell_resource: parse_resref_option(bytes, position, 8)?,
            price: parse_u32(bytes, position + 8)?,
        })
    })
}

fn parse_purchased_categories(
    bytes: &[u8],
    offset: u32,
    count: u32,
) -> Result<Vec<StorePurchasedCategoryJson>, StoreParseError> {
    parse_table(
        bytes,
        offset,
        count,
        STO_PURCHASED_ITEM_SIZE,
        |bytes, position| {
            let raw = parse_u32(bytes, position)?;
            Ok(StorePurchasedCategoryJson {
                raw,
                decoded: decode_item_category(raw).map(str::to_string),
            })
        },
    )
}

fn parse_table<T, F>(
    bytes: &[u8],
    offset: u32,
    count: u32,
    record_size: usize,
    mut parser: F,
) -> Result<Vec<T>, StoreParseError>
where
    F: FnMut(&[u8], usize) -> Result<T, StoreParseError>,
{
    if count == 0 {
        return Ok(Vec::new());
    }

    let start = offset as usize;
    let total_size = (count as usize).checked_mul(record_size).ok_or_else(|| {
        StoreParseError::InvalidField("table count and record size overflow".to_string())
    })?;
    let end = start.checked_add(total_size).ok_or_else(|| {
        StoreParseError::InvalidField("table offset and size overflow".to_string())
    })?;
    if end > bytes.len() {
        return Err(StoreParseError::UnexpectedEof(format!(
            "table at {} with {} records of {} bytes exceeds STO length {}",
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
) -> Result<Option<ResRef>, StoreParseError> {
    let raw = bytes
        .get(offset..offset + length)
        .ok_or_else(|| StoreParseError::UnexpectedEof(format!("missing resref at {}", offset)))?;
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    let string = String::from_utf8_lossy(&raw[..end])
        .trim()
        .to_ascii_uppercase();
    if string.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ResRef::new(string).map_err(|err| {
            StoreParseError::InvalidField(err.to_string())
        })?))
    }
}

fn parse_strref(
    bytes: &[u8],
    offset: usize,
    resolver: Option<&dyn StrRefResolver>,
) -> Result<ResolvedStrRef, StoreParseError> {
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
) -> Result<String, StoreParseError> {
    let raw = bytes.get(offset..offset + length).ok_or_else(|| {
        StoreParseError::UnexpectedEof(format!("missing ascii string at {}", offset))
    })?;
    Ok(String::from_utf8_lossy(raw).to_string())
}

fn parse_u16(bytes: &[u8], offset: usize) -> Result<u16, StoreParseError> {
    let raw = bytes.get(offset..offset + 2).ok_or_else(|| {
        StoreParseError::UnexpectedEof(format!("unable to read u16 at {}", offset))
    })?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn parse_u32(bytes: &[u8], offset: usize) -> Result<u32, StoreParseError> {
    let raw = bytes.get(offset..offset + 4).ok_or_else(|| {
        StoreParseError::UnexpectedEof(format!("unable to read u32 at {}", offset))
    })?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn decode_store_type(value: u32) -> Option<&'static str> {
    match value {
        0 => Some("Store"),
        1 => Some("Tavern"),
        2 => Some("Inn"),
        3 => Some("Temple"),
        5 => Some("Container"),
        _ => None,
    }
}

fn decode_store_flags(value: u32) -> Vec<String> {
    decode_bits(
        value,
        &[
            (0x0001, "CanBuy"),
            (0x0002, "CanSell"),
            (0x0004, "CanIdentify"),
            (0x0008, "CanSteal"),
            (0x0010, "CanDonate"),
            (0x0020, "CanBuyCures"),
            (0x0040, "CanBuyDrinks"),
            (0x0200, "Quality1"),
            (0x0400, "Quality2"),
            (0x1000, "BuyFencedGoods"),
            (0x2000, "IgnoreReputationPrices"),
            (0x4000, "ToggleItemRecharge"),
            (0x8000, "CanSellCriticalItems"),
        ],
    )
}

fn decode_room_flags(value: u32) -> Vec<String> {
    decode_bits(
        value,
        &[
            (0x0001, "Peasant"),
            (0x0002, "Merchant"),
            (0x0004, "Noble"),
            (0x0008, "Royal"),
        ],
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

fn decode_item_category(value: u32) -> Option<&'static str> {
    match value {
        0x00 => Some("BooksMisc"),
        0x01 => Some("AmuletsAndNecklaces"),
        0x02 => Some("Armor"),
        0x03 => Some("BeltsAndGirdles"),
        0x04 => Some("Boots"),
        0x05 => Some("Arrows"),
        0x06 => Some("BracersAndGauntlets"),
        0x07 => Some("Headgear"),
        0x08 => Some("Keys"),
        0x09 => Some("Potions"),
        0x0A => Some("Rings"),
        0x0B => Some("Scrolls"),
        0x0C => Some("Shields"),
        0x0D => Some("Food"),
        0x0E => Some("Bullets"),
        0x0F => Some("Bows"),
        0x10 => Some("Daggers"),
        0x11 => Some("Maces"),
        0x12 => Some("Slings"),
        0x13 => Some("SmallSwords"),
        0x14 => Some("LargeSwords"),
        0x15 => Some("Hammers"),
        0x16 => Some("MorningStars"),
        0x17 => Some("Flails"),
        0x18 => Some("Darts"),
        0x19 => Some("Axes"),
        0x1A => Some("Quarterstaff"),
        0x1B => Some("Crossbow"),
        0x1C => Some("HandToHandWeapons"),
        0x1D => Some("Spears"),
        0x1E => Some("Halberds"),
        0x1F => Some("CrossbowBolts"),
        0x20 => Some("CloaksAndRobes"),
        0x21 => Some("Gold"),
        0x22 => Some("Gems"),
        0x23 => Some("Wands"),
        0x24 => Some("ContainersEyeBrokenArmor"),
        0x25 => Some("BooksBrokenShieldsBracelets"),
        0x26 => Some("FamiliarsBrokenSwordsEarrings"),
        0x27 => Some("Tattoos"),
        0x28 => Some("Lenses"),
        0x29 => Some("BucklersTeeth"),
        0x2A => Some("Candles"),
        0x2C => Some("Clubs"),
        0x2F => Some("LargeShields"),
        0x31 => Some("MediumShields"),
        0x32 => Some("Notes"),
        0x35 => Some("SmallShields"),
        0x37 => Some("Telescopes"),
        0x38 => Some("Drinks"),
        0x39 => Some("GreatSwords"),
        0x3A => Some("Container"),
        0x3B => Some("FurPelt"),
        0x3C => Some("LeatherArmor"),
        0x3D => Some("StuddedLeatherArmor"),
        0x3E => Some("ChainMail"),
        0x3F => Some("SplintMail"),
        0x40 => Some("HalfPlate"),
        0x41 => Some("FullPlate"),
        0x42 => Some("HideArmor"),
        0x43 => Some("Robe"),
        0x45 => Some("BastardSword"),
        0x46 => Some("Scarf"),
        0x47 => Some("FoodIwd2"),
        0x48 => Some("Hat"),
        0x49 => Some("Gauntlet"),
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
    fn parse_minimal_sto_header_and_tables() {
        let mut bytes =
            vec![0u8; STO_HEADER_SIZE + STO_ITEM_SIZE + STO_DRINK_SIZE + STO_CURE_SIZE + 4];
        let items_offset = STO_HEADER_SIZE as u32;
        let drinks_offset = items_offset + STO_ITEM_SIZE as u32;
        let cures_offset = drinks_offset + STO_DRINK_SIZE as u32;
        let purchased_offset = cures_offset + STO_CURE_SIZE as u32;

        bytes[0..4].copy_from_slice(b"STOR");
        bytes[4..8].copy_from_slice(b"V1.0");
        bytes[0x08..0x0C].copy_from_slice(&3u32.to_le_bytes());
        bytes[0x0C..0x10].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x10..0x14].copy_from_slice(&0x1027u32.to_le_bytes());
        bytes[0x14..0x18].copy_from_slice(&150u32.to_le_bytes());
        bytes[0x18..0x1C].copy_from_slice(&75u32.to_le_bytes());
        bytes[0x20..0x22].copy_from_slice(&10u16.to_le_bytes());
        bytes[0x22..0x24].copy_from_slice(&0u16.to_le_bytes());
        bytes[0x2C..0x30].copy_from_slice(&purchased_offset.to_le_bytes());
        bytes[0x30..0x34].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x34..0x38].copy_from_slice(&items_offset.to_le_bytes());
        bytes[0x38..0x3C].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x3C..0x40].copy_from_slice(&5u32.to_le_bytes());
        bytes[0x40..0x44].copy_from_slice(&100u32.to_le_bytes());
        bytes[0x44..0x4C].copy_from_slice(b"TAVERN\0\0");
        bytes[0x4C..0x50].copy_from_slice(&drinks_offset.to_le_bytes());
        bytes[0x50..0x54].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x54..0x5C].copy_from_slice(b"TEMPLE\0\0");
        bytes[0x5C..0x60].copy_from_slice(&0x0003u32.to_le_bytes());
        bytes[0x60..0x64].copy_from_slice(&10u32.to_le_bytes());
        bytes[0x64..0x68].copy_from_slice(&25u32.to_le_bytes());
        bytes[0x68..0x6C].copy_from_slice(&50u32.to_le_bytes());
        bytes[0x6C..0x70].copy_from_slice(&100u32.to_le_bytes());
        bytes[0x70..0x74].copy_from_slice(&cures_offset.to_le_bytes());
        bytes[0x74..0x78].copy_from_slice(&1u32.to_le_bytes());

        let item_pos = items_offset as usize;
        bytes[item_pos..item_pos + 8].copy_from_slice(b"SW1H01\0\0");
        bytes[item_pos + 0x0A..item_pos + 0x0C].copy_from_slice(&1u16.to_le_bytes());
        bytes[item_pos + 0x10..item_pos + 0x14].copy_from_slice(&0x0001u32.to_le_bytes());
        bytes[item_pos + 0x14..item_pos + 0x18].copy_from_slice(&2u32.to_le_bytes());
        bytes[item_pos + 0x18..item_pos + 0x1C].copy_from_slice(&1u32.to_le_bytes());

        let drink_pos = drinks_offset as usize;
        bytes[drink_pos..drink_pos + 8].copy_from_slice(b"RUMOUR\0\0");
        bytes[drink_pos + 8..drink_pos + 12].copy_from_slice(&2u32.to_le_bytes());
        bytes[drink_pos + 12..drink_pos + 16].copy_from_slice(&5u32.to_le_bytes());
        bytes[drink_pos + 16..drink_pos + 20].copy_from_slice(&3u32.to_le_bytes());

        let cure_pos = cures_offset as usize;
        bytes[cure_pos..cure_pos + 8].copy_from_slice(b"SPPR103\0");
        bytes[cure_pos + 8..cure_pos + 12].copy_from_slice(&50u32.to_le_bytes());

        bytes[purchased_offset as usize..purchased_offset as usize + 4]
            .copy_from_slice(&0x09u32.to_le_bytes());

        let store = parse_sto(&bytes, "TEMPLE.STO", Some(&NullResolver)).expect("STO should parse");
        assert_eq!(store.resource_name, "TEMPLE.STO");
        assert_eq!(store.version, "V1.0");
        assert_eq!(store.header.store_type.decoded.as_deref(), Some("Temple"));
        assert!(store.header.flags.decoded.contains(&"CanBuy".to_string()));
        assert!(
            store
                .header
                .flags
                .decoded
                .contains(&"CanBuyCures".to_string())
        );
        assert_eq!(store.header.room_flags.decoded, vec!["Peasant", "Merchant"]);
        assert_eq!(store.items_for_sale.len(), 1);
        assert_eq!(store.drinks.len(), 1);
        assert_eq!(store.cures.len(), 1);
        assert_eq!(store.items_purchased[0].decoded.as_deref(), Some("Potions"));
        assert_eq!(
            store.items_for_sale[0]
                .resource
                .as_ref()
                .map(|r| r.as_str()),
            Some("SW1H01")
        );
        assert!(store.items_for_sale[0].infinite_supply);
        assert_eq!(
            store.drinks[0].rumour_resource.as_ref().map(|r| r.as_str()),
            Some("RUMOUR")
        );
        assert_eq!(
            store.cures[0].spell_resource.as_ref().map(|r| r.as_str()),
            Some("SPPR103")
        );
    }
}
