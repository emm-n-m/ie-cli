//! Value goldens for command output that does not pass through the decoded
//! resource JSON golden suite.
//!
//! This covers install-level JSON (`list`, `locate`, `verify`, `override-diff`,
//! `save-list`, and `tlk`), DLG graph output, and the human-readable text modes.
//! Neither the per-format value goldens in `ie-formats` nor the real-install
//! shape goldens cover these interfaces.
//!
//! These build a synthetic install in a temp directory, so they run in CI with no
//! game data and can pin values exactly. Building the install rather than reading
//! one also settles what would otherwise be unpinnable: precedence between an
//! override and a KEY-backed BIF is *stated* by the fixture, so a golden can
//! assert which one won rather than describe whichever the local install had.
//!
//! One redaction is unavoidable. These outputs carry absolute paths, which differ
//! per machine and per run, so `<install>` is substituted for the temp root and
//! `\` is normalized to `/` before comparison. Everything else is pinned as-is,
//! including the trailing path components, which is what tells an override apart
//! from a BIF-backed hit.
//!
//! Regenerate with `UPDATE_GOLDENS=1`, matching `ie-formats`.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const ITM_TYPE_CODE: u16 = 0x03ED;
const RESOURCE_LOCATOR: u32 = 0;

#[test]
fn list_json_matches_golden() {
    let install = TempInstall::new("list");
    assert_json_golden("list", &install, &["list", "--format", "json"]);
}

#[test]
fn list_filtered_by_type_and_source_matches_golden() {
    let install = TempInstall::new("list-filtered");
    assert_json_golden(
        "list-override-itm",
        &install,
        &[
            "list", "--type", "ITM", "--source", "override", "--format", "json",
        ],
    );
}

#[test]
fn locate_prefers_override_and_reports_it() {
    // SHADOW.ITM exists in both the BIF and the override, so this pins the
    // precedence rule itself rather than just the output shape.
    let install = TempInstall::new("locate-override");
    assert_json_golden(
        "locate-override",
        &install,
        &["locate", "--resource", "SHADOW.ITM"],
    );
}

#[test]
fn locate_with_explicit_bif_source_reaches_past_the_override() {
    let install = TempInstall::new("locate-bif");
    assert_json_golden(
        "locate-bif",
        &install,
        &["locate", "--resource", "SHADOW.ITM", "--source", "bif"],
    );
}

#[test]
fn verify_reports_a_dead_travel_link() {
    // The area's Travel region points at AR9999, which the install does not
    // contain. That is the issue `verify` exists to find.
    let install = TempInstall::new("verify");
    assert_json_golden(
        "verify",
        &install,
        &["verify", "--source", "override", "--format", "json"],
    );
}

#[test]
fn override_diff_reports_the_shadowed_resource() {
    let install = TempInstall::new("override-diff");
    assert_json_golden(
        "override-diff",
        &install,
        &["override-diff", "--format", "json"],
    );
}

#[test]
fn dump_dot_matches_golden() {
    let install = TempInstall::new("dump-dot");
    install.write_override("TEST.DLG", &graph_dlg());
    assert_text_golden(
        "dump-dot",
        "dot",
        &install,
        &[
            "dump",
            "--resource",
            "TEST.DLG",
            "--format",
            "dot",
            "--strings",
            "both",
        ],
    );
}

#[test]
fn dump_mermaid_matches_golden() {
    let install = TempInstall::new("dump-mermaid");
    install.write_override("TEST.DLG", &graph_dlg());
    assert_text_golden(
        "dump-mermaid",
        "mmd",
        &install,
        &[
            "dump",
            "--resource",
            "TEST.DLG",
            "--format",
            "mermaid",
            "--strings",
            "both",
        ],
    );
}

#[test]
fn save_list_json_matches_golden() {
    let install = TempInstall::new("save-list-json");
    install.create_save();
    assert_json_golden("save-list", &install, &["save-list", "--format", "json"]);
}

