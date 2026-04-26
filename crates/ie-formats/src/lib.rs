mod are;
mod bcs;
mod cre;
mod dlg;
mod itm;
mod spl;
mod sto;

use ie_core::{ResolverBundle, ResourceBytes, ResourceType};
use serde_json::Value;
use thiserror::Error;

pub use are::{AreaScalarPatch, patch_are_scalars};
pub use cre::{CreatureScalarPatch, patch_cre_scalars};

pub fn decode_to_json(
    resource: &ResourceBytes,
    resolvers: ResolverBundle<'_>,
) -> Result<Value, FormatError> {
    let resource_type = resource.metadata.resource_type;

    match resource_type {
        ResourceType::Are => {
            let area = are::parse_are(
                &resource.bytes,
                &resource.metadata.resource_name,
                resolvers.links,
            )?;
            serde_json::to_value(&area).map_err(|err| FormatError::Serialization(err.to_string()))
        }
        ResourceType::Itm => {
            let item = itm::parse_itm(
                &resource.bytes,
                &resource.metadata.resource_name,
                resolvers.strref,
            )?;
            serde_json::to_value(&item).map_err(|err| FormatError::Serialization(err.to_string()))
        }
        ResourceType::Spl => {
            let spell = spl::parse_spl(
                &resource.bytes,
                &resource.metadata.resource_name,
                resolvers.strref,
            )?;
            serde_json::to_value(&spell).map_err(|err| FormatError::Serialization(err.to_string()))
        }
        ResourceType::Cre => {
            let creature = cre::parse_cre(
                &resource.bytes,
                &resource.metadata.resource_name,
                resolvers.strref,
            )?;
            serde_json::to_value(&creature)
                .map_err(|err| FormatError::Serialization(err.to_string()))
        }
        ResourceType::Sto => {
            let store = sto::parse_sto(
                &resource.bytes,
                &resource.metadata.resource_name,
                resolvers.strref,
            )?;
            serde_json::to_value(&store).map_err(|err| FormatError::Serialization(err.to_string()))
        }
        ResourceType::Dlg => {
            let dialog = dlg::parse_dlg(
                &resource.bytes,
                &resource.metadata.resource_name,
                resolvers.strref,
            )?;
            serde_json::to_value(&dialog).map_err(|err| FormatError::Serialization(err.to_string()))
        }
        ResourceType::Bcs => {
            let script = bcs::parse_bcs(
                &resource.bytes,
                &resource.metadata.resource_name,
                resolvers.ids,
            )?;
            serde_json::to_value(&script).map_err(|err| FormatError::Serialization(err.to_string()))
        }
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
