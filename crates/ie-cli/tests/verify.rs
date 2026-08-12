use serde_json::Value;
use std::ffi::OsString;
use std::process::Command;

// A stock install is the strongest available oracle for verify: whatever the
// shipped game does is by definition not breakage. Every issue reported against
// an unmodded IWDEE was a false positive — six phantom entrances that differed
// only in case (Fr3501 vs FR3501) and one NONE script sentinel — so this test
// pins the clean result rather than a specific issue list.
#[test]
fn verify_reports_no_issues_for_stock_iwdee_when_iwdee_game_path_is_set() {
    let Some(game_path) = iwdee_game_path() else {
        return;
    };

    let issues = run_verify(&game_path);

    assert!(
        issues.is_empty(),
        "stock IWDEE should verify clean; got {}",
        serde_json::to_string_pretty(&issues).unwrap_or_default()
    );
}

fn run_verify(game_path: &OsString) -> Vec<Value> {
    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("verify")
        .arg("--game")
        .arg(game_path)
        .arg("--format")
        .arg("json")
        .output()
        .expect("iecli should run");

    assert!(
        output.status.success(),
        "iecli failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("verify should emit JSON")
}

fn iwdee_game_path() -> Option<OsString> {
    std::env::var_os("IE_IWDEE_PATH").or_else(|| std::env::var_os("IE_IWDEE_GAME_PATH"))
}
