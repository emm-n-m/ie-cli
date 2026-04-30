use serde_json::Value;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn dump_raw_writes_known_bg2ee_resource_when_ie_game_path_is_set() {
    let Some(game_path) = std::env::var_os("IE_GAME_PATH") else {
        return;
    };

    let mut output_path = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    output_path.push(format!("iecli-dump-raw-{unique}.itm"));

    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("dump-raw")
        .arg("--game")
        .arg(&game_path)
        .arg("--resource")
        .arg("ACIDBL.ITM")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("iecli should run");

    assert!(
        output.status.success(),
        "iecli failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let written = fs::read(&output_path).expect("dump-raw should write the requested file");
    assert!(written.starts_with(b"ITM "));

    let stdout: Value =
        serde_json::from_slice(&output.stdout).expect("dump-raw should emit JSON summary");
    assert_eq!(stdout["resource_name"], "ACIDBL.ITM");
    assert_eq!(stdout["bytes_written"], written.len());

    let _ = fs::remove_file(output_path);
}
