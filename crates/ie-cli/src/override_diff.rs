use ie_core::ResourceName;
use ie_io::{ListedResource, ResourceListOptions, ResourceLocator, ResourceReader, ResourceSource};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct OverrideShadowReport {
    shadows: Vec<OverrideShadowEntry>,
    override_only: Vec<String>,
    counts: OverrideShadowCounts,
}

#[derive(Debug, Clone)]
struct OverrideShadowEntry {
    resource: String,
    override_sha1: String,
    bif_sha1: String,
    identical: bool,
}

#[derive(Debug, Clone)]
struct OverrideShadowCounts {
    override_total: usize,
    shadowing_bif: usize,
    override_only: usize,
}

pub(crate) fn build_override_shadow_report(
    locator: &ResourceLocator,
    resource_type: Option<String>,
    resource: Option<&str>,
) -> Result<OverrideShadowReport, Box<dyn std::error::Error>> {
    let resource_filter = resource
        .map(ResourceName::parse)
        .transpose()?
        .map(|resource| resource.file_name());
    let overrides = locator.list(ResourceListOptions {
        resource_type: resource_type.clone(),
        name_glob: resource_filter.clone(),
        source: Some(ResourceSource::Override),
    })?;
    let bifs = locator.list(ResourceListOptions {
        resource_type,
        name_glob: resource_filter,
        source: Some(ResourceSource::Bif),
    })?;
    let bif_by_name = bifs
        .into_iter()
        .map(|resource| (resource.resource_name.to_ascii_uppercase(), resource))
        .collect::<BTreeMap<_, _>>();

    let reader = ResourceReader;
    let mut shadows = Vec::new();
    let mut override_only = Vec::new();

    for override_resource in &overrides {
        let key = override_resource.resource_name.to_ascii_uppercase();
        if bif_by_name.contains_key(&key) {
            let resource_name = ResourceName::parse(&override_resource.resource_name)?;
            let override_bytes =
                reader.read_with_source(locator, &resource_name, ResourceSource::Override)?;
            let bif_bytes =
                reader.read_with_source(locator, &resource_name, ResourceSource::Bif)?;
            let override_sha1 = sha1_hex(&override_bytes.bytes);
            let bif_sha1 = sha1_hex(&bif_bytes.bytes);

            shadows.push(OverrideShadowEntry {
                resource: resource_name.file_name(),
                identical: override_sha1 == bif_sha1,
                override_sha1,
                bif_sha1,
            });
        } else {
            override_only.push(override_resource.resource_name.clone());
        }
    }

    shadows.sort_by(|left, right| left.resource.cmp(&right.resource));
    override_only.sort();

    Ok(OverrideShadowReport {
        counts: OverrideShadowCounts {
            override_total: overrides.len(),
            shadowing_bif: shadows.len(),
            override_only: override_only.len(),
        },
        shadows,
        override_only,
    })
}

pub(crate) fn print_override_shadow_report_text(report: &OverrideShadowReport) {
    println!("resource\tstatus\tidentical\toverride_sha1\tbif_sha1");
    for shadow in &report.shadows {
        println!(
            "{}\tshadow\t{}\t{}\t{}",
            shadow.resource, shadow.identical, shadow.override_sha1, shadow.bif_sha1
        );
    }
    for resource in &report.override_only {
        println!("{resource}\toverride_only\t\t\t");
    }
    println!(
        "counts\toverride_total={}\tshadowing_bif={}\toverride_only={}",
        report.counts.override_total, report.counts.shadowing_bif, report.counts.override_only
    );
}

pub(crate) fn override_shadow_report_json(report: &OverrideShadowReport) -> serde_json::Value {
    serde_json::json!({
        "shadows": report.shadows.iter().map(|shadow| {
            serde_json::json!({
                "resource": shadow.resource,
                "in_override": true,
                "in_bif": true,
                "override_sha1": shadow.override_sha1,
                "bif_sha1": shadow.bif_sha1,
                "identical": shadow.identical,
            })
        }).collect::<Vec<_>>(),
        "override_only": report.override_only,
        "counts": {
            "override_total": report.counts.override_total,
            "shadowing_bif": report.counts.shadowing_bif,
            "override_only": report.counts.override_only,
        },
    })
}

#[derive(Debug, Clone)]
pub(crate) enum OverrideReferenceReport {
    Single(OverrideReferenceSingle),
    Set(OverrideReferenceSet),
}

#[derive(Debug, Clone)]
pub(crate) struct OverrideReferenceSingle {
    resource: String,
    status: OverrideReferenceStatus,
    override_sha1: String,
    reference_sha1: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OverrideReferenceSet {
    added: Vec<OverrideReferenceHashEntry>,
    removed: Vec<OverrideReferenceHashEntry>,
    changed: Vec<OverrideReferenceChangedEntry>,
    counts: OverrideReferenceCounts,
}

#[derive(Debug, Clone)]
struct OverrideReferenceHashEntry {
    resource: String,
    sha1: String,
}

#[derive(Debug, Clone)]
struct OverrideReferenceChangedEntry {
    resource: String,
    override_sha1: String,
    reference_sha1: String,
}

#[derive(Debug, Clone)]
struct OverrideReferenceCounts {
    override_total: usize,
    reference_total: usize,
    added: usize,
    removed: usize,
    changed: usize,
    unchanged: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverrideReferenceStatus {
    Match,
    Differ,
}

impl OverrideReferenceStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::Differ => "differ",
        }
    }
}

