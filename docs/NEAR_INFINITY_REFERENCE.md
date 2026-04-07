# Near Infinity Reference

Near Infinity is still a comparison target for parser behavior and validation.

The legacy Java snapshot was removed from this repository to keep the Rust workspace focused and smaller.

When a task needs Near Infinity for comparison:

- use a separate local checkout or archive of Near Infinity
- inspect the relevant code path there
- compare behavior against real game resources
- encode the decision in a Rust test or docs note

This repository should not depend on the vendored Java tree for building or testing the Rust workspace.