#[test]
fn tlk_json_matches_golden() {
    let install = TempInstall::new("tlk");
    assert_json_golden("tlk", &install, &["tlk", "--strref", "1"]);
}

#[test]
fn list_text_matches_golden() {
    let install = TempInstall::new("list-text");
    assert_text_golden("list", "txt", &install, &["list"]);
}

#[test]
fn override_diff_text_matches_golden() {
    let install = TempInstall::new("override-diff-text");
    assert_text_golden("override-diff", "txt", &install, &["override-diff"]);
}

#[test]
fn verify_text_matches_golden() {
    let install = TempInstall::new("verify-text");
    assert_text_golden(
        "verify",
        "txt",
        &install,
        &["verify", "--source", "override"],
    );
}

#[test]
fn save_list_text_matches_golden() {
    let install = TempInstall::new("save-list-text");
    install.create_save();
    assert_text_golden("save-list", "txt", &install, &["save-list"]);
}

fn command_output(name: &str, install: &TempInstall, args: &[&str]) -> std::process::Output {
    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .args(args)
        .arg("--game")
        .arg(install.root())
        .output()
        .expect("iecli should run");

    assert!(
        output.status.success(),
        "{name} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn assert_json_golden(name: &str, install: &TempInstall, args: &[&str]) {
    let output = command_output(name, install, args);

    let value: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{name} should emit JSON: {error}"));
    let value = redact(value, install.root());

    let actual = format!(
        "{}\n",
        serde_json::to_string_pretty(&value).expect("output should serialize")
    );
    let path = golden_path(name);

    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        std::fs::create_dir_all(path.parent().expect("golden path should have a parent"))
            .expect("golden directory should be creatable");
        std::fs::write(&path, &actual).expect("golden should be writable");
        return;
    }

    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| {
            panic!(
                "no golden at {}; regenerate with UPDATE_GOLDENS=1",
                path.display()
            )
        })
        .replace("\r\n", "\n");

    assert_eq!(
        actual,
        expected,
        "{name} output no longer matches {}. The skills read this by field name. \
         If the change is intended, regenerate with UPDATE_GOLDENS=1 and review the diff.",
        path.display()
    );
}

fn assert_text_golden(name: &str, extension: &str, install: &TempInstall, args: &[&str]) {
    let output = command_output(name, install, args);
    let actual = String::from_utf8(output.stdout)
        .unwrap_or_else(|error| panic!("{name} should emit UTF-8: {error}"))
        .replace("\r\n", "\n");
    let actual = redact_text(&actual, install.root());
    let path = text_golden_path(name, extension);

    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        std::fs::create_dir_all(path.parent().expect("golden path should have a parent"))
            .expect("golden directory should be creatable");
        std::fs::write(&path, &actual).expect("golden should be writable");
        return;
    }

    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| {
            panic!(
                "no golden at {}; regenerate with UPDATE_GOLDENS=1",
                path.display()
            )
        })
        .replace("\r\n", "\n");

    assert_eq!(
        actual,
        expected,
        "{name} output no longer matches {}. If the change is intended, regenerate with \
         UPDATE_GOLDENS=1 and review the diff.",
        path.display()
    );
}

/// Redacts path-bearing lines without rewriting graph escapes such as `\n` and
/// `\"`. A global backslash replacement makes a Windows path portable but also
/// silently changes the DOT document being pinned.
fn redact_text(text: &str, root: &Path) -> String {
    let native_root = root.to_string_lossy();
    let slash_root = native_root.replace('\\', "/");
    text.split_inclusive('\n')
        .map(|line| {
            if line.contains(native_root.as_ref()) || line.contains(&slash_root) {
                line.replace(native_root.as_ref(), "<install>")
                    .replace(&slash_root, "<install>")
                    .replace('\\', "/")
            } else {
                line.to_string()
            }
        })
        .collect()
}

