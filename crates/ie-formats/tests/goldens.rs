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
    CreatureResourceLink, GameVariant, IdsResolver, ResRef, ResolvedStrRef, ResolverBundle,
    ResourceBytes, ResourceLink, ResourceLinkResolver, ResourceMetadata, ResourceType, SourceKind,
    StrRef, StrRefResolver,
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

/// Names every opcode and IDS lookup deterministically.
///
/// The real resolver reads `.ids` files out of an install, so pinning its output
/// would pin that install. This keeps the decoded-name path exercised -- BCS
/// output is mostly these names -- without depending on one.
struct MarkerIds;

impl IdsResolver for MarkerIds {
    fn resolve_trigger(&self, opcode: i32) -> Option<String> {
        Some(format!("Trigger{opcode}"))
    }

    fn resolve_action(&self, opcode: i32) -> Option<String> {
        Some(format!("Action{opcode}"))
    }

    fn resolve_ids(&self, file: &str, value: i32) -> Option<String> {
        Some(format!("{file}#{value}"))
    }
}

/// Resolves every link as present, with no source path.
///
/// `source_path` is the one field here that would otherwise carry a machine
/// path into a committed golden. A link fixture states the *shape* of a
/// resolved link; where the file sat on the machine that generated it is not
/// part of that.
struct MarkerLinks;

impl MarkerLinks {
    fn link(resref: &ResRef, resource_type: ResourceType) -> ResourceLink {
        ResourceLink {
            resref: resref.clone(),
            resource_name: format!("{}.{}", resref.as_str(), resource_type.as_str()),
            resource_type: resource_type.as_str().to_string(),
            exists: true,
            source_kind: Some(SourceKind::Override),
            source_path: None,
        }
    }
}

impl ResourceLinkResolver for MarkerLinks {
    fn resolve_resource_link(&self, resref: &ResRef, resource_type: ResourceType) -> ResourceLink {
        Self::link(resref, resource_type)
    }

    fn resolve_creature_link(&self, resref: &ResRef) -> CreatureResourceLink {
        CreatureResourceLink {
            link: Self::link(resref, ResourceType::Cre),
            short_name: Some(ResolvedStrRef {
                strref: StrRef(1),
                text: Some("<string 1>".to_string()),
            }),
            long_name: None,
        }
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

#[test]
fn sto_json_matches_golden() {
    assert_golden("sto", ResourceType::Sto, "TEMPLE.STO", &minimal_sto());
}

#[test]
fn dlg_json_matches_golden() {
    assert_golden("dlg", ResourceType::Dlg, "IMOEN.DLG", &minimal_dlg());
}

#[test]
fn are_json_matches_golden() {
    assert_golden("are", ResourceType::Are, "AR0202.ARE", &minimal_are());
}

#[test]
fn bcs_json_matches_golden() {
    assert_golden("bcs", ResourceType::Bcs, "RASAAD.BCS", MINIMAL_BCS);
}

/// Saves are reached through `save-info` rather than `dump`, so they never pass
/// through `decode_to_json` and the shape goldens never see them either. These
/// two are the only thing pinning either output.
#[test]
fn gam_json_matches_golden() {
    let parsed = ie_formats::parse_gam(&minimal_gam(), "BALDUR.GAM", Some(&MarkerResolver))
        .expect("GAM fixture should parse");
    assert_golden_value("gam", &parsed);
}

#[test]
fn sav_json_matches_golden() {
    let parsed = ie_formats::parse_sav(&minimal_sav(), "BALDUR.SAV").expect("SAV should parse");
    assert_golden_value("sav", &parsed);
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
            ids: Some(&MarkerIds),
            links: Some(&MarkerLinks),
        },
    )
    .unwrap_or_else(|error| panic!("{name} fixture should decode: {error}"));

    assert_golden_value(name, &value);
}

