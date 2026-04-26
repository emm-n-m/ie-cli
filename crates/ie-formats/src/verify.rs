use crate::are::AreaJson;
use ie_core::{ResourceLink, ScriptSlots};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifySeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifyCategory {
    DeadLink,
    PhantomEntrance,
    MissingAreaScript,
    MissingRegionScript,
    MissingActorCre,
    MissingActorDialog,
    MissingActorScript,
    MissingKeyItem,
    ParseError,
}

impl VerifyCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeadLink => "dead_link",
            Self::PhantomEntrance => "phantom_entrance",
            Self::MissingAreaScript => "missing_area_script",
            Self::MissingRegionScript => "missing_region_script",
            Self::MissingActorCre => "missing_actor_cre",
            Self::MissingActorDialog => "missing_actor_dialog",
            Self::MissingActorScript => "missing_actor_script",
            Self::MissingKeyItem => "missing_key_item",
            Self::ParseError => "parse_error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifyIssue {
    pub resource: String,
    pub issue: VerifyCategory,
    pub severity: VerifySeverity,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_in: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_entrances: Option<Vec<String>>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VerifyOptions {
    pub severity: Option<VerifySeverity>,
    pub max_issues: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct AreaSourceEntry {
    pub resource_name: String,
    pub area: AreaJson,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AreaSourceError {
    pub resource_name: String,
    pub message: String,
}

pub trait AreaSource {
    fn areas(&self) -> Vec<Result<AreaSourceEntry, AreaSourceError>>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntranceRegistry {
    entrances_by_area: BTreeMap<String, BTreeSet<String>>,
}

impl EntranceRegistry {
    pub fn insert_area(&mut self, area: &AreaJson) {
        let entrances = area
            .entrances
            .iter()
            .map(|entrance| entrance.name.clone())
            .collect::<BTreeSet<_>>();
        self.entrances_by_area
            .insert(area.resource_name.to_ascii_uppercase(), entrances);
    }

    pub fn contains_area(&self, resource_name: &str) -> bool {
        self.entrances_by_area
            .contains_key(&resource_name.to_ascii_uppercase())
    }

    pub fn entrance_names(&self, resource_name: &str) -> Vec<String> {
        self.entrances_by_area
            .get(&resource_name.to_ascii_uppercase())
            .map(|names| names.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn has_entrance(&self, resource_name: &str, entrance_name: &str) -> bool {
        self.entrances_by_area
            .get(&resource_name.to_ascii_uppercase())
            .is_some_and(|names| names.contains(entrance_name))
    }
}

pub fn build_entrance_registry(source: &dyn AreaSource) -> EntranceRegistry {
    let mut registry = EntranceRegistry::default();
    for entry in source.areas().into_iter().flatten() {
        registry.insert_area(&entry.area);
    }
    registry
}

pub fn verify_are(area: &AreaJson, registry: &EntranceRegistry) -> Vec<VerifyIssue> {
    let mut issues = Vec::new();

    if let Some(link) = area.header.area_script.as_ref()
        && !link.exists
    {
        issues.push(missing_link_issue(
            area,
            VerifyCategory::MissingAreaScript,
            "header.area_script",
            link,
        ));
    }

    for (index, region) in area.regions.iter().enumerate() {
        let region_path = |field: &str| format!("regions[{index}].{field}");

        if let Some(link) = region.destination_area.as_ref()
            && !link.exists
        {
            issues.push(VerifyIssue {
                resource: area.resource_name.clone(),
                issue: VerifyCategory::DeadLink,
                severity: VerifySeverity::Error,
                path: region_path("destination_area"),
                expected_in: None,
                expected_value: Some(link.resource_name.clone()),
                available_entrances: None,
                message: format!(
                    "Travel region references missing destination area '{}'",
                    link.resource_name
                ),
            });
        }

        if let (Some(destination), Some(entrance)) = (
            region.destination_area.as_ref(),
            region.destination_entrance.as_ref(),
        ) && destination.exists
            && registry.contains_area(&destination.resource_name)
            && !registry.has_entrance(&destination.resource_name, entrance)
        {
            let available_entrances = registry.entrance_names(&destination.resource_name);
            issues.push(VerifyIssue {
                resource: area.resource_name.clone(),
                issue: VerifyCategory::PhantomEntrance,
                severity: VerifySeverity::Error,
                path: region_path("destination_entrance"),
                expected_in: Some(destination.resource_name.clone()),
                expected_value: Some(entrance.clone()),
                available_entrances: Some(available_entrances.clone()),
                message: format!(
                    "destination_entrance '{}' is not present in {}; available entrances: {}",
                    entrance,
                    destination.resource_name,
                    format_available(&available_entrances)
                ),
            });
        }

        if let Some(link) = region.region_script.as_ref()
            && !link.exists
        {
            issues.push(missing_link_issue(
                area,
                VerifyCategory::MissingRegionScript,
                &region_path("region_script"),
                link,
            ));
        }

        if let Some(link) = region.key_item.as_ref()
            && !link.exists
        {
            issues.push(missing_link_issue(
                area,
                VerifyCategory::MissingKeyItem,
                &region_path("key_item"),
                link,
            ));
        }
    }

    for (index, actor) in area.actors.iter().enumerate() {
        let actor_path = |field: &str| format!("actors[{index}].{field}");

        if let Some(link) = actor.dialog.as_ref()
            && !link.exists
        {
            issues.push(missing_link_issue(
                area,
                VerifyCategory::MissingActorDialog,
                &actor_path("dialog"),
                link,
            ));
        }

        if let Some(creature) = actor.cre_file.as_ref()
            && !creature.link.exists
        {
            issues.push(missing_link_issue(
                area,
                VerifyCategory::MissingActorCre,
                &actor_path("cre_file"),
                &creature.link,
            ));
        }

        push_missing_actor_scripts(area, index, &actor.scripts, &mut issues);
    }

    issues.sort_by(|left, right| {
        (&left.resource, &left.path, left.issue).cmp(&(&right.resource, &right.path, right.issue))
    });
    issues
}

pub fn filter_issues(mut issues: Vec<VerifyIssue>, options: VerifyOptions) -> Vec<VerifyIssue> {
    issues.retain(|issue| {
        options
            .severity
            .is_none_or(|severity| issue.severity == severity)
    });
    issues.sort_by(|left, right| {
        (&left.resource, &left.path, left.issue).cmp(&(&right.resource, &right.path, right.issue))
    });
    if let Some(max_issues) = options.max_issues {
        issues.truncate(max_issues);
    }
    issues
}

fn push_missing_actor_scripts(
    area: &AreaJson,
    actor_index: usize,
    scripts: &ScriptSlots<ResourceLink>,
    issues: &mut Vec<VerifyIssue>,
) {
    let slots = [
        ("override_script", scripts.override_script.as_ref()),
        ("general_script", scripts.general_script.as_ref()),
        ("class_script", scripts.class_script.as_ref()),
        ("race_script", scripts.race_script.as_ref()),
        ("default_script", scripts.default_script.as_ref()),
        ("specific_script", scripts.specific_script.as_ref()),
    ];

    for (name, link) in slots {
        if let Some(link) = link
            && !link.exists
        {
            issues.push(missing_link_issue(
                area,
                VerifyCategory::MissingActorScript,
                &format!("actors[{actor_index}].scripts.{name}"),
                link,
            ));
        }
    }
}

fn missing_link_issue(
    area: &AreaJson,
    category: VerifyCategory,
    path: &str,
    link: &ResourceLink,
) -> VerifyIssue {
    VerifyIssue {
        resource: area.resource_name.clone(),
        issue: category,
        severity: VerifySeverity::Warning,
        path: path.to_string(),
        expected_in: None,
        expected_value: Some(link.resource_name.clone()),
        available_entrances: None,
        message: format!(
            "{} references missing resource '{}'",
            path, link.resource_name
        ),
    }
}

fn format_available(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_string()
    } else {
        values.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::are::parse_are;
    use ie_core::{
        CreatureResourceLink, ResRef, ResolvedStrRef, ResourceLinkResolver, ResourceType,
        SourceKind, StrRef,
    };
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    const ARE_HEADER_SIZE: usize = 0x11C;
    const ARE_ACTOR_SIZE: usize = 0x110;
    const ARE_REGION_SIZE: usize = 0xC4;
    const ARE_ENTRANCE_SIZE: usize = 0x68;

    struct TestLinks {
        existing: BTreeSet<String>,
    }

    impl TestLinks {
        fn new(existing: &[&str]) -> Self {
            Self {
                existing: existing
                    .iter()
                    .map(|name| name.to_ascii_uppercase())
                    .collect(),
            }
        }
    }

    impl ResourceLinkResolver for TestLinks {
        fn resolve_resource_link(
            &self,
            resref: &ResRef,
            resource_type: ResourceType,
        ) -> ie_core::ResourceLink {
            let resource_name = format!("{}.{}", resref.as_str(), resource_type.as_str());
            let exists = self.existing.contains(&resource_name);
            ie_core::ResourceLink {
                resref: resref.clone(),
                resource_name,
                resource_type: resource_type.as_str().to_string(),
                exists,
                source_kind: exists.then_some(SourceKind::Override),
                source_path: exists.then_some(PathBuf::from("override")),
            }
        }

        fn resolve_creature_link(&self, resref: &ResRef) -> CreatureResourceLink {
            CreatureResourceLink {
                link: self.resolve_resource_link(resref, ResourceType::Cre),
                short_name: Some(ResolvedStrRef {
                    strref: StrRef(1),
                    text: Some("Short".to_string()),
                }),
                long_name: None,
            }
        }
    }

    struct TestAreaSource {
        areas: Vec<AreaJson>,
    }

    impl AreaSource for TestAreaSource {
        fn areas(&self) -> Vec<Result<AreaSourceEntry, AreaSourceError>> {
            self.areas
                .iter()
                .cloned()
                .map(|area| {
                    Ok(AreaSourceEntry {
                        resource_name: area.resource_name.clone(),
                        area,
                    })
                })
                .collect()
        }
    }

    #[test]
    fn phantom_entrance_detected_when_destination_lacks_entrance() {
        let source = area_with_travel("ARSRC.ARE", "ToDest", "ARDEST", "Foo", &["ARDEST.ARE"]);
        let dest = area_with_entrances("ARDEST.ARE", &["Bar"]);
        let registry = registry_for(vec![source.clone(), dest]);

        let issues = verify_are(&source, &registry);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue, VerifyCategory::PhantomEntrance);
        assert_eq!(issues[0].severity, VerifySeverity::Error);
        assert_eq!(issues[0].path, "regions[0].destination_entrance");
        assert_eq!(issues[0].expected_in.as_deref(), Some("ARDEST.ARE"));
        assert_eq!(issues[0].expected_value.as_deref(), Some("Foo"));
        assert_eq!(
            issues[0].available_entrances.as_deref(),
            Some(&["Bar".to_string()][..])
        );
    }

    #[test]
    fn entrance_match_clean_pass() {
        let source = area_with_travel("ARSRC.ARE", "ToDest", "ARDEST", "Foo", &["ARDEST.ARE"]);
        let dest = area_with_entrances("ARDEST.ARE", &["Foo"]);
        let registry = registry_for(vec![source.clone(), dest]);

        let issues = verify_are(&source, &registry);

        assert!(issues.is_empty());
    }

    #[test]
    fn dead_link_detected_when_destination_does_not_exist() {
        let source = area_with_travel("ARSRC.ARE", "ToDest", "MISSING", "Foo", &[]);
        let registry = registry_for(vec![source.clone()]);

        let issues = verify_are(&source, &registry);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue, VerifyCategory::DeadLink);
        assert_eq!(issues[0].severity, VerifySeverity::Error);
        assert_eq!(issues[0].path, "regions[0].destination_area");
    }

    #[test]
    fn non_travel_regions_skipped() {
        let area = area_with_non_travel_region_garbage();
        let registry = registry_for(vec![area.clone(), area_with_entrances("GHOST.ARE", &[])]);

        let issues = verify_are(&area, &registry);

        assert!(issues.is_empty());
    }

    #[test]
    fn missing_area_script_warning() {
        let area = area_with_missing_area_script();
        let registry = registry_for(vec![area.clone()]);

        let issues = verify_are(&area, &registry);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue, VerifyCategory::MissingAreaScript);
        assert_eq!(issues[0].severity, VerifySeverity::Warning);
        assert_eq!(issues[0].path, "header.area_script");
    }

    #[test]
    fn missing_actor_dialog_warning() {
        let area = area_with_actor_dialog(Some("MISSING"));
        let registry = registry_for(vec![area.clone()]);

        let issues = verify_are(&area, &registry);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue, VerifyCategory::MissingActorDialog);
        assert_eq!(issues[0].path, "actors[0].dialog");
    }

    #[test]
    fn missing_actor_dialog_no_issue_when_resref_empty() {
        let area = area_with_actor_dialog(None);
        let registry = registry_for(vec![area.clone()]);

        let issues = verify_are(&area, &registry);

        assert!(issues.is_empty());
    }

    #[test]
    fn missing_region_script_warning() {
        let area = area_with_region_refs(Some("MISSBCS"), None);
        let registry = registry_for(vec![area.clone()]);

        let issues = verify_are(&area, &registry);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue, VerifyCategory::MissingRegionScript);
        assert_eq!(issues[0].path, "regions[0].region_script");
    }

    #[test]
    fn missing_key_item_warning() {
        let area = area_with_region_refs(None, Some("MISSITM"));
        let registry = registry_for(vec![area.clone()]);

        let issues = verify_are(&area, &registry);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue, VerifyCategory::MissingKeyItem);
        assert_eq!(issues[0].path, "regions[0].key_item");
    }

    #[test]
    fn missing_actor_cre_warning() {
        let area = area_with_actor_refs(None, Some("MISS_CRE"), None);
        let registry = registry_for(vec![area.clone()]);

        let issues = verify_are(&area, &registry);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue, VerifyCategory::MissingActorCre);
        assert_eq!(issues[0].path, "actors[0].cre_file");
    }

    #[test]
    fn missing_actor_script_warning() {
        let area = area_with_actor_refs(None, None, Some("MISSBCS"));
        let registry = registry_for(vec![area.clone()]);

        let issues = verify_are(&area, &registry);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue, VerifyCategory::MissingActorScript);
        assert_eq!(issues[0].path, "actors[0].scripts.override_script");
    }

    #[test]
    fn severity_filter_drops_warnings_when_error_only() {
        let warning = VerifyIssue {
            resource: "A.ARE".to_string(),
            issue: VerifyCategory::MissingAreaScript,
            severity: VerifySeverity::Warning,
            path: "header.area_script".to_string(),
            expected_in: None,
            expected_value: None,
            available_entrances: None,
            message: "warning".to_string(),
        };
        let error = VerifyIssue {
            resource: "B.ARE".to_string(),
            issue: VerifyCategory::DeadLink,
            severity: VerifySeverity::Error,
            path: "regions[0].destination_area".to_string(),
            expected_in: None,
            expected_value: None,
            available_entrances: None,
            message: "error".to_string(),
        };

        let issues = filter_issues(
            vec![warning, error],
            VerifyOptions {
                severity: Some(VerifySeverity::Error),
                max_issues: None,
            },
        );

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, VerifySeverity::Error);
    }

    fn registry_for(areas: Vec<AreaJson>) -> EntranceRegistry {
        let source = TestAreaSource { areas };
        build_entrance_registry(&source)
    }

    fn area_with_travel(
        resource_name: &str,
        region_name: &str,
        destination_area: &str,
        destination_entrance: &str,
        existing: &[&str],
    ) -> AreaJson {
        let mut bytes = minimal_are_bytes();
        let regions_offset = ARE_HEADER_SIZE;
        bytes.resize(regions_offset + ARE_REGION_SIZE, 0);
        bytes[0x5A..0x5C].copy_from_slice(&1u16.to_le_bytes());
        bytes[0x5C..0x60].copy_from_slice(&(regions_offset as u32).to_le_bytes());

        bytes[regions_offset..regions_offset + region_name.len()]
            .copy_from_slice(region_name.as_bytes());
        bytes[regions_offset + 0x20..regions_offset + 0x22].copy_from_slice(&2u16.to_le_bytes());
        bytes[regions_offset + 0x38..regions_offset + 0x38 + destination_area.len()]
            .copy_from_slice(destination_area.as_bytes());
        bytes[regions_offset + 0x40..regions_offset + 0x40 + destination_entrance.len()]
            .copy_from_slice(destination_entrance.as_bytes());

        parse_are(&bytes, resource_name, Some(&TestLinks::new(existing))).expect("ARE should parse")
    }

    fn area_with_entrances(resource_name: &str, entrances: &[&str]) -> AreaJson {
        let mut bytes = minimal_are_bytes();
        let entrances_offset = ARE_HEADER_SIZE;
        bytes.resize(entrances_offset + entrances.len() * ARE_ENTRANCE_SIZE, 0);
        bytes[0x68..0x6C].copy_from_slice(&(entrances_offset as u32).to_le_bytes());
        bytes[0x6C..0x70].copy_from_slice(&(entrances.len() as u32).to_le_bytes());

        for (index, entrance) in entrances.iter().enumerate() {
            let offset = entrances_offset + index * ARE_ENTRANCE_SIZE;
            bytes[offset..offset + entrance.len()].copy_from_slice(entrance.as_bytes());
        }

        parse_are(&bytes, resource_name, Some(&TestLinks::new(&[]))).expect("ARE should parse")
    }

    fn area_with_non_travel_region_garbage() -> AreaJson {
        let mut bytes = minimal_are_bytes();
        let regions_offset = ARE_HEADER_SIZE;
        bytes.resize(regions_offset + ARE_REGION_SIZE, 0);
        bytes[0x5A..0x5C].copy_from_slice(&1u16.to_le_bytes());
        bytes[0x5C..0x60].copy_from_slice(&(regions_offset as u32).to_le_bytes());
        bytes[regions_offset..regions_offset + 7].copy_from_slice(b"Trigger");
        bytes[regions_offset + 0x20..regions_offset + 0x22].copy_from_slice(&0u16.to_le_bytes());
        bytes[regions_offset + 0x38..regions_offset + 0x40].copy_from_slice(b"GHOST\0\0\0");
        bytes[regions_offset + 0x40..regions_offset + 0x43].copy_from_slice(b"Foo");

        parse_are(&bytes, "ARSRC.ARE", Some(&TestLinks::new(&["GHOST.ARE"])))
            .expect("ARE should parse")
    }

    fn area_with_missing_area_script() -> AreaJson {
        let mut bytes = minimal_are_bytes();
        bytes[0x94..0x9C].copy_from_slice(b"MISSING\0");
        parse_are(&bytes, "ARSRC.ARE", Some(&TestLinks::new(&[]))).expect("ARE should parse")
    }

    fn area_with_actor_dialog(dialog: Option<&str>) -> AreaJson {
        area_with_actor_refs(dialog, None, None)
    }

    fn area_with_actor_refs(
        dialog: Option<&str>,
        cre_file: Option<&str>,
        override_script: Option<&str>,
    ) -> AreaJson {
        let mut bytes = minimal_are_bytes();
        let actor_offset = ARE_HEADER_SIZE;
        bytes.resize(actor_offset + ARE_ACTOR_SIZE, 0);
        bytes[0x54..0x58].copy_from_slice(&(actor_offset as u32).to_le_bytes());
        bytes[0x58..0x5A].copy_from_slice(&1u16.to_le_bytes());
        if let Some(dialog) = dialog {
            bytes[actor_offset + 0x48..actor_offset + 0x48 + dialog.len()]
                .copy_from_slice(dialog.as_bytes());
        }
        if let Some(cre_file) = cre_file {
            bytes[actor_offset + 0x80..actor_offset + 0x80 + cre_file.len()]
                .copy_from_slice(cre_file.as_bytes());
        }
        if let Some(override_script) = override_script {
            bytes[actor_offset + 0x50..actor_offset + 0x50 + override_script.len()]
                .copy_from_slice(override_script.as_bytes());
        }

        parse_are(&bytes, "ARSRC.ARE", Some(&TestLinks::new(&[]))).expect("ARE should parse")
    }

    fn area_with_region_refs(region_script: Option<&str>, key_item: Option<&str>) -> AreaJson {
        let mut bytes = minimal_are_bytes();
        let regions_offset = ARE_HEADER_SIZE;
        bytes.resize(regions_offset + ARE_REGION_SIZE, 0);
        bytes[0x5A..0x5C].copy_from_slice(&1u16.to_le_bytes());
        bytes[0x5C..0x60].copy_from_slice(&(regions_offset as u32).to_le_bytes());
        bytes[regions_offset..regions_offset + 6].copy_from_slice(b"Region");
        bytes[regions_offset + 0x20..regions_offset + 0x22].copy_from_slice(&0u16.to_le_bytes());
        if let Some(key_item) = key_item {
            bytes[regions_offset + 0x74..regions_offset + 0x74 + key_item.len()]
                .copy_from_slice(key_item.as_bytes());
        }
        if let Some(region_script) = region_script {
            bytes[regions_offset + 0x7C..regions_offset + 0x7C + region_script.len()]
                .copy_from_slice(region_script.as_bytes());
        }

        parse_are(&bytes, "ARSRC.ARE", Some(&TestLinks::new(&[]))).expect("ARE should parse")
    }

    fn minimal_are_bytes() -> Vec<u8> {
        let mut bytes = vec![0u8; ARE_HEADER_SIZE];
        bytes[0..4].copy_from_slice(b"AREA");
        bytes[4..8].copy_from_slice(b"V1.0");
        bytes[0x08..0x10].copy_from_slice(b"ARSRC\0\0\0");
        bytes[0x54..0x58].copy_from_slice(&(ARE_HEADER_SIZE as u32).to_le_bytes());
        bytes
    }
}
