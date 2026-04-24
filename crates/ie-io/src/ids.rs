use crate::{IoError, ResourceLocator, ResourceReader};
use ie_core::{IdsResolver, ResourceName};
use std::collections::HashMap;
use std::sync::Mutex;

type IdsEntries = HashMap<i32, String>;

#[derive(Debug)]
pub struct FileBackedIdsResolver {
    locator: ResourceLocator,
    cache: Mutex<HashMap<String, Option<IdsEntries>>>,
}

impl FileBackedIdsResolver {
    pub fn new(locator: ResourceLocator) -> Self {
        Self {
            locator,
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn lookup(&self, ids_file: &str, value: i32) -> Option<String> {
        let key = normalize_ids_name(ids_file);
        let mut cache = self.cache.lock().ok()?;

        if !cache.contains_key(&key) {
            let loaded = self.load_ids_file(&key).ok();
            cache.insert(key.clone(), loaded);
        }

        cache
            .get(&key)
            .and_then(|entries| entries.as_ref())
            .and_then(|entries| entries.get(&value))
            .cloned()
    }

    fn load_ids_file(&self, ids_file: &str) -> Result<IdsEntries, IoError> {
        let resource_name = ResourceName::parse(format!("{ids_file}.IDS"))
            .map_err(|err| IoError::InvalidIds(err.to_string()))?;
        let reader = ResourceReader;
        let bytes = reader.read(&self.locator, &resource_name)?;
        parse_ids(&bytes.bytes)
    }
}

impl IdsResolver for FileBackedIdsResolver {
    fn resolve_trigger(&self, opcode: i32) -> Option<String> {
        self.lookup("TRIGGER", opcode)
    }

    fn resolve_action(&self, opcode: i32) -> Option<String> {
        self.lookup("ACTION", opcode)
    }

    fn resolve_ids(&self, file: &str, value: i32) -> Option<String> {
        self.lookup(file, value)
    }
}

pub fn parse_ids(bytes: &[u8]) -> Result<HashMap<i32, String>, IoError> {
    let text = String::from_utf8_lossy(bytes);
    parse_ids_text(&text)
}

pub fn parse_ids_text(text: &str) -> Result<HashMap<i32, String>, IoError> {
    let mut entries = HashMap::new();

    for (index, raw_line) in text.lines().enumerate() {
        let line_number = index + 1;
        let line = strip_comment(raw_line).trim();

        if line.is_empty()
            || line.eq_ignore_ascii_case("IDS")
            || line.eq_ignore_ascii_case("V1.0")
            || line.eq_ignore_ascii_case("V1")
            || matches_ids_version_header(line)
        {
            continue;
        }

        let Some((value_token, name_part)) = split_mapping_line(line) else {
            if parse_ids_value(line).is_ok() {
                continue;
            }

            // Some IDS files contain standalone header lines; tolerate them.
            if !line.contains(char::is_whitespace) {
                continue;
            }

            return Err(IoError::InvalidIds(format!(
                "unrecognized IDS line {line_number}: {line}"
            )));
        };

        let value = parse_ids_value(value_token).map_err(|err| {
            IoError::InvalidIds(format!(
                "invalid IDS value on line {line_number}: {value_token} ({err})"
            ))
        })?;
        let name = strip_signature(name_part);

        if name.is_empty() {
            return Err(IoError::InvalidIds(format!(
                "missing IDS name on line {line_number}"
            )));
        }

        entries.insert(value, name.to_string());
    }

    Ok(entries)
}

fn normalize_ids_name(ids_file: &str) -> String {
    ids_file
        .trim()
        .trim_end_matches(".IDS")
        .trim_end_matches(".ids")
        .to_ascii_uppercase()
}

fn strip_comment(line: &str) -> &str {
    line.split_once("//").map_or(line, |(prefix, _)| prefix)
}

fn split_mapping_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, char::is_whitespace);
    let value = parts.next()?.trim();
    let rest = parts.next()?.trim();

    if value.is_empty() || rest.is_empty() {
        None
    } else {
        Some((value, rest))
    }
}

fn parse_ids_value(token: &str) -> Result<i32, String> {
    if let Some(hex) = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
    {
        i32::from_str_radix(hex, 16).map_err(|err| err.to_string())
    } else if let Some(hex) = token
        .strip_prefix("-0x")
        .or_else(|| token.strip_prefix("-0X"))
    {
        i32::from_str_radix(hex, 16)
            .map(|value| -value)
            .map_err(|err| err.to_string())
    } else {
        token.parse::<i32>().map_err(|err| err.to_string())
    }
}

fn strip_signature(name: &str) -> &str {
    let trimmed = name.trim();

    if let Some((bare, _)) = trimmed.split_once('(') {
        bare.trim()
    } else {
        trimmed.split_whitespace().next().unwrap_or_default().trim()
    }
}

fn matches_ids_version_header(line: &str) -> bool {
    let mut parts = line.split_whitespace();
    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(ids), Some(version), None)
            if ids.eq_ignore_ascii_case("IDS")
                && (version.eq_ignore_ascii_case("V1.0") || version.eq_ignore_ascii_case("V1"))
    )
}

#[cfg(test)]
mod tests {
    use super::parse_ids_text;

    #[test]
    fn parses_ids_with_standard_header_and_signatures() {
        let ids = parse_ids_text(
            r#"
IDS
V1.0
3
0x0001 Global(S:Name*,S:Area*,I:Value*)
-1 LastAttackerOf(O:Object*)
2 PC
"#,
        )
        .expect("IDS should parse");

        assert_eq!(ids.get(&1).map(String::as_str), Some("Global"));
        assert_eq!(ids.get(&-1).map(String::as_str), Some("LastAttackerOf"));
        assert_eq!(ids.get(&2).map(String::as_str), Some("PC"));
    }

    #[test]
    fn parses_ids_with_countless_header_variant() {
        let ids = parse_ids_text(
            r#"
2
1 HUMAN
2 ELF
"#,
        )
        .expect("IDS should parse");

        assert_eq!(ids.get(&1).map(String::as_str), Some("HUMAN"));
        assert_eq!(ids.get(&2).map(String::as_str), Some("ELF"));
    }

    #[test]
    fn tolerates_comments_and_unknown_single_token_headers() {
        let ids = parse_ids_text(
            r#"
MY_HEADER
1 FOO // inline comment
2 BAR
"#,
        )
        .expect("IDS should parse");

        assert_eq!(ids.get(&1).map(String::as_str), Some("FOO"));
        assert_eq!(ids.get(&2).map(String::as_str), Some("BAR"));
    }

    #[test]
    fn parses_ids_with_combined_ids_version_header_line() {
        let ids = parse_ids_text(
            r#"
IDS V1.0
0x400F Global(S:Name*,S:Area*,I:Value*)
0x4040 GlobalTimerExpired(S:Name*,S:Area*)
"#,
        )
        .expect("IDS should parse");

        assert_eq!(ids.get(&0x400F).map(String::as_str), Some("Global"));
        assert_eq!(
            ids.get(&0x4040).map(String::as_str),
            Some("GlobalTimerExpired")
        );
    }
}
