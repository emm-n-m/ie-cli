# Parser Coverage Matrix

Current as of 2026-08-14. This matrix tracks what the typed JSON parsers expose today and what remains shallow or deferred.

`Deferred (offset/count only)` means the parser reads section pointers from the header but does not decode the referenced records. `Other gaps` covers fields or semantics that are omitted, preserved as raw bytes, or only lightly decoded.

Effect opcode names are **game-variant aware**: `ITM`, `SPL`, and `CRE` decode effects against the PST opcode table on a PST install and the standard table elsewhere. `iwd` shares the standard table, confirmed against IWDEE items whose names state the stat they grant. Coverage is partial — 28,885 of 70,696 opcode instances in a whole-install IWDEE sweep resolve to a name — but an unrecognized opcode emits `decoded: null` beside its raw value rather than a wrong name. The variant is detected from install root files, not folder names, and `iecli locate` reports it as `game_variant` so a misdetected install can be caught before its effect names are trusted. Both tables cover the commonly-seen opcodes rather than the full IESDP range; an unrecognized opcode surfaces as a raw value with `decoded: null` rather than a wrong name.

| Resource | Parsed | Deferred (offset/count only) | Other gaps |
| --- | --- | --- | --- |
| ARE | header, actors, regions, entrances | spawn_points, containers, items, vertices, ambients, variables, doors, animations, tiled_objects, automap_notes, projectile_traps; song_entries and rest_interruptions offsets only | actor embedded CRE bytes are not decoded; actor tail bytes preserved raw; region vertices are referenced but not expanded |
| BCS | condition/response blocks, triggers, responses, actions, objects, optional action points, IDS names when available | none | arguments stay positional; no BAF-style pretty rendering; limited semantic typing of action/trigger parameters |
| CRE | header, names, flags, stats, portraits, AC, saves, resistances, thief skills, soundset, class levels, attributes, kit, scripts, classification, actor IDs, death variable, known spells, spell memorization, memorized spells, effects, items, item slots | none | proficiency bytes and object references are preserved as raw byte ranges; EFF V2 prefix/tail bytes preserved raw; item slot names are fixed parser labels |
| DLG | header, states, transitions, state triggers, transition triggers, actions, script text, linked transitions per state, strrefs | none | script text is not parsed as BCS/BAF; external-dialog references are surfaced as resrefs but not recursively resolved |
| ITM | header, strrefs, flags, category, requirements, icons, descriptions, abilities, ability-local effects, global effects | none | ITM effect tail fields after resource are not surfaced; several enum/bitfield values remain raw or shallowly decoded; no opaque byte preservation for omitted fields |
| SPL | header, strrefs, flags, spell type/school/secondary type, extended headers, feature blocks | none | extended header has omitted bytes/fields; enum/opcode coverage is partial; no opaque byte preservation for omitted fields |
| STO | header, store type, flags, room prices, items for sale, drinks, cures, purchased item categories | none | unknown header byte ranges are preserved raw; version-specific store variants are not deeply distinguished |

## Save Formats

Save decoding is reached through `save-info` rather than `dump`, so it is tracked separately.

| Format | Parsed | Deferred (offset/count only) | Other gaps |
| --- | --- | --- | --- |
| GAM | header (game time, gold, reputation, current/main area), party members (name, area, position, orientation, happiness, talk count, party order, vanquished/time-in-party stats), globals | party inventory, non-party members, journal entries, familiar/extra sections | embedded party CRE bytes are located but not decoded in `save-info` output; `v1.1` (classic PST) is not supported, only `v2.0/2.1/2.2` |
| SAV | archive manifest: per-entry filename, resolved resource name/type, compressed and uncompressed sizes | contained resource payloads | entries are inventoried and inflated for sizing, not decoded as their own formats |

See [SAVE_SUPPORT_TODO.md](SAVE_SUPPORT_TODO.md) for the format notes behind these rows, and [SPEC_SAVE_ITEM_WRITE_COMPLETE.md](SPEC_SAVE_ITEM_WRITE_COMPLETE.md) for the gate that keeps save writes PSTEE-only.

## Unsupported Resource Parsers

No typed JSON parser is present yet for common follow-on resource families such as `WED`, `TIS`, `BAM`, `MOS`, `2DA`, `EFF` as a standalone resource, `PRO`, `VEF`, `VVC`, or `IDS` as exported JSON. Some of these are already read indirectly as links or resolver inputs, but they are not first-class `dump` decoders.

Two absences have consequences beyond "that resource type does not dump", so they are worth stating plainly:

- **`WMP` (worldmap).** Without it, area reachability can only be inferred from ARE Travel regions. On BG-family installs most outdoor areas are entered from the worldmap instead, so a zero-inbound-Travel-link area is not necessarily unreachable — 53 BGEE areas are in that state. `verify` therefore cannot currently separate a broken exit a player will walk into from a dangling link in content nothing reaches.
- **DLC archives (`dlc/*.zip`).** Not a resource type but a container: an Enhanced Edition DLC zip is a complete game-root overlay carrying its own `data/*.BIF`, `override/`, and `lang/<locale>/dialog.tlk`. Nothing in the loader reads zips, so on an unmerged install those resources resolve as not-found with no warning, and SoD strrefs fall out of range against the base-game TLK. Installs that ran DlcMerger are unaffected for resources, though not necessarily for strings. Measured behaviour and an implementation plan are in [SPEC_DLC_MOUNTING.md](./SPEC_DLC_MOUNTING.md).
