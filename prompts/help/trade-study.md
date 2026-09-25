# trade-study — trade studies: TradeStudy elements, or MoE-weighted configuration scoring

## SYNOPSIS
    syscribe -m <root> trade-study [<TRD-id>] [--json]                  # TradeStudy elements (§15)
    syscribe -m <root> trade-study [--json] [--config <id|qname> ...]   # MagicGrid MoE fallback

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
  id), or the subset named by repeated `--config <id|qname>`.

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
violation** (a knock-out). Each cell prints `VALUE (SCORE)` to two decimals, with
` !` appended on a violation.

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
