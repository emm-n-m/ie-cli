mod dialog_graph;
mod override_diff;
mod patch_input;
mod resource_links;
mod save_support;
mod verify_command;

use clap::{Args, Parser, Subcommand, ValueEnum};
use dialog_graph::collect_followed_dialogs;
use ie_core::{ResRef, ResolverBundle, ResourceName, ResourceType};
use ie_formats::{
    DialogGraphOptions, DialogGraphStringMode, NewItem, VerifyOptions, VerifySeverity,
    add_item_to_save_gam, decode_to_json, dialog_json_to_dot, dialog_json_to_mermaid,
    dialog_jsons_to_dot, dialog_jsons_to_mermaid, parse_dlg, parse_gam, parse_sav,
    patch_are_scalars, patch_cre_scalars,
};
use ie_io::{
    FileBackedIdsResolver, GameInstallation, ListedResource, ListedSave, ResourceListOptions,
    ResourceLocator, ResourceReader, ResourceSource, TlkResolver, append_tlk_string, list_saves,
    read_save_member, resolve_save_folder,
};
use override_diff::{
    build_override_reference_report, build_override_shadow_report, override_reference_report_json,
    override_shadow_report_json, print_override_reference_report_text,
    print_override_shadow_report_text,
};
use patch_input::{collect_are_patches, collect_cre_patches};
use resource_links::CliResourceLinkResolver;
use save_support::{
    copy_save_folder, parse_item_flags, parse_member_selector, parse_slot_choice,
    resolve_child_file_case_insensitive, warn_if_item_missing,
};
use std::fs;
use std::path::PathBuf;
use verify_command::{format_verify_issue_text, verify_installation};

#[derive(Debug, Parser)]
#[command(name = "iecli")]
#[command(about = "CLI-first Infinity Engine inspection tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Locate(ResourceArgs),
    DumpRaw(DumpRawArgs),
    Dump(DumpArgs),
    Patch(PatchArgs),
    List(ListArgs),
    OverrideDiff(OverrideDiffArgs),
    Tlk(TlkArgs),
    TlkAppend(TlkAppendArgs),
    Verify(VerifyArgs),
    SaveList(SaveListArgs),
    SaveInfo(SaveInfoArgs),
    SaveAddItem(SaveAddItemArgs),
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
    #[arg(long, default_value_t = 40)]
    max_label_len: usize,
    #[arg(long)]
    no_triggers: bool,
    #[arg(long)]
    no_actions: bool,
    #[arg(long, value_enum, default_value_t = GraphStringModeArg::Resolved)]
    strings: GraphStringModeArg,
    #[arg(long, num_args = 0..=1, default_missing_value = "1")]
    follow_extern: Option<usize>,
}

#[derive(Debug, Args)]
struct PatchArgs {
    #[command(flatten)]
    resource: ResourceArgs,
    #[arg(long = "set")]
    sets: Vec<String>,
    #[arg(long)]
    patch_json: Option<PathBuf>,
    #[arg(long)]
    output: PathBuf,
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

#[derive(Debug, Args)]
struct OverrideDiffArgs {
    #[arg(long)]
    game: PathBuf,
    #[arg(long = "type")]
    resource_type: Option<String>,
    #[arg(long)]
    resource: Option<String>,
    #[arg(long)]
    against: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = OverrideDiffFormat::Text)]
    format: OverrideDiffFormat,
}

#[derive(Debug, Args)]
struct VerifyArgs {
    #[arg(long)]
    game: PathBuf,
    #[command(flatten)]
    source: SourceArgs,
    #[arg(long = "resource-type", default_value = "ARE")]
    resource_type: String,
    #[arg(long, value_enum)]
    severity: Option<SeverityArg>,
    #[arg(long, value_enum, default_value_t = VerifyFormat::Text)]
    format: VerifyFormat,
    #[arg(long)]
    max_issues: Option<usize>,
}

