# Format References

[IESDP](https://gibberlings3.github.io/iesdp/) is the specification reference for Infinity
Engine file-format layouts, offsets, field widths, and known enum values. Work directly from
it rather than from another tool's behavior.

When a task needs format details:

- start with the relevant IESDP page
- cite the page or offset table in the implementation notes, test comments, or PR body when it
  drove a parser decision
- preserve raw values when IESDP is ambiguous or incomplete, and say so in the field name
  (`unknown_*`) or a decision note

IESDP is a specification, not a data source. It tells you where a field sits, how wide it is,
and what its bits and enums mean — it cannot tell you what any particular resource contains.
Every expected *value* therefore comes from a real file:

- read the value out of the resource's own bytes at the offset IESDP names
- decode the same resource across several installs and variants
- record it as a real-resource expectation whose provenance names both halves: the resource and
  install it was read from, and the IESDP table used to read it (see [GOLDENS.md](./GOLDENS.md)
  and [TESTING.md](./TESTING.md))
- when a real file's layout disagrees with IESDP, the file wins — document the discrepancy in
  `docs/decisions/`

Early parser validation read those real files through Near Infinity's GUI rather than through
`dump-raw`, and some historical notes and expectation records still name it that way. It was a
reading tool, never a specification source. New format work reads the bytes directly.
