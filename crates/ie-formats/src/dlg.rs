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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogGraphStringMode {
    Resolved,
    StrRef,
    Both,
}

#[derive(Debug, Clone, Copy)]
pub struct DialogGraphOptions {
    pub max_label_len: usize,
    pub include_triggers: bool,
    pub include_actions: bool,
    pub string_mode: DialogGraphStringMode,
}

impl Default for DialogGraphOptions {
    fn default() -> Self {
        Self {
            max_label_len: 40,
            include_triggers: true,
            include_actions: true,
            string_mode: DialogGraphStringMode::Resolved,
        }
    }
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

pub fn dialog_json_to_dot(dialog: &DialogJson, options: &DialogGraphOptions) -> String {
    let mut output = String::new();
    output.push_str("digraph DLG {\n");
    output.push_str("  rankdir=LR;\n");
    output.push_str("  node [fontname=\"Arial\"];\n");
    output.push_str("  edge [fontname=\"Arial\"];\n");
    output.push_str("  END [shape=doublecircle,label=\"END\"];\n");

    for state in &dialog.states {
        output.push_str(&format!(
            "  S{} [shape=box,label=\"{}\"];\n",
            state.index,
            escape_dot_label(&state_label(state, options))
        ));
    }

    let mut external_nodes = Vec::new();
    for state in &dialog.states {
        for transition in &state.transitions {
            if let Some(external) = external_node_id(dialog, transition)
                && !external_nodes.contains(&external)
            {
                external_nodes.push(external);
            }
        }
    }
    external_nodes.sort();
    for node in external_nodes {
        output.push_str(&format!(
            "  {node} [shape=box,style=\"dashed,filled\",fillcolor=\"#fff2cc\",label=\"{}\"];\n",
            escape_dot_label(&external_node_label(&node))
        ));
    }

    for state in &dialog.states {
        for transition in &state.transitions {
            let target = transition_target(dialog, transition).unwrap_or_else(|| "END".to_string());
            let mut attrs = vec![format!(
                "label=\"{}\"",
                escape_dot_label(&transition_label(transition, options))
            )];
            if options.include_triggers && non_empty(transition.trigger_text.as_deref()).is_some() {
                attrs.push("color=\"#b00020\"".to_string());
                attrs.push("fontcolor=\"#b00020\"".to_string());
            }
            output.push_str(&format!(
                "  S{} -> {} [{}];\n",
                state.index,
                target,
                attrs.join(",")
            ));
        }
    }

    output.push_str("}\n");
    output
}

pub fn dialog_json_to_mermaid(dialog: &DialogJson, options: &DialogGraphOptions) -> String {
    let mut output = String::new();
    output.push_str("graph LR\n");
    output.push_str("  END((END))\n");

    for state in &dialog.states {
        output.push_str(&format!(
            "  S{}[\"{}\"]\n",
            state.index,
            escape_mermaid_label(&state_label(state, options))
        ));
    }

    let mut external_nodes = Vec::new();
    for state in &dialog.states {
        for transition in &state.transitions {
            if let Some(external) = external_node_id(dialog, transition)
                && !external_nodes.contains(&external)
            {
                external_nodes.push(external);
            }
        }
    }
    external_nodes.sort();
    for node in &external_nodes {
        output.push_str(&format!(
            "  {node}[\"{}\"]\n",
            escape_mermaid_label(&external_node_label(node))
        ));
    }

    for state in &dialog.states {
        for transition in &state.transitions {
            let target = transition_target(dialog, transition).unwrap_or_else(|| "END".to_string());
            let label = transition_label(transition, options);
            if label.is_empty() {
                output.push_str(&format!("  S{} --> {}\n", state.index, target));
            } else {
                output.push_str(&format!(
                    "  S{} -->|\"{}\"| {}\n",
                    state.index,
                    escape_mermaid_label(&label),
                    target
                ));
            }
        }
    }

    if !external_nodes.is_empty() {
        output.push_str("  classDef external fill:#fff2cc,stroke-dasharray: 5 5\n");
        output.push_str(&format!("  class {} external\n", external_nodes.join(",")));
    }

    output
}

fn state_label(state: &DialogStateJson, options: &DialogGraphOptions) -> String {
    let mut parts = vec![format!(
        "S{}: {}",
        state.index,
        truncate(
            &resolved_label(&state.response_text, options.string_mode),
            options.max_label_len
        )
    )];

    if options.include_triggers
        && let Some(trigger) = non_empty(state.trigger_text.as_deref())
    {
        parts.push(format!("if {}", truncate(trigger, options.max_label_len)));
    }

    parts.join("\n")
}

fn transition_label(transition: &DialogTransitionJson, options: &DialogGraphOptions) -> String {
    let mut parts = Vec::new();

    if let Some(player_text) = &transition.player_text {
        parts.push(truncate(
            &resolved_label(player_text, options.string_mode),
            options.max_label_len,
        ));
    }

    if options.include_triggers
        && let Some(trigger) = non_empty(transition.trigger_text.as_deref())
    {
        parts.push(format!("if {}", truncate(trigger, options.max_label_len)));
    }

    if transition.journal_text.is_some() {
        parts.push("[journal]".to_string());
    }

    if options.include_actions
        && let Some(action) = non_empty(transition.action_text.as_deref())
    {
        if let Some(cutscene) = cutscene_tag(action) {
            parts.push(cutscene);
        } else {
            parts.push(truncate(action, options.max_label_len));
        }
    }

    parts.join("\n")
}

fn transition_target(dialog: &DialogJson, transition: &DialogTransitionJson) -> Option<String> {
    if transition.terminates_dialog {
        return Some("END".to_string());
    }

    if let Some(external) = external_node_id(dialog, transition) {
        return Some(external);
    }

    transition
        .next_state_index
        .map(|state_index| format!("S{state_index}"))
}

fn external_node_id(dialog: &DialogJson, transition: &DialogTransitionJson) -> Option<String> {
    let next_dialog = transition.next_dialog.as_ref()?;
    if next_dialog
        .as_str()
        .eq_ignore_ascii_case(dialog_resref(dialog))
    {
        return None;
    }

    let state = transition
        .next_state_index
        .map(|index| index.to_string())
        .unwrap_or_else(|| "UNKNOWN".to_string());
    Some(format!(
        "EXT_{}_{}",
        graph_id_component(next_dialog.as_str()),
        graph_id_component(&state)
    ))
}

fn external_node_label(node_id: &str) -> String {
    node_id
        .strip_prefix("EXT_")
        .unwrap_or(node_id)
        .replace('_', " ")
}

fn dialog_resref(dialog: &DialogJson) -> &str {
    dialog
        .resource_name
        .rsplit_once('.')
        .map(|(resref, _)| resref)
        .unwrap_or(&dialog.resource_name)
}

fn resolved_label(value: &ResolvedStrRef, mode: DialogGraphStringMode) -> String {
    match mode {
        DialogGraphStringMode::Resolved => value
            .text
            .clone()
            .unwrap_or_else(|| format!("#{}", value.strref.0)),
        DialogGraphStringMode::StrRef => format!("#{}", value.strref.0),
        DialogGraphStringMode::Both => value
            .text
            .as_ref()
            .map(|text| format!("{text} (#{})", value.strref.0))
            .unwrap_or_else(|| format!("#{}", value.strref.0)),
    }
}

fn cutscene_tag(action: &str) -> Option<String> {
    // Find a StartCutScene / StartCutSceneEx call that carries a quoted cutscene
    // name. In practice these are preceded by a nameless StartCutSceneMode() call
    // (e.g. `StartCutSceneMode()StartCutSceneEx("finmel1",FALSE)`), so we must skip
    // the "Mode" variant and keep scanning rather than short-circuit on it.
    for marker in ["StartCutSceneEx", "StartCutScene"] {
        let mut search_from = 0;
        while let Some(rel) = action[search_from..].find(marker) {
            let start = search_from + rel;
            let after_marker = &action[start + marker.len()..];
            search_from = start + marker.len();

            // "StartCutScene" is a prefix of "StartCutSceneMode"; the mode call has
            // no name argument, so skip it.
            if after_marker.starts_with("Mode") {
                continue;
            }

            let Some(open) = after_marker.find('(') else {
                continue;
            };
            let after_open = after_marker[open + 1..].trim_start();
            let end = after_open.find([',', ')']).unwrap_or(after_open.len());
            let name = after_open[..end].trim().trim_matches('"');
            if !name.is_empty() {
                return Some(format!("[cutscene: {name}]"));
            }
        }
    }

    // A cutscene is fired but we could not recover a name (e.g. mode-only).
    if action.contains("StartCutScene") {
        Some("[cutscene mode]".to_string())
    } else {
        None
    }
}

fn truncate(value: &str, max_len: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if max_len == 0 || normalized.chars().count() <= max_len {
        return normalized;
    }

    if max_len <= 3 {
        return ".".repeat(max_len);
    }

    let mut truncated = normalized
        .chars()
        .take(max_len.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn graph_id_component(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_uppercase());
        } else {
            output.push('_');
        }
    }
    if output.is_empty() {
        "UNKNOWN".to_string()
    } else {
        output
    }
}

