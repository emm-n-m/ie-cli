use clap::{Args, Parser, Subcommand, ValueEnum};
use ie_core::{
    CreatureResourceLink, ResRef, ResolvedStrRef, ResolverBundle, ResourceLink,
    ResourceLinkResolver, ResourceName, ResourceType,
};
use ie_formats::{
    AreaJson, AreaScalarPatch, CreatureScalarPatch, DialogGraphOptions, DialogGraphStringMode,
    DialogJson, EntranceRegistry, MemberSelector, NewItem, SlotChoice, VerifyCategory, VerifyIssue,
    VerifyOptions, VerifySeverity, add_item_to_save_gam, decode_to_json, dialog_json_to_dot,
    dialog_json_to_mermaid, dialog_jsons_to_dot, dialog_jsons_to_mermaid, filter_issues, parse_are,
    parse_dlg, parse_gam, parse_sav, patch_are_scalars, patch_cre_scalars, verify_are,
};
use ie_io::{
    FileBackedIdsResolver, GameInstallation, ListedResource, ListedSave, ResourceListOptions,
    ResourceLocator, ResourceReader, ResourceSource, TlkResolver, append_tlk_string, list_saves,
    read_save_member, resolve_save_folder,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

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
                    let patches = collect_cre_patches(&args.sets, args.patch_json.as_ref())?;
                    let count = patches.len();
                    let out = patch_cre_scalars(&bytes.bytes, &patches)?;
                    (out, count)
                }
                ResourceType::Are => {
                    let patches = collect_are_patches(&args.sets, args.patch_json.as_ref())?;
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

fn verify_installation(
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

fn collect_followed_dialogs(
    locator: &ResourceLocator,
    reader: &ResourceReader,
    root: &DialogJson,
    max_depth: usize,
    source: ResourceSource,
    tlk_resolver: Option<&TlkResolver>,
) -> Result<Vec<DialogJson>, Box<dyn std::error::Error>> {
    let mut dialogs = vec![root.clone()];
    let mut depths = vec![0usize];
    let mut visited = BTreeSet::from([root.resource_name.to_ascii_uppercase()]);
    let mut cursor = 0usize;

    while cursor < dialogs.len() {
        let depth = depths[cursor];
        let dialog = dialogs[cursor].clone();
        cursor += 1;

        if depth >= max_depth {
            continue;
        }

        for state in &dialog.states {
            for transition in &state.transitions {
                let Some(next_dialog) = transition.next_dialog.as_ref() else {
                    continue;
                };
                let resource_name = format!("{}.DLG", next_dialog.as_str());
                if resource_name.eq_ignore_ascii_case(&dialog.resource_name) {
                    continue;
                }
                let normalized = resource_name.to_ascii_uppercase();
                if visited.contains(&normalized) {
                    continue;
                }

                let parsed = ResourceName::parse(&resource_name)?;
                let Ok(bytes) = reader.read_with_source(locator, &parsed, source) else {
                    continue;
                };
                // A present-but-corrupt extern degrades to a dashed external node
                // (like a read miss) rather than aborting the whole graph — this tool
                // is for inspecting possibly-broken installs.
                let parsed_dialog = match parse_dlg(
                    &bytes.bytes,
                    &bytes.metadata.resource_name,
                    tlk_resolver.map(|resolver| resolver as _),
                ) {
                    Ok(parsed_dialog) => parsed_dialog,
                    Err(err) => {
                        eprintln!(
                            "warning: skipping unparseable extern DLG {resource_name}: {err}"
                        );
                        continue;
                    }
                };

                visited.insert(normalized);
                dialogs.push(parsed_dialog);
                depths.push(depth + 1);
            }
        }
    }

    dialogs.sort_by(|left, right| left.resource_name.cmp(&right.resource_name));
    Ok(dialogs)
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

fn format_verify_issue_text(issue: &VerifyIssue) -> String {
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

fn collect_are_patches(
    sets: &[String],
    patch_json: Option<&PathBuf>,
) -> Result<Vec<AreaScalarPatch>, Box<dyn std::error::Error>> {
    let mut patches = Vec::new();

    for set in sets {
        let (field, value) = set
            .split_once('=')
            .ok_or_else(|| format!("invalid --set value '{set}', expected field=value"))?;
        patches.push(AreaScalarPatch {
            field: field.to_string(),
            value: value.to_string(),
        });
    }

    if let Some(path) = patch_json {
        let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
        match &value {
            serde_json::Value::Object(fields) => {
                patches.extend(fields.iter().map(|(field, v)| AreaScalarPatch {
                    field: field.clone(),
                    value: scalar_json_value_to_string(v),
                }))
            }
            serde_json::Value::Array(rows) => {
                for row in rows {
                    let field = row
                        .get("field")
                        .and_then(serde_json::Value::as_str)
                        .ok_or("ARE patch array entries must include string field")?;
                    let value = row
                        .get("value")
                        .ok_or("ARE patch array entries must include value")?;
                    patches.push(AreaScalarPatch {
                        field: field.to_string(),
                        value: scalar_json_value_to_string(value),
                    });
                }
            }
            _ => return Err("ARE patch JSON must be an object or array".into()),
        }
    }

    Ok(patches)
}

fn collect_cre_patches(
    sets: &[String],
    patch_json: Option<&PathBuf>,
) -> Result<Vec<CreatureScalarPatch>, Box<dyn std::error::Error>> {
    let mut patches = Vec::new();

    for set in sets {
        let (field, value) = set
            .split_once('=')
            .ok_or_else(|| format!("invalid --set value '{set}', expected field=value"))?;
        patches.push(CreatureScalarPatch {
            field: field.to_string(),
            value: value.to_string(),
        });
    }

    if let Some(path) = patch_json {
        let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
        patches.extend(parse_patch_json(&value)?);
    }

    Ok(patches)
}

fn parse_patch_json(
    value: &serde_json::Value,
) -> Result<Vec<CreatureScalarPatch>, Box<dyn std::error::Error>> {
    match value {
        serde_json::Value::Object(fields) => Ok(fields
            .iter()
            .map(|(field, value)| CreatureScalarPatch {
                field: field.clone(),
                value: scalar_json_value_to_string(value),
            })
            .collect()),
        serde_json::Value::Array(rows) => rows
            .iter()
            .map(|row| {
                let field = row
                    .get("field")
                    .and_then(serde_json::Value::as_str)
                    .ok_or("patch array entries must include string field")?;
                let value = row
                    .get("value")
                    .ok_or("patch array entries must include value")?;
                Ok(CreatureScalarPatch {
                    field: field.to_string(),
                    value: scalar_json_value_to_string(value),
                })
            })
            .collect(),
        _ => Err("patch JSON must be an object or array".into()),
    }
}

fn scalar_json_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn parse_member_selector(
    value: Option<&str>,
) -> Result<MemberSelector, Box<dyn std::error::Error>> {
    let Some(value) = value else {
        return Ok(MemberSelector::Index(0));
    };
    if let Ok(index) = value.parse::<usize>() {
        return Ok(MemberSelector::Index(index));
    }
    Ok(MemberSelector::CreResRef(
        ResRef::new(value)?.as_str().to_string(),
    ))
}

fn parse_slot_choice(value: &str) -> Result<SlotChoice, Box<dyn std::error::Error>> {
    if value.eq_ignore_ascii_case("auto") {
        return Ok(SlotChoice::AutoInventory);
    }
    Ok(SlotChoice::Index(value.parse::<usize>()?))
}

fn parse_item_flags(value: &str) -> Result<u32, Box<dyn std::error::Error>> {
    if value.eq_ignore_ascii_case("identified") {
        return Ok(0x0000_0001);
    }
    if value.eq_ignore_ascii_case("none") {
        return Ok(0);
    }
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return Ok(u32::from_str_radix(hex, 16)?);
    }
    Ok(value.parse::<u32>()?)
}

fn warn_if_item_missing(installation: &GameInstallation, item: &ResRef) {
    let Ok(locator) = ResourceLocator::new(installation.clone()) else {
        return;
    };
    let Ok(resource) = ResourceName::parse(format!("{}.ITM", item.as_str())) else {
        return;
    };
    if locator.locate(&resource).is_err() {
        eprintln!(
            "warning: item {}.ITM was not found in the installation; writing resref anyway",
            item.as_str()
        );
    }
}

fn copy_save_folder(source: &Path, target: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if target.exists() {
        return Err(format!("--output already exists: {}", target.display()).into());
    }
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_save_folder(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

fn resolve_child_file_case_insensitive(parent: &Path, file_name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(parent).ok()?;
    for entry in entries.filter_map(Result::ok) {
        if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(file_name)
            && entry.path().is_file()
        {
            return Some(entry.path());
        }
    }
    None
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

#[derive(Debug, Clone)]
struct OverrideShadowReport {
    shadows: Vec<OverrideShadowEntry>,
    override_only: Vec<String>,
    counts: OverrideShadowCounts,
}

#[derive(Debug, Clone)]
struct OverrideShadowEntry {
    resource: String,
    override_sha1: String,
    bif_sha1: String,
    identical: bool,
}

#[derive(Debug, Clone)]
struct OverrideShadowCounts {
    override_total: usize,
    shadowing_bif: usize,
    override_only: usize,
}

fn build_override_shadow_report(
    locator: &ResourceLocator,
    resource_type: Option<String>,
    resource: Option<&str>,
) -> Result<OverrideShadowReport, Box<dyn std::error::Error>> {
    let resource_filter = resource
        .map(ResourceName::parse)
        .transpose()?
        .map(|resource| resource.file_name());
    let overrides = locator.list(ResourceListOptions {
        resource_type: resource_type.clone(),
        name_glob: resource_filter.clone(),
        source: Some(ResourceSource::Override),
    })?;
    let bifs = locator.list(ResourceListOptions {
        resource_type,
        name_glob: resource_filter,
        source: Some(ResourceSource::Bif),
    })?;
    let bif_by_name = bifs
        .into_iter()
        .map(|resource| (resource.resource_name.to_ascii_uppercase(), resource))
        .collect::<BTreeMap<_, _>>();

    let reader = ResourceReader;
    let mut shadows = Vec::new();
    let mut override_only = Vec::new();

    for override_resource in &overrides {
        let key = override_resource.resource_name.to_ascii_uppercase();
        if bif_by_name.contains_key(&key) {
            let resource_name = ResourceName::parse(&override_resource.resource_name)?;
            let override_bytes =
                reader.read_with_source(locator, &resource_name, ResourceSource::Override)?;
            let bif_bytes =
                reader.read_with_source(locator, &resource_name, ResourceSource::Bif)?;
            let override_sha1 = sha1_hex(&override_bytes.bytes);
            let bif_sha1 = sha1_hex(&bif_bytes.bytes);

            shadows.push(OverrideShadowEntry {
                resource: resource_name.file_name(),
                identical: override_sha1 == bif_sha1,
                override_sha1,
                bif_sha1,
            });
        } else {
            override_only.push(override_resource.resource_name.clone());
        }
    }

    shadows.sort_by(|left, right| left.resource.cmp(&right.resource));
    override_only.sort();

    Ok(OverrideShadowReport {
        counts: OverrideShadowCounts {
            override_total: overrides.len(),
            shadowing_bif: shadows.len(),
            override_only: override_only.len(),
        },
        shadows,
        override_only,
    })
}

fn print_override_shadow_report_text(report: &OverrideShadowReport) {
    println!("resource\tstatus\tidentical\toverride_sha1\tbif_sha1");
    for shadow in &report.shadows {
        println!(
            "{}\tshadow\t{}\t{}\t{}",
            shadow.resource, shadow.identical, shadow.override_sha1, shadow.bif_sha1
        );
    }
    for resource in &report.override_only {
        println!("{resource}\toverride_only\t\t\t");
    }
    println!(
        "counts\toverride_total={}\tshadowing_bif={}\toverride_only={}",
        report.counts.override_total, report.counts.shadowing_bif, report.counts.override_only
    );
}

fn override_shadow_report_json(report: &OverrideShadowReport) -> serde_json::Value {
    serde_json::json!({
        "shadows": report.shadows.iter().map(|shadow| {
            serde_json::json!({
                "resource": shadow.resource,
                "in_override": true,
                "in_bif": true,
                "override_sha1": shadow.override_sha1,
                "bif_sha1": shadow.bif_sha1,
                "identical": shadow.identical,
            })
        }).collect::<Vec<_>>(),
        "override_only": report.override_only,
        "counts": {
            "override_total": report.counts.override_total,
            "shadowing_bif": report.counts.shadowing_bif,
            "override_only": report.counts.override_only,
        },
    })
}

#[derive(Debug, Clone)]
enum OverrideReferenceReport {
    Single(OverrideReferenceSingle),
    Set(OverrideReferenceSet),
}

#[derive(Debug, Clone)]
struct OverrideReferenceSingle {
    resource: String,
    status: OverrideReferenceStatus,
    override_sha1: String,
    reference_sha1: String,
}

#[derive(Debug, Clone)]
struct OverrideReferenceSet {
    added: Vec<OverrideReferenceHashEntry>,
    removed: Vec<OverrideReferenceHashEntry>,
    changed: Vec<OverrideReferenceChangedEntry>,
    counts: OverrideReferenceCounts,
}

#[derive(Debug, Clone)]
struct OverrideReferenceHashEntry {
    resource: String,
    sha1: String,
}

#[derive(Debug, Clone)]
struct OverrideReferenceChangedEntry {
    resource: String,
    override_sha1: String,
    reference_sha1: String,
}

#[derive(Debug, Clone)]
struct OverrideReferenceCounts {
    override_total: usize,
    reference_total: usize,
    added: usize,
    removed: usize,
    changed: usize,
    unchanged: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverrideReferenceStatus {
    Match,
    Differ,
}

impl OverrideReferenceStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::Differ => "differ",
        }
    }
}

fn build_override_reference_report(
    locator: &ResourceLocator,
    resource_type: Option<String>,
    resource: Option<&str>,
    against: &Path,
) -> Result<OverrideReferenceReport, Box<dyn std::error::Error>> {
    if against.is_file() {
        let resource = resource
            .ok_or_else(|| "--resource is required when --against points to a file".to_string())?;
        return build_single_override_reference_report(locator, resource, against)
            .map(OverrideReferenceReport::Single);
    }

    if !against.is_dir() {
        return Err(format!("--against path does not exist: {}", against.display()).into());
    }

    build_set_override_reference_report(locator, resource_type, resource, against)
        .map(OverrideReferenceReport::Set)
}

fn build_single_override_reference_report(
    locator: &ResourceLocator,
    resource: &str,
    against: &Path,
) -> Result<OverrideReferenceSingle, Box<dyn std::error::Error>> {
    let resource_name = ResourceName::parse(resource)?;
    let reader = ResourceReader;
    let override_bytes =
        reader.read_with_source(locator, &resource_name, ResourceSource::Override)?;
    let reference_bytes = fs::read(against)?;
    let override_sha1 = sha1_hex(&override_bytes.bytes);
    let reference_sha1 = sha1_hex(&reference_bytes);

    Ok(OverrideReferenceSingle {
        resource: resource_name.file_name(),
        status: if override_sha1 == reference_sha1 {
            OverrideReferenceStatus::Match
        } else {
            OverrideReferenceStatus::Differ
        },
        override_sha1,
        reference_sha1,
    })
}

fn build_set_override_reference_report(
    locator: &ResourceLocator,
    resource_type: Option<String>,
    resource: Option<&str>,
    against: &Path,
) -> Result<OverrideReferenceSet, Box<dyn std::error::Error>> {
    let resource_filter = resource
        .map(ResourceName::parse)
        .transpose()?
        .map(|resource| resource.file_name());
    let overrides = locator.list(ResourceListOptions {
        resource_type: resource_type.clone(),
        name_glob: resource_filter.clone(),
        source: Some(ResourceSource::Override),
    })?;

    let override_hashes = override_hashes(locator, &overrides)?;
    let reference_hashes = reference_hashes(against, resource_type.as_deref(), resource_filter)?;
    let all_resources = override_hashes
        .keys()
        .chain(reference_hashes.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    let mut unchanged = 0usize;

    for resource in all_resources {
        match (
            override_hashes.get(&resource),
            reference_hashes.get(&resource),
        ) {
            (Some(override_sha1), Some(reference_sha1)) if override_sha1 == reference_sha1 => {
                unchanged += 1;
            }
            (Some(override_sha1), Some(reference_sha1)) => {
                changed.push(OverrideReferenceChangedEntry {
                    resource,
                    override_sha1: override_sha1.clone(),
                    reference_sha1: reference_sha1.clone(),
                });
            }
            (Some(override_sha1), None) => {
                added.push(OverrideReferenceHashEntry {
                    resource,
                    sha1: override_sha1.clone(),
                });
            }
            (None, Some(reference_sha1)) => {
                removed.push(OverrideReferenceHashEntry {
                    resource,
                    sha1: reference_sha1.clone(),
                });
            }
            (None, None) => {}
        }
    }

    Ok(OverrideReferenceSet {
        counts: OverrideReferenceCounts {
            override_total: override_hashes.len(),
            reference_total: reference_hashes.len(),
            added: added.len(),
            removed: removed.len(),
            changed: changed.len(),
            unchanged,
        },
        added,
        removed,
        changed,
    })
}

fn override_hashes(
    locator: &ResourceLocator,
    resources: &[ListedResource],
) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let reader = ResourceReader;
    let mut hashes = BTreeMap::new();
    for resource in resources {
        let resource_name = ResourceName::parse(&resource.resource_name)?;
        let bytes = reader.read_with_source(locator, &resource_name, ResourceSource::Override)?;
        hashes.insert(resource_name.file_name(), sha1_hex(&bytes.bytes));
    }
    Ok(hashes)
}

fn reference_hashes(
    against: &Path,
    resource_type: Option<&str>,
    resource_filter: Option<String>,
) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let mut hashes = BTreeMap::new();
    for entry in fs::read_dir(against)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Ok(resource_name) = ResourceName::parse(file_name) else {
            continue;
        };
        if let Some(resource_type) = resource_type
            && !resource_name
                .extension()
                .eq_ignore_ascii_case(resource_type)
        {
            continue;
        }
        if let Some(resource_filter) = resource_filter.as_deref()
            && !resource_name
                .file_name()
                .eq_ignore_ascii_case(resource_filter)
        {
            continue;
        }

        hashes.insert(resource_name.file_name(), sha1_hex(&fs::read(path)?));
    }
    Ok(hashes)
}

fn print_override_reference_report_text(report: &OverrideReferenceReport) {
    match report {
        OverrideReferenceReport::Single(single) => {
            println!(
                "resource\tstatus\toverride_sha1\treference_sha1\n{}\t{}\t{}\t{}",
                single.resource,
                single.status.as_str(),
                single.override_sha1,
                single.reference_sha1
            );
        }
        OverrideReferenceReport::Set(set) => {
            println!("resource\tstatus\toverride_sha1\treference_sha1");
            for entry in &set.added {
                println!("{}\tadded\t{}\t", entry.resource, entry.sha1);
            }
            for entry in &set.removed {
                println!("{}\tremoved\t\t{}", entry.resource, entry.sha1);
            }
            for entry in &set.changed {
                println!(
                    "{}\tchanged\t{}\t{}",
                    entry.resource, entry.override_sha1, entry.reference_sha1
                );
            }
            println!(
                "counts\toverride_total={}\treference_total={}\tadded={}\tremoved={}\tchanged={}\tunchanged={}",
                set.counts.override_total,
                set.counts.reference_total,
                set.counts.added,
                set.counts.removed,
                set.counts.changed,
                set.counts.unchanged
            );
        }
    }
}

fn override_reference_report_json(report: &OverrideReferenceReport) -> serde_json::Value {
    match report {
        OverrideReferenceReport::Single(single) => serde_json::json!({
            "resource": single.resource,
            "status": single.status.as_str(),
            "override_sha1": single.override_sha1,
            "reference_sha1": single.reference_sha1,
        }),
        OverrideReferenceReport::Set(set) => serde_json::json!({
            "added": set.added.iter().map(|entry| {
                serde_json::json!({
                    "resource": entry.resource,
                    "override_sha1": entry.sha1,
                })
            }).collect::<Vec<_>>(),
            "removed": set.removed.iter().map(|entry| {
                serde_json::json!({
                    "resource": entry.resource,
                    "reference_sha1": entry.sha1,
                })
            }).collect::<Vec<_>>(),
            "changed": set.changed.iter().map(|entry| {
                serde_json::json!({
                    "resource": entry.resource,
                    "override_sha1": entry.override_sha1,
                    "reference_sha1": entry.reference_sha1,
                })
            }).collect::<Vec<_>>(),
            "counts": {
                "override_total": set.counts.override_total,
                "reference_total": set.counts.reference_total,
                "added": set.counts.added,
                "removed": set.counts.removed,
                "changed": set.counts.changed,
                "unchanged": set.counts.unchanged,
            },
        }),
    }
}

fn sha1_hex(bytes: &[u8]) -> String {
    sha1(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn sha1(bytes: &[u8]) -> [u8; 20] {
    let mut h0 = 0x6745_2301u32;
    let mut h1 = 0xEFCD_AB89u32;
    let mut h2 = 0x98BA_DCFEu32;
    let mut h3 = 0x1032_5476u32;
    let mut h4 = 0xC3D2_E1F0u32;

    let bit_len = (bytes.len() as u64) * 8;
    let mut padded = bytes.to_vec();
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut words = [0u32; 80];
        for (index, word) in words.iter_mut().take(16).enumerate() {
            let start = index * 4;
            *word = u32::from_be_bytes([
                chunk[start],
                chunk[start + 1],
                chunk[start + 2],
                chunk[start + 3],
            ]);
        }
        for index in 16..80 {
            words[index] =
                (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16])
                    .rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for (index, word) in words.iter().enumerate() {
            let (function, constant) = match index {
                0..=19 => ((b & c) | ((!b) & d), 0x5A82_7999),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(function)
                .wrapping_add(e)
                .wrapping_add(constant)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut digest = [0u8; 20];
    digest[0..4].copy_from_slice(&h0.to_be_bytes());
    digest[4..8].copy_from_slice(&h1.to_be_bytes());
    digest[8..12].copy_from_slice(&h2.to_be_bytes());
    digest[12..16].copy_from_slice(&h3.to_be_bytes());
    digest[16..20].copy_from_slice(&h4.to_be_bytes());
    digest
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

struct CliResourceLinkResolver<'a> {
    locator: &'a ResourceLocator,
    tlk_resolver: Option<&'a TlkResolver>,
    source: ResourceSource,
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

#[cfg(test)]
mod tests {
    use super::*;
    use ie_core::SourceKind;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sha1_hex_matches_known_vectors() {
        // "abc" is a single padded block; the longer cases exercise the
        // multi-block padding path (the usual source of hand-rolled SHA-1 bugs):
        // 56 bytes spills the length word into a second block, 64 is an exact
        // block boundary, and 1000 bytes spans many blocks.
        assert_eq!(sha1_hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(
            sha1_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
        );
        assert_eq!(
            sha1_hex(&[b'a'; 64]),
            "0098ba824b5c16427bd7a1122a5a442a25ec644d"
        );
        assert_eq!(
            sha1_hex(&[b'a'; 1000]),
            "291e9a6c66994949b57ba5e650361e98fc36b1ba"
        );
    }

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
