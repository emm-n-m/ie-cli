use serde_json::Value;
use std::env;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn packed_bgee_sod_dlc_is_visible_to_list_dump_and_tlk() {
    let Some(game) = env::var_os("IE_BGEE_SOD_PATH") else {
        return;
    };
    let game = PathBuf::from(game);

    let list = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .args([
            "list",
            "--game",
            game.to_str().expect("game path should be UTF-8"),
            "--type",
            "CRE",
            "--name",
            "BD*",
            "--format",
            "json",
        ])
        .output()
        .expect("iecli should run");
    assert!(
        list.status.success(),
        "list failed: {}",
        String::from_utf8_lossy(&list.stderr)
    );
    let entries: Value = serde_json::from_slice(&list.stdout).expect("list should emit JSON");
    let entry = entries
        .as_array()
        .and_then(|entries| entries.iter().find(|entry| entry["source_kind"] == "dlc"))
        .expect("packed SoD should list at least one DLC-backed BD CRE");
    let resource = entry["resource_name"]
        .as_str()
        .expect("listed resource should have a name");

    let dump = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .args([
            "dump",
            "--game",
            game.to_str().expect("game path should be UTF-8"),
            "--resource",
            resource,
            "--format",
            "json",
        ])
        .output()
        .expect("iecli should run");
    assert!(
        dump.status.success(),
        "dump failed for {resource}: {}",
        String::from_utf8_lossy(&dump.stderr)
    );
    let decoded: Value = serde_json::from_slice(&dump.stdout).expect("dump should emit JSON");
    assert_eq!(decoded["resource_type"], "CRE");

    let tlk = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .args([
            "tlk",
            "--game",
            game.to_str().expect("game path should be UTF-8"),
            "--strref",
            "50000",
        ])
        .output()
        .expect("iecli should run");
    assert!(
        tlk.status.success(),
        "tlk failed: {}",
        String::from_utf8_lossy(&tlk.stderr)
    );
    let resolved: Value = serde_json::from_slice(&tlk.stdout).expect("tlk should emit JSON");
    assert_eq!(resolved["text"], "If I could, I would. But I can't, so...");
    assert!(
        resolved["dialog_tlk"]
            .as_str()
            .is_some_and(|path| path.contains("!"))
    );
}
