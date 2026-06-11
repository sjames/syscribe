# matrix — requirement × configuration coverage grid

## SYNOPSIS
    syscribe -m <root> matrix [--json] [--tag <t>]
        [--status <s>] [--gaps-only] [--linked-only]
        [--config <C>] [--plan TP-X]
    syscribe -m <root> matrix --features [--json]
    syscribe -m <root> matrix --allocations [--json]

## DESCRIPTION
Rows are requirements, columns are Configurations; cells are covered (✓), gap
(✗), or N/A (—). When a results sidecar is present, a covered cell shows ✓ for
covered-and-passing vs ▣ for covered-but-not-passing. A per-config and overall
coverage-% footer is printed. With no feature model, falls back to a flat
requirement/test view.

## OPTIONS
    --tag <t>       Restrict rows to requirements tagged t.
    --status <s>    Restrict rows to requirements whose status: equals s.
    --gaps-only     Drop fully-covered and all-N/A rows (keep rows with a gap).
    --linked-only   Ignore ingested results (covered cells stay ✓).
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
