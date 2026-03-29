use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StrRef(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResRef(String);

impl ResRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(CoreError::InvalidResRef(
                "resource reference cannot be empty".to_string(),
            ));
        }

        if trimmed.len() > 8 {
            return Err(CoreError::InvalidResRef(format!(
                "resource reference exceeds 8 characters: {trimmed}"
            )));
        }

        Ok(Self(trimmed.to_ascii_uppercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceName {
    resref: ResRef,
    extension: String,
}

impl ResourceName {
    pub fn parse(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        let trimmed = value.trim();

        let (resref, extension) = trimmed
            .rsplit_once('.')
            .ok_or_else(|| CoreError::InvalidResourceName(trimmed.to_string()))?;

        if extension.trim().is_empty() {
            return Err(CoreError::InvalidResourceName(trimmed.to_string()));
        }

        Ok(Self {
            resref: ResRef::new(resref)?,
            extension: extension.trim().to_ascii_uppercase(),
        })
    }

    pub fn resref(&self) -> &ResRef {
        &self.resref
    }

    pub fn extension(&self) -> &str {
        &self.extension
    }

    pub fn resource_type(&self) -> ResourceType {
        ResourceType::from_extension(&self.extension)
    }

    pub fn file_name(&self) -> String {
        format!("{}.{}", self.resref.as_str(), self.extension)
    }
}

impl fmt::Display for ResourceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.file_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Itm,
    Spl,
    Cre,
    Sto,
    Dlg,
    Bcs,
    Unknown,
}

impl ResourceType {
    pub fn from_extension(extension: &str) -> Self {
        match extension.trim().to_ascii_uppercase().as_str() {
            "ITM" => Self::Itm,
            "SPL" => Self::Spl,
            "CRE" | "CHR" => Self::Cre,
            "STO" => Self::Sto,
            "DLG" => Self::Dlg,
            "BCS" | "BS" => Self::Bcs,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Itm => "ITM",
            Self::Spl => "SPL",
            Self::Cre => "CRE",
            Self::Sto => "STO",
            Self::Dlg => "DLG",
            Self::Bcs => "BCS",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceKind {
    Override,
    Bif,
    LooseFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub source_path: PathBuf,
    pub source_kind: SourceKind,
    pub resource_type: ResourceType,
    pub resource_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceBytes {
    pub metadata: ResourceMetadata,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedStrRef {
    pub strref: StrRef,
    pub text: Option<String>,
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid resource reference: {0}")]
    InvalidResRef(String),
    #[error("invalid resource name: {0}")]
    InvalidResourceName(String),
}

#[cfg(test)]
mod tests {
    use super::{ResRef, ResourceName, ResourceType};

    #[test]
    fn normalizes_resrefs_to_uppercase() {
        let resref = ResRef::new("foo").expect("resref should be valid");
        assert_eq!(resref.as_str(), "FOO");
    }

    #[test]
    fn rejects_resrefs_longer_than_eight_characters() {
        let result = ResRef::new("ABCDEFGHI");
        assert!(result.is_err());
    }

    #[test]
    fn maps_known_extensions() {
        assert_eq!(ResourceType::from_extension("itm"), ResourceType::Itm);
        assert_eq!(ResourceType::from_extension("chr"), ResourceType::Cre);
        assert_eq!(ResourceType::from_extension("zzz"), ResourceType::Unknown);
    }

    #[test]
    fn parses_resource_names() {
        let resource = ResourceName::parse("foo.itm").expect("resource name should be valid");
        assert_eq!(resource.file_name(), "FOO.ITM");
        assert_eq!(resource.resource_type(), ResourceType::Itm);
    }
}
