use ie_core::{ResRef, ResourceName};
use ie_formats::{MemberSelector, SlotChoice};
use ie_io::{GameInstallation, ResourceLocator};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn parse_member_selector(
    value: Option<&str>,
) -> Result<MemberSelector, Box<dyn std::error::Error>> {
    let Some(value) = value else {
        return Ok(MemberSelector::Index(0));
    };
    if let Ok(index) = value.parse::<usize>() {
        return Ok(MemberSelector::Index(index));
    }
    Ok(MemberSelector::CreResRef(
        ResRef::new(value)?.as_str().to_string(),
    ))
}

pub(crate) fn parse_slot_choice(value: &str) -> Result<SlotChoice, Box<dyn std::error::Error>> {
    if value.eq_ignore_ascii_case("auto") {
        return Ok(SlotChoice::AutoInventory);
    }
    Ok(SlotChoice::Index(value.parse::<usize>()?))
}

pub(crate) fn parse_item_flags(value: &str) -> Result<u32, Box<dyn std::error::Error>> {
    if value.eq_ignore_ascii_case("identified") {
        return Ok(0x0000_0001);
    }
    if value.eq_ignore_ascii_case("none") {
        return Ok(0);
    }
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return Ok(u32::from_str_radix(hex, 16)?);
    }
    Ok(value.parse::<u32>()?)
}

pub(crate) fn warn_if_item_missing(installation: &GameInstallation, item: &ResRef) {
    let Ok(locator) = ResourceLocator::new(installation.clone()) else {
        return;
    };
    let Ok(resource) = ResourceName::parse(format!("{}.ITM", item.as_str())) else {
        return;
    };
    if locator.locate(&resource).is_err() {
        eprintln!(
            "warning: item {}.ITM was not found in the installation; writing resref anyway",
            item.as_str()
        );
    }
}

pub(crate) fn copy_save_folder(
    source: &Path,
    target: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if target.exists() {
        return Err(format!("--output already exists: {}", target.display()).into());
    }
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_save_folder(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

pub(crate) fn resolve_child_file_case_insensitive(
    parent: &Path,
    file_name: &str,
) -> Option<PathBuf> {
    let entries = fs::read_dir(parent).ok()?;
    for entry in entries.filter_map(Result::ok) {
        if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(file_name)
            && entry.path().is_file()
        {
            return Some(entry.path());
        }
    }
    None
}
