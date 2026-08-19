# Testing

The default test suite is self-contained. Synthetic resources and a synthetic installation cover
parsers, loaders, command output, exact-value goldens, and error handling without requiring game
data.

```bash
cargo test --workspace
```

## Real-install tests

Real-install tests are env-gated: if a path is unset, that installation's tests return without
doing external I/O. Set only the paths available on the current machine.

| Variable | Installation | Typical coverage |
| --- | --- | --- |
| `IE_GAME_PATH` | BG2EE | historical value checks and smoke tests |
| `IE_BGEE_PATH` | BGEE or BGEE+SoD | packed-DLC smoke, shape goldens, known `verify` errors |
| `IE_IWDEE_PATH` | IWDEE | shape goldens, opcode anchors, clean `verify` baseline |
| `IE_PSTEE_PATH` | PSTEE | shape goldens, ARE/BCS/effect smoke tests |
| `IE_BGEE_SOD_PATH` | BGEE+SoD with packed DLC | DLC mounting acceptance test |

Some older tests also accept the corresponding `*_GAME_PATH` alias. Prefer the names in the table
for new automation.

Example:

```bash
IE_BGEE_PATH=/path/to/bgee \
IE_BGEE_SOD_PATH=/path/to/bgee \
cargo test --workspace
```

The real-install shape test launches the CLI once per sampled resource and can take several minutes
on a Windows-mounted filesystem. Run it directly when validating output compatibility:

```bash
IE_BGEE_PATH=/path/to/bgee cargo test -p iecli --test shape
```

The stock-install `verify` baselines are also intentionally env-gated and may be I/O-heavy:

```bash
IE_BGEE_PATH=/path/to/bgee cargo test -p iecli --test verify
IE_IWDEE_PATH=/path/to/iwdee cargo test -p iecli --test verify
```

### Factual expectations

[`real_resources.json`](../crates/ie-cli/tests/expectations/real_resources.json) records small,
non-localized facts read from real resources at the offsets the relevant IESDP
layout documents. Every case states its provenance; the harness reports that provenance with a failing JSON pointer. It currently
covers ITM, SPL, CRE, STO, DLG, BCS, and ARE, including an external-dialog transition.

```bash
IE_BGEE_PATH=/path/to/bgee cargo test -p iecli --test real_expectations
```

Malformed-input coverage has two layers: targeted truncated-header/table tests beside the parsers,
and a deterministic adversarial-input test that gives every decoder valid signatures with
randomized bodies and asserts that it never panics. The latter runs in the default suite and can be
run directly with `cargo test -p ie-formats --test robustness`.

## Goldens

Exact-value goldens use synthetic fixtures and run in the default suite. Shape goldens sample real
installs and compare observed JSON paths and types against the committed per-variant union. See
[GOLDENS.md](./GOLDENS.md) for the normalization and regeneration rules.

Do not set `UPDATE_GOLDENS` or `UPDATE_SHAPE_GOLDENS` during an ordinary verification run. Golden
updates must be reviewed as interface changes.

## Game-data policy

Do not commit raw game resources, full JSON dumps of game resources, or TLK text dumps. Record
independently verified factual constants as focused assertions instead. The full validation matrix
and validation procedure are in [REGRESSION_PLAN.md](./REGRESSION_PLAN.md).
