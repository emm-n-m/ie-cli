use ie_core::{ResourceBytes, ResourceType};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct JsonResource {
    pub resource_type: String,
    pub resource_name: String,
    pub note: String,
}

pub fn decode_to_json(resource: &ResourceBytes) -> Result<JsonResource, FormatError> {
    let resource_type = resource.metadata.resource_type;

    match resource_type {
        ResourceType::Itm
        | ResourceType::Spl
        | ResourceType::Cre
        | ResourceType::Sto
        | ResourceType::Dlg
        | ResourceType::Bcs => Err(FormatError::NotImplemented(resource_type)),
        ResourceType::Unknown => Err(FormatError::UnsupportedResourceType),
    }
}

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("resource decoding is not implemented yet for {0:?}")]
    NotImplemented(ResourceType),
    #[error("unsupported resource type")]
    UnsupportedResourceType,
}
