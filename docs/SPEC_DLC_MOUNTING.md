# Spec: DLC archive mounting (Enhanced Edition `dlc/*.zip`)

Status: implemented.

Enhanced Edition titles ship expansion content as a zip under `dlc/`. `iecli` has no zip reader, so on
an install that still keeps its DLC packed, every resource inside it resolves as not-found and every
string it adds is out of range — **silently, with no warning that a whole content set is missing**.
That failure shape is the reason this outranks the remaining format work: a tool selling trustworthy
extraction must not quietly under-report.

The findings below were measured against a real BGEE+SoD install; the numbers are reproducible with
the commands in each section.

## 1. Goal

Resolve resources and strings that live inside `dlc/*.zip` exactly as an unpacked install resolves
them, so that a merged install and an unmerged install of the same game produce the same answers.

Non-goals: writing into a DLC, repacking a zip, or emulating DlcMerger.

## 2. What the format actually does (measured, not assumed)

### 2.1 The zip carries no `chitin.key`

`dlc/sod-dlc.zip` contains `data/`, `override/`, `characters/`, and `lang/<locale>/` — but no KEY:

```bash
unzip -l "$BGEE/dlc/sod-dlc.zip" | grep -icE "chitin|\.key"   # 0
```

### 2.2 The base KEY already indexes the DLC's archives

The pre-DlcMerger key names BIFs that, unpacked, exist only inside the zip:

```bash
strings -n 6 "$BGEE/chitin.original" | grep -E "25CREANI|CD4CREA2|CRIWANIM|BDTP_DLC"
#   data/25CREANI.BIF  data/CD4CREA2.BIF  data/CRIWANIM.BIF  data/BDTP_DLC.BIF
```

**This is the load-bearing finding.** KEY parsing needs no change and there is no second resource
index to merge. The DLC is an overlay on *path resolution*: when the KEY asks for `data/X.BIF` and
that path is not on disk, the archive is inside a DLC zip.

### 2.3 The DLC's TLK is an exact prefix-superset

Both `dialog.tlk` files were extracted and compared entry by entry:

| | strings | size |
| --- | --- | --- |
| base `lang/en_US/dialog.tlk` | 34,000 | 4.7 MB |
| DLC `lang/en_US/dialog.tlk` | 71,404 | 8.7 MB |

All 34,000 base strrefs are **byte-identical** in the DLC copy (`34000/34000`, zero mismatches);
strrefs `34000..71403` are SoD-only additions.

That makes the rule safe: preferring the DLC TLK never changes the meaning of an existing strref, it
only extends the range. Any other relationship — renumbering, or a delta needing concatenation —
would have made this far harder, so it is worth re-checking per title rather than assuming.

### 2.4 The symptom this fixes

On a merged install whose TLK was *not* replaced, SoD strrefs are simply unreachable:

```
$ iecli tlk --game "$BGEE" --strref 3000     # base range  -> resolves
$ iecli tlk --game "$BGEE" --strref 50000    # SoD range
strref 50000 is out of range for TLK with 34000 entries
```

The DLC TLK holds `"If I could, I would. But I can't, so..."` at 50000.

## 3. Scope

In scope:

- Read-only resolution of resources whose BIF path lives inside a DLC zip.
- DLC `override/` participating in override precedence.
- DLC `lang/<locale>/dialog.tlk` selection.
- `list` enumerating DLC-backed resources with a source that names the zip.

Out of scope for the first slice: multiple simultaneous DLCs beyond deterministic ordering, and
writing.

## 4. Algorithm

### 4.1 Discovery

At `GameInstallation::discover`, enumerate `dlc/*.zip` in a **deterministic order** (sort by file
name; do not rely on directory iteration order — the override index already learned this lesson).
Ignore `*.disabled`; a real install carries `sod-dlc.disabled` beside the live zip and it must not be
mounted.

### 4.2 Archive path resolution

When resolving a KEY BIF path:

