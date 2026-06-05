use serde_json::Value;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn dump_confusion_matches_validated_bg2ee_spell_fields_when_ie_game_path_is_set() {
    let Some(game_path) = std::env::var_os("IE_GAME_PATH") else {
        return;
    };

    let stdout = dump_json(&game_path, "SPWI401.SPL");

    assert_eq!(stdout["resource_name"], "SPWI401.SPL");
    assert_eq!(stdout["version"], "V1  ");
    assert_eq!(stdout["header"]["completion_sound"], "CAS_M05");
    assert_eq!(stdout["header"]["flags"]["raw"], 512);
    assert_eq!(stdout["header"]["flags"]["decoded"][0], "Break Sanctuary");
    assert_eq!(stdout["header"]["spell_type"]["decoded"], "Wizard");
    assert_eq!(stdout["header"]["exclusion_flags"]["raw"], 2048);
    assert_eq!(
        stdout["header"]["exclusion_flags"]["decoded"][0],
        "Invokers"
    );
    assert_eq!(stdout["header"]["school"]["decoded"], "Enchanter");
    assert_eq!(stdout["header"]["secondary_type"]["decoded"], "Disabling");
    assert_eq!(stdout["header"]["spell_level"], 4);
    assert_eq!(stdout["header"]["spellbook_icon"], "SPWI401C");
    assert_eq!(stdout["header"]["extended_headers_count"], 4);
    assert_eq!(stdout["header"]["feature_block_offset"], 274);

    assert_eq!(stdout["extended_headers"].as_array().map(Vec::len), Some(4));
    assert_eq!(
        stdout["extended_headers"][0]["spell_form"]["decoded"],
        "Melee"
    );
    assert_eq!(
        stdout["extended_headers"][0]["target_type"]["decoded"],
        "Any point within range"
    );
    assert_eq!(stdout["extended_headers"][0]["range"], 35);
    assert_eq!(stdout["extended_headers"][0]["level_required"], 1);
    assert_eq!(stdout["extended_headers"][0]["casting_time"], 4);

    assert_eq!(
        stdout["feature_blocks"][0]["opcode"]["decoded"],
        "Confusion"
    );
    assert_eq!(
        stdout["feature_blocks"][1]["opcode"]["decoded"],
        "Display String"
    );
    assert_eq!(
        stdout["feature_blocks"][2]["opcode"]["decoded"],
        "Display Special Effect Icon"
    );
    assert_eq!(
        stdout["feature_blocks"][3]["opcode"]["decoded"],
        "Play Sound Effect"
    );
    assert_eq!(
        stdout["feature_blocks"][4]["opcode"]["decoded"],
        "Play 3D Effect"
    );
}

#[test]
fn dump_silence_matches_validated_bg2ee_priest_spell_fields_when_ie_game_path_is_set() {
    let Some(game_path) = std::env::var_os("IE_GAME_PATH") else {
        return;
    };

    let stdout = dump_json(&game_path, "SPPR211.SPL");

    assert_eq!(stdout["resource_name"], "SPPR211.SPL");
    assert_eq!(stdout["version"], "V1  ");
    assert_eq!(stdout["header"]["completion_sound"], "CAS_P08");
    assert_eq!(stdout["header"]["flags"]["decoded"][0], "Break Sanctuary");
    assert_eq!(stdout["header"]["spell_type"]["decoded"], "Priest");
    assert_eq!(stdout["header"]["exclusion_flags"]["raw"], 2147483648u64);
    assert_eq!(
        stdout["header"]["exclusion_flags"]["decoded"][0],
        "Druid/Ranger/Shaman"
    );
    assert_eq!(stdout["header"]["school"]["decoded"], "Transmuter");
    assert_eq!(stdout["header"]["secondary_type"]["decoded"], "Disabling");
    assert_eq!(stdout["header"]["spell_level"], 2);
    assert_eq!(stdout["header"]["spellbook_icon"], "SPPR211C");
    assert_eq!(stdout["header"]["extended_headers_count"], 20);
    assert_eq!(stdout["header"]["feature_block_offset"], 914);

    assert_eq!(
        stdout["extended_headers"].as_array().map(Vec::len),
        Some(20)
    );
    assert_eq!(
        stdout["extended_headers"][0]["spell_form"]["decoded"],
        "Melee"
    );
    assert_eq!(
        stdout["extended_headers"][0]["target_type"]["decoded"],
        "Any point within range"
    );
    assert_eq!(stdout["extended_headers"][0]["range"], 80);
    assert_eq!(stdout["extended_headers"][0]["level_required"], 1);
    assert_eq!(stdout["extended_headers"][19]["level_required"], 20);
    assert_eq!(stdout["extended_headers"][0]["casting_time"], 5);

    assert_eq!(stdout["feature_blocks"][0]["opcode"]["decoded"], "Silence");
    assert_eq!(
        stdout["feature_blocks"][1]["opcode"]["decoded"],
        "Display String"
    );
    assert_eq!(
        stdout["feature_blocks"][2]["opcode"]["decoded"],
        "Display Special Effect Icon"
    );
    assert_eq!(
        stdout["feature_blocks"][4]["opcode"]["decoded"],
        "Lighting Effects"
    );
}