/// Replaces the temp install root with `<install>` and normalizes separators.
///
/// Without this the goldens would pin a path containing a per-run temp directory
/// and, on Windows, backslashes -- neither of which says anything about the
/// output being correct.
fn redact(value: Value, root: &Path) -> Value {
    let root = root.to_string_lossy().replace('\\', "/");

    match value {
        Value::String(text) => {
            let text = text.replace('\\', "/");
            Value::String(text.replace(&root, "<install>"))
        }
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(|item| redact(item, Path::new(&root)))
                .collect(),
        ),
        Value::Object(fields) => Value::Object(
            fields
                .into_iter()
                .map(|(key, nested)| (key, redact(nested, Path::new(&root))))
                .collect(),
        ),
        scalar => scalar,
    }
}

fn golden_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens/cli")
        .join(format!("{name}.json"))
}

fn text_golden_path(name: &str, extension: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens/cli")
        .join(format!("{name}.{extension}"))
}

/// A minimal but complete game root: a KEY naming one BIF, that BIF, an override
/// that shadows one of its resources and adds another, an area with a dead Travel
/// link, and a TLK.
struct TempInstall {
    root: PathBuf,
}

impl TempInstall {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!("iecli-cli-goldens-{label}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("data")).expect("data dir should be creatable");
        std::fs::create_dir_all(root.join("override")).expect("override dir should be creatable");
        std::fs::create_dir_all(root.join("lang/en_US")).expect("lang dir should be creatable");

        std::fs::write(root.join("chitin.key"), build_key()).expect("KEY should be writable");
        std::fs::write(root.join("data/base.bif"), build_biff(&stock_itm()))
            .expect("BIF should be writable");
        std::fs::write(root.join("override/SHADOW.ITM"), overridden_itm())
            .expect("override should be writable");
        std::fs::write(root.join("override/AR0202.ARE"), area_with_dead_link())
            .expect("area should be writable");
        std::fs::write(
            root.join("lang/en_US/dialog.tlk"),
            build_tlk(&["<string 0>", "<string 1>", "<string 2>"]),
        )
        .expect("TLK should be writable");

        Self { root }
    }

    fn write_override(&self, resource_name: &str, bytes: &[u8]) {
        std::fs::write(self.root.join("override").join(resource_name), bytes)
            .expect("override resource should be writable");
    }

