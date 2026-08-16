//! Shape goldens for decoded JSON, checked against real installs.
//!
//! The exported JSON is the product's interface: skills, guides, and any script
//! downstream read it by field name. Nothing pinned that shape until now, so a
//! renamed or newly nested field passed CI silently and broke consumers.
//!
//! Pinning *values* against a real install does not work. A modded install
//! carries the mod's bytes, not the game's; `tlk-append` shifts strrefs; store
//! variants ship different patch levels; and `lang/<locale>` changes every
//! resolved string. Those all move values without the parser changing at all.
//!
//! Pinning the *shape* does work, and was measured to: unioned over 60 CREs per
//! install and normalized as below, a 16-mod BGEE install, a clean BG2EE, IWDEE,
//! and PSTEE agree on all but a handful of paths, and every residual difference
//! is a resource the sample happened not to include.
//!
//! So the assertion here is deliberately one-directional: what an install
//! produces must be a **subset** of the golden — every observed path known to it,
//! and every observed type among the types it records for that path. A smaller
//! sample or a thinner mod set can only leave paths out, never invent one, so
//! neither can fail this, while a renamed, added, re-nested, or retyped field
//! fails immediately. Deletions are caught by the synthetic value goldens in
//! `goldens.rs`, which pin exact output for fixtures that need no install.
//!
//! The honest limit: a *richer* install than the one that generated the goldens
//! can legitimately surface a path none of them reached — a mod that populates a
//! section the reference installs leave empty, say. That reads as a failure and
//! is really a gap in the golden, which is why generation samples far harder than
//! assertion does, and why the failure message says to regenerate. It is a
//! one-line union, not a debugging session, but it is not nothing.
//!
//! Regenerate with `UPDATE_SHAPE_GOLDENS=1`, which unions into the existing file
//! rather than replacing it, so running with one install cannot drop the paths
//! another install contributed.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

/// Serializes golden updates.
///
/// BG2EE and BGEE are both `standard`, so they regenerate the same files, and
/// cargo runs these tests as parallel threads of one process. Without this the
/// union is a racing read-modify-write and one install's paths are silently lost
/// -- which produces a golden that looks fine and is quietly incomplete.
static GOLDEN_WRITE: Mutex<()> = Mutex::new(());

/// Resources decoded per type when asserting. Under the subset rule a sample can
/// only omit paths, never invent one, so this trades coverage for runtime without
/// risking a false failure on the install it was generated from.
const SAMPLE_PER_TYPE: usize = 150;

/// Resources decoded per type when regenerating.
///
/// Generation is deliberate and rare, so it looks harder than an assertion run
/// does. The reason is not thoroughness for its own sake: a path that a real
/// install can produce but the golden has never seen fails on the *next*
/// person's install, whose mods or patch level reach code this sample did not.
/// Widening here is what keeps that from being someone else's surprise.
///
/// Not "every resource", though. Each dump is a process that rediscovers the
/// install and reparses a half-megabyte KEY, which measures at roughly half a
/// second over a Windows-mounted install; a whole-install regeneration across
/// four installs runs into hours. This is the point where the curve flattens
/// against a cost someone has to actually sit through.
const GENERATE_SAMPLE_PER_TYPE: usize = 400;

const DUMPABLE_TYPES: &[&str] = &["ITM", "SPL", "CRE", "STO", "DLG", "ARE", "BCS"];

#[test]
fn shape_matches_golden_for_bg2ee_when_ie_game_path_is_set() {
    check_install("IE_GAME_PATH", "standard");
}

#[test]
fn shape_matches_golden_for_bgee_when_ie_bgee_path_is_set() {
    check_install("IE_BGEE_PATH", "standard");
}

#[test]
fn shape_matches_golden_for_iwdee_when_ie_iwdee_path_is_set() {
    check_install("IE_IWDEE_PATH", "iwd");
}

#[test]
fn shape_matches_golden_for_pstee_when_ie_pstee_path_is_set() {
    check_install("IE_PSTEE_PATH", "pst");
}

