use serde_json::Value;
use std::ffi::OsString;
use std::process::Command;

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