#[test]
fn dump_baldur_bcs_exposes_named_blocks_when_ie_game_path_is_set() {
    let Some(game_path) = std::env::var_os("IE_GAME_PATH") else {
        return;
    };

    let stdout = dump_json(&game_path, "BALDUR.BCS");

    assert_eq!(stdout["resource_name"], "BALDUR.BCS");
    let blocks = stdout["blocks"]
        .as_array()
        .expect("blocks should be an array");
    assert!(
        !blocks.is_empty(),
        "BALDUR.BCS should contain at least one block"
    );

    let has_named_trigger = blocks.iter().any(|block| {
        block["triggers"]
            .as_array()
            .is_some_and(|triggers| triggers.iter().any(|trigger| trigger["name"].is_string()))
    });
    let has_named_action = blocks.iter().any(|block| {
        block["responses"].as_array().is_some_and(|responses| {
            responses.iter().any(|response| {
                response["actions"]
                    .as_array()
                    .is_some_and(|actions| actions.iter().any(|action| action["name"].is_string()))
            })
        })
    });

    assert!(
        has_named_trigger,
        "expected at least one decoded trigger name in BALDUR.BCS"
    );
    assert!(
        has_named_action,
        "expected at least one decoded action name in BALDUR.BCS"
    );
}

#[test]
fn dump_pstee_area_exposes_actor_links_when_pstee_game_path_is_set() {
    let Some(game_path) = pstee_game_path() else {
        return;
    };

    let stdout = dump_json(&game_path, "AR0202.ARE");

    assert_eq!(stdout["resource_name"], "AR0202.ARE");
    assert_eq!(stdout["resource_type"], "ARE");
    let actors = stdout["actors"]
        .as_array()
        .expect("ARE actors should be an array");
    assert!(!actors.is_empty(), "AR0202.ARE should list actors");
    assert!(
        actors
            .iter()
            .any(|actor| actor["position"]["x"].is_number() && actor["position"]["y"].is_number()),
        "at least one actor should expose placement coordinates"
    );
    assert!(
        actors
            .iter()
            .any(|actor| actor["cre_file"]["exists"] == true
                && actor["cre_file"]["short_name"]["strref"].is_number()),
        "at least one actor should expose CRE display-name enrichment"
    );
    assert!(
        actors
            .iter()
            .any(|actor| actor["dialog"]["exists"].is_boolean()
                || actor["scripts"]["override_script"]["exists"].is_boolean()
                || actor["scripts"]["default_script"]["exists"].is_boolean()),
        "expected actor dialog or script link metadata"
    );

    let deferred = &stdout["deferred_sections"];
    let automap_notes_offset = deferred["automap_notes_offset"]
        .as_u64()
        .expect("automap_notes_offset should be u32");
    let automap_notes_count = deferred["automap_notes_count"]
        .as_u64()
        .expect("automap_notes_count should be u32");
    assert!(
        automap_notes_count < 1024,
        "automap_notes_count looks like an offset, not a count: {automap_notes_count}"
    );
    assert!(
        automap_notes_offset == 0 || automap_notes_offset >= 0x11C,
        "automap_notes_offset must clear the ARE header: {automap_notes_offset:#x}"
    );
    assert_ne!(
        automap_notes_count,
        deferred["projectile_traps_offset"].as_u64().unwrap_or(0),
        "automap_notes_count and projectile_traps_offset must not alias the same bytes"
    );
}

#[test]
fn dump_pstee_mortuary_parses_without_error_when_pstee_game_path_is_set() {
    let Some(game_path) = pstee_game_path() else {
        return;
    };

    // AR0500 is the Mortuary — first area of the game, always present, structurally different
    // from AR0202, satisfying the M6 "2+ real PSTEE area files" acceptance criterion.
    let stdout = dump_json(&game_path, "AR0500.ARE");

    assert_eq!(stdout["resource_type"], "ARE");
    assert_eq!(stdout["version"], "V1.0");
    assert!(
        stdout["actors"].is_array(),
        "AR0500.ARE actors field must be a JSON array"
    );
    let deferred = &stdout["deferred_sections"];
    assert!(
        deferred["regions_count"].is_number(),
        "deferred_sections.regions_count must be present"
    );
    assert!(
        deferred["regions_offset"].is_number(),
        "deferred_sections.regions_offset must be present"
    );
}

