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

// Stock BGEE is not clean in the same way as IWDEE. These seven errors are
// shipped by an unmodded Steam BGEE+SoD installation with SoD still packed in
// dlc/sod-dlc.zip. Pinning their identifying fields gives new errors nowhere to
// hide without making the test depend on the much noisier warning set.
#[test]
fn verify_matches_known_errors_for_stock_bgee_when_ie_bgee_path_is_set() {
    let Some(game_path) = bgee_game_path() else {
        return;
    };

    let issues = run_verify_with_args(&game_path, &["--severity", "error"]);
    let fingerprints = issues
        .iter()
        .map(|issue| {
            (
                issue["resource"].as_str().unwrap_or_default(),
                issue["issue"].as_str().unwrap_or_default(),
                issue["path"].as_str().unwrap_or_default(),
                issue["expected_in"].as_str(),
                issue["expected_value"].as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        fingerprints,
        vec![
            (
                "AR2622.ARE",
                "dead_link",
                "regions[0].destination_area",
                None,
                Some("AR2621.ARE"),
            ),
            (
                "AR2624.ARE",
                "dead_link",
                "regions[0].destination_area",
                None,
                Some("AR2621.ARE"),
            ),
            (
                "AR2637.ARE",
                "phantom_entrance",
                "regions[1].destination_entrance",
                Some("AR2638.ARE"),
                Some("Exit2621"),
            ),
            (
                "AR2638.ARE",
                "phantom_entrance",
                "regions[0].destination_entrance",
                Some("AR2637.ARE"),
                Some("Exit2622"),
            ),
            (
                "AR4201.ARE",
                "phantom_entrance",
                "regions[0].destination_entrance",
                Some("AR4200.ARE"),
                Some("Exit4201"),
            ),
            (
                "BD0121.ARE",
                "phantom_entrance",
                "regions[0].destination_entrance",
                Some("BD0112.ARE"),
                Some("Exitbd0121"),
            ),
            (
                "PH0001.ARE",
                "dead_link",
                "regions[0].destination_area",
                None,
                Some("FW0123.ARE"),
            ),
        ]
    );
}

fn run_verify(game_path: &OsString) -> Vec<Value> {
    run_verify_with_args(game_path, &[])
}

fn run_verify_with_args(game_path: &OsString, extra_args: &[&str]) -> Vec<Value> {
    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("verify")
        .arg("--game")
        .arg(game_path)
        .arg("--format")
        .arg("json")
        .args(extra_args)
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

fn bgee_game_path() -> Option<OsString> {
    std::env::var_os("IE_BGEE_PATH").or_else(|| std::env::var_os("IE_BGEE_GAME_PATH"))
}
