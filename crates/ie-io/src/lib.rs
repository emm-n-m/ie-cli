mod biff;
mod bytes;
mod ids;

use crate::bytes::{read_u16_le, read_u32_le};
use ie_core::{
    ResourceBytes, ResourceMetadata, ResourceName, ResourceType, SourceKind, StrRef, StrRefResolver,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub use ids::{FileBackedIdsResolver, parse_ids, parse_ids_text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameInstallation {
    pub root: PathBuf,
    pub chitin_key: PathBuf,
    pub language: Option<String>,
    pub language_dir: Option<PathBuf>,
    pub dialog_tlk: Option<PathBuf>,
    pub override_dirs: Vec<PathBuf>,
}

impl GameInstallation {
    pub fn discover(root: impl AsRef<Path>) -> Result<Self, IoError> {
        let root = root.as_ref().to_path_buf();
        let chitin_key = root.join("chitin.key");

        if !root.is_dir() {
            return Err(IoError::InvalidGamePath(root));
        }

        if !chitin_key.is_file() {
            return Err(IoError::MissingChitinKey(chitin_key));
        }

        let (language, language_dir, dialog_tlk) = discover_dialog_tlk(&root)?;
        let override_dirs = discover_override_dirs(&root, language_dir.as_deref());

        Ok(Self {
            root,
            chitin_key,
            language,
            language_dir,
            dialog_tlk,
            override_dirs,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLocator {
    pub installation: GameInstallation,
    key_file: KeyFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceSource {
    Auto,
    Override,
    Bif,
}

impl ResourceSource {
    fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Override => "override",
            Self::Bif => "bif",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedResource {
    pub resource_name: String,
    pub resref: String,
    pub extension: String,
    pub source_kind: SourceKind,
    pub source_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveDirectory {
    pub kind: SaveDirectoryKind,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveDirectoryKind {
    Save,
    MultiplayerSave,
}

impl SaveDirectoryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Save => "save",
            Self::MultiplayerSave => "mpsave",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedSave {
    pub folder_name: String,
    pub display_name: String,
    pub path: PathBuf,
    pub save_dir_kind: SaveDirectoryKind,
    pub has_gam: bool,
    pub has_sav: bool,
    pub portraits: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceListOptions {
    pub resource_type: Option<String>,
    pub name_glob: Option<String>,
    pub source: Option<ResourceSource>,
}

impl ResourceLocator {
    pub fn new(installation: GameInstallation) -> Result<Self, IoError> {
        let key_file = KeyFile::parse(&installation)?;
        Ok(Self {
            installation,
            key_file,
        })
    }

    pub fn locate(&self, resource: &ResourceName) -> Result<LocatedResource, IoError> {
        self.locate_with_source(resource, ResourceSource::Auto)
    }

    pub fn locate_with_source(
        &self,
        resource: &ResourceName,
        source: ResourceSource,
    ) -> Result<LocatedResource, IoError> {
        if matches!(source, ResourceSource::Auto | ResourceSource::Override)
            && let Some(located) = self.locate_override(resource)
        {
            return Ok(located);
        }

        if matches!(source, ResourceSource::Auto | ResourceSource::Bif) {
            return self.locate_bif(resource);
        }

        Err(IoError::ResourceNotFoundInSource {
            resource: resource.file_name(),
            source_name: source.label().to_string(),
        })
    }

    pub fn list(&self, options: ResourceListOptions) -> Result<Vec<ListedResource>, IoError> {
        let source = options.source.unwrap_or(ResourceSource::Auto);
        let mut entries = HashMap::new();

        if matches!(source, ResourceSource::Auto | ResourceSource::Override) {
            for entry in self.override_entries()? {
                entries.entry(entry.resource_name.clone()).or_insert(entry);
            }
        }

        if matches!(source, ResourceSource::Auto | ResourceSource::Bif) {
            for entry in self.bif_entries()? {
                if matches!(source, ResourceSource::Auto) {
                    entries.entry(entry.resource_name.clone()).or_insert(entry);
                } else {
                    entries.insert(entry.resource_name.clone(), entry);
                }
            }
        }

        let mut entries = entries
            .into_values()
            .filter(|entry| resource_matches_filters(entry, &options))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.resource_name.cmp(&right.resource_name));
        Ok(entries)
    }

    fn locate_override(&self, resource: &ResourceName) -> Option<LocatedResource> {
        let file_name = resource.file_name();

        for override_dir in &self.installation.override_dirs {
            if let Some((path, resource_name)) = resolve_override_path(override_dir, &file_name) {
                return Some(LocatedResource {
                    metadata: ResourceMetadata {
                        source_path: path,
                        source_kind: SourceKind::Override,
                        resource_type: resource.resource_type(),
                        resource_name,
                    },
                    locator: None,
                });
            }
        }
        None
    }

    fn locate_bif(&self, resource: &ResourceName) -> Result<LocatedResource, IoError> {
        let file_name = resource.file_name();
        let key = file_name.to_ascii_uppercase();
        let entry = self
            .key_file
            .resource(resource)
            .ok_or_else(|| IoError::ResourceNotFound(file_name.clone()))?;

        let biff = self
            .key_file
            .biffs
            .get(entry.biff_index as usize)
            .ok_or_else(|| {
                IoError::InvalidKey(format!("invalid BIFF index {}", entry.biff_index))
            })?;

        let source_path = biff
            .actual_path
            .clone()
            .unwrap_or_else(|| self.installation.root.join(&biff.relative_path));

        Ok(LocatedResource {
            metadata: ResourceMetadata {
                source_path,
                source_kind: SourceKind::Bif,
                resource_type: entry.resource_type,
                resource_name: key,
            },
            locator: Some(entry.locator),
        })
    }

    fn override_entries(&self) -> Result<Vec<ListedResource>, IoError> {
        let mut entries = Vec::new();

        for override_dir in &self.installation.override_dirs {
            let mut dir_entries = fs::read_dir(override_dir)
                .map_err(IoError::FileIo)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_file())
                .collect::<Vec<_>>();
            dir_entries
                .sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());

            for entry in dir_entries {
                let file_name = entry.file_name().to_string_lossy().to_string();
                let Ok(resource) = ResourceName::parse(&file_name) else {
                    continue;
                };

                entries.push(ListedResource {
                    resource_name: resource.file_name(),
                    resref: resource.resref().as_str().to_string(),
                    extension: resource.extension().to_string(),
                    source_kind: SourceKind::Override,
                    source_path: entry.path(),
                });
            }
        }

        Ok(entries)
    }

    fn bif_entries(&self) -> Result<Vec<ListedResource>, IoError> {
        let mut entries = Vec::with_capacity(self.key_file.resources.len());

        for entry in &self.key_file.resources {
            let biff = self
                .key_file
                .biffs
                .get(entry.biff_index as usize)
                .ok_or_else(|| {
                    IoError::InvalidKey(format!("invalid BIFF index {}", entry.biff_index))
                })?;
            let source_path = biff
                .actual_path
                .clone()
                .unwrap_or_else(|| self.installation.root.join(&biff.relative_path));

            entries.push(ListedResource {
                resource_name: entry.resource_name.clone(),
                resref: entry.resref.clone(),
                extension: entry.extension.clone(),
                source_kind: SourceKind::Bif,
                source_path,
            });
        }

        Ok(entries)
    }

    pub fn key_file(&self) -> &KeyFile {
        &self.key_file
    }
}

pub fn discover_save_directories(
    installation: &GameInstallation,
    explicit_saves_dir: Option<&Path>,
) -> Vec<SaveDirectory> {
    let mut dirs = Vec::new();

    if let Some(path) = explicit_saves_dir {
        push_save_dir_candidate(&mut dirs, path.to_path_buf(), SaveDirectoryKind::Save);
        push_save_dir_candidate(
            &mut dirs,
            path.join("mpsave"),
            SaveDirectoryKind::MultiplayerSave,
        );
        return dirs;
    }

    push_save_dir_candidate(
        &mut dirs,
        installation.root.join("save"),
        SaveDirectoryKind::Save,
    );
    push_save_dir_candidate(
        &mut dirs,
        installation.root.join("mpsave"),
        SaveDirectoryKind::MultiplayerSave,
    );

    for base in user_document_dirs() {
        for game_folder in candidate_documents_game_folders(&installation.root) {
            push_save_dir_candidate(
                &mut dirs,
                base.join(&game_folder).join("save"),
                SaveDirectoryKind::Save,
            );
            push_save_dir_candidate(
                &mut dirs,
                base.join(&game_folder).join("mpsave"),
                SaveDirectoryKind::MultiplayerSave,
            );
        }
    }

    dirs.sort_by(|left, right| {
        (
            left.path.to_string_lossy().to_ascii_lowercase(),
            left.kind.as_str(),
        )
            .cmp(&(
                right.path.to_string_lossy().to_ascii_lowercase(),
                right.kind.as_str(),
            ))
    });
    dirs.dedup_by(|left, right| left.path == right.path && left.kind == right.kind);
    dirs
}

pub fn list_saves(
    installation: &GameInstallation,
    explicit_saves_dir: Option<&Path>,
) -> Result<Vec<ListedSave>, IoError> {
    let mut saves = Vec::new();
    for save_dir in discover_save_directories(installation, explicit_saves_dir) {
        let mut entries = fs::read_dir(&save_dir.path)
            .map_err(IoError::FileIo)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());

        for entry in entries {
            let path = entry.path();
            let folder_name = entry.file_name().to_string_lossy().to_string();
            let has_gam = resolve_child_file_case_insensitive(&path, "BALDUR.gam").is_some();
            let has_sav = resolve_child_file_case_insensitive(&path, "BALDUR.SAV").is_some();
            if !has_gam && !has_sav {
                continue;
            }

            saves.push(ListedSave {
                display_name: save_display_name(&folder_name),
                folder_name,
                path: path.clone(),
                save_dir_kind: save_dir.kind,
                has_gam,
                has_sav,
                portraits: list_save_portraits(&path)?,
            });
        }
    }

    saves.sort_by(|left, right| {
        (
            left.save_dir_kind.as_str(),
            left.folder_name.to_ascii_lowercase(),
            left.path.to_string_lossy().to_ascii_lowercase(),
        )
            .cmp(&(
                right.save_dir_kind.as_str(),
                right.folder_name.to_ascii_lowercase(),
                right.path.to_string_lossy().to_ascii_lowercase(),
            ))
    });
    Ok(saves)
}

pub fn resolve_save_folder(
    installation: &GameInstallation,
    explicit_saves_dir: Option<&Path>,
    selector: &str,
) -> Result<ListedSave, IoError> {
    let selector_path = Path::new(selector);
    if selector_path.is_dir() {
        return listed_save_from_path(selector_path, SaveDirectoryKind::Save);
    }

    let selector_lower = selector.to_ascii_lowercase();
    let saves = list_saves(installation, explicit_saves_dir)?;
    saves
        .into_iter()
        .find(|save| {
            save.folder_name.eq_ignore_ascii_case(selector)
                || save.display_name.eq_ignore_ascii_case(selector)
                || save.path.to_string_lossy().to_ascii_lowercase() == selector_lower
        })
        .ok_or_else(|| IoError::SaveNotFound(selector.to_string()))
}

pub fn read_save_member(save: &ListedSave, file_name: &str) -> Result<Vec<u8>, IoError> {
    let path = resolve_child_file_case_insensitive(&save.path, file_name).ok_or_else(|| {
        IoError::SaveMemberNotFound {
            save: save.path.clone(),
            member: file_name.to_string(),
        }
    })?;
    fs::read(path).map_err(IoError::FileIo)
}

#[derive(Debug, Default)]
pub struct ResourceReader;

impl ResourceReader {
    pub fn read(
        &self,
        locator: &ResourceLocator,
        resource: &ResourceName,
    ) -> Result<ResourceBytes, IoError> {
        self.read_with_source(locator, resource, ResourceSource::Auto)
    }

    pub fn read_with_source(
        &self,
        locator: &ResourceLocator,
        resource: &ResourceName,
        source: ResourceSource,
    ) -> Result<ResourceBytes, IoError> {
        let located = locator.locate_with_source(resource, source)?;

        match located.metadata.source_kind {
            SourceKind::Override | SourceKind::LooseFile => {
                let bytes = fs::read(&located.metadata.source_path).map_err(IoError::FileIo)?;
                Ok(ResourceBytes {
                    metadata: located.metadata,
                    bytes,
                })
            }
            SourceKind::Bif => {
                let locator = located.locator.ok_or_else(|| {
                    IoError::InvalidBiff("missing BIFF locator for archived resource".to_string())
                })?;
                let bytes = biff::read_resource(&located.metadata.source_path, locator)?;
                Ok(ResourceBytes {
                    metadata: located.metadata,
                    bytes,
                })
            }
        }
    }
}

fn resource_matches_filters(entry: &ListedResource, options: &ResourceListOptions) -> bool {
    if let Some(resource_type) = options.resource_type.as_deref()
        && !entry.extension.eq_ignore_ascii_case(resource_type)
    {
        return false;
    }

    if let Some(pattern) = options.name_glob.as_deref() {
        let candidate = if pattern.contains('.') {
            entry.resource_name.as_str()
        } else {
            entry.resref.as_str()
        };

        if !matches_glob(candidate, pattern) {
            return false;
        }
    }

    true
}

fn matches_glob(candidate: &str, pattern: &str) -> bool {
    let candidate = candidate.as_bytes();
    let pattern = pattern.as_bytes();
    let (mut candidate_index, mut pattern_index) = (0usize, 0usize);
    let (mut star_index, mut candidate_after_star) = (None, 0usize);

    while candidate_index < candidate.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == b'?'
                || pattern[pattern_index].eq_ignore_ascii_case(&candidate[candidate_index]))
        {
            candidate_index += 1;
            pattern_index += 1;
            continue;
        }

        if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
            star_index = Some(pattern_index);
            pattern_index += 1;
            candidate_after_star = candidate_index;
            continue;
        }

        if let Some(index) = star_index {
            pattern_index = index + 1;
            candidate_after_star += 1;
            candidate_index = candidate_after_star;
            continue;
        }

        return false;
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }

    pattern_index == pattern.len()
}

fn resolve_override_path(override_dir: &Path, file_name: &str) -> Option<(PathBuf, String)> {
    let direct_path = override_dir.join(file_name);
    if direct_path.is_file() {
        return Some((direct_path, file_name.to_string()));
    }

    let mut matches = fs::read_dir(override_dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| {
            let candidate = entry.file_name().to_string_lossy().to_string();
            candidate
                .eq_ignore_ascii_case(file_name)
                .then(|| (entry.path(), candidate))
        })
        .collect::<Vec<_>>();
    matches.sort_by_key(|(_, candidate)| candidate.to_ascii_lowercase());
    matches.into_iter().next()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedResource {
    pub metadata: ResourceMetadata,
    pub locator: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTlkEntry {
    pub strref: u32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlkResolver {
    bytes: Vec<u8>,
    entry_count: usize,
    strings_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlkAppendResult {
    pub strref: u32,
    pub output_path: PathBuf,
    pub bytes_written: usize,
}

impl TlkResolver {
    pub fn new(installation: &GameInstallation) -> Result<Self, IoError> {
        let path = installation
            .dialog_tlk
            .clone()
            .ok_or_else(|| IoError::DialogTlkNotFound(installation.root.clone()))?;
        let bytes = fs::read(&path).map_err(IoError::FileIo)?;
        let header = parse_tlk_header(&bytes)?;

        Ok(Self {
            bytes,
            entry_count: header.entry_count,
            strings_offset: header.strings_offset,
        })
    }

    pub fn resolve(&self, strref: u32) -> Result<ResolvedTlkEntry, IoError> {
        let strref = strref as usize;

        if strref >= self.entry_count {
            return Err(IoError::StrRefOutOfRange {
                requested: strref as u32,
                entry_count: self.entry_count as u32,
            });
        }

        let entry_offset = 18usize
            .checked_add(
                strref
                    .checked_mul(26)
                    .ok_or_else(|| IoError::InvalidTlk("TLK entry offset overflow".to_string()))?,
            )
            .ok_or_else(|| IoError::InvalidTlk("TLK entry offset overflow".to_string()))?;
        let entry_end = entry_offset
            .checked_add(26)
            .ok_or_else(|| IoError::InvalidTlk("TLK entry offset overflow".to_string()))?;
        if entry_end > self.bytes.len() {
            return Err(IoError::InvalidTlk("truncated TLK entry table".to_string()));
        }

        let relative_text_offset = read_u32_le(&self.bytes, entry_offset + 18)? as usize;
        let text_length = read_u32_le(&self.bytes, entry_offset + 22)? as usize;
        let text_start = self
            .strings_offset
            .checked_add(relative_text_offset)
            .ok_or_else(|| IoError::InvalidTlk("TLK string offset overflow".to_string()))?;
        let text_end = text_start
            .checked_add(text_length)
            .ok_or_else(|| IoError::InvalidTlk("TLK string offset overflow".to_string()))?;

        if text_end > self.bytes.len() {
            return Err(IoError::InvalidTlk(
                "TLK string points outside the file".to_string(),
            ));
        }

        let text = String::from_utf8_lossy(&self.bytes[text_start..text_end]).to_string();

        Ok(ResolvedTlkEntry {
            strref: strref as u32,
            text,
        })
    }
}

impl StrRefResolver for TlkResolver {
    fn resolve_strref(&self, strref: StrRef) -> Option<String> {
        self.resolve(strref.0).ok().map(|entry| entry.text)
    }
}

pub fn append_tlk_string(
    input_path: impl AsRef<Path>,
    text: &str,
    output_path: impl AsRef<Path>,
) -> Result<TlkAppendResult, IoError> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    let bytes = fs::read(input_path).map_err(IoError::FileIo)?;
    let header = parse_tlk_header(&bytes)?;
    validate_tlk_entries(&bytes, header)?;

    if header.entry_count > u32::MAX as usize - 1 {
        return Err(IoError::InvalidTlk(
            "TLK entry count cannot be incremented".to_string(),
        ));
    }

    let new_strings_offset = header
        .strings_offset
        .checked_add(26)
        .ok_or_else(|| IoError::InvalidTlk("TLK strings offset overflow".to_string()))?;
    if new_strings_offset > u32::MAX as usize {
        return Err(IoError::InvalidTlk(
            "TLK strings offset exceeds u32 range".to_string(),
        ));
    }

    let existing_string_bytes = bytes.len() - header.strings_offset;
    if existing_string_bytes > u32::MAX as usize {
        return Err(IoError::InvalidTlk(
            "TLK string block exceeds u32 range".to_string(),
        ));
    }

    let text_bytes = text.as_bytes();
    if text_bytes.len() > u32::MAX as usize {
        return Err(IoError::InvalidTlk(
            "appended TLK string exceeds u32 range".to_string(),
        ));
    }

    let mut out = Vec::with_capacity(bytes.len() + 26 + text_bytes.len());
    out.extend_from_slice(&bytes[0..10]);
    out.extend_from_slice(&((header.entry_count as u32) + 1).to_le_bytes());
    out.extend_from_slice(&(new_strings_offset as u32).to_le_bytes());
    out.extend_from_slice(&bytes[18..header.table_end]);
    out.extend_from_slice(&build_tlk_text_entry(
        existing_string_bytes as u32,
        text_bytes.len() as u32,
    ));
    out.extend_from_slice(&bytes[header.table_end..header.strings_offset]);
    out.extend_from_slice(&bytes[header.strings_offset..]);
    out.extend_from_slice(text_bytes);

    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(IoError::FileIo)?;
    }
    fs::write(output_path, &out).map_err(IoError::FileIo)?;

    Ok(TlkAppendResult {
        strref: header.entry_count as u32,
        output_path: output_path.to_path_buf(),
        bytes_written: out.len(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TlkHeader {
    entry_count: usize,
    strings_offset: usize,
    table_end: usize,
}

fn parse_tlk_header(bytes: &[u8]) -> Result<TlkHeader, IoError> {
    if bytes.len() < 18 {
        return Err(IoError::InvalidTlk("TLK file is too small".to_string()));
    }

    if &bytes[0..8] != b"TLK V1  " {
        return Err(IoError::InvalidTlk("invalid TLK signature".to_string()));
    }

    let entry_count = read_u32_le(bytes, 10)? as usize;
    let strings_offset = read_u32_le(bytes, 14)? as usize;
    let table_end =
        18usize
            .checked_add(entry_count.checked_mul(26).ok_or_else(|| {
                IoError::InvalidTlk("TLK entry table offset overflow".to_string())
            })?)
            .ok_or_else(|| IoError::InvalidTlk("TLK entry table offset overflow".to_string()))?;

    if table_end > bytes.len() || strings_offset > bytes.len() {
        return Err(IoError::InvalidTlk("truncated TLK entry table".to_string()));
    }

    if strings_offset < table_end {
        return Err(IoError::InvalidTlk(
            "TLK string data overlaps entry table".to_string(),
        ));
    }

    Ok(TlkHeader {
        entry_count,
        strings_offset,
        table_end,
    })
}

fn build_tlk_text_entry(relative_text_offset: u32, text_length: u32) -> [u8; 26] {
    let mut entry = [0u8; 26];
    entry[0..2].copy_from_slice(&1u16.to_le_bytes());
    entry[18..22].copy_from_slice(&relative_text_offset.to_le_bytes());
    entry[22..26].copy_from_slice(&text_length.to_le_bytes());
    entry
}

fn validate_tlk_entries(bytes: &[u8], header: TlkHeader) -> Result<(), IoError> {
    for index in 0..header.entry_count {
        let entry_offset = 18usize
            .checked_add(
                index
                    .checked_mul(26)
                    .ok_or_else(|| IoError::InvalidTlk("TLK entry offset overflow".to_string()))?,
            )
            .ok_or_else(|| IoError::InvalidTlk("TLK entry offset overflow".to_string()))?;
        let relative_text_offset = read_u32_le(bytes, entry_offset + 18)? as usize;
        let text_length = read_u32_le(bytes, entry_offset + 22)? as usize;
        let text_start = header
            .strings_offset
            .checked_add(relative_text_offset)
            .ok_or_else(|| IoError::InvalidTlk("TLK string offset overflow".to_string()))?;
        let text_end = text_start
            .checked_add(text_length)
            .ok_or_else(|| IoError::InvalidTlk("TLK string offset overflow".to_string()))?;

        if text_end > bytes.len() {
            return Err(IoError::InvalidTlk(format!(
                "TLK entry {index} string points outside the file"
            )));
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyFile {
    pub header: KeyHeader,
    pub biffs: Vec<KeyBiffEntry>,
    pub resources: Vec<KeyResourceEntry>,
    resource_lookup: HashMap<String, usize>,
}

impl KeyFile {
    pub fn parse(installation: &GameInstallation) -> Result<Self, IoError> {
        let bytes = fs::read(&installation.chitin_key).map_err(IoError::FileIo)?;

        if bytes.len() < 24 {
            return Err(IoError::InvalidKey("KEY file is too small".to_string()));
        }

        if &bytes[0..4] != b"KEY " || &bytes[4..8] != b"V1  " {
            return Err(IoError::InvalidKey("invalid KEY signature".to_string()));
        }

        let bif_count = read_u32_le(&bytes, 8)? as usize;
        let resource_count = read_u32_le(&bytes, 12)? as usize;
        let bif_offset = read_u32_le(&bytes, 16)? as usize;
        let resource_offset = read_u32_le(&bytes, 20)? as usize;
        let header = KeyHeader {
            signature: "KEY ".to_string(),
            version: "V1  ".to_string(),
            bif_count: bif_count as u32,
            resource_count: resource_count as u32,
            bif_offset: bif_offset as u32,
            resource_offset: resource_offset as u32,
        };

        let mut biffs = Vec::with_capacity(bif_count);
        for index in 0..bif_count {
            let entry_offset =
                bif_offset
                    .checked_add(index.checked_mul(12).ok_or_else(|| {
                        IoError::InvalidKey("BIFF table offset overflow".to_string())
                    })?)
                    .ok_or_else(|| IoError::InvalidKey("BIFF table offset overflow".to_string()))?;
            let entry_end = entry_offset
                .checked_add(12)
                .ok_or_else(|| IoError::InvalidKey("BIFF table offset overflow".to_string()))?;
            if entry_end > bytes.len() {
                return Err(IoError::InvalidKey("truncated BIFF table".to_string()));
            }

            let file_size = read_u32_le(&bytes, entry_offset)?;
            let string_offset = read_u32_le(&bytes, entry_offset + 4)? as usize;
            let string_length = read_u16_le(&bytes, entry_offset + 8)? as usize;
            let string_end = string_offset
                .checked_add(string_length)
                .ok_or_else(|| IoError::InvalidKey("BIFF filename offset overflow".to_string()))?;

            if string_length == 0 || string_end > bytes.len() {
                return Err(IoError::InvalidKey(
                    "invalid BIFF filename entry".to_string(),
                ));
            }

            let mut relative_path =
                String::from_utf8_lossy(&bytes[string_offset..(string_end - 1)]).to_string();
            if relative_path.starts_with('\\') || relative_path.starts_with('/') {
                relative_path.remove(0);
            }
            relative_path = relative_path.replace(['\\', ':'], "/");

            let actual_path = resolve_biff_path(installation, &relative_path);
            biffs.push(KeyBiffEntry {
                index: index as u32,
                file_size,
                string_offset: string_offset as u32,
                string_length: string_length as u16,
                relative_path,
                actual_path,
            });
        }

        let mut resources = Vec::with_capacity(resource_count);
        let mut resource_lookup = HashMap::with_capacity(resource_count);
        for index in 0..resource_count {
            let entry_offset = resource_offset
                .checked_add(index.checked_mul(14).ok_or_else(|| {
                    IoError::InvalidKey("resource table offset overflow".to_string())
                })?)
                .ok_or_else(|| IoError::InvalidKey("resource table offset overflow".to_string()))?;
            let entry_end = entry_offset
                .checked_add(14)
                .ok_or_else(|| IoError::InvalidKey("resource table offset overflow".to_string()))?;
            if entry_end > bytes.len() {
                return Err(IoError::InvalidKey("truncated resource table".to_string()));
            }

            let resref = read_resref(&bytes[entry_offset..entry_offset + 8]);
            let type_code = read_u16_le(&bytes, entry_offset + 8)?;
            let locator = read_u32_le(&bytes, entry_offset + 10)?;
            let extension =
                extension_from_type(type_code).unwrap_or_else(|| format!("0x{type_code:04X}"));
            let resource_name = format!("{resref}.{extension}");
            let resource_type = ResourceType::from_extension(&extension);

            let entry = KeyResourceEntry {
                index: index as u32,
                resource_name: resource_name.to_ascii_uppercase(),
                resref,
                extension,
                type_code,
                locator,
                biff_index: locator >> 20,
                resource_index: locator & 0x000F_FFFF,
                resource_type,
            };
            resource_lookup.insert(entry.resource_name.clone(), resources.len());
            resources.push(entry);
        }

        Ok(Self {
            header,
            biffs,
            resources,
            resource_lookup,
        })
    }

    pub fn resource(&self, resource: &ResourceName) -> Option<&KeyResourceEntry> {
        let key = resource.file_name().to_ascii_uppercase();
        self.resource_lookup
            .get(&key)
            .and_then(|index| self.resources.get(*index))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyHeader {
    pub signature: String,
    pub version: String,
    pub bif_count: u32,
    pub resource_count: u32,
    pub bif_offset: u32,
    pub resource_offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBiffEntry {
    pub index: u32,
    pub file_size: u32,
    pub string_offset: u32,
    pub string_length: u16,
    pub relative_path: String,
    pub actual_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyResourceEntry {
    pub index: u32,
    pub resource_name: String,
    pub resref: String,
    pub extension: String,
    pub type_code: u16,
    pub locator: u32,
    pub biff_index: u32,
    pub resource_index: u32,
    pub resource_type: ResourceType,
}

// (language_name, language_dir, dialog_tlk_path)
type DialogTlkDiscovery = (Option<String>, Option<PathBuf>, Option<PathBuf>);

fn discover_dialog_tlk(root: &Path) -> Result<DialogTlkDiscovery, IoError> {
    let direct = root.join("dialog.tlk");
    if direct.is_file() {
        return Ok((None, None, Some(direct)));
    }

    let lang_root = root.join("lang");
    if !lang_root.is_dir() {
        return Ok((None, None, None));
    }

    let mut languages = fs::read_dir(&lang_root)
        .map_err(IoError::FileIo)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .collect::<Vec<_>>();

    languages.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());

    if let Some(entry) = languages.iter().find(|entry| {
        entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case("en_US")
    }) {
        let language = entry.file_name().to_string_lossy().to_string();
        let language_dir = entry.path();
        let dialog_tlk = language_dir.join("dialog.tlk");
        if dialog_tlk.is_file() {
            return Ok((Some(language), Some(language_dir), Some(dialog_tlk)));
        }
    }

    for entry in languages {
        let language = entry.file_name().to_string_lossy().to_string();
        let language_dir = entry.path();
        let dialog_tlk = language_dir.join("dialog.tlk");
        if dialog_tlk.is_file() {
            return Ok((Some(language), Some(language_dir), Some(dialog_tlk)));
        }
    }

    Ok((None, None, None))
}

fn discover_override_dirs(root: &Path, language_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(language_dir) = language_dir
        && let Some(language_override) =
            resolve_child_dir_case_insensitive(language_dir, "override")
    {
        dirs.push(language_override);
    }

    if let Some(root_override) = resolve_child_dir_case_insensitive(root, "override") {
        dirs.push(root_override);
    }

    dirs
}

fn resolve_child_dir_case_insensitive(parent: &Path, child_name: &str) -> Option<PathBuf> {
    let direct = parent.join(child_name);
    if direct.is_dir() {
        return Some(direct);
    }

    let mut matches = fs::read_dir(parent)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let candidate = entry.file_name().to_string_lossy().to_string();
            candidate
                .eq_ignore_ascii_case(child_name)
                .then(|| entry.path())
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.into_iter().next()
}

fn resolve_child_file_case_insensitive(parent: &Path, child_name: &str) -> Option<PathBuf> {
    let direct = parent.join(child_name);
    if direct.is_file() {
        return Some(direct);
    }

    let mut matches = fs::read_dir(parent)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| {
            let candidate = entry.file_name().to_string_lossy().to_string();
            candidate
                .eq_ignore_ascii_case(child_name)
                .then(|| entry.path())
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.into_iter().next()
}

fn push_save_dir_candidate(dirs: &mut Vec<SaveDirectory>, path: PathBuf, kind: SaveDirectoryKind) {
    if path.is_dir() {
        dirs.push(SaveDirectory { kind, path });
    }
}

fn user_document_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for env_name in ["USERPROFILE", "HOME", "OneDrive", "OneDriveConsumer"] {
        if let Some(base) = std::env::var_os(env_name) {
            let base = PathBuf::from(base);
            push_existing_path(&mut dirs, base.join("Documents"));
            push_existing_path(&mut dirs, base.join("My Documents"));
            push_existing_path(&mut dirs, base.join("Έγγραφα"));
        }
    }
    dirs.sort();
    dirs.dedup();
    dirs
}

fn push_existing_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if path.is_dir() {
        paths.push(path);
    }
}

fn candidate_documents_game_folders(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(root_name) = root.file_name().and_then(|name| name.to_str()) {
        names.push(root_name.to_string());
        match root_name.to_ascii_lowercase().as_str() {
            "baldur's gate enhanced edition" | "baldur's gate - enhanced edition" => {
                names.push("Baldur's Gate - Enhanced Edition".to_string());
            }
            "baldur's gate ii enhanced edition" | "baldur's gate ii - enhanced edition" => {
                names.push("Baldur's Gate II - Enhanced Edition".to_string());
            }
            "icewind dale enhanced edition" | "icewind dale - enhanced edition" => {
                names.push("Icewind Dale - Enhanced Edition".to_string());
            }
            "planescape torment enhanced edition" | "planescape torment - enhanced edition" => {
                names.push("Planescape Torment - Enhanced Edition".to_string());
            }
            _ => {}
        }
    }

    names.sort_by_key(|name| name.to_ascii_lowercase());
    names.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    names
}

fn save_display_name(folder_name: &str) -> String {
    match folder_name.split_once('-') {
        Some((prefix, name))
            if !name.trim().is_empty() && prefix.chars().all(|ch| ch.is_ascii_digit()) =>
        {
            name.trim().to_string()
        }
        _ => folder_name.to_string(),
    }
}

fn listed_save_from_path(path: &Path, kind: SaveDirectoryKind) -> Result<ListedSave, IoError> {
    let folder_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| IoError::InvalidSavePath(path.to_path_buf()))?
        .to_string();
    let has_gam = resolve_child_file_case_insensitive(path, "BALDUR.gam").is_some();
    let has_sav = resolve_child_file_case_insensitive(path, "BALDUR.SAV").is_some();
    if !has_gam && !has_sav {
        return Err(IoError::InvalidSavePath(path.to_path_buf()));
    }

    Ok(ListedSave {
        display_name: save_display_name(&folder_name),
        folder_name,
        path: path.to_path_buf(),
        save_dir_kind: kind,
        has_gam,
        has_sav,
        portraits: list_save_portraits(path)?,
    })
}

fn list_save_portraits(path: &Path) -> Result<Vec<PathBuf>, IoError> {
    let mut portraits = fs::read_dir(path)
        .map_err(IoError::FileIo)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter(|entry| {
            let file_name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            file_name.starts_with("portrt") && file_name.ends_with(".bmp")
        })
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    portraits.sort();
    Ok(portraits)
}

fn resolve_biff_path(installation: &GameInstallation, relative_path: &str) -> Option<PathBuf> {
    let candidates = [
        installation.root.join(relative_path),
        installation.root.join(relative_path.replace('/', "\\")),
    ];

    for path in candidates {
        if path.is_file() {
            return Some(path);
        }
    }

    let cbf_relative = replace_extension(relative_path, "cbf");
    let cbf_candidates = [
        installation.root.join(&cbf_relative),
        installation.root.join(cbf_relative.replace('/', "\\")),
    ];

    cbf_candidates.into_iter().find(|path| path.is_file())
}

fn replace_extension(path: &str, new_extension: &str) -> String {
    match path.rsplit_once('.') {
        Some((base, _)) => format!("{base}.{new_extension}"),
        None => path.to_string(),
    }
}

fn read_resref(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end])
        .trim()
        .to_ascii_uppercase()
}

fn extension_from_type(type_code: u16) -> Option<String> {
    let extension = match type_code {
        0x001 => "BMP",
        0x002 => "MVE",
        0x004 => "WAV",
        0x005 => "WFX",
        0x006 => "PLT",
        0x3B8 => "TGA",
        0x3E8 => "BAM",
        0x3E9 => "WED",
        0x3EA => "CHU",
        0x3EB => "TIS",
        0x3EC => "MOS",
        0x3ED => "ITM",
        0x3EE => "SPL",
        0x3EF => "BCS",
        0x3F0 => "IDS",
        0x3F1 => "CRE",
        0x3F2 => "ARE",
        0x3F3 => "DLG",
        0x3F4 => "2DA",
        0x3F5 => "GAM",
        0x3F6 => "STO",
        0x3F7 => "WMP",
        0x3F8 => "EFF",
        0x3F9 => "BS",
        0x3FA => "CHR",
        0x3FB => "VVC",
        0x3FC => "VEF",
        0x3FD => "PRO",
        0x3FE => "BIO",
        0x3FF => "WBM",
        0x400 => "FNT",
        0x402 => "GUI",
        0x403 => "SQL",
        0x404 => "PVRZ",
        0x405 => "GLSL",
        0x406 => "TOT",
        0x407 => "TOH",
        0x408 => "MENU",
        0x409 => "LUA",
        0x40A => "TTF",
        0x40B => "PNG",
        0x44C => "BAH",
        0x802 => "INI",
        0x803 => "SRC",
        0x804 => "MAZE",
        0xFFE => "MUS",
        0xFFF => "ACM",
        _ => return None,
    };

    Some(extension.to_string())
}

#[derive(Debug, Error)]
pub enum IoError {
    #[error("game path is not a directory: {0}")]
    InvalidGamePath(PathBuf),
    #[error("missing chitin.key: {0}")]
    MissingChitinKey(PathBuf),
    #[error("resource not found: {0}")]
    ResourceNotFound(String),
    #[error("resource not found in {source_name}: {resource}")]
    ResourceNotFoundInSource {
        resource: String,
        source_name: String,
    },
    #[error("save not found: {0}")]
    SaveNotFound(String),
    #[error("save folder is invalid or contains no BALDUR.gam/BALDUR.SAV: {0}")]
    InvalidSavePath(PathBuf),
    #[error("save member {member} not found in {save}")]
    SaveMemberNotFound { save: PathBuf, member: String },
    #[error("dialog.tlk not found for installation: {0}")]
    DialogTlkNotFound(PathBuf),
    #[error("strref {requested} is out of range for TLK with {entry_count} entries")]
    StrRefOutOfRange { requested: u32, entry_count: u32 },
    #[error("invalid KEY file: {0}")]
    InvalidKey(String),
    #[error("invalid TLK file: {0}")]
    InvalidTlk(String),
    #[error("invalid IDS file: {0}")]
    InvalidIds(String),
    #[error("invalid BIFF archive: {0}")]
    InvalidBiff(String),
    #[error("unexpected end of file while reading {0}")]
    UnexpectedEof(String),
    #[error("{0}")]
    FileIo(std::io::Error),
}