#[test]
fn dump_dot_renders_override_dlg_graph() {
    let fixture = GraphTestInstallation::new("dump-dot-dlg");
    fixture.write_override("TEST.DLG", &build_test_dlg());

    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("dump")
        .arg("--game")
        .arg(fixture.root())
        .arg("--resource")
        .arg("TEST.DLG")
        .arg("--format")
        .arg("dot")
        .arg("--strings")
        .arg("strref")
        .output()
        .expect("iecli should run");

    assert!(
        output.status.success(),
        "iecli failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("digraph DLG"));
    assert!(stdout.contains("S0 [shape=box,label=\"S0: #10\"]"));
    assert!(stdout.contains("S0 -> END [label=\"#20\"]"));
}

#[test]
fn dump_dot_rejects_non_dlg_resources() {
    let fixture = GraphTestInstallation::new("dump-dot-non-dlg");
    fixture.write_override("TEST.ITM", b"not an item");

    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("dump")
        .arg("--game")
        .arg(fixture.root())
        .arg("--resource")
        .arg("TEST.ITM")
        .arg("--format")
        .arg("dot")
        .output()
        .expect("iecli should run");

    assert!(!output.status.success(), "dump should fail");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("--format dot is only supported for DLG"),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn dump_dot_follow_extern_renders_referenced_override_dlg() {
    let fixture = GraphTestInstallation::new("dump-dot-follow-extern");
    fixture.write_override("TEST.DLG", &build_external_jump_dlg("OTHER"));
    fixture.write_override("OTHER.DLG", &build_test_dlg());

    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("dump")
        .arg("--game")
        .arg(fixture.root())
        .arg("--resource")
        .arg("TEST.DLG")
        .arg("--format")
        .arg("dot")
        .arg("--strings")
        .arg("strref")
        .arg("--follow-extern")
        .output()
        .expect("iecli should run");

    assert!(
        output.status.success(),
        "iecli failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("subgraph cluster_TEST"));
    assert!(stdout.contains("subgraph cluster_OTHER"));
    assert!(stdout.contains("TEST_S0 -> OTHER_S0 [label=\"#20\"]"));
}

#[test]
fn dump_pstee_tattoos_use_pst_opcode_labels_when_pstee_game_path_is_set() {
    let Some(game_path) = pstee_game_path() else {
        return;
    };

    let cases = [
        ("TTSTR2.ITM", 44, "Strength"),
        ("TTDEX2.ITM", 15, "Dexterity"),
        ("TTCON2.ITM", 10, "Constitution"),
        ("TTINT2.ITM", 19, "Intelligence"),
        ("TTWIS2.ITM", 49, "Wisdom"),
        ("TTCHR2.ITM", 6, "Charisma"),
        ("TTAC2.ITM", 0, "AC"),
        ("TTMAXHP2.ITM", 18, "Max HP"),
    ];

    for (resource_name, raw_opcode, expected_label) in cases {
        let stdout = dump_json(&game_path, resource_name);
        assert_eq!(stdout["resource_name"], resource_name);
        assert!(
            item_has_effect_opcode(&stdout, raw_opcode, expected_label),
            "{resource_name} should contain opcode {raw_opcode} decoded as {expected_label}; stdout={stdout}"
        );
    }
}

fn pstee_game_path() -> Option<OsString> {
    std::env::var_os("IE_PSTEE_PATH").or_else(|| std::env::var_os("IE_PSTEE_GAME_PATH"))
}

fn dump_json(game_path: &OsString, resource_name: &str) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("dump")
        .arg("--game")
        .arg(game_path)
        .arg("--resource")
        .arg(resource_name)
        .arg("--format")
        .arg("json")
        .output()
        .expect("iecli should run");

    assert!(
        output.status.success(),
        "iecli failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("dump should emit JSON")
}

fn item_has_effect_opcode(stdout: &Value, raw_opcode: u64, expected_label: &str) -> bool {
    stdout["global_effects"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(
            stdout["abilities"]
                .as_array()
                .into_iter()
                .flatten()
                .flat_map(|ability| ability["effects"].as_array().into_iter().flatten()),
        )
        .any(|effect| {
            effect["opcode"]["raw"].as_u64() == Some(raw_opcode)
                && effect["opcode"]["decoded"].as_str() == Some(expected_label)
        })
}

struct GraphTestInstallation {
    root: PathBuf,
}

impl GraphTestInstallation {
    fn new(label: &str) -> Self {
        let mut root = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        root.push(format!("iecli-{label}-{unique}-{}", std::process::id()));
        fs::create_dir_all(&root).expect("temporary installation root should be creatable");
        fs::write(root.join("dialog.tlk"), build_minimal_tlk()).expect("dialog.tlk should exist");
        fs::write(root.join("chitin.key"), build_empty_key()).expect("chitin.key should exist");
        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn write_override(&self, resource_name: &str, bytes: &[u8]) {
        let override_dir = self.root.join("override");
        fs::create_dir_all(&override_dir).expect("override directory should be creatable");
        fs::write(override_dir.join(resource_name), bytes)
            .expect("override resource should be writable");
    }
}

impl Drop for GraphTestInstallation {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn build_test_dlg() -> Vec<u8> {
    let offset_states = 0x34u32;
    let offset_transitions = offset_states + 16;
    let mut bytes = vec![0u8; offset_transitions as usize + 32];

    bytes[0..4].copy_from_slice(b"DLG ");
    bytes[4..8].copy_from_slice(b"V1.0");
    bytes[0x08..0x0C].copy_from_slice(&1u32.to_le_bytes());
    bytes[0x0C..0x10].copy_from_slice(&offset_states.to_le_bytes());
    bytes[0x10..0x14].copy_from_slice(&1u32.to_le_bytes());
    bytes[0x14..0x18].copy_from_slice(&offset_transitions.to_le_bytes());
    bytes[0x30..0x34].copy_from_slice(&0u32.to_le_bytes());

    let state = offset_states as usize;
    bytes[state..state + 4].copy_from_slice(&10u32.to_le_bytes());
    bytes[state + 4..state + 8].copy_from_slice(&0u32.to_le_bytes());
    bytes[state + 8..state + 12].copy_from_slice(&1u32.to_le_bytes());
    bytes[state + 12..state + 16].copy_from_slice(&u32::MAX.to_le_bytes());

    let transition = offset_transitions as usize;
    bytes[transition..transition + 4].copy_from_slice(&0x0009u32.to_le_bytes());
    bytes[transition + 4..transition + 8].copy_from_slice(&20u32.to_le_bytes());
    bytes[transition + 8..transition + 12].copy_from_slice(&u32::MAX.to_le_bytes());
    bytes[transition + 12..transition + 16].copy_from_slice(&u32::MAX.to_le_bytes());
    bytes[transition + 16..transition + 20].copy_from_slice(&u32::MAX.to_le_bytes());
    bytes[transition + 28..transition + 32].copy_from_slice(&u32::MAX.to_le_bytes());

    bytes
}

fn build_external_jump_dlg(next_dialog: &str) -> Vec<u8> {
    let offset_states = 0x34u32;
    let offset_transitions = offset_states + 16;
    let mut bytes = vec![0u8; offset_transitions as usize + 32];

    bytes[0..4].copy_from_slice(b"DLG ");
    bytes[4..8].copy_from_slice(b"V1.0");
    bytes[0x08..0x0C].copy_from_slice(&1u32.to_le_bytes());
    bytes[0x0C..0x10].copy_from_slice(&offset_states.to_le_bytes());
    bytes[0x10..0x14].copy_from_slice(&1u32.to_le_bytes());
    bytes[0x14..0x18].copy_from_slice(&offset_transitions.to_le_bytes());
    bytes[0x30..0x34].copy_from_slice(&0u32.to_le_bytes());

    let state = offset_states as usize;
    bytes[state..state + 4].copy_from_slice(&10u32.to_le_bytes());
    bytes[state + 4..state + 8].copy_from_slice(&0u32.to_le_bytes());
    bytes[state + 8..state + 12].copy_from_slice(&1u32.to_le_bytes());
    bytes[state + 12..state + 16].copy_from_slice(&u32::MAX.to_le_bytes());

    let transition = offset_transitions as usize;
    bytes[transition..transition + 4].copy_from_slice(&0x0001u32.to_le_bytes());
    bytes[transition + 4..transition + 8].copy_from_slice(&20u32.to_le_bytes());
    bytes[transition + 8..transition + 12].copy_from_slice(&u32::MAX.to_le_bytes());
    bytes[transition + 12..transition + 16].copy_from_slice(&u32::MAX.to_le_bytes());
    bytes[transition + 16..transition + 20].copy_from_slice(&u32::MAX.to_le_bytes());
    let mut resref = [0u8; 8];
    resref[..next_dialog.len()].copy_from_slice(next_dialog.as_bytes());
    bytes[transition + 20..transition + 28].copy_from_slice(&resref);
    bytes[transition + 28..transition + 32].copy_from_slice(&0u32.to_le_bytes());

    bytes
}

fn build_empty_key() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"KEY V1  ");
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&24u32.to_le_bytes());
    bytes.extend_from_slice(&24u32.to_le_bytes());
    bytes
}

fn build_minimal_tlk() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"TLK V1  ");
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&18u32.to_le_bytes());
    bytes
}
