use ie_core::{ResRef, ResolvedStrRef, ResourceType, StrRef, StrRefResolver};
use serde::Serialize;
use thiserror::Error;

const DLG_HEADER_WITHOUT_FLAGS: usize = 0x30;
const DLG_HEADER_WITH_FLAGS: usize = 0x34;
const DLG_STATE_SIZE: usize = 16;
const DLG_TRANSITION_SIZE: usize = 32;
const DLG_SCRIPT_ENTRY_SIZE: usize = 8;

#[derive(Debug, Clone, Serialize)]
pub struct DialogJson {
    pub resource_type: String,
    pub resource_name: String,
    pub version: String,
    pub header: DialogHeaderJson,
    pub states: Vec<DialogStateJson>,
    pub state_triggers: Vec<DialogScriptJson>,
    pub transition_triggers: Vec<DialogScriptJson>,
    pub actions: Vec<DialogScriptJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DialogHeaderJson {
    pub num_states: u32,
    pub offset_states: u32,
    pub num_transitions: u32,
    pub offset_transitions: u32,
    pub offset_state_triggers: u32,
    pub num_state_triggers: u32,
    pub offset_transition_triggers: u32,
    pub num_transition_triggers: u32,
    pub offset_actions: u32,
    pub num_actions: u32,
    pub flags: Option<RawDecodedFlags>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DialogStateJson {
    pub index: u32,
    pub response_text: ResolvedStrRef,
    pub first_transition_index: u32,
    pub num_transitions: u32,
    pub trigger_index: Option<u32>,
    pub trigger_text: Option<String>,
    pub transitions: Vec<DialogTransitionJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DialogTransitionJson {
    pub index: u32,
    pub flags: RawDecodedFlags,
    pub terminates_dialog: bool,
    pub player_text: Option<ResolvedStrRef>,
    pub journal_text: Option<ResolvedStrRef>,
    pub trigger_index: Option<u32>,
    pub trigger_text: Option<String>,
    pub action_index: Option<u32>,
    pub action_text: Option<String>,
    pub next_dialog: Option<ResRef>,
    pub next_state_index: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DialogScriptJson {
    pub offset: u32,
    pub length: u32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RawDecodedFlags {
    pub raw: u32,
    pub decoded: Vec<String>,
}

#[derive(Debug, Error)]
pub enum DialogParseError {
    #[error("invalid DLG header: {0}")]
    InvalidHeader(String),
    #[error("unexpected end of DLG file: {0}")]
    UnexpectedEof(String),
    #[error("invalid DLG field: {0}")]
    InvalidField(String),
}

impl From<DialogParseError> for crate::FormatError {
    fn from(err: DialogParseError) -> Self {
        crate::FormatError::Parse(err.to_string())
    }
}

pub fn parse_dlg(
    bytes: &[u8],
    resource_name: &str,
    resolver: Option<&dyn StrRefResolver>,
) -> Result<DialogJson, crate::FormatError> {
    if bytes.len() < DLG_HEADER_WITHOUT_FLAGS {
        return Err(DialogParseError::UnexpectedEof(format!(
            "DLG resource must contain at least {} bytes",
            DLG_HEADER_WITHOUT_FLAGS
        ))
        .into());
    }

    if &bytes[0..4] != b"DLG " {
        return Err(DialogParseError::InvalidHeader("missing DLG signature".to_string()).into());
    }

    let version = parse_ascii_string(bytes, 4, 4)?;

    let num_states = parse_u32(bytes, 0x08)?;
    let offset_states = parse_u32(bytes, 0x0C)?;
    let num_transitions = parse_u32(bytes, 0x10)?;
    let offset_transitions = parse_u32(bytes, 0x14)?;
    let offset_state_triggers = parse_u32(bytes, 0x18)?;
    let num_state_triggers = parse_u32(bytes, 0x1C)?;
    let offset_transition_triggers = parse_u32(bytes, 0x20)?;
    let num_transition_triggers = parse_u32(bytes, 0x24)?;
    let offset_actions = parse_u32(bytes, 0x28)?;
    let num_actions = parse_u32(bytes, 0x2C)?;

    let min_table_offset = [
        (num_states, offset_states),
        (num_transitions, offset_transitions),
        (num_state_triggers, offset_state_triggers),
        (num_transition_triggers, offset_transition_triggers),
        (num_actions, offset_actions),
    ]
    .iter()
    .filter(|(count, _)| *count > 0)
    .map(|(_, offset)| *offset)
    .min();

    let flags_present = match min_table_offset {
        Some(offset) => offset as usize >= DLG_HEADER_WITH_FLAGS,
        None => bytes.len() >= DLG_HEADER_WITH_FLAGS,
    };

    let flags = if flags_present && bytes.len() >= DLG_HEADER_WITH_FLAGS {
        let raw = parse_u32(bytes, 0x30)?;
        Some(RawDecodedFlags {
            raw,
            decoded: decode_dialog_flags(raw),
        })
    } else {
        None
    };

    let header = DialogHeaderJson {
        num_states,
        offset_states,
        num_transitions,
        offset_transitions,
        offset_state_triggers,
        num_state_triggers,
        offset_transition_triggers,
        num_transition_triggers,
        offset_actions,
        num_actions,
        flags,
    };

    let state_triggers = parse_script_table(
        bytes,
        offset_state_triggers,
        num_state_triggers,
        "state trigger",
    )?;
    let transition_triggers = parse_script_table(
        bytes,
        offset_transition_triggers,
        num_transition_triggers,
        "transition trigger",
    )?;
    let actions = parse_script_table(bytes, offset_actions, num_actions, "action")?;

    let transitions = parse_transitions(
        bytes,
        offset_transitions,
        num_transitions,
        resolver,
        &transition_triggers,
        &actions,
    )?;

    let states = parse_states(
        bytes,
        offset_states,
        num_states,
        num_transitions,
        resolver,
        &state_triggers,
        &transitions,
    )?;

    Ok(DialogJson {
        resource_type: ResourceType::Dlg.as_str().to_string(),
        resource_name: resource_name.to_string(),
        version,
        header,
        states,
        state_triggers,
        transition_triggers,
        actions,
    })
}

fn parse_states(
    bytes: &[u8],
    offset: u32,
    count: u32,
    transition_count: u32,
    resolver: Option<&dyn StrRefResolver>,
    state_triggers: &[DialogScriptJson],
    transitions: &[DialogTransitionJson],
) -> Result<Vec<DialogStateJson>, DialogParseError> {
    parse_table(
        bytes,
        offset,
        count,
        DLG_STATE_SIZE,
        |bytes, position, index| {
            let response_text = parse_strref(bytes, position, resolver)?;
            let first_transition_index = parse_u32(bytes, position + 0x04)?;
            let num_transitions = parse_u32(bytes, position + 0x08)?;
            let trigger_index_raw = parse_u32(bytes, position + 0x0C)?;

            let trigger_index = optional_index(trigger_index_raw);
            let trigger_text = trigger_index
                .and_then(|idx| state_triggers.get(idx as usize))
                .map(|script| script.text.clone());

            let state_transitions = slice_transitions(
                transitions,
                first_transition_index,
                num_transitions,
                transition_count,
                index,
            )?;

            Ok(DialogStateJson {
                index,
                response_text,
                first_transition_index,
                num_transitions,
                trigger_index,
                trigger_text,
                transitions: state_transitions,
            })
        },
    )
}

fn slice_transitions(
    transitions: &[DialogTransitionJson],
    first: u32,
    count: u32,
    total: u32,
    state_index: u32,
) -> Result<Vec<DialogTransitionJson>, DialogParseError> {
    if count == 0 {
        return Ok(Vec::new());
    }

    let end = first.checked_add(count).ok_or_else(|| {
        DialogParseError::InvalidField(format!(
            "state {} transition range overflows u32",
            state_index
        ))
    })?;

    if end > total {
        return Err(DialogParseError::InvalidField(format!(
            "state {} references transitions {}..{} but only {} exist",
            state_index, first, end, total
        )));
    }

    Ok(transitions[first as usize..end as usize].to_vec())
}

fn parse_transitions(
    bytes: &[u8],
    offset: u32,
    count: u32,
    resolver: Option<&dyn StrRefResolver>,
    transition_triggers: &[DialogScriptJson],
    actions: &[DialogScriptJson],
) -> Result<Vec<DialogTransitionJson>, DialogParseError> {
    parse_table(
        bytes,
        offset,
        count,
        DLG_TRANSITION_SIZE,
        |bytes, position, index| {
            let flags_raw = parse_u32(bytes, position)?;
            let player_text_raw = parse_u32(bytes, position + 0x04)?;
            let journal_text_raw = parse_u32(bytes, position + 0x08)?;
            let trigger_index_raw = parse_u32(bytes, position + 0x0C)?;
            let action_index_raw = parse_u32(bytes, position + 0x10)?;
            let next_dialog = parse_resref_option(bytes, position + 0x14, 8)?;
            let next_state_raw = parse_u32(bytes, position + 0x1C)?;

            let flags = RawDecodedFlags {
                raw: flags_raw,
                decoded: decode_transition_flags(flags_raw),
            };
            let has_text = flags_raw & 0x0001 != 0;
            let has_trigger = flags_raw & 0x0002 != 0;
            let has_action = flags_raw & 0x0004 != 0;
            let terminates_dialog = flags_raw & 0x0008 != 0;
            let has_journal = flags_raw & 0x0010 != 0;

            let player_text = if has_text {
                Some(resolve_strref_value(player_text_raw, resolver))
            } else {
                None
            };
            let journal_text = if has_journal {
                Some(resolve_strref_value(journal_text_raw, resolver))
            } else {
                None
            };

            let trigger_index = if has_trigger {
                optional_index(trigger_index_raw)
            } else {
                None
            };
            let trigger_text = trigger_index
                .and_then(|idx| transition_triggers.get(idx as usize))
                .map(|script| script.text.clone());

            let action_index = if has_action {
                optional_index(action_index_raw)
            } else {
                None
            };
            let action_text = action_index
                .and_then(|idx| actions.get(idx as usize))
                .map(|script| script.text.clone());

            let next_state_index = if terminates_dialog {
                None
            } else {
                optional_index(next_state_raw)
            };

            Ok(DialogTransitionJson {
                index,
                flags,
                terminates_dialog,
                player_text,
                journal_text,
                trigger_index,
                trigger_text,
                action_index,
                action_text,
                next_dialog,
                next_state_index,
            })
        },
    )
}

fn parse_script_table(
    bytes: &[u8],
    offset: u32,
    count: u32,
    label: &str,
) -> Result<Vec<DialogScriptJson>, DialogParseError> {
    parse_table(
        bytes,
        offset,
        count,
        DLG_SCRIPT_ENTRY_SIZE,
        |bytes, position, _| {
            let script_offset = parse_u32(bytes, position)?;
            let script_length = parse_u32(bytes, position + 0x04)?;
            let text = read_script_text(bytes, script_offset, script_length, label)?;
            Ok(DialogScriptJson {
                offset: script_offset,
                length: script_length,
                text,
            })
        },
    )
}

fn read_script_text(
    bytes: &[u8],
    offset: u32,
    length: u32,
    label: &str,
) -> Result<String, DialogParseError> {
    if length == 0 {
        return Ok(String::new());
    }

    let start = offset as usize;
    let end = start.checked_add(length as usize).ok_or_else(|| {
        DialogParseError::InvalidField(format!("{} offset and length overflow", label))
    })?;

    if end > bytes.len() {
        return Err(DialogParseError::UnexpectedEof(format!(
            "{} text at {}..{} exceeds DLG length {}",
            label,
            start,
            end,
            bytes.len()
        )));
    }

    Ok(String::from_utf8_lossy(&bytes[start..end]).to_string())
}

fn parse_table<T, F>(
    bytes: &[u8],
    offset: u32,
    count: u32,
    record_size: usize,
    mut parser: F,
) -> Result<Vec<T>, DialogParseError>
where
    F: FnMut(&[u8], usize, u32) -> Result<T, DialogParseError>,
{
    if count == 0 {
        return Ok(Vec::new());
    }

    let start = offset as usize;
    let total_size = (count as usize).checked_mul(record_size).ok_or_else(|| {
        DialogParseError::InvalidField("table count and record size overflow".to_string())
    })?;
    let end = start.checked_add(total_size).ok_or_else(|| {
        DialogParseError::InvalidField("table offset and size overflow".to_string())
    })?;
    if end > bytes.len() {
        return Err(DialogParseError::UnexpectedEof(format!(
            "table at {} with {} records of {} bytes exceeds DLG length {}",
            offset,
            count,
            record_size,
            bytes.len()
        )));
    }

    let mut rows = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let position = start + index * record_size;
        rows.push(parser(bytes, position, index as u32)?);
    }
    Ok(rows)
}

fn optional_index(value: u32) -> Option<u32> {
    if value == u32::MAX { None } else { Some(value) }
}

fn parse_resref_option(
    bytes: &[u8],
    offset: usize,
    length: usize,
) -> Result<Option<ResRef>, DialogParseError> {
    let raw = bytes
        .get(offset..offset + length)
        .ok_or_else(|| DialogParseError::UnexpectedEof(format!("missing resref at {}", offset)))?;
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    let string = String::from_utf8_lossy(&raw[..end])
        .trim()
        .to_ascii_uppercase();
    if string.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ResRef::new(string).map_err(|err| {
            DialogParseError::InvalidField(err.to_string())
        })?))
    }
}

fn parse_strref(
    bytes: &[u8],
    offset: usize,
    resolver: Option<&dyn StrRefResolver>,
) -> Result<ResolvedStrRef, DialogParseError> {
    let raw = parse_u32(bytes, offset)?;
    Ok(resolve_strref_value(raw, resolver))
}

fn resolve_strref_value(raw: u32, resolver: Option<&dyn StrRefResolver>) -> ResolvedStrRef {
    let text = if raw == u32::MAX {
        None
    } else {
        resolver.and_then(|resolver| resolver.resolve_strref(StrRef(raw)))
    };
    ResolvedStrRef {
        strref: StrRef(raw),
        text,
    }
}

fn parse_ascii_string(
    bytes: &[u8],
    offset: usize,
    length: usize,
) -> Result<String, DialogParseError> {
    let raw = bytes.get(offset..offset + length).ok_or_else(|| {
        DialogParseError::UnexpectedEof(format!("missing ascii string at {}", offset))
    })?;
    Ok(String::from_utf8_lossy(raw).to_string())
}

fn parse_u32(bytes: &[u8], offset: usize) -> Result<u32, DialogParseError> {
    let raw = bytes.get(offset..offset + 4).ok_or_else(|| {
        DialogParseError::UnexpectedEof(format!("unable to read u32 at {}", offset))
    })?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn decode_dialog_flags(value: u32) -> Vec<String> {
    decode_bits(
        value,
        &[
            (0x0001, "Enemy"),
            (0x0002, "EscapeArea"),
            (0x0004, "Nothing"),
        ],
    )
}

fn decode_transition_flags(value: u32) -> Vec<String> {
    decode_bits(
        value,
        &[
            (0x0001, "HasText"),
            (0x0002, "HasTrigger"),
            (0x0004, "HasAction"),
            (0x0008, "TerminatesDialog"),
            (0x0010, "HasJournal"),
            (0x0020, "Interrupt"),
            (0x0040, "AddUnsolvedQuest"),
            (0x0080, "AddJournalNote"),
            (0x0100, "AddSolvedQuest"),
            (0x0200, "Immediate"),
            (0x0400, "ClearActions"),
        ],
    )
}

fn decode_bits(value: u32, mapping: &[(u32, &str)]) -> Vec<String> {
    mapping
        .iter()
        .filter_map(|(bit, label)| (value & bit != 0).then_some((*label).to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NullResolver;

    impl StrRefResolver for NullResolver {
        fn resolve_strref(&self, _strref: StrRef) -> Option<String> {
            None
        }
    }

    #[test]
    fn parses_synthetic_dlg_with_states_transitions_and_scripts() {
        let state_trigger_text = b"CheckStatGT(Myself,12,STR)";
        let transition_trigger_text = b"Global(\"X\",\"GLOBAL\",0)";
        let action_text = b"SetGlobal(\"X\",\"GLOBAL\",1)";

        let offset_states: u32 = DLG_HEADER_WITH_FLAGS as u32;
        let offset_transitions = offset_states + (2 * DLG_STATE_SIZE as u32);
        let offset_state_triggers = offset_transitions + (3 * DLG_TRANSITION_SIZE as u32);
        let offset_transition_triggers = offset_state_triggers + (1 * DLG_SCRIPT_ENTRY_SIZE as u32);
        let offset_actions = offset_transition_triggers + (1 * DLG_SCRIPT_ENTRY_SIZE as u32);
        let offset_strings = offset_actions + (1 * DLG_SCRIPT_ENTRY_SIZE as u32);

        let state_trigger_offset = offset_strings;
        let transition_trigger_offset = state_trigger_offset + state_trigger_text.len() as u32;
        let action_offset = transition_trigger_offset + transition_trigger_text.len() as u32;
        let total_len = (action_offset + action_text.len() as u32) as usize;

        let mut bytes = vec![0u8; total_len];

        bytes[0..4].copy_from_slice(b"DLG ");
        bytes[4..8].copy_from_slice(b"V1.0");
        bytes[0x08..0x0C].copy_from_slice(&2u32.to_le_bytes());
        bytes[0x0C..0x10].copy_from_slice(&offset_states.to_le_bytes());
        bytes[0x10..0x14].copy_from_slice(&3u32.to_le_bytes());
        bytes[0x14..0x18].copy_from_slice(&offset_transitions.to_le_bytes());
        bytes[0x18..0x1C].copy_from_slice(&offset_state_triggers.to_le_bytes());
        bytes[0x1C..0x20].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x20..0x24].copy_from_slice(&offset_transition_triggers.to_le_bytes());
        bytes[0x24..0x28].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x28..0x2C].copy_from_slice(&offset_actions.to_le_bytes());
        bytes[0x2C..0x30].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x30..0x34].copy_from_slice(&0u32.to_le_bytes());

        let state0 = offset_states as usize;
        bytes[state0..state0 + 4].copy_from_slice(&100u32.to_le_bytes());
        bytes[state0 + 4..state0 + 8].copy_from_slice(&0u32.to_le_bytes());
        bytes[state0 + 8..state0 + 12].copy_from_slice(&2u32.to_le_bytes());
        bytes[state0 + 12..state0 + 16].copy_from_slice(&0u32.to_le_bytes());

        let state1 = state0 + DLG_STATE_SIZE;
        bytes[state1..state1 + 4].copy_from_slice(&101u32.to_le_bytes());
        bytes[state1 + 4..state1 + 8].copy_from_slice(&2u32.to_le_bytes());
        bytes[state1 + 8..state1 + 12].copy_from_slice(&1u32.to_le_bytes());
        bytes[state1 + 12..state1 + 16].copy_from_slice(&u32::MAX.to_le_bytes());

        let trans0 = offset_transitions as usize;
        write_transition(
            &mut bytes,
            trans0,
            0x0003,
            200,
            u32::MAX,
            0,
            u32::MAX,
            None,
            1,
        );
        write_transition(
            &mut bytes,
            trans0 + DLG_TRANSITION_SIZE,
            0x0009,
            201,
            u32::MAX,
            u32::MAX,
            u32::MAX,
            None,
            u32::MAX,
        );
        write_transition(
            &mut bytes,
            trans0 + 2 * DLG_TRANSITION_SIZE,
            0x0005,
            202,
            u32::MAX,
            u32::MAX,
            0,
            None,
            u32::MAX,
        );

        let state_trigger_entry = offset_state_triggers as usize;
        bytes[state_trigger_entry..state_trigger_entry + 4]
            .copy_from_slice(&state_trigger_offset.to_le_bytes());
        bytes[state_trigger_entry + 4..state_trigger_entry + 8]
            .copy_from_slice(&(state_trigger_text.len() as u32).to_le_bytes());

        let transition_trigger_entry = offset_transition_triggers as usize;
        bytes[transition_trigger_entry..transition_trigger_entry + 4]
            .copy_from_slice(&transition_trigger_offset.to_le_bytes());
        bytes[transition_trigger_entry + 4..transition_trigger_entry + 8]
            .copy_from_slice(&(transition_trigger_text.len() as u32).to_le_bytes());

        let action_entry = offset_actions as usize;
        bytes[action_entry..action_entry + 4].copy_from_slice(&action_offset.to_le_bytes());
        bytes[action_entry + 4..action_entry + 8]
            .copy_from_slice(&(action_text.len() as u32).to_le_bytes());

        bytes[state_trigger_offset as usize
            ..state_trigger_offset as usize + state_trigger_text.len()]
            .copy_from_slice(state_trigger_text);
        bytes[transition_trigger_offset as usize
            ..transition_trigger_offset as usize + transition_trigger_text.len()]
            .copy_from_slice(transition_trigger_text);
        bytes[action_offset as usize..action_offset as usize + action_text.len()]
            .copy_from_slice(action_text);

        let dlg = parse_dlg(&bytes, "QUENASH.DLG", Some(&NullResolver)).expect("DLG should parse");

        assert_eq!(dlg.resource_name, "QUENASH.DLG");
        assert_eq!(dlg.version, "V1.0");
        assert_eq!(dlg.states.len(), 2);
        assert_eq!(dlg.header.num_transitions, 3);
        assert!(dlg.header.flags.is_some());

        let state0_json = &dlg.states[0];
        assert_eq!(state0_json.index, 0);
        assert_eq!(state0_json.trigger_index, Some(0));
        assert_eq!(
            state0_json.trigger_text.as_deref(),
            Some("CheckStatGT(Myself,12,STR)")
        );
        assert_eq!(state0_json.transitions.len(), 2);

        let trans0 = &state0_json.transitions[0];
        assert!(trans0.flags.decoded.contains(&"HasText".to_string()));
        assert!(trans0.flags.decoded.contains(&"HasTrigger".to_string()));
        assert!(!trans0.terminates_dialog);
        assert_eq!(
            trans0.trigger_text.as_deref(),
            Some("Global(\"X\",\"GLOBAL\",0)")
        );
        assert_eq!(trans0.next_state_index, Some(1));

        let trans1 = &state0_json.transitions[1];
        assert!(trans1.terminates_dialog);
        assert!(trans1.next_state_index.is_none());

        let state1_json = &dlg.states[1];
        assert!(state1_json.trigger_index.is_none());
        assert_eq!(state1_json.transitions.len(), 1);
        let trans2 = &state1_json.transitions[0];
        assert_eq!(
            trans2.action_text.as_deref(),
            Some("SetGlobal(\"X\",\"GLOBAL\",1)")
        );
        assert_eq!(trans2.action_index, Some(0));

        assert_eq!(dlg.state_triggers.len(), 1);
        assert_eq!(dlg.transition_triggers.len(), 1);
        assert_eq!(dlg.actions.len(), 1);
    }

    #[test]
    fn rejects_dlg_without_signature() {
        let bytes = vec![0u8; DLG_HEADER_WITH_FLAGS];
        let err = parse_dlg(&bytes, "BAD.DLG", None).expect_err("should fail");
        let message = format!("{err}");
        assert!(message.contains("signature"), "unexpected error: {message}");
    }

    #[test]
    fn rejects_dlg_shorter_than_header() {
        let bytes = vec![0u8; 8];
        let err = parse_dlg(&bytes, "BAD.DLG", None).expect_err("should fail");
        let message = format!("{err}");
        assert!(message.contains("at least"), "unexpected error: {message}");
    }

    #[allow(clippy::too_many_arguments)]
    fn write_transition(
        bytes: &mut [u8],
        position: usize,
        flags: u32,
        player_text: u32,
        journal_text: u32,
        trigger_index: u32,
        action_index: u32,
        next_dialog: Option<&[u8; 8]>,
        next_state_index: u32,
    ) {
        bytes[position..position + 4].copy_from_slice(&flags.to_le_bytes());
        bytes[position + 4..position + 8].copy_from_slice(&player_text.to_le_bytes());
        bytes[position + 8..position + 12].copy_from_slice(&journal_text.to_le_bytes());
        bytes[position + 12..position + 16].copy_from_slice(&trigger_index.to_le_bytes());
        bytes[position + 16..position + 20].copy_from_slice(&action_index.to_le_bytes());
        let resref = next_dialog.copied().unwrap_or([0u8; 8]);
        bytes[position + 20..position + 28].copy_from_slice(&resref);
        bytes[position + 28..position + 32].copy_from_slice(&next_state_index.to_le_bytes());
    }
}