    fn create_save(&self) {
        let save = self.root.join("save/000000007-Chapter 1 Start");
        std::fs::create_dir_all(&save).expect("save folder should be creatable");
        std::fs::write(save.join("BALDUR.gam"), []).expect("GAM marker should be writable");
        std::fs::write(save.join("BALDUR.SAV"), []).expect("SAV marker should be writable");
        std::fs::write(save.join("PORTRT0L.BMP"), []).expect("portrait marker should be writable");
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for TempInstall {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn build_key() -> Vec<u8> {
    let path = b"data\\base.bif\0";
    let resource_offset = 36u32;
    let string_offset = resource_offset + 14;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"KEY V1  ");
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&24u32.to_le_bytes());
    bytes.extend_from_slice(&resource_offset.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&string_offset.to_le_bytes());
    bytes.extend_from_slice(&(path.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    let mut resref = [0u8; 8];
    resref[..6].copy_from_slice(b"SHADOW");
    bytes.extend_from_slice(&resref);
    bytes.extend_from_slice(&ITM_TYPE_CODE.to_le_bytes());
    bytes.extend_from_slice(&RESOURCE_LOCATOR.to_le_bytes());
    bytes.extend_from_slice(path);
    bytes
}

fn build_biff(payload: &[u8]) -> Vec<u8> {
    let data_offset = 36u32;
    let mut archive = Vec::new();
    archive.extend_from_slice(b"BIFFV1  ");
    archive.extend_from_slice(&1u32.to_le_bytes());
    archive.extend_from_slice(&0u32.to_le_bytes());
    archive.extend_from_slice(&20u32.to_le_bytes());
    archive.extend_from_slice(&RESOURCE_LOCATOR.to_le_bytes());
    archive.extend_from_slice(&data_offset.to_le_bytes());
    archive.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    archive.extend_from_slice(&ITM_TYPE_CODE.to_le_bytes());
    archive.extend_from_slice(&0u16.to_le_bytes());
    archive.extend_from_slice(payload);
    archive
}

fn build_tlk<T: AsRef<str>>(entries: &[T]) -> Vec<u8> {
    let strings_offset = 18u32 + entries.len() as u32 * 26;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"TLK V1  ");
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&strings_offset.to_le_bytes());
    let mut text = Vec::new();
    for entry in entries {
        let entry = entry.as_ref();
        bytes.extend_from_slice(&[0u8; 18]);
        bytes.extend_from_slice(&(text.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(entry.len() as u32).to_le_bytes());
        text.extend_from_slice(entry.as_bytes());
    }
    bytes.extend_from_slice(&text);
    bytes
}

const DLG_HEADER_SIZE: usize = 0x34;
const DLG_STATE_SIZE: usize = 16;
const DLG_TRANSITION_SIZE: usize = 32;
const DLG_SCRIPT_ENTRY_SIZE: usize = 8;

/// Two states and three transitions, including every optional script table and
/// both terminating and external edges. This pins the meaningful graph labels,
/// not just the renderer prologue.
fn graph_dlg() -> Vec<u8> {
    let state_trigger = b"CheckStatGT(Myself,12,STR)";
    let transition_trigger = b"Global(\"X\",\"GLOBAL\",0)";
    let action = b"SetGlobal(\"X\",\"GLOBAL\",1)";

    let states_offset = DLG_HEADER_SIZE as u32;
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
    bytes[state0..state0 + 4].copy_from_slice(&0u32.to_le_bytes());
    bytes[state0 + 8..state0 + 12].copy_from_slice(&2u32.to_le_bytes());

    let state1 = state0 + DLG_STATE_SIZE;
    bytes[state1..state1 + 4].copy_from_slice(&1u32.to_le_bytes());
    bytes[state1 + 4..state1 + 8].copy_from_slice(&2u32.to_le_bytes());
    bytes[state1 + 8..state1 + 12].copy_from_slice(&1u32.to_le_bytes());
    bytes[state1 + 12..state1 + 16].copy_from_slice(&u32::MAX.to_le_bytes());

    let transition0 = transitions_offset as usize;
    bytes[transition0..transition0 + 4].copy_from_slice(&0b0000_0111u32.to_le_bytes());
    bytes[transition0 + 4..transition0 + 8].copy_from_slice(&2u32.to_le_bytes());
    bytes[transition0 + 12..transition0 + 16].copy_from_slice(&0u32.to_le_bytes());
    bytes[transition0 + 16..transition0 + 20].copy_from_slice(&0u32.to_le_bytes());
    bytes[transition0 + 20..transition0 + 28].copy_from_slice(b"IMOEN\0\0\0");
    bytes[transition0 + 28..transition0 + 32].copy_from_slice(&1u32.to_le_bytes());

    let transition1 = transition0 + DLG_TRANSITION_SIZE;
    bytes[transition1..transition1 + 4].copy_from_slice(&0b0000_1000u32.to_le_bytes());

    let transition2 = transition1 + DLG_TRANSITION_SIZE;
    bytes[transition2..transition2 + 4].copy_from_slice(&0b0000_0001u32.to_le_bytes());
    bytes[transition2 + 4..transition2 + 8].copy_from_slice(&1u32.to_le_bytes());
    bytes[transition2 + 20..transition2 + 28].copy_from_slice(b"JAHEIRA\0");

    for (entry_offset, string_offset, string) in [
        (
            state_triggers_offset,
            state_trigger_at,
            state_trigger.as_slice(),
        ),
        (
            transition_triggers_offset,
            transition_trigger_at,
            transition_trigger.as_slice(),
        ),
        (actions_offset, action_at, action.as_slice()),
    ] {
        let entry = entry_offset as usize;
        bytes[entry..entry + 4].copy_from_slice(&string_offset.to_le_bytes());
        bytes[entry + 4..entry + 8].copy_from_slice(&(string.len() as u32).to_le_bytes());
        let at = string_offset as usize;
        bytes[at..at + string.len()].copy_from_slice(string);
    }

    bytes
}

const ITM_HEADER_SIZE: usize = 0x72;

fn itm(price: u32) -> Vec<u8> {
    let mut bytes = vec![0u8; ITM_HEADER_SIZE];
    bytes[0..4].copy_from_slice(b"ITM ");
    bytes[4..8].copy_from_slice(b"V1  ");
    bytes[0x08..0x0C].copy_from_slice(&1u32.to_le_bytes());
    bytes[0x0C..0x10].copy_from_slice(&2u32.to_le_bytes());
    bytes[0x34..0x38].copy_from_slice(&price.to_le_bytes());
    bytes[0x64..0x68].copy_from_slice(&(ITM_HEADER_SIZE as u32).to_le_bytes());
    bytes
}

/// The BIF copy and the override copy differ in price, so `override-diff` has a
/// real difference to report rather than two identical files.
fn stock_itm() -> Vec<u8> {
    itm(100)
}

fn overridden_itm() -> Vec<u8> {
    itm(250)
}

const ARE_HEADER_SIZE: usize = 0x11C;
const ARE_REGION_SIZE: usize = 0xC4;

fn area_with_dead_link() -> Vec<u8> {
    let region_offset = ARE_HEADER_SIZE;
    let mut bytes = vec![0u8; region_offset + ARE_REGION_SIZE];

    bytes[0..4].copy_from_slice(b"AREA");
    bytes[4..8].copy_from_slice(b"V1.0");
    bytes[0x08..0x10].copy_from_slice(b"AR0202\0\0");
    bytes[0x54..0x58].copy_from_slice(&(region_offset as u32).to_le_bytes());
    bytes[0x5A..0x5C].copy_from_slice(&1u16.to_le_bytes());
    bytes[0x5C..0x60].copy_from_slice(&(region_offset as u32).to_le_bytes());
    bytes[0x94..0x9C].copy_from_slice(b"AR0202\0\0");

    let region = region_offset;
    bytes[region..region + 6].copy_from_slice(b"Door01");
    bytes[region + 0x20..region + 0x22].copy_from_slice(&2u16.to_le_bytes());
    bytes[region + 0x38..region + 0x40].copy_from_slice(b"AR9999\0\0");
    bytes[region + 0x40..region + 0x46].copy_from_slice(b"Exit01");

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redaction_replaces_the_install_root_and_normalizes_separators() {
        let root = Path::new("/tmp/iecli-cli-goldens-list");
        let value = json!({
            "source_path": "/tmp/iecli-cli-goldens-list/override/SHADOW.ITM",
            "resource_name": "SHADOW.ITM",
            "count": 1
        });

        assert_eq!(
            redact(value, root),
            json!({
                "source_path": "<install>/override/SHADOW.ITM",
                "resource_name": "SHADOW.ITM",
                "count": 1
            })
        );
    }

    #[test]
    fn redaction_keeps_the_path_tail_that_identifies_the_source() {
        // An override hit and a BIF hit are told apart by what follows the root,
        // so redaction must not swallow it.
        let root = Path::new("/tmp/install");
        let redacted = redact(json!("/tmp/install/data/base.bif"), root);

        assert_eq!(redacted, json!("<install>/data/base.bif"));
    }

    #[test]
    fn text_redaction_preserves_dot_backslash_escapes() {
        let root = Path::new("/tmp/install");
        let text = "label=\"line one\\nline two\\\"quoted\\\"\"\n/tmp/install/save/slot\n";

        assert_eq!(
            redact_text(text, root),
            "label=\"line one\\nline two\\\"quoted\\\"\"\n<install>/save/slot\n"
        );
    }
}
