# Save Item Write

Status: accepted for the first scoped save-write path.

`save-add-item` edits only `BALDUR.gam` and only the embedded CRE for a party member.
The command appends one 20-byte CRE item entry, assigns it to an empty slot, and repairs
stored offsets affected by the splice. It does not edit `BALDUR.SAV`, containers,
stores, globals, journal entries, party gold, or standalone save archives.

The write path is intentionally copy-first: without `--in-place`, the CLI copies the
entire save folder to `--output` and edits the copy. In-place edits write
`BALDUR.gam.bak` by default.

Offset handling uses a table-driven pass over known V2.x header offset fields, not only
fields currently rendered by the reader. For CRE, section offsets at or after the insertion
point shift, except `items_offset` itself because it must continue to point at the first
item when the item table was previously empty. For GAM, header section offsets and embedded
CRE offsets at or after the insertion point shift; this covers the common case where the
next CRE starts exactly where the edited CRE ended.

The GAM table includes party members (`0x20`), party inventory (`0x28`), non-party members
(`0x30`), globals (`0x38`), journal entries (`0x50`), and PSTEE tail section offsets
observed at `0x68`, `0x6C`, and `0x78`. The PSTEE `Quick-Save-4` regression specifically
checks that `0x68`, `0x6C`, and `0x78` shift by 20 after adding `CUBE`; they otherwise point
into stale tail data after the inserted CRE bytes.

Auto slot selection is populated only for PST/PSTEE. It is covered by a real-save
regression against `Quick-Save-4`: slots 20..32 are occupied and slot 33 is the first
empty cell selected for `CUBE`. A Near Infinity visual check is still useful before
calling the slot-map fully validated. Other variants reject `--slot auto` until their
general-inventory ranges are verified against real CREs and Near Infinity. Explicit
slot indices are still allowed when the target slot exists and is empty.
