# exec-coverage — which architecture elements are modelled and exercised by executable scenarios

## SYNOPSIS
    syscribe exec-coverage [<element>...] [--config <C>] [--min-modelled <percent>] [--require <element>]... [--min-states <percent>] [--min-transitions <percent>] [--min-guards <percent>] [--min-score <percent>] [--mutate] [--json]

## DESCRIPTION
The architecture-element view of the evidence `requirement-coverage` computes (the two share one engine, so their
numbers agree). For every `Part`/`PartDef` whose `domain:` is `software` or `hardware` it shows:

* whether it is **executable** itself (it, or its definition chain, `exhibitsStates:` a machine), **modelled by**
  executable parts that declare `models:` it, or **not modelled**;
* the `models:` correspondence findings (`E991`, `W990`, `W991`) that name it;
* how many executable scenarios have a subject that exercises it, and how many of them passed;
* the mutation scores of the machines that carry or model it;
* how the requirements it `satisfies:` are verified: executable (by scenarios that exercise it), executable through
  another element that satisfies the same requirement, descriptive-only, unverified or not applicable.

Without arguments every software and hardware element is listed; with element names, those are shown with the
requirement ids in each group. The threshold options grade requirements as in `requirement-coverage`.

## OPTIONS
    --config <C>             evaluate the model projected onto this Configuration
    --min-modelled <pct>     gate: exit 2 when fewer of the elements are modelled
    --require <element>      gate: exit 2 unless this element is modelled and a passing executable scenario exercises it (repeatable)
    --min-states <pct>       threshold: states covered of each carrying machine
    --min-transitions <pct>  threshold: transitions covered
    --min-guards <pct>       threshold: guard conditions covered
    --min-score <pct>        threshold: mutation score of each carrying machine
    --mutate                 compute mutation scores now instead of reading the recorded ones
    --json                   machine-readable report

## EXIT STATUS
0 done (and every gate met) · 1 usage, an unknown element or Configuration · 2 a gate missed.
