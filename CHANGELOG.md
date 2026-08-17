# Changelog

Notable changes per release. This file is the source for GitHub release notes:
the `Release` workflow extracts the section matching the pushed tag.

Versions before `0.3.0-rc.1` shipped without notes; their entries here were
reconstructed from history and are summaries rather than contemporaneous records.

## [0.3.0-rc.1] - 2026-08-17

The theme is validation. All four Enhanced Edition titles now have real-install
coverage, both install shapes do, and the JSON output finally has tests that
notice when it changes.

### Added

- **DLC archive mounting.** A default Steam BGEE+SoD install keeps the expansion
  packed in `dlc/sod-dlc.zip`, and every resource inside it previously resolved as
  not-found *silently*, with SoD strrefs out of range. The loader now mounts
  `dlc/*.zip` read-only: a DLC's own KEY is merged into the resource index,
  KEY-backed BIFFs resolve from inside the zip, game overrides outrank DLC
  overrides, later-sorted DLCs win among themselves, and the largest matching DLC
  TLK is selected. `source_kind: "dlc"` and a `zip!interior/path` source identify
  packed resources. DlcMerger is neither required nor emulated. Specified in
  [docs/SPEC_DLC_MOUNTING.md](docs/SPEC_DLC_MOUNTING.md).
- **CHR decoding and patching.** A CHR is a wrapper around a complete CRE, not a
  creature file, so every CHR in every install was undecodable — including the
  base game's own pregenerated characters. It is now its own resource type,
  decoded by locating the embedded CRE at the offset its header records, with the
  version-specific quick-slot region preserved raw. `patch` works on CHRs too.
- **JSON goldens**, in two tiers. Exact-value goldens over synthetic fixtures and
  a synthetic install cover every decoded format, both save formats, and the
  `list` / `locate` / `verify` / `override-diff` outputs; they run in CI with no
  game data. Normalized shape goldens check four real installs per game variant.
  See [docs/GOLDENS.md](docs/GOLDENS.md).
- `locate` reports the detected `game_variant`, so a misdetected install is
  visible rather than silently decoding every effect against the wrong opcode
  table.

### Changed

- `list --format text` quotes any resref that is not plain graphic ASCII. Stock
  BGEE ships `MONKTU 8.DLG`, whose 8-byte padding space falls mid-name, so
  splitting the unquoted listing on whitespace was never safe. `--format json`
  remains the interface for machine consumers.
- `locate` and `dump` report the normalized resource name rather than echoing the
  on-disk filename's casing. Previously the reported name depended on whether the
  filesystem was case-sensitive, which made output non-deterministic across
  machines for the same install.
- Override directories are indexed once at load instead of being rescanned per
  lookup. The old fallback ran a full `read_dir` with a stat per entry for every
  resource that lives in a BIF.

### Fixed

- `verify` no longer reports false positives on stock installs. Two checks flagged
  shipped, working game data — worse than not checking at all, since the
  `explore-dungeon` skill tells users to run it.
- A signature mismatch now says what it actually found (`expected "SPL ", found
  "ITM "`) instead of failing opaquely.

### Validated

- **IWDEE**: whole-install sweep, 5,995 of 5,996 resources decoded. The exception
  is stock `#BONECIR.SPL`, which ships with one corrupt signature byte.
- **BGEE**: whole-install sweep of a heavily modded install (16 WeiDU mods, 7,762
  override files), 10,965 of 10,966 decoded. The exception is `CDDETECT`, which
  stock `chitin.key` indexes as `.SPL` over ITM payload bytes.
- **BGEE+SoD with the DLC still packed**, against a real unmerged install. This
  closes the last install-shape gap: every earlier target was a single merged
  game root.

## [0.2.0] - 2026-07-05

- `ARE` parsing, and `verify` for install-wide cross-resource integrity.
- DLG graph export (`dump --format dot|mermaid`) and `override-diff`.
- Tier 1 scalar patching for `CRE`/`CHR` and `ARE` Travel regions.
- Save inspection (`save-list`, `save-info`) and the scoped PSTEE
  `save-add-item` write.
- `tlk-append`, and agent skills packaged for both Claude Code and Codex.

## [0.1.0-rc.1] - 2026-04-30

First release candidate of the Rust rewrite: game discovery, `chitin.key`
parsing, BIFF/BIF/BIFC reads with override precedence, TLK lookup, and typed JSON
export for `ITM`, `SPL`, `CRE`, `STO`, `DLG`, and `BCS`.
