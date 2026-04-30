# Format References

`iecli` uses two main external references when implementing and validating resource formats:

- [IESDP](https://gibberlings3.github.io/iesdp/) is the primary specification reference for Infinity Engine file-format layouts, offsets, field widths, and known enum values.
- [Near Infinity](https://github.com/Argent77/NearInfinity) is the behavioral comparison target for parser behavior, loader behavior, and real-resource validation.



When a task needs format details:

- start with the relevant IESDP page
- cite the page or offset table in the implementation notes, test comments, or PR body when it drove a parser decision
- preserve raw values when IESDP is ambiguous or incomplete

When a task needs Near Infinity for comparison:

- use a separate local checkout or archive of Near Infinity
- inspect the relevant code path there
- compare behavior against real game resources
- encode the decision in a Rust test or docs note

This repository should not depend on the vendored Java tree for building or testing the Rust workspace.
