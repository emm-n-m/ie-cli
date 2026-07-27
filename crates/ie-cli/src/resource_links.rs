use ie_core::{
    CreatureResourceLink, ResRef, ResolvedStrRef, ResolverBundle, ResourceLink,
    ResourceLinkResolver, ResourceName, ResourceType,
};
use ie_formats::decode_to_json;
use ie_io::{ResourceLocator, ResourceReader, ResourceSource, TlkResolver};

pub(crate) struct CliResourceLinkResolver<'a> {
    pub(crate) locator: &'a ResourceLocator,
    pub(crate) tlk_resolver: Option<&'a TlkResolver>,
    pub(crate) source: ResourceSource,
}

impl ResourceLinkResolver for CliResourceLinkResolver<'_> {
    fn resolve_resource_link(&self, resref: &ResRef, resource_type: ResourceType) -> ResourceLink {
        let resource_name = format!("{}.{}", resref.as_str(), resource_type.as_str());
        let parsed = ResourceName::parse(&resource_name);

        if let Ok(resource) = parsed
            && let Ok(located) = self.locator.locate_with_source(&resource, self.source)
        {
            return ResourceLink {
                resref: resref.clone(),
                resource_name,
                resource_type: resource_type.as_str().to_string(),
                exists: true,
                source_kind: Some(located.metadata.source_kind),
                source_path: Some(located.metadata.source_path),
            };
        }

        ResourceLink {
            resref: resref.clone(),
            resource_name,
            resource_type: resource_type.as_str().to_string(),
            exists: false,
            source_kind: None,
            source_path: None,
        }
    }

    fn resolve_creature_link(&self, resref: &ResRef) -> CreatureResourceLink {
        let link = self.resolve_resource_link(resref, ResourceType::Cre);
        let mut creature_link = CreatureResourceLink {
            link,
            short_name: None,
            long_name: None,
        };

        if !creature_link.link.exists {
            return creature_link;
        }

        let resource_name = creature_link.link.resource_name.clone();
        let Ok(resource) = ResourceName::parse(&resource_name) else {
            return creature_link;
        };
        let reader = ResourceReader;
        let Ok(bytes) = reader.read_with_source(self.locator, &resource, self.source) else {
            return creature_link;
        };
        let Ok(value) = decode_to_json(
            &bytes,
            ResolverBundle {
                strref: self.tlk_resolver.map(|resolver| resolver as _),
                ids: None,
                links: None,
            },
        ) else {
            return creature_link;
        };

        creature_link.short_name =
            serde_json::from_value::<ResolvedStrRef>(value["header"]["short_name"].clone()).ok();
        creature_link.long_name =
            serde_json::from_value::<ResolvedStrRef>(value["header"]["long_name"].clone()).ok();
        creature_link
    }
}
