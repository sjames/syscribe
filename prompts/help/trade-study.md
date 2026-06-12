# trade-study — MoE-weighted trade study comparing configurations

```
syscribe -m <root> trade-study [--json] [--config <id|qname> ...]
```

Score and rank the model's `Configuration`s against its Measures of Effectiveness
(MagicGrid B4), so the MoEs actually drive solution selection. Read-only;
available regardless of profile.

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

## Examples

```
syscribe -m model/ trade-study
syscribe -m model/ trade-study --config CONF-A --config CONF-B
syscribe -m model/ trade-study --json
```

See also: `matrix` (Requirement × Configuration), `matrix --allocations`,
`magicgrid`.
