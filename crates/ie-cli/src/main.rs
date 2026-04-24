use clap::{Args, Parser, Subcommand, ValueEnum};
use ie_core::{
    CreatureResourceLink, ResRef, ResolvedStrRef, ResolverBundle, ResourceLink,
    ResourceLinkResolver, ResourceName, ResourceType,
};
use ie_formats::decode_to_json;
use ie_io::{
    FileBackedIdsResolver, GameInstallation, ListedResource, ResourceListOptions,
    ResourceLocator, ResourceReader, ResourceSource, TlkResolver,
};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "iecli")]
#[command(about = "CLI-first Infinity Engine inspection tool")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Locate(ResourceArgs),
    DumpRaw(DumpRawArgs),
    Dump(DumpArgs),
    List(ListArgs),
    Tlk(TlkArgs),
}

#[derive(Debug, Args)]
struct ResourceArgs {
    #[arg(long)]
    game: PathBuf,
    #[arg(long)]
    resource: String,
    #[command(flatten)]
    source: SourceArgs,
}

#[derive(Debug, Args)]
struct DumpRawArgs {
    #[command(flatten)]
    resource: ResourceArgs,
    #[arg(long)]
    output: PathBuf,
}

#[derive(Debug, Args)]
struct DumpArgs {
    #[command(flatten)]
    resource: ResourceArgs,
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,
}

#[derive(Debug, Args)]
struct ListArgs {
    #[arg(long)]
    game: PathBuf,
    #[arg(long = "type")]
    resource_type: Option<String>,
    #[arg(long)]
    name: Option<String>,
    #[command(flatten)]
    source: SourceArgs,
    #[arg(long, value_enum, default_value_t = ListFormat::Text)]
    format: ListFormat,
}

#[derive(Debug, Args, Default)]
struct SourceArgs {
    #[arg(long, value_enum)]
    source: Option<SourceArg>,
    #[arg(long, conflicts_with = "source")]
    skip_override: bool,
}

impl SourceArgs {
    fn selection(&self) -> ResourceSource {
        if self.skip_override {
            ResourceSource::Bif
        } else {
            self.source.unwrap_or(SourceArg::Auto).into()
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ListFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SourceArg {
    Auto,
    Override,
    Bif,
}

impl From<SourceArg> for ResourceSource {
    fn from(value: SourceArg) -> Self {
        match value {
            SourceArg::Auto => ResourceSource::Auto,
            SourceArg::Override => ResourceSource::Override,
            SourceArg::Bif => ResourceSource::Bif,
        }
    }
}

#[derive(Debug, Args)]
struct TlkArgs {
    #[arg(long)]
    game: PathBuf,
    #[arg(long)]
    strref: u32,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Locate(args) => {
            let installation = GameInstallation::discover(args.game)?;
            let resource = ResourceName::parse(args.resource)?;
            let locator = ResourceLocator::new(installation)?;
            let located = locator.locate_with_source(&resource, args.source.selection())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "resource_name": located.metadata.resource_name,
                    "resource_type": located.metadata.resource_type.as_str(),
                    "source_kind": format!("{:?}", located.metadata.source_kind),
                    "source_path": located.metadata.source_path,
                    "locator": located.locator,
                }))?
            );
        }
        Command::DumpRaw(args) => {
            let installation = GameInstallation::discover(args.resource.game)?;
            let resource = ResourceName::parse(args.resource.resource)?;
            let locator = ResourceLocator::new(installation)?;
            let reader = ResourceReader;
            let bytes = reader.read_with_source(&locator, &resource, args.resource.source.selection())?;

            if let Some(parent) = args.output.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)?;
                }
            }

            fs::write(&args.output, &bytes.bytes)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "resource_name": bytes.metadata.resource_name,
                    "resource_type": bytes.metadata.resource_type.as_str(),
                    "source_kind": format!("{:?}", bytes.metadata.source_kind),
                    "source_path": bytes.metadata.source_path,
                    "output_path": args.output,
                    "bytes_written": bytes.bytes.len(),
                }))?
            );
        }
        Command::Dump(args) => {
            let installation = GameInstallation::discover(args.resource.game)?;
            let resource = ResourceName::parse(args.resource.resource)?;
            let locator = ResourceLocator::new(installation.clone())?;
            let reader = ResourceReader;
            let bytes = reader.read_with_source(&locator, &resource, args.resource.source.selection())?;
            let tlk_resolver = installation
                .dialog_tlk
                .as_ref()
                .map(|_| TlkResolver::new(&installation))
                .transpose()?;
            let ids_resolver = FileBackedIdsResolver::new(locator.clone());
            let link_resolver = CliResourceLinkResolver {
                locator: &locator,
                tlk_resolver: tlk_resolver.as_ref(),
            };

            match args.format {
                OutputFormat::Json => {
                    let value = decode_to_json(
                        &bytes,
                        ResolverBundle {
                            strref: tlk_resolver.as_ref().map(|resolver| resolver as _),
                            ids: Some(&ids_resolver),
                            links: Some(&link_resolver),
                        },
                    )?;
                    println!("{}", serde_json::to_string_pretty(&value)?);
                }
            }
        }
        Command::List(args) => {
            let installation = GameInstallation::discover(args.game)?;
            let locator = ResourceLocator::new(installation)?;
            let resources = locator.list(ResourceListOptions {
                resource_type: args.resource_type.map(|value| value.trim().to_ascii_uppercase()),
                name_glob: args.name,
                source: Some(args.source.selection()),
            })?;

            match args.format {
                ListFormat::Text => {
                    for resource in resources {
                        println!("{}", resource.resref);
                    }
                }
                ListFormat::Json => {
                    let payload = resources
                        .iter()
                        .map(listed_resource_json)
                        .collect::<Vec<_>>();
                    println!("{}", serde_json::to_string_pretty(&payload)?);
                }
            }
        }
        Command::Tlk(args) => {
            let installation = GameInstallation::discover(args.game)?;
            let resolver = TlkResolver::new(&installation)?;
            let entry = resolver.resolve(args.strref)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "strref": entry.strref,
                    "text": entry.text,
                    "dialog_tlk": installation.dialog_tlk,
                    "language": installation.language,
                }))?
            );
        }
    }

    Ok(())
}

fn listed_resource_json(resource: &ListedResource) -> serde_json::Value {
    serde_json::json!({
        "resref": resource.resref,
        "type": resource.extension,
        "resource_name": resource.resource_name,
        "source_kind": format!("{:?}", resource.source_kind),
        "source_path": resource.source_path,
    })
}

struct CliResourceLinkResolver<'a> {
    locator: &'a ResourceLocator,
    tlk_resolver: Option<&'a TlkResolver>,
}

impl ResourceLinkResolver for CliResourceLinkResolver<'_> {
    fn resolve_resource_link(&self, resref: &ResRef, resource_type: ResourceType) -> ResourceLink {
        let resource_name = format!("{}.{}", resref.as_str(), resource_type.as_str());
        let parsed = ResourceName::parse(&resource_name);

        if let Ok(resource) = parsed
            && let Ok(located) = self.locator.locate(&resource)
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
        let Ok(bytes) = reader.read(self.locator, &resource) else {
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
