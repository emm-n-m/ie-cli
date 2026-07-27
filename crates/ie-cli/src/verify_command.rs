use crate::resource_links::CliResourceLinkResolver;
use ie_core::ResourceName;
use ie_formats::{
    AreaJson, EntranceRegistry, VerifyCategory, VerifyIssue, VerifyOptions, VerifySeverity,
    filter_issues, parse_are, verify_are,
};
use ie_io::{
    GameInstallation, ResourceListOptions, ResourceLocator, ResourceReader, ResourceSource,
};
use std::path::PathBuf;

pub(crate) fn verify_installation(
    game: PathBuf,
    target_source: ResourceSource,
    options: VerifyOptions,
) -> Result<Vec<VerifyIssue>, Box<dyn std::error::Error>> {
    let installation = GameInstallation::discover(game)?;
    let locator = ResourceLocator::new(installation)?;
    let reader = ResourceReader;
    let link_resolver = CliResourceLinkResolver {
        locator: &locator,
        tlk_resolver: None,
        source: ResourceSource::Auto,
    };

    let registry_entries =
        parse_are_resources(&locator, &reader, ResourceSource::Auto, &link_resolver)?;
    let mut registry = EntranceRegistry::default();
    for area in registry_entries
        .iter()
        .filter_map(|entry| entry.as_ref().ok())
    {
        registry.insert_area(area);
    }

    let target_entries = if target_source == ResourceSource::Auto {
        registry_entries
    } else {
        parse_are_resources(&locator, &reader, target_source, &link_resolver)?
    };

    let mut issues = Vec::new();
    for entry in target_entries {
        match entry {
            Ok(area) => issues.extend(verify_are(&area, &registry)),
            Err(issue) => issues.push(issue),
        }
    }

    Ok(filter_issues(issues, options))
}

fn parse_are_resources(
    locator: &ResourceLocator,
    reader: &ResourceReader,
    source: ResourceSource,
    link_resolver: &CliResourceLinkResolver<'_>,
) -> Result<Vec<Result<AreaJson, VerifyIssue>>, Box<dyn std::error::Error>> {
    let resources = locator.list(ResourceListOptions {
        resource_type: Some("ARE".to_string()),
        name_glob: None,
        source: Some(source),
    })?;

    let mut areas = Vec::with_capacity(resources.len());
    for listed in resources {
        let parsed_name = ResourceName::parse(&listed.resource_name)?;
        let result = match reader.read_with_source(locator, &parsed_name, source) {
            Ok(bytes) => parse_are(
                &bytes.bytes,
                &bytes.metadata.resource_name,
                Some(link_resolver),
            )
            .map_err(|err| err.to_string()),
            Err(error) => Err(error.to_string()),
        };

        areas.push(result.map_err(|message| parse_error_issue(listed.resource_name, message)));
    }

    Ok(areas)
}

fn parse_error_issue(resource_name: String, message: String) -> VerifyIssue {
    VerifyIssue {
        resource: resource_name,
        issue: VerifyCategory::ParseError,
        severity: VerifySeverity::Error,
        path: "$".to_string(),
        expected_in: None,
        expected_value: None,
        available_entrances: None,
        message,
    }
}

pub(crate) fn format_verify_issue_text(issue: &VerifyIssue) -> String {
    format!(
        "{} {} {} {}: {}",
        match issue.severity {
            VerifySeverity::Error => "ERROR",
            VerifySeverity::Warning => "WARNING",
        },
        issue.resource,
        issue.path,
        issue.issue.as_str(),
        issue.message
    )
}