fn escape_dot_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn escape_mermaid_label(value: &str) -> String {
    value
        .replace('"', "#quot;")
        .replace('|', "&#124;")
        .replace('\n', "<br/>")
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
            let trigger_text = script_text_at(trigger_index, state_triggers);

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
            let trigger_text = script_text_at(trigger_index, transition_triggers);

            let action_index = if has_action {
                optional_index(action_index_raw)
            } else {
                None
            };
            let action_text = script_text_at(action_index, actions);

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

fn script_text_at(index: Option<u32>, scripts: &[DialogScriptJson]) -> Option<String> {
    index
        .and_then(|idx| scripts.get(idx as usize))
        .map(|script| script.text.clone())
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
        let offset_transition_triggers = offset_state_triggers + DLG_SCRIPT_ENTRY_SIZE as u32;
        let offset_actions = offset_transition_triggers + DLG_SCRIPT_ENTRY_SIZE as u32;
        let offset_strings = offset_actions + DLG_SCRIPT_ENTRY_SIZE as u32;

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

        let json = serde_json::to_value(&dlg).expect("DLG JSON should serialize");
        assert_eq!(
            json["states"][0]["trigger_text"],
            "CheckStatGT(Myself,12,STR)"
        );
        assert_eq!(
            json["states"][0]["transitions"][0]["trigger_text"],
            "Global(\"X\",\"GLOBAL\",0)"
        );
        assert_eq!(
            json["states"][1]["transitions"][0]["action_text"],
            "SetGlobal(\"X\",\"GLOBAL\",1)"
        );
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

    #[test]
    fn renders_dlg_as_dot_with_internal_external_and_terminal_edges() {
        let dialog = graph_test_dialog();
        let dot = dialog_json_to_dot(&dialog, &DialogGraphOptions::default());

        assert!(dot.contains("digraph DLG"));
        assert!(dot.contains("S0 [shape=box,label=\"S0: Hello there\\nif StateCheck()\"]"));
        assert!(
            dot.contains("S0 -> S1 [label=\"Continue\\nif Global(\\\"X\\\",\\\"GLOBAL\\\",0)\"")
        );
        assert!(dot.contains("S0 -> END [label=\"Leave\"]"));
        assert!(dot.contains("EXT_OTHER_2 [shape=box"));
        assert!(dot.contains(
            "S1 -> EXT_OTHER_2 [label=\"Ask about cutscene\\n[journal]\\n[cutscene: CUT01]\"]"
        ));
    }

    #[test]
    fn renders_dlg_as_mermaid_with_string_modes_and_options() {
        let dialog = graph_test_dialog();
        let options = DialogGraphOptions {
            max_label_len: 12,
            include_triggers: false,
            include_actions: false,
            string_mode: DialogGraphStringMode::Both,
        };
        let mermaid = dialog_json_to_mermaid(&dialog, &options);

        assert!(mermaid.contains("graph LR"));
        assert!(mermaid.contains("S0[\"S0: Hello the...\"]"));
        assert!(mermaid.contains("S0 -->|\"Continue ...\"| S1"));
        assert!(mermaid.contains("S0 -->|\"Leave (#20)\"| END"));
        assert!(mermaid.contains("S1 -->|\"Ask about...<br/>[journal]\"| EXT_OTHER_2"));
        assert!(!mermaid.contains("Global("));
        assert!(!mermaid.contains("cutscene"));
    }

    fn graph_test_dialog() -> DialogJson {
        DialogJson {
            resource_type: "DLG".to_string(),
            resource_name: "TEST.DLG".to_string(),
            version: "V1.0".to_string(),
            header: DialogHeaderJson {
                num_states: 2,
                offset_states: 0,
                num_transitions: 3,
                offset_transitions: 0,
                offset_state_triggers: 0,
                num_state_triggers: 1,
                offset_transition_triggers: 0,
                num_transition_triggers: 1,
                offset_actions: 0,
                num_actions: 1,
                flags: None,
            },
            states: vec![
                DialogStateJson {
                    index: 0,
                    response_text: resolved(10, "Hello there"),
                    first_transition_index: 0,
                    num_transitions: 2,
                    trigger_index: Some(0),
                    trigger_text: Some("StateCheck()".to_string()),
                    transitions: vec![
                        DialogTransitionJson {
                            index: 0,
                            flags: flags(0x0003),
                            terminates_dialog: false,
                            player_text: Some(resolved(11, "Continue")),
                            journal_text: None,
                            trigger_index: Some(0),
                            trigger_text: Some("Global(\"X\",\"GLOBAL\",0)".to_string()),
                            action_index: None,
                            action_text: None,
                            next_dialog: None,
                            next_state_index: Some(1),
                        },
                        DialogTransitionJson {
                            index: 1,
                            flags: flags(0x0009),
                            terminates_dialog: true,
                            player_text: Some(resolved(20, "Leave")),
                            journal_text: None,
                            trigger_index: None,
                            trigger_text: None,
                            action_index: None,
                            action_text: None,
                            next_dialog: None,
                            next_state_index: None,
                        },
                    ],
                },
                DialogStateJson {
                    index: 1,
                    response_text: resolved(12, "Second state"),
                    first_transition_index: 2,
                    num_transitions: 1,
                    trigger_index: None,
                    trigger_text: None,
                    transitions: vec![DialogTransitionJson {
                        index: 2,
                        flags: flags(0x0005),
                        terminates_dialog: false,
                        player_text: Some(resolved(21, "Ask about cutscene")),
                        journal_text: Some(resolved(22, "Journal note")),
                        trigger_index: None,
                        trigger_text: None,
                        action_index: Some(0),
                        action_text: Some(
                            "ClearAllActions()StartCutSceneMode()StartCutSceneEx(\"CUT01\",FALSE)"
                                .to_string(),
                        ),
                        next_dialog: Some(ResRef::new("OTHER").expect("valid resref")),
                        next_state_index: Some(2),
                    }],
                },
            ],
            state_triggers: Vec::new(),
            transition_triggers: Vec::new(),
            actions: Vec::new(),
        }
    }

    fn resolved(strref: u32, text: &str) -> ResolvedStrRef {
        ResolvedStrRef {
            strref: StrRef(strref),
            text: Some(text.to_string()),
        }
    }

    fn flags(raw: u32) -> RawDecodedFlags {
        RawDecodedFlags {
            raw,
            decoded: Vec::new(),
        }
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
