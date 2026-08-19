//! Data-driven factual checks against real installations.
//!
//! Keep the expectation file small: it records individual facts checked against
//! the IESDP format specification or the raw bytes, never complete resource
//! dumps or localized text.

use serde_json::Value;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

#[test]
fn real_resources_match_recorded_expectations_when_game_paths_are_set() {
    let suites: Value = serde_json::from_str(include_str!("expectations/real_resources.json"))
        .expect("real-resource expectations should be valid JSON");

    for suite in suites
        .as_array()
        .expect("expectation root should be an array")
    {
        let Some(game_path) = first_configured_path(suite) else {
            continue;
        };
        let suite_name = suite["name"].as_str().expect("suite should have a name");
        let cases = suite["cases"].as_array().expect("suite should have cases");

        for case in cases {
            check_case(suite_name, &game_path, case);
        }
    }
}

fn first_configured_path(suite: &Value) -> Option<OsString> {
    suite["environment"]
        .as_array()
        .expect("suite should list environment variables")
        .iter()
        .filter_map(Value::as_str)
        .find_map(std::env::var_os)
}

fn check_case(suite_name: &str, game_path: &OsString, case: &Value) {
    let resource = case["resource"]
        .as_str()
        .expect("case should name a resource");
    let provenance = case["provenance"]
        .as_str()
        .expect("case should state its provenance");
    assert!(
        !provenance.trim().is_empty(),
        "{suite_name}/{resource} must state its provenance"
    );

    let output = Command::new(env!("CARGO_BIN_EXE_iecli"))
        .arg("dump")
        .arg("--game")
        .arg(game_path)
        .args(["--resource", resource, "--format", "json"])
        .output()
        .expect("iecli should run");
    assert!(
        output.status.success(),
        "{suite_name}/{resource} should dump successfully: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: Value =
        serde_json::from_slice(&output.stdout).expect("dump should emit a JSON document");

    for assertion in case["assertions"]
        .as_array()
        .expect("case should contain assertions")
    {
        check_assertion(suite_name, resource, provenance, &actual, assertion);
    }
}

fn check_assertion(
    suite_name: &str,
    resource: &str,
    provenance: &str,
    actual: &Value,
    assertion: &Value,
) {
    let pointer = assertion["pointer"]
        .as_str()
        .expect("assertion should contain a JSON pointer");
    let observed = actual.pointer(pointer).unwrap_or_else(|| {
        panic!("{suite_name}/{resource}: missing {pointer} (expectation provenance: {provenance})")
    });

    if let Some(expected) = assertion.get("equals") {
        assert_eq!(
            observed, expected,
            "{suite_name}/{resource}: mismatch at {pointer} (expectation provenance: {provenance})"
        );
        return;
    }

    if let Some(expected) = assertion.get("length").and_then(Value::as_u64) {
        let observed_length = observed
            .as_array()
            .map(Vec::len)
            .or_else(|| observed.as_object().map(serde_json::Map::len))
            .unwrap_or_else(|| {
                panic!("{suite_name}/{resource}: {pointer} is not an array or object")
            });
        assert_eq!(
            observed_length, expected as usize,
            "{suite_name}/{resource}: wrong length at {pointer} (expectation provenance: {provenance})"
        );
        return;
    }

    panic!("{suite_name}/{resource}: assertion at {pointer} has no supported operation");
}

#[test]
fn expectation_manifest_is_well_formed_without_game_installations() {
    let suites: Value = serde_json::from_str(include_str!("expectations/real_resources.json"))
        .expect("real-resource expectations should be valid JSON");

    for suite in suites
        .as_array()
        .expect("expectation root should be an array")
    {
        assert!(suite["name"].is_string());
        assert!(suite["environment"].is_array());
        for case in suite["cases"].as_array().expect("suite should have cases") {
            let resource = case["resource"]
                .as_str()
                .expect("resource should be a string");
            assert!(Path::new(resource).extension().is_some());
            assert!(case["provenance"].is_string());
            let assertions = case["assertions"]
                .as_array()
                .expect("case should have assertions");
            assert!(!assertions.is_empty());
            for assertion in assertions {
                assert!(
                    assertion["pointer"]
                        .as_str()
                        .is_some_and(|p| p.starts_with('/'))
                );
                assert!(assertion.get("equals").is_some() ^ assertion.get("length").is_some());
            }
        }
    }
}
