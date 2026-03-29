use clap::{Args, Parser, Subcommand, ValueEnum};
use ie_core::ResourceName;
use ie_formats::decode_to_json;
use ie_io::{GameInstallation, ResourceLocator, ResourceReader, TlkResolver};
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
    Tlk(TlkArgs),
}

#[derive(Debug, Args)]
struct ResourceArgs {
    #[arg(long)]
    game: PathBuf,
    #[arg(long)]
    resource: String,
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

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
}

#[derive(Debug, Args)]
struct TlkArgs {
    #[arg(long)]
    game: PathBuf,
    #[arg(long)]
    strref: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Locate(args) => {
            let installation = GameInstallation::discover(args.game)?;
            let resource = ResourceName::parse(args.resource)?;
            let locator = ResourceLocator::new(installation)?;
            let located = locator.locate(&resource)?;
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
            let bytes = reader.read(&locator, &resource)?;

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
            let locator = ResourceLocator::new(installation)?;
            let reader = ResourceReader;
            let bytes = reader.read(&locator, &resource)?;

            match args.format {
                OutputFormat::Json => {
                    let value = decode_to_json(&bytes)?;
                    println!("{}", serde_json::to_string_pretty(&value)?);
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
