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
    assert_eq!(stdout["extended_headers"][0]["spell_form"]["decoded"], "Melee");
    assert_eq!(
        stdout["extended_headers"][0]["target_type"]["decoded"],
        "Any point within range"
    );
    assert_eq!(stdout["extended_headers"][0]["range"], 35);
    assert_eq!(stdout["extended_headers"][0]["level_required"], 1);
    assert_eq!(stdout["extended_headers"][0]["casting_time"], 4);

    assert_eq!(stdout["feature_blocks"][0]["opcode"]["decoded"], "Confusion");
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
    assert_eq!(stdout["feature_blocks"][4]["opcode"]["decoded"], "Play 3D Effect");
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

    assert_eq!(stdout["extended_headers"].as_array().map(Vec::len), Some(20));
    assert_eq!(stdout["extended_headers"][0]["spell_form"]["decoded"], "Melee");
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
