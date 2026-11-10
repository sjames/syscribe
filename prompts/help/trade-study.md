# trade-study — trade studies: TradeStudy elements, or MoE-weighted configuration scoring

## SYNOPSIS
    syscribe -m <root> trade-study [<TRD-id>] [--json]                  # TradeStudy elements (§15)
    syscribe -m <root> trade-study [--json] [--config <id|qname> ...] [--results <dir | CONF=file> ...]   # MagicGrid MoE fallback

## DESCRIPTION

Read-only; never writes to disk. The command has two modes, chosen by the model:

**TradeStudy elements (§15).** When the model contains any `type: TradeStudy`
element (`TRD-*`), the command works on those. With no argument it lists every
study (id, name, status, number of alternatives, complete ✓/✗); with a
`<TRD-id>` it prints that study's full scoring table — each alternative's
min-max **normalised** score per criterion, the **weighted** total
(`criteria[].weight`, `maximize`/`minimize` direction) and its **rank**. `--json`
emits `{tradeStudies:[…]}` for the list, or the study's criteria, per-alternative
normalised scores, totals and ranks for one study. `template TradeStudy` prints a
skeleton; validation codes `E869`/`E870`/`W061`/`W062` cover the element itself.
`--config` does not apply in this mode.

**MagicGrid MoE fallback (REQ-TRS-MG-007).** With no `TradeStudy` element, the
command scores and ranks the model's `Configuration`s against its Measures of
Effectiveness (MagicGrid B4), so the MoEs actually drive solution selection;
available regardless of profile. The rest of this page describes that mode.

## Grid

- **Rows** — every element with `custom_fields: { mg_moe: true }` (labelled by name).
- **Columns** — the model's `Configuration`s (labelled by `name`, falling back to
  id), or the subset named by repeated `--config <id|qname>`. A `--config` that does not resolve to a
  Configuration is a usage error (exit 1).

## Per-cell evaluation

For each (MoE, Configuration) the MoE host's `expression:` right-hand side is
evaluated, resolving each variable from the configuration's `parameterBindings:`
(an exact key wins; otherwise a binding key's final `.`/`::` segment must match,
and only when **exactly one** binding does — a bare token matching two or more
bindings is **ambiguous**). A variable that does not resolve (or is ambiguous)
makes the cell **unevaluable** → printed `n/a` and excluded from that column's
weight normalisation.

The value is normalised to a score in `[0,1]`:

- `maximize`: `clamp((value − threshold) / (objective − threshold), 0, 1)`
- `minimize`: `clamp((threshold − value) / (threshold − objective), 0, 1)`

A value worse than `mg_moe_threshold` scores `0` **and** is a **threshold
violation** (a knock-out). Each cell prints `VALUE (SCORE)`: the value as measured (not rounded, e.g. `2.966666666666667 (0.52)`) and the score to two decimals, with
` !` appended on a violation.

## Measured values (GH #467)

A MoE can rank Configurations on the measurements recorded for them, not only on their
`parameterBindings:`. Ingest per Configuration — `ingest-results --format records --config <CONF>
<file>` — and name the measurement on the MoE host:

    mg_moe_measurement: closeTime     # the recorded measurement name
    mg_moe_aggregate: max             # mean | max | p95 | last (default last = latest by finish time)
    mg_moe_unit: ms                   # the unit the bounds are in; each recorded value is converted to it
    mg_moe_source: measurement        # score the recorded value (default `expression` scores the prediction)

Only records ingested with `--config` count for that Configuration; unscoped records do not. The other
basis is shown beside the scored one, `1650 (0.65) [pred 1500]` or `1500 (0.50) [meas 2300]`; a cell with
nothing recorded is `n/a`, and a Configuration with no evaluable cell is never the `WINNER`.

Recorded evidence knocks a cell out whichever value is scored: a `fail` verdict on a record that carries the
measurement, or a measurement that breaks its own `limit`, makes the Configuration `FAIL`, in addition to the
threshold rule. A `Knock-outs:` list under the table says why. A recorded value that cannot be converted to
the unit (a different dimension) is reported there and the cell is `n/a`. `--json` gives each cell
`predicted`, `measured` (`value`, `unit`, `aggregate`, `recordCount`, the contributing `records` and the
`evidence` they came from) and `violationReasons`. `MG033` checks the fields.

**Records from outside the model (`--results`).** `--results <dir>` reads `<Configuration id>.json` for each
compared Configuration, and `--results <Configuration>=<file>` one file (repeatable; a named file wins over the
directory). Each file is the records format of `ingest-results --format records`, parsed, validated and joined to
the model's TestCases as at ingest; a record naming no TestCase is ignored and counted. The files replace the
sidecar for the Configurations they cover, never merged with it: one the directory has no file for has **no
evidence** (`n/a`), and one not covered reads the sidecar. So a run kept as a CI artefact can be ranked, and two
runs compared by running the command twice, with nothing ingested or written. A text `Recorded evidence
(--results):` list and each cell's `evidence` (`path`, `hash` = `blake3:` of the file, `count`, `unmatched`) name
exactly what was used. Two limits: the file is not sealed by the model, and its records are stamped against the
model as it is now, so a TestCase or requirement changed since the run is **not** flagged (`W091` applies only to
ingested records). The sidecar's `evidence` describes the Configuration's last records ingest, which understates
the sources when several ingests contributed; `recordCount` and `records` are always exact. `ingest-results
--run` keeps verdicts, not measurements, so name the run's files here instead.

## Rollup

A footer reports each configuration's weighted total (`mg_moe_weight × score`,
weights renormalised to sum 1 over the evaluable rows). The top-scoring
non-failing configuration is marked `WINNER`; any configuration with a threshold
violation is marked `FAIL`.

`--json` emits the full grid (values, scores, weighted contributions) and the
per-configuration `rollup`.

## EXAMPLES
    # MoE fallback, against the bundled MagicGrid model (model_mg/)
    syscribe -m model_mg/ trade-study
    syscribe -m model_mg/ trade-study --config CONF-EVCS-001 --config CONF-EVCS-002
    syscribe -m model_mg/ trade-study --json
    # TradeStudy elements (no bundled model carries one yet)
    syscribe -m <root> trade-study
    syscribe -m <root> trade-study TRD-PROPULSION-001 --json

## SEE ALSO
    matrix (Requirement × Configuration), matrix --allocations, magicgrid,
    template TradeStudy