pub(crate) fn build_override_reference_report(
    locator: &ResourceLocator,
    resource_type: Option<String>,
    resource: Option<&str>,
    against: &Path,
) -> Result<OverrideReferenceReport, Box<dyn std::error::Error>> {
    if against.is_file() {
        let resource = resource
            .ok_or_else(|| "--resource is required when --against points to a file".to_string())?;
        return build_single_override_reference_report(locator, resource, against)
            .map(OverrideReferenceReport::Single);
    }

    if !against.is_dir() {
        return Err(format!("--against path does not exist: {}", against.display()).into());
    }

    build_set_override_reference_report(locator, resource_type, resource, against)
        .map(OverrideReferenceReport::Set)
}

fn build_single_override_reference_report(
    locator: &ResourceLocator,
    resource: &str,
    against: &Path,
) -> Result<OverrideReferenceSingle, Box<dyn std::error::Error>> {
    let resource_name = ResourceName::parse(resource)?;
    let reader = ResourceReader;
    let override_bytes =
        reader.read_with_source(locator, &resource_name, ResourceSource::Override)?;
    let reference_bytes = fs::read(against)?;
    let override_sha1 = sha1_hex(&override_bytes.bytes);
    let reference_sha1 = sha1_hex(&reference_bytes);

    Ok(OverrideReferenceSingle {
        resource: resource_name.file_name(),
        status: if override_sha1 == reference_sha1 {
            OverrideReferenceStatus::Match
        } else {
            OverrideReferenceStatus::Differ
        },
        override_sha1,
        reference_sha1,
    })
}

fn build_set_override_reference_report(
    locator: &ResourceLocator,
    resource_type: Option<String>,
    resource: Option<&str>,
    against: &Path,
) -> Result<OverrideReferenceSet, Box<dyn std::error::Error>> {
    let resource_filter = resource
        .map(ResourceName::parse)
        .transpose()?
        .map(|resource| resource.file_name());
    let overrides = locator.list(ResourceListOptions {
        resource_type: resource_type.clone(),
        name_glob: resource_filter.clone(),
        source: Some(ResourceSource::Override),
    })?;

    let override_hashes = override_hashes(locator, &overrides)?;
    let reference_hashes = reference_hashes(against, resource_type.as_deref(), resource_filter)?;
    let all_resources = override_hashes
        .keys()
        .chain(reference_hashes.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    let mut unchanged = 0usize;

    for resource in all_resources {
        match (
            override_hashes.get(&resource),
            reference_hashes.get(&resource),
        ) {
            (Some(override_sha1), Some(reference_sha1)) if override_sha1 == reference_sha1 => {
                unchanged += 1;
            }
            (Some(override_sha1), Some(reference_sha1)) => {
                changed.push(OverrideReferenceChangedEntry {
                    resource,
                    override_sha1: override_sha1.clone(),
                    reference_sha1: reference_sha1.clone(),
                });
            }
            (Some(override_sha1), None) => added.push(OverrideReferenceHashEntry {
                resource,
                sha1: override_sha1.clone(),
            }),
            (None, Some(reference_sha1)) => removed.push(OverrideReferenceHashEntry {
                resource,
                sha1: reference_sha1.clone(),
            }),
            (None, None) => {}
        }
    }

    Ok(OverrideReferenceSet {
        counts: OverrideReferenceCounts {
            override_total: override_hashes.len(),
            reference_total: reference_hashes.len(),
            added: added.len(),
            removed: removed.len(),
            changed: changed.len(),
            unchanged,
        },
        added,
        removed,
        changed,
    })
}

fn override_hashes(
    locator: &ResourceLocator,
    resources: &[ListedResource],
) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let reader = ResourceReader;
    let mut hashes = BTreeMap::new();
    for resource in resources {
        let resource_name = ResourceName::parse(&resource.resource_name)?;
        let bytes = reader.read_with_source(locator, &resource_name, ResourceSource::Override)?;
        hashes.insert(resource_name.file_name(), sha1_hex(&bytes.bytes));
    }
    Ok(hashes)
}

fn reference_hashes(
    against: &Path,
    resource_type: Option<&str>,
    resource_filter: Option<String>,
) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let mut hashes = BTreeMap::new();
    for entry in fs::read_dir(against)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Ok(resource_name) = ResourceName::parse(file_name) else {
            continue;
        };
        if let Some(resource_type) = resource_type
            && !resource_name
                .extension()
                .eq_ignore_ascii_case(resource_type)
        {
            continue;
        }
        if let Some(resource_filter) = resource_filter.as_deref()
            && !resource_name
                .file_name()
                .eq_ignore_ascii_case(resource_filter)
        {
            continue;
        }

        hashes.insert(resource_name.file_name(), sha1_hex(&fs::read(path)?));
    }
    Ok(hashes)
}

