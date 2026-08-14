use crate::IoError;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use zip::ZipArchive;

/// A mounted Enhanced Edition DLC archive.
///
/// The archive is opened, and its central directory indexed, during game
/// discovery. Reads reuse that archive handle; the individual entry is still
/// decompressed on demand rather than extracting the DLC to disk.
#[derive(Debug, Clone)]
pub struct DlcArchive {
    pub path: PathBuf,
    entries: HashMap<String, String>,
    archive: Arc<Mutex<ZipArchive<File>>>,
}

impl PartialEq for DlcArchive {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.entries == other.entries
    }
}

impl Eq for DlcArchive {}

impl DlcArchive {
    pub(crate) fn open(path: PathBuf) -> Result<Self, IoError> {
        let file = File::open(&path).map_err(|error| IoError::DlcArchive {
            path: path.clone(),
            message: error.to_string(),
        })?;
        let archive = ZipArchive::new(file).map_err(|error| IoError::DlcArchive {
            path: path.clone(),
            message: error.to_string(),
        })?;

        let mut entries = HashMap::new();
        for name in archive.file_names() {
            let name = name.map_err(|error| IoError::DlcArchive {
                path: path.clone(),
                message: error.to_string(),
            })?;
            let name = name.into_owned();
            if name.ends_with('/') {
                continue;
            }

            let key = normalize_entry_name(&name);
            if key.is_empty() {
                continue;
            }

            match entries.get(&key) {
                Some(existing) if existing <= &name => {}
                _ => {
                    entries.insert(key, name);
                }
            }
        }

        Ok(Self {
            path,
            entries,
            archive: Arc::new(Mutex::new(archive)),
        })
    }

    pub(crate) fn entry_name(&self, name: &str) -> Option<&str> {
        self.entries
            .get(&normalize_entry_name(name))
            .map(String::as_str)
    }

    pub(crate) fn display_path(&self, interior: &str) -> PathBuf {
        PathBuf::from(format!("{}!{}", self.path.display(), interior))
    }

    pub(crate) fn read_entry(&self, name: &str) -> Result<Vec<u8>, IoError> {
        let canonical = self
            .entry_name(name)
            .ok_or_else(|| IoError::DlcEntryNotFound {
                archive: self.path.clone(),
                entry: name.to_string(),
            })?;
        let mut archive = self.archive.lock().map_err(|_| IoError::DlcArchive {
            path: self.path.clone(),
            message: "archive lock poisoned".to_string(),
        })?;
        let mut entry = archive
            .by_name(canonical)
            .map_err(|error| IoError::DlcArchive {
                path: self.path.clone(),
                message: error.to_string(),
            })?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(IoError::FileIo)?;
        Ok(bytes)
    }

    pub(crate) fn read_biff_resource(&self, name: &str, locator: u32) -> Result<Vec<u8>, IoError> {
        let canonical = self
            .entry_name(name)
            .ok_or_else(|| IoError::DlcEntryNotFound {
                archive: self.path.clone(),
                entry: name.to_string(),
            })?;
        let mut archive = self.archive.lock().map_err(|_| IoError::DlcArchive {
            path: self.path.clone(),
            message: "archive lock poisoned".to_string(),
        })?;

        match archive.by_name_seek(canonical) {
            Ok(mut entry) => crate::biff::read_resource_from_reader(&mut entry, locator),
            Err(_) => {
                // Compressed ZIP entries cannot expose a seekable reader in the
                // zip crate. This is uncommon for game BIFs, but remains valid
                // and is handled without changing the ordinary on-disk path.
                let mut entry =
                    archive
                        .by_name(canonical)
                        .map_err(|error| IoError::DlcArchive {
                            path: self.path.clone(),
                            message: error.to_string(),
                        })?;
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes).map_err(IoError::FileIo)?;
                crate::biff::read_resource_from_bytes_for_dlc(&bytes, locator)
            }
        }
    }

    pub(crate) fn override_entries(&self) -> Vec<(String, String)> {
        let mut entries = self
            .entries
            .iter()
            .filter_map(|(normalized, original)| {
                let remainder = normalized.strip_prefix("OVERRIDE/")?;
                if remainder.is_empty() || remainder.contains('/') {
                    return None;
                }
                Some((remainder.to_string(), original.clone()))
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
        entries
    }

    pub(crate) fn dialog_tlk_entries(&self) -> Vec<(String, String)> {
        let mut entries = self
            .entries
            .iter()
            .filter_map(|(normalized, original)| {
                let remainder = normalized.strip_prefix("LANG/")?;
                let (language, file_name) = remainder.split_once('/')?;
                if language.is_empty() || !file_name.eq_ignore_ascii_case("dialog.tlk") {
                    return None;
                }
                let original_parts = original.replace('\\', "/");
                let original_language = original_parts.split('/').nth(1)?;
                Some((original_language.to_string(), original.clone()))
            })
            .collect::<Vec<_>>();
        entries.sort();
        entries
    }
}

pub(crate) fn normalize_entry_name(name: &str) -> String {
    name.replace('\\', "/")
        .trim_start_matches('/')
        .to_ascii_uppercase()
}