#[derive(Debug, Args)]
struct SaveListArgs {
    #[arg(long)]
    game: PathBuf,
    #[arg(long)]
    saves_dir: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = ListFormat::Text)]
    format: ListFormat,
}

#[derive(Debug, Args)]
struct SaveInfoArgs {
    #[arg(long)]
    game: PathBuf,
    #[arg(long)]
    save: String,
    #[arg(long)]
    saves_dir: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = SaveInfoPart::All)]
    part: SaveInfoPart,
    #[arg(long, value_enum, default_value_t = SaveInfoFormat::Json)]
    format: SaveInfoFormat,
}

#[derive(Debug, Args)]
struct SaveAddItemArgs {
    #[arg(long)]
    game: PathBuf,
    #[arg(long)]
    save: String,
    #[arg(long)]
    saves_dir: Option<PathBuf>,
    #[arg(long)]
    item: String,
    #[arg(long)]
    member: Option<String>,
    #[arg(long, default_value = "auto")]
    slot: String,
    #[arg(long, default_value_t = 0)]
    charges: u16,
    #[arg(long, default_value_t = 0)]
    charges2: u16,
    #[arg(long, default_value_t = 0)]
    charges3: u16,
    #[arg(long, default_value = "identified")]
    flags: String,
    #[arg(long, conflicts_with = "output")]
    in_place: bool,
    #[arg(long, conflicts_with = "in_place")]
    output: Option<PathBuf>,
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    backup: bool,
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
    Dot,
    Mermaid,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum GraphStringModeArg {
    Resolved,
    Strref,
    Both,
}

impl From<GraphStringModeArg> for DialogGraphStringMode {
    fn from(value: GraphStringModeArg) -> Self {
        match value {
            GraphStringModeArg::Resolved => DialogGraphStringMode::Resolved,
            GraphStringModeArg::Strref => DialogGraphStringMode::StrRef,
            GraphStringModeArg::Both => DialogGraphStringMode::Both,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ListFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum VerifyFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OverrideDiffFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SaveInfoFormat {
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SaveInfoPart {
    All,
    Gam,
    Sav,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SeverityArg {
    Error,
    Warning,
}

impl From<SeverityArg> for VerifySeverity {
    fn from(value: SeverityArg) -> Self {
        match value {
            SeverityArg::Error => VerifySeverity::Error,
            SeverityArg::Warning => VerifySeverity::Warning,
        }
    }
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

#[derive(Debug, Args)]
struct TlkAppendArgs {
    #[arg(long)]
    game: PathBuf,
    #[arg(long)]
    text: String,
    #[arg(long)]
    tlk_out: Option<PathBuf>,
    #[arg(long)]
    output_strref_to: Option<PathBuf>,
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
                    "source_kind": located.metadata.source_kind.as_str(),
                    "source_path": located.metadata.source_path,
                    "game_variant": located.metadata.game_variant.as_str(),
                    "locator": located.locator,
                }))?
            );
        }
        Command::DumpRaw(args) => {
            let installation = GameInstallation::discover(args.resource.game)?;
            let resource = ResourceName::parse(args.resource.resource)?;
            let locator = ResourceLocator::new(installation)?;
            let reader = ResourceReader;
            let bytes =
                reader.read_with_source(&locator, &resource, args.resource.source.selection())?;

            if let Some(parent) = args.output.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent)?;
            }

