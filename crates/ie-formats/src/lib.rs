mod cre;
mod dlg;
mod itm;
mod spl;
mod sto;

use ie_core::{ResourceBytes, ResourceType, StrRefResolver};
use serde_json::Value;
use thiserror::Error;

pub fn decode_to_json(
    resource: &ResourceBytes,
    resolver: Option<&dyn StrRefResolver>,
) -> Result<Value, FormatError> {
    let resource_type = resource.metadata.resource_type;

    match resource_type {
        ResourceType::Itm => {
            let item = itm::parse_itm(&resource.bytes, &resource.metadata.resource_name, resolver)?;
            serde_json::to_value(&item).map_err(|err| FormatError::Serialization(err.to_string()))
        }
        ResourceType::Spl => {
            let spell =
                spl::parse_spl(&resource.bytes, &resource.metadata.resource_name, resolver)?;
            serde_json::to_value(&spell).map_err(|err| FormatError::Serialization(err.to_string()))
        }
        ResourceType::Cre => {
            let creature =
                cre::parse_cre(&resource.bytes, &resource.metadata.resource_name, resolver)?;
            serde_json::to_value(&creature)
                .map_err(|err| FormatError::Serialization(err.to_string()))
        }
        ResourceType::Sto => {
            let store =
                sto::parse_sto(&resource.bytes, &resource.metadata.resource_name, resolver)?;
            serde_json::to_value(&store).map_err(|err| FormatError::Serialization(err.to_string()))
        }
        ResourceType::Dlg => {
            let dialog =
                dlg::parse_dlg(&resource.bytes, &resource.metadata.resource_name, resolver)?;
            serde_json::to_value(&dialog).map_err(|err| FormatError::Serialization(err.to_string()))
        }
        ResourceType::Bcs => Err(FormatError::NotImplemented(resource_type)),
        ResourceType::Unknown => Err(FormatError::UnsupportedResourceType),
    }
}

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("resource decoding is not implemented yet for {0:?}")]
    NotImplemented(ResourceType),
    #[error("unsupported resource type")]
    UnsupportedResourceType,
    #[error("resource parsing failure: {0}")]
    Parse(String),
    #[error("JSON serialization failure: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for FormatError {
    fn from(err: serde_json::Error) -> Self {
        FormatError::Serialization(err.to_string())
    }
}
