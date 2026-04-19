mod biff;
mod bytes;
mod ids;

use crate::bytes::{read_u16_le, read_u32_le};
use ie_core::{ResourceBytes, ResourceMetadata, ResourceName, ResourceType, SourceKind, StrRef, StrRefResolver};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub use ids::{parse_ids, parse_ids_text, FileBackedIdsResolver};

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
            let path = override_dir.join(&file_name);
            if path.is_file() {
                return Some(LocatedResource {
                    metadata: ResourceMetadata {
                        source_path: path,
                        source_kind: SourceKind::Override,
                        resource_type: resource.resource_type(),
                        resource_name: file_name,
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
            dir_entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());

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
    path: PathBuf,
}

impl TlkResolver {
    pub fn new(installation: &GameInstallation) -> Result<Self, IoError> {
        let path = installation
            .dialog_tlk
            .clone()
            .ok_or_else(|| IoError::DialogTlkNotFound(installation.root.clone()))?;

        Ok(Self { path })
    }

    pub fn resolve(&self, strref: u32) -> Result<ResolvedTlkEntry, IoError> {
        let bytes = fs::read(&self.path).map_err(IoError::FileIo)?;

        if bytes.len() < 18 {
            return Err(IoError::InvalidTlk("TLK file is too small".to_string()));
        }

        if &bytes[0..8] != b"TLK V1  " {
            return Err(IoError::InvalidTlk("invalid TLK signature".to_string()));
        }

        let entry_count = read_u32_le(&bytes, 10)? as usize;
        let strings_offset = read_u32_le(&bytes, 14)? as usize;
        let strref = strref as usize;

        if strref >= entry_count {
            return Err(IoError::StrRefOutOfRange {
                requested: strref as u32,
                entry_count: entry_count as u32,
            });
        }

        let entry_offset = 18 + (strref * 26);
        if entry_offset + 26 > bytes.len() {
            return Err(IoError::InvalidTlk("truncated TLK entry table".to_string()));
        }

        let relative_text_offset = read_u32_le(&bytes, entry_offset + 18)? as usize;
        let text_length = read_u32_le(&bytes, entry_offset + 22)? as usize;
        let text_start = strings_offset + relative_text_offset;
        let text_end = text_start.saturating_add(text_length);

        if text_end > bytes.len() {
            return Err(IoError::InvalidTlk(
                "TLK string points outside the file".to_string(),
            ));
        }

        let text = String::from_utf8_lossy(&bytes[text_start..text_end]).to_string();

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
            let entry_offset = bif_offset + (index * 12);
            if entry_offset + 12 > bytes.len() {
                return Err(IoError::InvalidKey("truncated BIFF table".to_string()));
            }

            let file_size = read_u32_le(&bytes, entry_offset)?;
            let string_offset = read_u32_le(&bytes, entry_offset + 4)? as usize;
            let string_length = read_u16_le(&bytes, entry_offset + 8)? as usize;

            if string_length == 0 || string_offset + string_length > bytes.len() {
                return Err(IoError::InvalidKey(
                    "invalid BIFF filename entry".to_string(),
                ));
            }

            let mut relative_path =
                String::from_utf8_lossy(&bytes[string_offset..(string_offset + string_length - 1)])
                    .to_string();
            if relative_path.starts_with('\\') || relative_path.starts_with('/') {
                relative_path.remove(0);
            }
            relative_path = relative_path.replace('\\', "/").replace(':', "/");

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
            let entry_offset = resource_offset + (index * 14);
            if entry_offset + 14 > bytes.len() {
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

fn discover_dialog_tlk(
    root: &Path,
) -> Result<(Option<String>, Option<PathBuf>, Option<PathBuf>), IoError> {
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

    if let Some(language_dir) = language_dir {
        let language_override = language_dir.join("Override");
        if language_override.is_dir() {
            dirs.push(language_override);
        }
    }

    let root_override = root.join("override");
    if root_override.is_dir() {
        dirs.push(root_override);
    }

    dirs
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

    for path in cbf_candidates {
        if path.is_file() {
            return Some(path);
        }
    }

    None
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