fn check_install(env_var: &str, variant: &str) {
    let Some(game) = std::env::var_os(env_var) else {
        return;
    };

    let updating = std::env::var_os("UPDATE_SHAPE_GOLDENS").is_some();

    let cap = if updating {
        GENERATE_SAMPLE_PER_TYPE
    } else {
        SAMPLE_PER_TYPE
    };

    for resource_type in DUMPABLE_TYPES {
        let observed = observed_shape(&game, resource_type, cap);
        if observed.is_empty() {
            continue;
        }

        let path = golden_path(variant, resource_type);
        if updating {
            let _guard = GOLDEN_WRITE
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            write_golden(&path, &union(read_golden(&path), observed));
            continue;
        }

        let golden = read_golden(&path);
        assert!(
            !golden.is_empty(),
            "no shape golden at {}; regenerate with UPDATE_SHAPE_GOLDENS=1",
            path.display()
        );

        let complaints = compare(&observed, &golden);
        assert!(
            complaints.is_empty(),
            "{env_var} produced {} {resource_type} JSON shape difference(s) against {}:\n  {}\n\
             A renamed, added, or retyped field is a breaking change for anything reading this \
             output. If the change is intended, regenerate with UPDATE_SHAPE_GOLDENS=1.",
            complaints.len(),
            path.display(),
            complaints.join("\n  ")
        );
    }
}

/// Compares observed shape against the golden, per path.
///
/// Types are compared as **sets**, not as the rendered string. A golden written
/// from a wider sample legitimately records `null|str` where a narrower run sees
/// only `str`, and those are the same shape: the field is there and it is a
/// string when it decodes. Comparing the rendered text instead called that a
/// breaking change and failed on all four installs over five such paths.
///
/// A path the golden has never seen is still a failure, and so is a type the
/// golden has no record of -- `int` where it only ever saw `str` is a real
/// change to how a consumer must read the field.
fn compare(
    observed: &BTreeMap<String, BTreeSet<String>>,
    golden: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<String> {
    let mut complaints = Vec::new();

    for (path, types) in observed {
        let Some(known) = golden.get(path) else {
            complaints.push(format!(
                "{path}: {} (path not in golden)",
                render_types(types)
            ));
            continue;
        };

        let novel = types.difference(known).cloned().collect::<BTreeSet<_>>();
        if !novel.is_empty() {
            complaints.push(format!(
                "{path}: {} (golden records {})",
                render_types(&novel),
                render_types(known)
            ));
        }
    }

    complaints
}

fn union(
    mut into: BTreeMap<String, BTreeSet<String>>,
    from: BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<String, BTreeSet<String>> {
    for (path, types) in from {
        into.entry(path).or_default().extend(types);
    }
    into
}

/// Decodes a deterministic spread of resources and unions their shapes.
fn observed_shape(
    game: &OsString,
    resource_type: &str,
    cap: usize,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut types = BTreeMap::<String, BTreeSet<String>>::new();

    for resource in sample(list_resources(game, resource_type), cap) {
        let Some(value) = dump_json(game, &resource) else {
            // Stock data ships resources that do not decode -- BGEE indexes
            // CDDETECT as SPL over ITM bytes, IWDEE ships a corrupt #BONECIR.
            // Those are pinned as their own regression tests; here they simply
            // contribute no shape.
            continue;
        };
        collect(&value, "$", &mut types);
    }

    types
}

/// Spreads the sample across the whole listing instead of taking a prefix, so a
/// type is not represented entirely by resources whose names begin with `A`.
///
/// The assertion sample must be a **subset** of the generation sample, which is
/// why this narrows in two steps rather than striding straight to `cap`. Striding
/// once lands on different resources for different caps -- 2,811 CREs stride by 8
/// at 400 and by 19 at 150, and 19 is not a multiple of 8 -- so an assertion run
/// would decode resources the generation run never saw and report their paths as
/// unknown. That is a false alarm about sampling dressed up as a shape change,
/// and it fired on all four installs before this was fixed.
fn sample(resources: Vec<String>, cap: usize) -> Vec<String> {
    let widest = stride_to(resources, GENERATE_SAMPLE_PER_TYPE);
    stride_to(widest, cap)
}

fn stride_to(mut resources: Vec<String>, cap: usize) -> Vec<String> {
    resources.sort();
    if resources.len() <= cap {
        return resources;
    }

    let stride = resources.len().div_ceil(cap);
    resources.into_iter().step_by(stride).collect()
}

fn list_resources(game: &OsString, resource_type: &str) -> Vec<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("list")
        .arg("--game")
        .arg(game)
        .args(["--type", resource_type, "--format", "json"])
        .output()
        .expect("iecli should run");

    if !output.status.success() {
        return Vec::new();
    }

    let listed: Value = serde_json::from_slice(&output.stdout).expect("list should emit JSON");
    listed
        .as_array()
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry["resource_name"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn dump_json(game: &OsString, resource: &str) -> Option<Value> {
    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("dump")
        .arg("--game")
        .arg(game)
        .args(["--resource", resource, "--format", "json"])
        .output()
        .expect("iecli should run");

    if !output.status.success() {
        return None;
    }

    serde_json::from_slice(&output.stdout).ok()
}

/// Walks a decoded document into `path -> observed JSON types`.
///
/// Two normalizations matter, and both exist because the alternative is a test
/// that fails on data rather than on code:
///
/// - Array indices collapse to `[]`, and an **empty** array contributes nothing.
///   A creature carrying no items says nothing about item shape; without this a
///   golden would encode which sampled creature happened to be holding something.
/// - `null` is recorded as a type rather than as absence, so a field that is
///   nullable reads as `str|null` instead of splitting into two rival shapes
///   depending on whether the sample hit a decodable value.
fn collect(value: &Value, path: &str, types: &mut BTreeMap<String, BTreeSet<String>>) {
    match value {
        Value::Object(fields) => {
            for (field, nested) in fields {
                collect(nested, &format!("{path}.{field}"), types);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect(item, &format!("{path}[]"), types);
            }
        }
        scalar => {
            types
                .entry(path.to_string())
                .or_default()
                .insert(scalar_type(scalar).to_string());
        }
    }
}

fn scalar_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(number) if number.is_f64() => "float",
        Value::Number(_) => "int",
        Value::String(_) => "str",
        // Containers are recursed before reaching here; an empty one contributes
        // no path at all, which is the point of the normalization.
        Value::Array(_) | Value::Object(_) => "container",
    }
}

fn render_types(types: &BTreeSet<String>) -> String {
    types.iter().cloned().collect::<Vec<_>>().join("|")
}

fn golden_path(variant: &str, resource_type: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens/shape")
        .join(format!("{variant}.{}.shape", resource_type.to_lowercase()))
}

/// Reads `<path>: <type>|<type>` lines back into the map the comparison uses.
///
/// A line the parser cannot split is skipped rather than guessed at: a golden
/// half-read would silently weaken every assertion made against it.
fn read_golden(path: &Path) -> BTreeMap<String, BTreeSet<String>> {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return BTreeMap::new();
    };

    let mut shape = BTreeMap::<String, BTreeSet<String>>::new();

    for line in contents.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((path, types)) = line.rsplit_once(": ") else {
            continue;
        };
        // Union rather than replace. Two lines for one path are what an older
        // writer produced, and taking the last one silently dropped `null` from
        // five paths -- which then failed as a shape change against goldens that
        // recorded the type perfectly well one line earlier.
        shape
            .entry(path.to_string())
            .or_default()
            .extend(types.split('|').map(str::to_string));
    }

    shape
}