1. Try the path on disk under the game root (current behaviour, unchanged).
2. On miss, try each mounted DLC zip's interior, using the same case-insensitive matching the
   override index uses. KEY paths use `\` separators and are compared case-insensitively.

Read the entry **streamed**, not by inflating the whole archive. See §7.

### 4.3 Override precedence

Order, highest first:

1. game `override/`
2. DLC `override/` (later-sorted DLC wins over earlier, documented explicitly)
3. KEY-backed archives, whether on disk or inside a DLC

The game's own `override/` must outrank a DLC's, or a user's mod stops taking effect once a DLC is
mounted.

### 4.4 TLK selection

If any mounted DLC supplies `lang/<locale>/dialog.tlk`, prefer the one with the **largest string
count** over the base file, and record which file was chosen. §2.3 justifies this; the chosen path
must appear in `tlk` output so a wrong choice is visible rather than silent.

### 4.5 Metadata

`source_kind` needs a value distinguishing DLC-backed resources, and `source_path` should name both
the zip and the interior path (e.g. `dlc/sod-dlc.zip!data/CD4CREA2.BIF`). `resource_name` stays
normalized, per the rule already established for override lookups.

## 5. Testing plan

Everything except §5.3 needs **no game data** and runs anywhere, including a cloud VM.

### 5.1 Synthetic fixtures (primary)

The existing helpers (`build_key_file`, `build_biff_archive`, `build_minimal_tlk`,
`TestInstallation`) already produce synthetic installs. Extend them to write into a zip:

- KEY names `data/items.bif`; the BIF exists **only** inside `dlc/test-dlc.zip` → lookup resolves.
- Same resource present on disk and in a DLC → disk wins.
- Resource in game `override/` and DLC `override/` → game override wins.
- Two DLC zips both providing the resource → deterministic, documented winner.
- `*.disabled` beside a live zip → not mounted.
- DLC supplies a larger `dialog.tlk` → strrefs beyond the base count resolve.

### 5.2 Zip64

Build a synthetic Zip64 archive and assert it reads. See §7 — this is a real risk, not a formality.

### 5.3 Acceptance, needs a real unmerged BGEE+SoD

Gate on a new `IE_BGEE_SOD_PATH` so it skips like every other real-install test:

- A `BD*` SoD resource resolves and decodes.
- `iecli tlk --strref 50000` returns `"If I could, I would. But I can't, so..."`.
- `list --type CRE` count is strictly greater than the same install with the zip removed.

**Oracle available:** a merged install answers these too, so the strongest check is that an unmerged
install and a DlcMerger-merged install of the same game agree resource-for-resource. This machine has
the merged side (plus `sod-dlc.zip` and `chitin.original`); a second machine has an unmerged install.

## 6. Correctness gates

- No behaviour change for installs with no `dlc/` directory — the common case must not regress.
- Mounting must not slow ordinary lookups; the zip central directory is read once, like the override
  index, not per lookup.
- A DLC that cannot be opened is reported, never silently skipped. Silent under-coverage is the exact
  bug being fixed, and a swallowed error recreates it.

## 7. Risks

**Zip64 is the main one.** The real archive is 1,926,932,220 bytes and Python's `zipfile` refuses it
outright while `unzip` reads it fine:

```
BadZipFile: Bad magic number for central directory
```

So the archive brushes the Zip64 boundary rather than being corrupt. Whichever Rust zip crate is
chosen must have verified Zip64 support and streaming entry reads — inflating 1.9 GB to answer one
BIF lookup is not acceptable. Validate the crate against a synthetic Zip64 fixture *before* building
on it.

Secondary:

- Per-title differences. These findings are BGEE+SoD. IWDEE/PSTEE DLC layouts, if any, need the same
  measurement rather than an assumption — particularly §2.3.
- `lang/` locale sets differ between base and DLC; selection must be per-locale.
- DlcMerger merged resources but left the base TLK in place on the reference install, so "merged"
  does not imply "strings merged". Do not treat a merged install as proof of TLK behaviour.

## 8. Deliverables

- Zip mounting in `ie-io` behind the existing locator API.
- New `source_kind` for DLC-backed resources, surfaced by `locate` and `list`.
- Tests per §5.1 and §5.2.
- `IE_BGEE_SOD_PATH`-gated acceptance test per §5.3, added to the table in
  [REGRESSION_PLAN.md](./REGRESSION_PLAN.md).
- Update [PARSER_COVERAGE.md](./PARSER_COVERAGE.md), which currently lists this as an unread
  container.
