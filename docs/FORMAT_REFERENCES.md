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

When IESDP is not enough, the tiebreaker is real game data, not another implementation:

- decode the same resource across several installs and variants
- record the observed values as a real-resource expectation with explicit provenance
  (see [GOLDENS.md](./GOLDENS.md) and [TESTING.md](./TESTING.md))
- if real files disagree with IESDP, the files win — document the discrepancy in
  `docs/decisions/`

Early parser validation also used Near Infinity as a comparison target, and some historical
notes and decision records still refer to it. That is history, not the current process:
new format work is developed against IESDP and pinned by real-resource expectations.
