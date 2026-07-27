use ie_core::ResourceName;
use ie_formats::{DialogJson, parse_dlg};
use ie_io::{ResourceLocator, ResourceReader, ResourceSource, TlkResolver};
use std::collections::BTreeSet;

pub(crate) fn collect_followed_dialogs(
    locator: &ResourceLocator,
    reader: &ResourceReader,
    root: &DialogJson,
    max_depth: usize,
    source: ResourceSource,
    tlk_resolver: Option<&TlkResolver>,
) -> Result<Vec<DialogJson>, Box<dyn std::error::Error>> {
    let mut dialogs = vec![root.clone()];
    let mut depths = vec![0usize];
    let mut visited = BTreeSet::from([root.resource_name.to_ascii_uppercase()]);
    let mut cursor = 0usize;

    while cursor < dialogs.len() {
        let depth = depths[cursor];
        let dialog = dialogs[cursor].clone();
        cursor += 1;

        if depth >= max_depth {
            continue;
        }

        for state in &dialog.states {
            for transition in &state.transitions {
                let Some(next_dialog) = transition.next_dialog.as_ref() else {
                    continue;
                };
                let resource_name = format!("{}.DLG", next_dialog.as_str());
                if resource_name.eq_ignore_ascii_case(&dialog.resource_name) {
                    continue;
                }
                let normalized = resource_name.to_ascii_uppercase();
                if visited.contains(&normalized) {
                    continue;
                }

                let parsed = ResourceName::parse(&resource_name)?;
                let Ok(bytes) = reader.read_with_source(locator, &parsed, source) else {
                    continue;
                };
                // A present-but-corrupt extern degrades to a dashed external node
                // rather than aborting inspection of a possibly-broken install.
                let parsed_dialog = match parse_dlg(
                    &bytes.bytes,
                    &bytes.metadata.resource_name,
                    tlk_resolver.map(|resolver| resolver as _),
                ) {
                    Ok(parsed_dialog) => parsed_dialog,
                    Err(err) => {
                        eprintln!(
                            "warning: skipping unparseable extern DLG {resource_name}: {err}"
                        );
                        continue;
                    }
                };

                visited.insert(normalized);
                dialogs.push(parsed_dialog);
                depths.push(depth + 1);
            }
        }
    }

    dialogs.sort_by(|left, right| left.resource_name.cmp(&right.resource_name));
    Ok(dialogs)
}