            fs::write(&args.output, &bytes.bytes)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "resource_name": bytes.metadata.resource_name,
                    "resource_type": bytes.metadata.resource_type.as_str(),
                    "source_kind": bytes.metadata.source_kind.as_str(),
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
            let source = args.resource.source.selection();
            let bytes = reader.read_with_source(&locator, &resource, source)?;
            let tlk_resolver = installation
                .dialog_tlk
                .as_ref()
                .map(|_| TlkResolver::new(&installation))
                .transpose()?;
            let ids_resolver = FileBackedIdsResolver::new(locator.clone());
            let link_resolver = CliResourceLinkResolver {
                locator: &locator,
                tlk_resolver: tlk_resolver.as_ref(),
                source,
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
                OutputFormat::Dot | OutputFormat::Mermaid => {
                    if resource.resource_type() != ResourceType::Dlg {
                        let format_name = match args.format {
                            OutputFormat::Dot => "dot",
                            OutputFormat::Mermaid => "mermaid",
                            OutputFormat::Json => unreachable!(),
                        };
                        return Err(
                            format!("--format {format_name} is only supported for DLG").into()
                        );
                    }

                    let dialog = parse_dlg(
                        &bytes.bytes,
                        &bytes.metadata.resource_name,
                        tlk_resolver.as_ref().map(|resolver| resolver as _),
                    )?;
                    let graph_options = DialogGraphOptions {
                        max_label_len: args.max_label_len,
                        include_triggers: !args.no_triggers,
                        include_actions: !args.no_actions,
                        string_mode: args.strings.into(),
                    };

                    if let Some(max_depth) = args.follow_extern {
                        let dialogs = collect_followed_dialogs(
                            &locator,
                            &reader,
                            &dialog,
                            max_depth,
                            source,
                            tlk_resolver.as_ref(),
                        )?;
                        match args.format {
                            OutputFormat::Dot => {
                                println!("{}", dialog_jsons_to_dot(&dialogs, &graph_options))
                            }
                            OutputFormat::Mermaid => {
                                println!("{}", dialog_jsons_to_mermaid(&dialogs, &graph_options))
                            }
                            OutputFormat::Json => unreachable!(),
                        }
                    } else {
                        match args.format {
                            OutputFormat::Dot => {
                                println!("{}", dialog_json_to_dot(&dialog, &graph_options))
                            }
                            OutputFormat::Mermaid => {
                                println!("{}", dialog_json_to_mermaid(&dialog, &graph_options))
                            }
                            OutputFormat::Json => unreachable!(),
                        }
                    }
                }
            }
        }
        Command::Patch(args) => {
            let installation = GameInstallation::discover(args.resource.game)?;
            let resource = ResourceName::parse(args.resource.resource)?;
            let resource_type = resource.resource_type();

            let locator = ResourceLocator::new(installation)?;
            let reader = ResourceReader;
            let bytes =
                reader.read_with_source(&locator, &resource, args.resource.source.selection())?;

            let (patched, patches_applied) = match resource_type {
                ResourceType::Cre => {
                    let patches = collect_cre_patches(&args.sets, args.patch_json.as_deref())?;
                    let count = patches.len();
                    let out = patch_cre_scalars(&bytes.bytes, &patches)?;
                    (out, count)
                }
                ResourceType::Are => {
                    let patches = collect_are_patches(&args.sets, args.patch_json.as_deref())?;
                    let count = patches.len();
                    let out = patch_are_scalars(&bytes.bytes, &patches)?;
                    (out, count)
                }
                _ => {
                    return Err("patch currently supports CRE/CHR and ARE resources only".into());
                }
            };

            if let Some(parent) = args.output.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent)?;
            }

            fs::write(&args.output, &patched)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "resource_name": bytes.metadata.resource_name,
                    "resource_type": bytes.metadata.resource_type.as_str(),
                    "source_kind": bytes.metadata.source_kind.as_str(),
                    "source_path": bytes.metadata.source_path,
                    "output_path": args.output,
                    "patches_applied": patches_applied,
                    "bytes_written": patched.len(),
                }))?
            );
        }
        Command::List(args) => {
            let installation = GameInstallation::discover(args.game)?;
            let locator = ResourceLocator::new(installation)?;
            let resources = locator.list(ResourceListOptions {
                resource_type: args
                    .resource_type
                    .map(|value| value.trim().to_ascii_uppercase()),
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
        Command::OverrideDiff(args) => {
            let installation = GameInstallation::discover(args.game)?;
            let locator = ResourceLocator::new(installation)?;
            let resource_type = args
                .resource_type
                .as_deref()
                .map(|value| value.trim().to_ascii_uppercase());

            if let Some(against) = args.against.as_ref() {
                let report = build_override_reference_report(
                    &locator,
                    resource_type,
                    args.resource.as_deref(),
                    against,
                )?;
                match args.format {
                    OverrideDiffFormat::Text => print_override_reference_report_text(&report),
                    OverrideDiffFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&override_reference_report_json(&report))?
                        );
                    }
                }
            } else {
                let report = build_override_shadow_report(
                    &locator,
                    resource_type,
                    args.resource.as_deref(),
                )?;

                match args.format {
                    OverrideDiffFormat::Text => print_override_shadow_report_text(&report),
                    OverrideDiffFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&override_shadow_report_json(&report))?
                        );
                    }
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
        Command::TlkAppend(args) => {
            let installation = GameInstallation::discover(args.game)?;
            let input_path = installation
                .dialog_tlk
                .clone()
                .ok_or_else(|| "dialog.tlk not found for installation".to_string())?;
            let output_path = args.tlk_out.unwrap_or_else(|| input_path.clone());

            if let Some(path) = args.output_strref_to.as_ref()
                && let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent)?;
            }

            let result = append_tlk_string(&input_path, &args.text, &output_path)?;

            if let Some(path) = args.output_strref_to {
                fs::write(&path, result.strref.to_string())?;
            }

            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "strref": result.strref,
                    "text": args.text,
                    "input_tlk": input_path,
                    "output_tlk": result.output_path,
                    "bytes_written": result.bytes_written,
                    "language": installation.language,
                }))?
            );
        }
        Command::Verify(args) => {
            if !args.resource_type.eq_ignore_ascii_case("ARE") {
                return Err(format!(
                    "verify currently supports --resource-type ARE only, got '{}'",
                    args.resource_type
                )
                .into());
            }

            let options = VerifyOptions {
                severity: args.severity.map(Into::into),
                max_issues: args.max_issues,
            };
            let issues = verify_installation(args.game, args.source.selection(), options)?;

            match args.format {
                VerifyFormat::Text => {
                    for issue in issues {
                        println!("{}", format_verify_issue_text(&issue));
                    }
                }
                VerifyFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&issues)?);
                }
            }
        }
        Command::SaveList(args) => {
            let installation = GameInstallation::discover(args.game)?;
            let saves = list_saves(&installation, args.saves_dir.as_deref())?;

            match args.format {
                ListFormat::Text => {
                    for save in saves {
                        println!(
                            "{}\t{}\t{}",
                            save.save_dir_kind.as_str(),
                            save.folder_name,
                            save.path.display()
                        );
                    }
                }
                ListFormat::Json => {
                    let payload = saves.iter().map(listed_save_json).collect::<Vec<_>>();
                    println!("{}", serde_json::to_string_pretty(&payload)?);
                }
            }
        }
        Command::SaveInfo(args) => {
            let installation = GameInstallation::discover(args.game)?;
            let save = resolve_save_folder(&installation, args.saves_dir.as_deref(), &args.save)?;
            let tlk_resolver = installation
                .dialog_tlk
                .as_ref()
                .map(|_| TlkResolver::new(&installation))
                .transpose()?;

            match args.format {
                SaveInfoFormat::Json => {
                    let value = match args.part {
                        SaveInfoPart::All => {
                            let gam = if save.has_gam {
                                let bytes = read_save_member(&save, "BALDUR.gam")?;
                                Some(serde_json::to_value(parse_gam(
                                    &bytes,
                                    "BALDUR.GAM",
                                    tlk_resolver.as_ref().map(|resolver| resolver as _),
                                )?)?)
                            } else {
                                None
                            };
                            let sav = if save.has_sav {
                                let bytes = read_save_member(&save, "BALDUR.SAV")?;
                                Some(serde_json::to_value(parse_sav(&bytes, "BALDUR.SAV")?)?)
                            } else {
                                None
                            };
                            serde_json::json!({
                                "save": listed_save_json(&save),
                                "gam": gam,
                                "sav": sav,
                            })
                        }
                        SaveInfoPart::Gam => {
                            let bytes = read_save_member(&save, "BALDUR.gam")?;
                            serde_json::to_value(parse_gam(
                                &bytes,
                                "BALDUR.GAM",
                                tlk_resolver.as_ref().map(|resolver| resolver as _),
                            )?)?
                        }
                        SaveInfoPart::Sav => {
                            let bytes = read_save_member(&save, "BALDUR.SAV")?;
                            serde_json::to_value(parse_sav(&bytes, "BALDUR.SAV")?)?
                        }
                    };
                    println!("{}", serde_json::to_string_pretty(&value)?);
                }
            }
        }
        Command::SaveAddItem(args) => {
            let installation = GameInstallation::discover(&args.game)?;
            let save = resolve_save_folder(&installation, args.saves_dir.as_deref(), &args.save)?;
            let item_resref = ResRef::new(&args.item)?;
            let member = parse_member_selector(args.member.as_deref())?;
            let slot = parse_slot_choice(&args.slot)?;
            let flags = parse_item_flags(&args.flags)?;
            let item = NewItem {
                resref: item_resref.clone(),
                expiration_time_days: 0,
                charges_1: args.charges,
                charges_2: args.charges2,
                charges_3: args.charges3,
                flags,
            };

            warn_if_item_missing(&installation, &item_resref);

            let target_save = if args.in_place {
                save.path.clone()
            } else {
                let output = args
                    .output
                    .as_ref()
                    .ok_or("--output <DIR> is required unless --in-place is passed")?;
                copy_save_folder(&save.path, output)?;
                output.clone()
            };
            let gam_path = resolve_child_file_case_insensitive(&target_save, "BALDUR.gam")
                .ok_or_else(|| format!("BALDUR.gam not found in {}", target_save.display()))?;

            if args.in_place && args.backup {
                let backup_path = gam_path.with_file_name(format!(
                    "{}.bak",
                    gam_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("BALDUR.gam")
                ));
                fs::copy(&gam_path, &backup_path)?;
            }

            let gam = fs::read(&gam_path)?;
            let result =
                add_item_to_save_gam(&gam, installation.game_variant, member, &item, slot)?;
            fs::write(&gam_path, &result.bytes)?;

            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "save_folder": target_save,
                    "gam_path": gam_path,
                    "member_index": result.member_index,
                    "member_name": result.member_name,
                    "item_resref": result.item_resref,
                    "slot_index": result.slot_index,
                    "new_item_index": result.new_item_index,
                    "old_items_count": result.old_items_count,
                    "new_items_count": result.new_items_count,
                    "byte_delta": result.byte_delta,
                    "in_place": args.in_place,
                    "backup_written": args.in_place && args.backup,
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
        "source_kind": resource.source_kind.as_str(),
        "source_path": resource.source_path,
    })
}