fn assert_golden_value<T: serde::Serialize>(name: &str, value: &T) {
    let actual = format!(
        "{}\n",
        serde_json::to_string_pretty(value).expect("decoded JSON should serialize")
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

const STO_HEADER_SIZE: usize = 0x9C;
const STO_ITEM_SIZE: usize = 0x1C;
const STO_DRINK_SIZE: usize = 0x14;
const STO_CURE_SIZE: usize = 0x0C;
const ARE_HEADER_SIZE: usize = 0x11C;
const ARE_ACTOR_SIZE: usize = 0x110;
const ARE_REGION_SIZE: usize = 0xC4;
const ARE_ENTRANCE_SIZE: usize = 0x68;
const DLG_HEADER_WITH_FLAGS: usize = 0x34;
const DLG_STATE_SIZE: usize = 16;
const DLG_TRANSITION_SIZE: usize = 32;
const DLG_SCRIPT_ENTRY_SIZE: usize = 8;

/// A tavern-and-temple store: one item, one drink, one cure, one purchase
/// category. Each table is populated because an empty one serializes as `[]` and
/// pins nothing about the records inside it.
fn minimal_sto() -> Vec<u8> {
    let items_offset = STO_HEADER_SIZE as u32;
    let drinks_offset = items_offset + STO_ITEM_SIZE as u32;
    let cures_offset = drinks_offset + STO_DRINK_SIZE as u32;
    let purchased_offset = cures_offset + STO_CURE_SIZE as u32;
    let mut bytes = vec![0u8; purchased_offset as usize + 4];

    bytes[0..4].copy_from_slice(b"STOR");
    bytes[4..8].copy_from_slice(b"V1.0");
    bytes[0x08..0x0C].copy_from_slice(&3u32.to_le_bytes()); // store type
    bytes[0x0C..0x10].copy_from_slice(&1u32.to_le_bytes()); // name strref
    bytes[0x10..0x14].copy_from_slice(&0x1027u32.to_le_bytes()); // flags
    bytes[0x14..0x18].copy_from_slice(&150u32.to_le_bytes()); // sell markup
    bytes[0x18..0x1C].copy_from_slice(&75u32.to_le_bytes()); // buy markup
    bytes[0x20..0x22].copy_from_slice(&10u16.to_le_bytes());
    bytes[0x2C..0x30].copy_from_slice(&purchased_offset.to_le_bytes());
    bytes[0x30..0x34].copy_from_slice(&1u32.to_le_bytes());
    bytes[0x34..0x38].copy_from_slice(&items_offset.to_le_bytes());
    bytes[0x38..0x3C].copy_from_slice(&1u32.to_le_bytes());
    bytes[0x3C..0x40].copy_from_slice(&5u32.to_le_bytes()); // lore
    bytes[0x40..0x44].copy_from_slice(&100u32.to_le_bytes()); // id price
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

    let item = items_offset as usize;
    bytes[item..item + 8].copy_from_slice(b"SW1H01\0\0");
    bytes[item + 0x0A..item + 0x0C].copy_from_slice(&1u16.to_le_bytes());
    bytes[item + 0x10..item + 0x14].copy_from_slice(&0x0001u32.to_le_bytes());
    bytes[item + 0x14..item + 0x18].copy_from_slice(&2u32.to_le_bytes());
    bytes[item + 0x18..item + 0x1C].copy_from_slice(&1u32.to_le_bytes());

    let drink = drinks_offset as usize;
    bytes[drink..drink + 8].copy_from_slice(b"RUMOUR\0\0");
    bytes[drink + 8..drink + 12].copy_from_slice(&2u32.to_le_bytes());
    bytes[drink + 12..drink + 16].copy_from_slice(&5u32.to_le_bytes());
    bytes[drink + 16..drink + 20].copy_from_slice(&3u32.to_le_bytes());

    let cure = cures_offset as usize;
    bytes[cure..cure + 8].copy_from_slice(b"SPPR103\0");
    bytes[cure + 8..cure + 12].copy_from_slice(&50u32.to_le_bytes());

    let purchased = purchased_offset as usize;
    bytes[purchased..purchased + 4].copy_from_slice(&0x09u32.to_le_bytes());

    bytes
}

/// One area with an actor, a Travel region, and an entrance.
///
/// All three tables are populated deliberately. A Travel region is what `verify`
/// walks and what `patch` edits, and an empty table serializes as `[]` and pins
/// nothing about the records it would hold -- so an empty-table golden would have
/// left the two structures this tool manipulates most completely unpinned.
fn minimal_are() -> Vec<u8> {
    let actor_offset = ARE_HEADER_SIZE;
    let region_offset = actor_offset + ARE_ACTOR_SIZE;
    let entrance_offset = region_offset + ARE_REGION_SIZE;
    let mut bytes = vec![0u8; entrance_offset + ARE_ENTRANCE_SIZE];

    bytes[0..4].copy_from_slice(b"AREA");
    bytes[4..8].copy_from_slice(b"V1.0");
    bytes[0x08..0x10].copy_from_slice(b"AR0202\0\0");
    bytes[0x14..0x18].copy_from_slice(&0x21u32.to_le_bytes()); // area flags
    bytes[0x48..0x4A].copy_from_slice(&0x0400u16.to_le_bytes());
    bytes[0x54..0x58].copy_from_slice(&(actor_offset as u32).to_le_bytes());
    bytes[0x58..0x5A].copy_from_slice(&1u16.to_le_bytes()); // actor count
    bytes[0x5A..0x5C].copy_from_slice(&1u16.to_le_bytes()); // region count
    bytes[0x5C..0x60].copy_from_slice(&(region_offset as u32).to_le_bytes());
    bytes[0x68..0x6C].copy_from_slice(&(entrance_offset as u32).to_le_bytes());
    bytes[0x6C..0x70].copy_from_slice(&1u32.to_le_bytes()); // entrance count
    bytes[0x94..0x9C].copy_from_slice(b"AR0202\0\0");

    // A Travel region: type 2 is what makes the destination fields decode.
    let region = region_offset;
    bytes[region..region + 6].copy_from_slice(b"Door19");
    bytes[region + 0x20..region + 0x22].copy_from_slice(&2u16.to_le_bytes());
    bytes[region + 0x22..region + 0x24].copy_from_slice(&10u16.to_le_bytes());
    bytes[region + 0x24..region + 0x26].copy_from_slice(&20u16.to_le_bytes());
    bytes[region + 0x26..region + 0x28].copy_from_slice(&30u16.to_le_bytes());
    bytes[region + 0x28..region + 0x2A].copy_from_slice(&40u16.to_le_bytes());
    bytes[region + 0x2A..region + 0x2C].copy_from_slice(&4u16.to_le_bytes());
    bytes[region + 0x38..region + 0x40].copy_from_slice(b"AR0203\0\0");
    bytes[region + 0x40..region + 0x46].copy_from_slice(b"Exit19");
    bytes[region + 0x74..region + 0x7C].copy_from_slice(b"KEY01\0\0\0");
    bytes[region + 0x7C..region + 0x84].copy_from_slice(b"RGNSCR\0\0");

    let entrance = entrance_offset;
    bytes[entrance..entrance + 6].copy_from_slice(b"Exit19");
    bytes[entrance + 0x20..entrance + 0x22].copy_from_slice(&256u16.to_le_bytes());
    bytes[entrance + 0x22..entrance + 0x24].copy_from_slice(&512u16.to_le_bytes());
    bytes[entrance + 0x24..entrance + 0x26].copy_from_slice(&8u16.to_le_bytes());

    let actor = actor_offset;
    bytes[actor..actor + 5].copy_from_slice(b"Grace");
    bytes[actor + 0x20..actor + 0x22].copy_from_slice(&100u16.to_le_bytes());
    bytes[actor + 0x22..actor + 0x24].copy_from_slice(&120u16.to_le_bytes());
    bytes[actor + 0x24..actor + 0x26].copy_from_slice(&140u16.to_le_bytes());
    bytes[actor + 0x26..actor + 0x28].copy_from_slice(&160u16.to_le_bytes());
    bytes[actor + 0x28..actor + 0x2C].copy_from_slice(&0x0006u32.to_le_bytes());
    bytes[actor + 0x2C..actor + 0x2E].copy_from_slice(&1u16.to_le_bytes());
    bytes[actor + 0x34..actor + 0x36].copy_from_slice(&12u16.to_le_bytes());
    bytes[actor + 0x38..actor + 0x3C].copy_from_slice(&(-1i32).to_le_bytes());
    bytes[actor + 0x40..actor + 0x44].copy_from_slice(&0x00FF_FFFFu32.to_le_bytes());
    bytes[actor + 0x48..actor + 0x50].copy_from_slice(b"DGRACE\0\0");
    bytes[actor + 0x50..actor + 0x58].copy_from_slice(b"OVR\0\0\0\0\0");
    bytes[actor + 0x70..actor + 0x78].copy_from_slice(b"DEF\0\0\0\0\0");
    bytes[actor + 0x80..actor + 0x88].copy_from_slice(b"DGRACE\0\0");

    bytes
}

/// Two states and three transitions, with a state trigger, a transition trigger,
/// and an action -- the four tables a DLG can carry, each non-empty.
fn minimal_dlg() -> Vec<u8> {
    let state_trigger = b"CheckStatGT(Myself,12,STR)";
    let transition_trigger = b"Global(\"X\",\"GLOBAL\",0)";
    let action = b"SetGlobal(\"X\",\"GLOBAL\",1)";

    let states_offset = DLG_HEADER_WITH_FLAGS as u32;
    let transitions_offset = states_offset + (2 * DLG_STATE_SIZE as u32);
    let state_triggers_offset = transitions_offset + (3 * DLG_TRANSITION_SIZE as u32);
    let transition_triggers_offset = state_triggers_offset + DLG_SCRIPT_ENTRY_SIZE as u32;
    let actions_offset = transition_triggers_offset + DLG_SCRIPT_ENTRY_SIZE as u32;
    let strings_offset = actions_offset + DLG_SCRIPT_ENTRY_SIZE as u32;

    let state_trigger_at = strings_offset;
    let transition_trigger_at = state_trigger_at + state_trigger.len() as u32;
    let action_at = transition_trigger_at + transition_trigger.len() as u32;
    let mut bytes = vec![0u8; (action_at + action.len() as u32) as usize];

    bytes[0..4].copy_from_slice(b"DLG ");
    bytes[4..8].copy_from_slice(b"V1.0");
    bytes[0x08..0x0C].copy_from_slice(&2u32.to_le_bytes());
    bytes[0x0C..0x10].copy_from_slice(&states_offset.to_le_bytes());
    bytes[0x10..0x14].copy_from_slice(&3u32.to_le_bytes());
    bytes[0x14..0x18].copy_from_slice(&transitions_offset.to_le_bytes());
    bytes[0x18..0x1C].copy_from_slice(&state_triggers_offset.to_le_bytes());
    bytes[0x1C..0x20].copy_from_slice(&1u32.to_le_bytes());
    bytes[0x20..0x24].copy_from_slice(&transition_triggers_offset.to_le_bytes());
    bytes[0x24..0x28].copy_from_slice(&1u32.to_le_bytes());
    bytes[0x28..0x2C].copy_from_slice(&actions_offset.to_le_bytes());
    bytes[0x2C..0x30].copy_from_slice(&1u32.to_le_bytes());

    let state0 = states_offset as usize;
    bytes[state0..state0 + 4].copy_from_slice(&100u32.to_le_bytes());
    bytes[state0 + 8..state0 + 12].copy_from_slice(&2u32.to_le_bytes());

    let state1 = state0 + DLG_STATE_SIZE;
    bytes[state1..state1 + 4].copy_from_slice(&101u32.to_le_bytes());
    bytes[state1 + 4..state1 + 8].copy_from_slice(&2u32.to_le_bytes());
    bytes[state1 + 8..state1 + 12].copy_from_slice(&1u32.to_le_bytes());
    bytes[state1 + 12..state1 + 16].copy_from_slice(&u32::MAX.to_le_bytes());

    // Transition 0 carries a reply, a trigger, an action, and a next-state link;
    // transition 1 ends the dialogue; transition 2 hands off to another DLG.
    let transition0 = transitions_offset as usize;
    bytes[transition0..transition0 + 4].copy_from_slice(&0b0000_0111u32.to_le_bytes());
    bytes[transition0 + 4..transition0 + 8].copy_from_slice(&200u32.to_le_bytes());
    bytes[transition0 + 12..transition0 + 16].copy_from_slice(&0u32.to_le_bytes());
    bytes[transition0 + 16..transition0 + 24].copy_from_slice(b"IMOEN\0\0\0");
    bytes[transition0 + 24..transition0 + 28].copy_from_slice(&1u32.to_le_bytes());

    let transition1 = transition0 + DLG_TRANSITION_SIZE;
    bytes[transition1..transition1 + 4].copy_from_slice(&0b0000_1000u32.to_le_bytes());

    let transition2 = transition1 + DLG_TRANSITION_SIZE;
    bytes[transition2..transition2 + 4].copy_from_slice(&0b0000_0001u32.to_le_bytes());
    bytes[transition2 + 4..transition2 + 8].copy_from_slice(&201u32.to_le_bytes());
    bytes[transition2 + 16..transition2 + 24].copy_from_slice(b"JAHEIRA\0");

    let entry = state_triggers_offset as usize;
    bytes[entry..entry + 4].copy_from_slice(&state_trigger_at.to_le_bytes());
    bytes[entry + 4..entry + 8].copy_from_slice(&(state_trigger.len() as u32).to_le_bytes());

    let entry = transition_triggers_offset as usize;
    bytes[entry..entry + 4].copy_from_slice(&transition_trigger_at.to_le_bytes());
    bytes[entry + 4..entry + 8].copy_from_slice(&(transition_trigger.len() as u32).to_le_bytes());

    let entry = actions_offset as usize;
    bytes[entry..entry + 4].copy_from_slice(&action_at.to_le_bytes());
    bytes[entry + 4..entry + 8].copy_from_slice(&(action.len() as u32).to_le_bytes());

    let at = state_trigger_at as usize;
    bytes[at..at + state_trigger.len()].copy_from_slice(state_trigger);
    let at = transition_trigger_at as usize;
    bytes[at..at + transition_trigger.len()].copy_from_slice(transition_trigger);
    let at = action_at as usize;
    bytes[at..at + action.len()].copy_from_slice(action);

    bytes
}

/// One condition block with two triggers and two responses.
///
/// BCS is a text format, so this fixture is legible as written -- which makes it
/// the one place in this file where a reader can see what is being pinned
/// without decoding offsets by hand.
const MINIMAL_BCS: &[u8] = br#"SC
CR
CO
TR
1 10 20 30 40 "PLOT" "GLOBAL" OB
2 0 0 0 0 0 0 7 0 0 0 0 "Myself" OB
TR
TR
-2 1 2 3 4 "" ""OB
0 0 0 0 0 0 0 -1 0 0 0 0 ""OB
TR
CO
RS
RE
100AC
30OB
2 0 0 0 0 0 0 0 0 0 0 0 ""OB
OB
0 0 0 0 0 0 0 0 0 0 0 0 ""OB
OB
0 0 0 0 0 0 0 0 1 2 3 4 "DV"OB
5 7 8 9 10 "RASAAD_PLOT" "GLOBAL" AC
AC
40OB
0 0 0 0 0 0 0 0 0 0 0 0 ""OB
OB
0 0 0 0 0 0 0 0 0 0 0 0 ""OB
OB
0 0 0 0 0 0 0 0 0 0 0 0 ""OB
6 0 0 0 0 "" "" AC
RE
RS
CR
SC"#;

const GAM_V2_HEADER_SIZE: usize = 0xB4;
const GAM_V2_NPC_SIZE: usize = 0x160;
const GAM_V2_VARIABLE_SIZE: usize = 0x54;

/// A `GAMEV2.0` save with one party member and one global.
fn minimal_gam() -> Vec<u8> {
    let npc = GAM_V2_HEADER_SIZE;
    let global = npc + GAM_V2_NPC_SIZE;
    let mut bytes = vec![0u8; global + GAM_V2_VARIABLE_SIZE];

    bytes[0..8].copy_from_slice(b"GAMEV2.0");
    write_u32(&mut bytes, 0x08, 2181); // game time
    write_u32(&mut bytes, 0x18, 1234); // party gold
    write_u16(&mut bytes, 0x1C, 0xFFFF);
    write_u32(&mut bytes, 0x20, GAM_V2_HEADER_SIZE as u32);
    write_u32(&mut bytes, 0x24, 1); // party count
    write_u32(&mut bytes, 0x38, global as u32);
    write_u32(&mut bytes, 0x3C, 1); // global count
    bytes[0x40..0x46].copy_from_slice(b"AR0602"); // main area
    write_u32(&mut bytes, 0x54, 125); // reputation
    bytes[0x58..0x5E].copy_from_slice(b"AR0602"); // current area
    write_u32(&mut bytes, 0x74, 9876);

    write_u16(&mut bytes, npc, 1); // in party
    write_u32(&mut bytes, npc + 0x04, 0x500);
    write_u32(&mut bytes, npc + 0x08, 0x300);
    bytes[npc + 0x0C..npc + 0x11].copy_from_slice(b"IMOEN");
    bytes[npc + 0x18..npc + 0x1E].copy_from_slice(b"AR0602");
    write_u16(&mut bytes, npc + 0x20, 100);
    write_u16(&mut bytes, npc + 0x22, 200);
    bytes[npc + 0xC0..npc + 0xCA].copy_from_slice(b"Imoen Long");
    write_u32(&mut bytes, npc + 0xE0, 7); // talk count
    write_u32(&mut bytes, npc + 0xE4, 42);
    bytes[npc + 0xF4] = 1;

    bytes[global..global + 7].copy_from_slice(b"CHAPTER");
    write_u16(&mut bytes, global + 0x20, 1);
    write_u32(&mut bytes, global + 0x28, 3);

    bytes
}

/// A `SAV V1.0` archive holding one zlib-compressed entry.
///
/// The payload is deliberately not a real ARE: `save-info` inventories entries
/// and inflates them for sizing rather than decoding them, and the golden should
/// pin that boundary rather than blur it.
fn minimal_sav() -> Vec<u8> {
    use flate2::Compression;
    use flate2::write::ZlibEncoder;
    use std::io::Write;

    let payload = b"AREA bytes";
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(payload).expect("payload should compress");
    let compressed = encoder.finish().expect("encoder should finish");

    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"SAV V1.0");
    bytes.extend_from_slice(&11u32.to_le_bytes());
    bytes.extend_from_slice(b"AR0602.ARE\0");
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&compressed);
    bytes
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
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