pub(crate) fn print_override_reference_report_text(report: &OverrideReferenceReport) {
    match report {
        OverrideReferenceReport::Single(single) => {
            println!(
                "resource\tstatus\toverride_sha1\treference_sha1\n{}\t{}\t{}\t{}",
                single.resource,
                single.status.as_str(),
                single.override_sha1,
                single.reference_sha1
            );
        }
        OverrideReferenceReport::Set(set) => {
            println!("resource\tstatus\toverride_sha1\treference_sha1");
            for entry in &set.added {
                println!("{}\tadded\t{}\t", entry.resource, entry.sha1);
            }
            for entry in &set.removed {
                println!("{}\tremoved\t\t{}", entry.resource, entry.sha1);
            }
            for entry in &set.changed {
                println!(
                    "{}\tchanged\t{}\t{}",
                    entry.resource, entry.override_sha1, entry.reference_sha1
                );
            }
            println!(
                "counts\toverride_total={}\treference_total={}\tadded={}\tremoved={}\tchanged={}\tunchanged={}",
                set.counts.override_total,
                set.counts.reference_total,
                set.counts.added,
                set.counts.removed,
                set.counts.changed,
                set.counts.unchanged
            );
        }
    }
}

pub(crate) fn override_reference_report_json(
    report: &OverrideReferenceReport,
) -> serde_json::Value {
    match report {
        OverrideReferenceReport::Single(single) => serde_json::json!({
            "resource": single.resource,
            "status": single.status.as_str(),
            "override_sha1": single.override_sha1,
            "reference_sha1": single.reference_sha1,
        }),
        OverrideReferenceReport::Set(set) => serde_json::json!({
            "added": set.added.iter().map(|entry| {
                serde_json::json!({
                    "resource": entry.resource,
                    "override_sha1": entry.sha1,
                })
            }).collect::<Vec<_>>(),
            "removed": set.removed.iter().map(|entry| {
                serde_json::json!({
                    "resource": entry.resource,
                    "reference_sha1": entry.sha1,
                })
            }).collect::<Vec<_>>(),
            "changed": set.changed.iter().map(|entry| {
                serde_json::json!({
                    "resource": entry.resource,
                    "override_sha1": entry.override_sha1,
                    "reference_sha1": entry.reference_sha1,
                })
            }).collect::<Vec<_>>(),
            "counts": {
                "override_total": set.counts.override_total,
                "reference_total": set.counts.reference_total,
                "added": set.counts.added,
                "removed": set.counts.removed,
                "changed": set.counts.changed,
                "unchanged": set.counts.unchanged,
            },
        }),
    }
}

fn sha1_hex(bytes: &[u8]) -> String {
    sha1(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn sha1(bytes: &[u8]) -> [u8; 20] {
    let mut h0 = 0x6745_2301u32;
    let mut h1 = 0xEFCD_AB89u32;
    let mut h2 = 0x98BA_DCFEu32;
    let mut h3 = 0x1032_5476u32;
    let mut h4 = 0xC3D2_E1F0u32;

    let bit_len = (bytes.len() as u64) * 8;
    let mut padded = bytes.to_vec();
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut words = [0u32; 80];
        for (index, word) in words.iter_mut().take(16).enumerate() {
            let start = index * 4;
            *word = u32::from_be_bytes([
                chunk[start],
                chunk[start + 1],
                chunk[start + 2],
                chunk[start + 3],
            ]);
        }
        for index in 16..80 {
            words[index] =
                (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16])
                    .rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for (index, word) in words.iter().enumerate() {
            let (function, constant) = match index {
                0..=19 => ((b & c) | ((!b) & d), 0x5A82_7999),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(function)
                .wrapping_add(e)
                .wrapping_add(constant)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut digest = [0u8; 20];
    digest[0..4].copy_from_slice(&h0.to_be_bytes());
    digest[4..8].copy_from_slice(&h1.to_be_bytes());
    digest[8..12].copy_from_slice(&h2.to_be_bytes());
    digest[12..16].copy_from_slice(&h3.to_be_bytes());
    digest[16..20].copy_from_slice(&h4.to_be_bytes());
    digest
}

#[cfg(test)]
mod tests {
    use super::sha1_hex;

    #[test]
    fn sha1_hex_matches_known_vectors() {
        assert_eq!(sha1_hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(
            sha1_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
        );
        assert_eq!(
            sha1_hex(&[b'a'; 64]),
            "0098ba824b5c16427bd7a1122a5a442a25ec644d"
        );
        assert_eq!(
            sha1_hex(&[b'a'; 1000]),
            "291e9a6c66994949b57ba5e650361e98fc36b1ba"
        );
    }
}