fn listed_save_json(save: &ListedSave) -> serde_json::Value {
    serde_json::json!({
        "folder_name": save.folder_name,
        "display_name": save.display_name,
        "save_dir_kind": save.save_dir_kind.as_str(),
        "path": save.path,
        "has_gam": save.has_gam,
        "has_sav": save.has_sav,
        "portraits": save.portraits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ie_core::{ResourceLinkResolver, SourceKind};
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn verify_smoke_against_ie_game_path_when_set() {
        let Ok(game_path) = std::env::var("IE_GAME_PATH") else {
            return;
        };

        let issues = verify_installation(
            PathBuf::from(game_path),
            ResourceSource::Override,
            VerifyOptions::default(),
        )
        .expect("verify should run against IE_GAME_PATH");

        let mut sorted = issues.clone();
        sorted.sort_by(|left, right| {
            (&left.resource, &left.path, left.issue).cmp(&(
                &right.resource,
                &right.path,
                right.issue,
            ))
        });
        assert_eq!(issues, sorted);
    }

    #[test]
    fn link_resolver_honors_selected_bif_source_when_override_exists() {
        let fixture = TestInstallation::new("link-source-bif");
        fixture.write_archive_install("data/creatures.bif", "KIRINH.CRE", b"CRE BASE");
        fixture.write_override("KIRINH.CRE", b"CRE OVERRIDE");

        let installation =
            GameInstallation::discover(fixture.root()).expect("synthetic installation should work");
        let locator = ResourceLocator::new(installation).expect("KEY should parse");
        let resolver = CliResourceLinkResolver {
            locator: &locator,
            tlk_resolver: None,
            source: ResourceSource::Bif,
        };

        let link = resolver.resolve_resource_link(
            &ResRef::new("KIRINH").expect("resref should parse"),
            ResourceType::Cre,
        );

        assert!(link.exists);
        assert_eq!(link.source_kind, Some(SourceKind::Bif));
        assert!(
            link.source_path
                .expect("link should include source path")
                .ends_with("creatures.bif")
        );
    }

    struct TestInstallation {
        root: PathBuf,
    }

    impl TestInstallation {
        fn new(label: &str) -> Self {
            let mut root = std::env::temp_dir();
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be monotonic")
                .as_nanos();
            root.push(format!(
                "nearinfinity-cli-{label}-{unique}-{}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("temporary installation root should be creatable");
            Self { root }
        }

        fn root(&self) -> &Path {
            &self.root
        }

        fn write_archive_install(
            &self,
            relative_archive_path: &str,
            resource_name: &str,
            resource_bytes: &[u8],
        ) {
            let archive_path = self.root.join(relative_archive_path);
            if let Some(parent) = archive_path.parent() {
                fs::create_dir_all(parent).expect("archive parent should be creatable");
            }
            fs::write(&archive_path, build_biff_archive(resource_bytes))
                .expect("archive should be writable");
            fs::write(
                self.root.join("chitin.key"),
                build_key_file(relative_archive_path, resource_name),
            )
            .expect("chitin.key should be writable");
        }

        fn write_override(&self, resource_name: &str, resource_bytes: &[u8]) {
            let override_dir = self.root.join("override");
            fs::create_dir_all(&override_dir).expect("override dir should be creatable");
            fs::write(override_dir.join(resource_name), resource_bytes)
                .expect("override resource should be writable");
        }
    }

    impl Drop for TestInstallation {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn build_key_file(relative_archive_path: &str, resource_name: &str) -> Vec<u8> {
        let mut archive_name_bytes = relative_archive_path.replace('/', "\\").into_bytes();
        archive_name_bytes.push(0);
        let (resref, extension) = resource_name
            .rsplit_once('.')
            .expect("test resource name should include extension");
        let type_code = match extension.to_ascii_uppercase().as_str() {
            "CRE" => 0x03F1u16,
            other => panic!("unsupported test extension {other}"),
        };
        let resource_locator = 0x0000_0001u32;
        let bif_offset = 24u32;
        let resource_offset = bif_offset + 12;
        let string_offset = resource_offset + 14;

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"KEY V1  ");
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&bif_offset.to_le_bytes());
        bytes.extend_from_slice(&resource_offset.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&string_offset.to_le_bytes());
        bytes.extend_from_slice(&(archive_name_bytes.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&padded_resref(resref));
        bytes.extend_from_slice(&type_code.to_le_bytes());
        bytes.extend_from_slice(&resource_locator.to_le_bytes());
        bytes.extend_from_slice(&archive_name_bytes);
        bytes
    }

    fn build_biff_archive(resource_bytes: &[u8]) -> Vec<u8> {
        let file_entry_offset = 20u32;
        let resource_offset = file_entry_offset + 16;

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"BIFFV1  ");
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&file_entry_offset.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&resource_offset.to_le_bytes());
        bytes.extend_from_slice(&(resource_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&0x03F1u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(resource_bytes);
        bytes
    }

    fn padded_resref(resref: &str) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[..resref.len()].copy_from_slice(resref.as_bytes());
        bytes
    }
}
