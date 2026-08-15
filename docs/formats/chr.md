# CHR (saved party member)

Status: header decoded, embedded CRE decoded, quick-slot region preserved raw.

A CHR is not a creature file. It is a small header wrapping a **complete CRE**, which begins at the
offset the header records. Decoding a CHR as a bare CRE fails immediately on the signature check:

```
$ iecli dump --game "$BGEE" --resource 01FIGHT.CHR --format json
resource parsing failure: invalid CRE header: missing CRE signature: expected "CRE ", found "CHR "
```

That was the behaviour until 2026-08-15, because `ResourceType::from_extension` mapped both `CRE` and
`CHR` onto `ResourceType::Cre`. Every CHR in every install was undecodable — 20 of them in a packed
BGEE+SoD install, and the base game's own pregenerated characters among them.

## Header

Measured against `01FIGHT.CHR` (base BGEE, `data/Misc.bif`) and `11FIGHT.CHR`
(`dlc/sod-dlc.zip!data/SODOVR.bif`):

| offset | size | field |
| --- | --- | --- |
| 0x00 | 4 | signature `CHR ` |
| 0x04 | 4 | version, e.g. `V2.0` |
| 0x08 | 32 | character name, NUL-padded |
| 0x28 | 4 | offset to the embedded CRE |
| 0x2C | 4 | length of the embedded CRE |
| 0x30 | .. | quick weapon / spell / item slots |

Both sample files record offset `0x64` (100) and lengths of 1608 and 1712, matching their 1708 and
1812 byte totals exactly.

## Deliberate gaps

**The quick-slot region (0x30 until the CRE offset) is not decoded.** Its layout differs across
`V1.0`, `V2.0`, `V2.2` and `V9.0`, and the only versions available to sample here are `V2.0`. Rather
than invent field names that cannot be checked, the bytes are preserved verbatim as
`header.unknown_header_bytes_0x30`, following the `unknown_header_bytes_0x24` precedent in the STO
decoder. Decoding them needs a Near Infinity comparison against files of each version.

**The header name is a literal string, not a strref.** It is whatever the player typed, so it has no
`dialog.tlk` entry. The embedded CRE's `long_name` / `short_name` are usually `-1` for a player
character, which is why the CHR-level name matters — it is the only name a CHR carries.

## Version coverage

Verified: `V2.0` only, across all 20 CHR files in a packed BGEE+SoD install. `V1.0`, `V2.2` and
`V9.0` are parsed by the same code path — the CRE is located from the header offset rather than from
an assumed header size, so version differences in the quick-slot region cannot move it — but no file
of those versions has been tested.

## Patching

`iecli patch` has always advertised `CRE/CHR`, but routed a CHR to the CRE patcher whole, where it
was rejected on the signature check. Scalar patches now apply to the embedded CRE and splice back at
the same offset. Because scalar edits are fixed-width and in-place, the file length is unchanged and
every header offset stays valid; `patch_chr_scalars` asserts that invariant rather than trusting it.
