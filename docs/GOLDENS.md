# JSON Goldens

The exported JSON is the product's interface. Skills, guides, and any downstream
script read it by field name, so a renamed or re-nested field is a breaking
change — and until these tests existed, one passed CI silently.

Goldens come in two kinds because no single kind can cover both failure
directions.

| | synthetic ([formats](../crates/ie-formats/tests/goldens.rs), [commands](../crates/ie-cli/tests/cli_goldens.rs)) | [`ie-cli/tests/shape.rs`](../crates/ie-cli/tests/shape.rs) |
| --- | --- | --- |
| Input | fixtures and a temp install, built byte by byte | real installs |
| Pinned | exact JSON, values included | normalized key paths and types |
| Assertion | equality | observed ⊆ golden |
| Runs in CI | yes | no — env-gated |
| Catches | renames, deletions, re-nesting, reordering, encoding changes | shapes only real data produces |
| Regenerate | `UPDATE_GOLDENS=1` | `UPDATE_SHAPE_GOLDENS=1` |

## Why real installs cannot pin values

This was measured before it was designed, and the measurement is the reason for
the split.

A golden taken from a real install pins that install, not the parser:

- **Mods.** The BGEE reference install carries 16 WeiDU mods over 7,762 override
  files. Its resources are the mods' bytes. Reinstalling in a different order
  changes them, and `tlk-append` shifts strrefs and every resolved string with
  them.
- **Store and patch level.** Steam auto-updates, GOG offline installers lag, and
  Beamdog differs again. Classic CD releases are a different engine generation
  entirely — different CRE versions, no `lang/` folder.
- **Language.** `lang/<locale>/dialog.tlk` changes every resolved string in the
  output.
- **DLC state.** A merged install and a packed one differ in `source_kind` and
  `source_path` for the same resource.

Shape survives all of that. The exploratory measurement that motivated this
design unioned 60 CREs per install and found BG2EE and IWDEE identical across 163
paths, with the 16-mod BGEE install differing from a clean BG2EE by 2 paths — both
of them resources the sample happened not to include.

The committed goldens agree to the same degree, and the residual differences are
now legible rather than mysterious:

```
        standard   shared with iwd   vs iwd   vs pst
CRE        170           166            4       11
ITM        114           111            4        8
ARE        188           162           33       22
```

`CRE` and `ITM` differ by a handful — mostly EFF V2 fields the other install's
sample did not reach. `ARE` differs far more because the variants genuinely differ
there, which is the kind of thing a per-variant golden is supposed to record
rather than paper over.

## The normalizations

Three rules turn a document into a shape. Each exists because the alternative is
a test that fails on data rather than on code.

1. **Array indices collapse to `[]`.** Length is data, not structure.
2. **Empty arrays contribute no path.** A creature carrying no items must not
   read as a different shape from one that does, or the golden encodes which
   sampled creature happened to be holding something.
3. **`null` is a type, not an absence.** A nullable field reads as `str|null`
   rather than splitting into two rival shapes depending on whether the sample
   hit a decodable value.

Resolved strings are pinned by *type*, never by value — `$.header.general_name.text: str`
says the field exists and is a string, and says nothing about the language it was
resolved in.

## The shape file format

One path per line, sorted, with the types observed at that path:

```
$.abilities[].damage_dice: null|str
$.header.price: int
```

Types are a **set**, not a sequence, and the comparison treats them as one: a
golden recording `null|str` is satisfied by a run that only ever saw `str`. One
path must appear on exactly one line — the writer guarantees it, and the reader
unions duplicates rather than letting a later line shadow an earlier one, since
that shadowing silently dropped `null` from five paths and turned a healthy
golden into five phantom shape changes.

## Why the real-install assertion is one-directional

`observed ⊆ golden`. A smaller sample or a thinner mod set can only leave paths
out, never invent one, so neither of those can fail the assertion — while a
renamed, added, or re-nested field produces a path the golden does not know and
fails immediately.

Two costs come with that, and both are real:

- **Deletions are invisible.** Remove a field and a subset assertion still passes.
  That is exactly what the synthetic value goldens cover, since they assert
  equality. Neither kind is sufficient alone.
- **A richer install can fail on a gap rather than a regression.** If a mod
  populates a section every reference install leaves empty, that path is
  legitimate and absent from the golden, and the test says so. To keep that rare,
  regeneration samples 400 resources per type where an assertion run samples 150.
  Not every resource: each dump is a process that rediscovers the install and
  reparses a half-megabyte KEY, so a four-install regeneration at 400 already
  takes about 40 minutes.
  The remedy is a regeneration, which unions the new path in — but read it first
  and confirm it is a shape the parser should be producing.

## Coverage and its edges

| Output | Value golden | Shape golden |
| --- | --- | --- |
| `ITM` `SPL` `CRE` `STO` `DLG` `ARE` `BCS` | yes | yes, per variant |
| `CHR` | yes | **no — impossible** |
| `GAM` `SAV` | yes | no — not reached by `dump` |
| `list` `locate` `verify` `override-diff` | yes | no — not a decoded resource |

Shape goldens are keyed per game variant (`standard`, `iwd`, `pst`) because the
variants genuinely differ: `ARE` differs between `standard` and `iwd` by 33 paths.

`CHR` can never get a shape golden. `list` enumerates `override` and KEY-backed
resources, and CHRs live in `characters/`, so the real-install sweep never sees
one. Its synthetic golden is the only thing pinning that shape, which matters
more than usual — CHR nests a whole CRE under a wrapper, and a refactor could
flatten it with nothing else noticing.

The command outputs in the last row are pinned by
[`ie-cli/tests/cli_goldens.rs`](../crates/ie-cli/tests/cli_goldens.rs), which
builds a synthetic install in a temp directory. Building the install rather than
reading one settles what a real install cannot: precedence between an override
and a KEY-backed BIF is *stated* by the fixture, so the golden asserts which one
won. `locate-override.json` and `locate-bif.json` are the same resource resolved
two ways, and they differ exactly where they should:

```
locate-override   "source_kind": "override"  "locator": null
locate-bif        "source_kind": "bif"       "locator": 0
```

Those outputs carry absolute paths, so `<install>` is substituted for the temp
root and `\` normalized to `/` before comparison. The path *tail* is kept, since
that is what tells an override hit from a BIF-backed one.

Still unpinned: `dump --format dot|mermaid`, `save-list`, `tlk`, and the text
(non-JSON) output modes.

## Regenerating

```bash
# synthetic value goldens (no install needed)
UPDATE_GOLDENS=1 cargo test -p ie-formats --test goldens
UPDATE_GOLDENS=1 cargo test -p iecli --test cli_goldens

# shape goldens, per install; unions into the existing file rather than
# replacing it, so running with one install cannot drop another's paths
UPDATE_SHAPE_GOLDENS=1 IE_GAME_PATH=... IE_BGEE_PATH=... \
  IE_IWDEE_PATH=... IE_PSTEE_PATH=... cargo test -p iecli --test shape
```

Read the diff before committing it. A golden updated without being read is worse
than no golden at all: it converts a caught regression into a recorded one.