fn write_golden(path: &Path, shape: &BTreeMap<String, BTreeSet<String>>) {
    std::fs::create_dir_all(path.parent().expect("golden path should have a parent"))
        .expect("golden directory should be creatable");
    let body = shape
        .iter()
        .map(|(path, types)| format!("{path}: {}", render_types(types)))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(path, format!("{body}\n")).expect("golden should be writable");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn shape(value: Value) -> Vec<String> {
        let mut types = BTreeMap::new();
        collect(&value, "$", &mut types);
        rendered(&types)
    }

    fn rendered(types: &BTreeMap<String, BTreeSet<String>>) -> Vec<String> {
        types
            .iter()
            .map(|(path, types)| format!("{path}: {}", render_types(types)))
            .collect()
    }

    #[test]
    fn array_indices_collapse_so_length_does_not_change_the_shape() {
        let one = shape(json!({"items": [{"resource": "SW1H01"}]}));
        let many = shape(json!({
            "items": [{"resource": "SW1H01"}, {"resource": "SW1H02"}]
        }));

        assert_eq!(one, many);
        assert_eq!(one, vec!["$.items[].resource: str".to_string()]);
    }

    #[test]
    fn empty_arrays_contribute_no_path() {
        // A creature carrying nothing must not read as a different shape from one
        // that does, or the golden encodes the sample rather than the format.
        assert_eq!(shape(json!({"items": []})), Vec::<String>::new());
    }

    #[test]
    fn nullable_fields_read_as_one_path_rather_than_two_shapes() {
        let mut types = BTreeMap::new();
        collect(&json!({"kit": {"decoded": null}}), "$", &mut types);
        collect(&json!({"kit": {"decoded": "BERSERKER"}}), "$", &mut types);

        assert_eq!(
            rendered(&types),
            vec!["$.kit.decoded: null|str".to_string()]
        );
    }

    /// The false alarm that failed all four installs: a golden written from a
    /// wide sample records `null|str`, a narrower run sees only `str`, and those
    /// are the same shape. Comparing rendered text instead of type sets called
    /// that a breaking change.
    #[test]
    fn a_narrower_type_set_than_the_golden_is_not_a_difference() {
        let golden = golden_from(&[("$.kit.decoded", "null|str")]);
        let observed = golden_from(&[("$.kit.decoded", "str")]);

        assert!(compare(&observed, &golden).is_empty());
    }

    #[test]
    fn a_type_the_golden_has_never_recorded_is_a_difference() {
        let golden = golden_from(&[("$.header.price", "int")]);
        let observed = golden_from(&[("$.header.price", "str")]);

        let complaints = compare(&observed, &golden);
        assert_eq!(complaints.len(), 1);
        assert!(
            complaints[0].contains("golden records int"),
            "unhelpful complaint: {}",
            complaints[0]
        );
    }

    #[test]
    fn a_path_the_golden_has_never_seen_is_a_difference() {
        let golden = golden_from(&[("$.header.price", "int")]);
        let observed = golden_from(&[("$.header.renamed_price", "int")]);

        let complaints = compare(&observed, &golden);
        assert_eq!(complaints.len(), 1);
        assert!(
            complaints[0].contains("path not in golden"),
            "unhelpful complaint: {}",
            complaints[0]
        );
    }

    #[test]
    fn reading_a_golden_unions_repeated_paths_rather_than_keeping_the_last() {
        let path = std::env::temp_dir().join("iecli-shape-duplicate.shape");
        std::fs::write(
            &path,
            "$.abilities[].location.decoded: null|str\n$.abilities[].location.decoded: str\n",
        )
        .expect("fixture should be writable");

        let shape = read_golden(&path);
        let _ = std::fs::remove_file(&path);

        assert_eq!(
            shape.get("$.abilities[].location.decoded"),
            Some(
                &["null".to_string(), "str".to_string()]
                    .into_iter()
                    .collect()
            )
        );
    }

    #[test]
    fn goldens_round_trip_through_the_file_format() {
        let shape = golden_from(&[
            ("$.abilities[].damage_dice", "null|str"),
            ("$.header.price", "int"),
        ]);
        let path = std::env::temp_dir().join("iecli-shape-roundtrip.shape");

        write_golden(&path, &shape);
        let read_back = read_golden(&path);
        let _ = std::fs::remove_file(&path);

        assert_eq!(read_back, shape);
    }

    fn golden_from(entries: &[(&str, &str)]) -> BTreeMap<String, BTreeSet<String>> {
        entries
            .iter()
            .map(|(path, types)| {
                (
                    path.to_string(),
                    types.split('|').map(str::to_string).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn numbers_separate_integers_from_floats() {
        assert_eq!(
            shape(json!({"count": 3, "ratio": 0.5})),
            vec!["$.count: int".to_string(), "$.ratio: float".to_string()]
        );
    }

    /// The invariant the subset assertion rests on. If an assertion run can reach
    /// a resource the generation run did not, it reports that resource's paths as
    /// unknown and the failure is about sampling, not about shape.
    #[test]
    fn asserted_sample_is_a_subset_of_the_generated_sample() {
        let resources = (0..2811)
            .map(|index| format!("{index:04}.CRE"))
            .collect::<Vec<_>>();

        let generated = sample(resources.clone(), GENERATE_SAMPLE_PER_TYPE)
            .into_iter()
            .collect::<BTreeSet<_>>();
        let asserted = sample(resources, SAMPLE_PER_TYPE);

        assert!(!asserted.is_empty());
        let outside = asserted
            .iter()
            .filter(|resource| !generated.contains(*resource))
            .collect::<Vec<_>>();
        assert!(
            outside.is_empty(),
            "asserting would decode {} resource(s) generation never saw, e.g. {:?}",
            outside.len(),
            outside.first()
        );
    }

    #[test]
    fn sampling_spreads_across_the_listing_instead_of_taking_a_prefix() {
        let resources = (0..SAMPLE_PER_TYPE * 3)
            .map(|index| format!("{index:04}.CRE"))
            .collect::<Vec<_>>();

        let sampled = sample(resources, SAMPLE_PER_TYPE);

        assert!(sampled.len() <= SAMPLE_PER_TYPE);
        assert_eq!(
            sampled.first().expect("sample should not be empty"),
            "0000.CRE"
        );
        // A prefix sample would stop in the 0100s; a spread reaches the tail.
        assert!(
            sampled
                .last()
                .expect("sample should not be empty")
                .starts_with("04"),
            "sample should reach the end of the listing, got {:?}",
            sampled.last()
        );
    }
}
