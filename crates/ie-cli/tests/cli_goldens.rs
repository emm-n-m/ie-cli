//! Value goldens for the commands that report *about* an install rather than
//! decode one resource.
//!
//! `list`, `locate`, `verify`, and `override-diff` never pass through
//! `decode_to_json`, so neither the per-format value goldens in `ie-formats` nor
//! the real-install shape goldens cover them -- and they are exactly the outputs
//! the skills drive their answers from. Until now nothing pinned them at all.
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
    assert_golden("list", &install, &["list", "--format", "json"]);
}

#[test]
fn list_filtered_by_type_and_source_matches_golden() {
    let install = TempInstall::new("list-filtered");
    assert_golden(
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
    assert_golden(
        "locate-override",
        &install,
        &["locate", "--resource", "SHADOW.ITM"],
    );
}

#[test]
fn locate_with_explicit_bif_source_reaches_past_the_override() {
    let install = TempInstall::new("locate-bif");
    assert_golden(
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
    assert_golden(
        "verify",
        &install,
        &["verify", "--source", "override", "--format", "json"],
    );
}

#[test]
fn override_diff_reports_the_shadowed_resource() {
    let install = TempInstall::new("override-diff");
    assert_golden(
        "override-diff",
        &install,
        &["override-diff", "--format", "json"],
    );
}

fn assert_golden(name: &str, install: &TempInstall, args: &[&str]) {
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

fn build_tlk(entries: &[&str]) -> Vec<u8> {
    let strings_offset = 18u32 + entries.len() as u32 * 26;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"TLK V1  ");
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&strings_offset.to_le_bytes());
    let mut text = Vec::new();
    for entry in entries {
        bytes.extend_from_slice(&[0u8; 18]);
        bytes.extend_from_slice(&(text.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(entry.len() as u32).to_le_bytes());
        text.extend_from_slice(entry.as_bytes());
    }
    bytes.extend_from_slice(&text);
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
}
