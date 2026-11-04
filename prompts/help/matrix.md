# matrix — requirement × configuration coverage grid

## SYNOPSIS
    syscribe -m <root> matrix [--json] [--tag <t>] [--select <sel>]...
        [--status <s>] [--gaps-only] [--linked-only]
        [--config <C>] [--plan TP-X] [--stage <stage>]
    syscribe -m <root> matrix --rollup [--json] [--tag <t>] [--status <s>] [--config <C>]
    syscribe -m <root> matrix --features [--json]
    syscribe -m <root> matrix --allocations [--json]
    syscribe -m <root> matrix --by environment [--json] [--tag <t>] [--status <s>] [--gaps-only]

## DESCRIPTION
Rows are requirements, columns are Configurations; cells are covered (✓), gap
(✗), or N/A (—). When a results sidecar is present, a covered cell shows ✓ for
covered-and-passing vs ▣ for covered-but-not-passing. A per-config and overall
coverage-% footer is printed. With no feature model, falls back to a flat
requirement/test view. A Configuration with a `derivedFrom:` base contributes
its effective (inherited + own) selection, spec §9.8.

`--stage <stage>` shows the grid as it can be known at that binding stage (ADR-SYS-BT-001):
a feature whose binding window extends beyond the stage and that the column's
`derivedFrom:` chain has not decided by it is unknown there, so a requirement (or
covering test) depending on it is `?` — neither covered nor a gap, and not counted as
applicable. Without `--stage` the grid is as authored.

`--by environment` (GH #437) shows each requirement against each TestEnvironment instead: a cell is the verdict of
the active verifying tests on that rig (`pass`, `FAIL`, `waived`, `gap`, `pass (stale)`) with the latest result's
build commit and date. `gap` is an environment a verifying test declares in `runsOn:` (or its plan does) without a
passing result there, including a retired or out-of-calibration rig. `stale` is a pass older than the model: the
test or requirement changed since ingest (`W091`), or the result's build predates the last git change to either.
The roll-up is "verified on every declared environment". `--gaps-only` keeps gap, failing and stale rows.

`--json` adds a `gaps` list (GH #424): one entry per cell that is `gap`, `planned`, `failing` or `open`, with the
requirement and its file, the configuration (null without a feature model), the `action` (`create-test`,
`activate-test`, `fix-failing-test`, `decide-binding`, `widen-test`), the linked `tests` (id, status, file) and, for
`create-test`, a `suggestedTestId` (the next free id in the requirement's area).

## OPTIONS
    --by environment  Group by TestEnvironment (see above); any other value is a usage error.
    --stage <stage> Show the grid as known at this binding stage (see above); unknown is a usage error.
    --tag <t>       Restrict rows to requirements tagged t.
    --select <sel>  Restrict rows to requirements a stereotype selector matches: `Safety`, or
                    `Safety where integrity >= 2` (a specialisation counts). Repeatable, ANDed.
    --status <s>    Restrict rows to requirements whose status: equals s.
    --gaps-only     Drop fully-covered and all-N/A rows (keep rows with a gap).
    --linked-only   Ignore ingested results (covered cells stay ✓).
    --rollup        One row per requirement: own coverage, coverage below it (leaf
                    descendants active/total/planned), verdict glyph and the
                    [coverage] rule applied, plus a per-reqClass footer. The
                    all-requirements view of `coverage tree`. Honours --tag,
                    --status, --config and --json; --gaps-only, --linked-only are not applied (results are always
                    used), --plan scopes the model first, and --allocations / --features take
                    precedence over it. The [coverage] integrity guard runs over all
                    requirements, whatever the filters keep.
    --features      Show the Feature × Configuration selection grid instead.
    --allocations   Show the MagicGrid Allocation source × target matrix instead:
                    rows are allocation sources, columns are targets, cells mark an
                    Allocation edge (✓). A rollup lists unallocated sources and
                    unused targets; when mg_layer is present, a logical→physical
                    partition is added. Read-only; works regardless of profile.
    --config <C>    Project onto a Configuration (id/qname or 'Features::A,…').
    --plan TP-X     Restrict rows to a TestPlan's in-scope requirements and the
                    TestCase universe to its members; composes with --config.
    --json          Emit the grid (+ a coverage object) as JSON.

## EXAMPLES
    syscribe -m model/ matrix
    syscribe -m model/ matrix --gaps-only --status approved
    syscribe -m model/ matrix --features --json
    syscribe -m model/ matrix --allocations

## SEE ALSO
    audit, trace, validate --config, magicgrid, trade-study
