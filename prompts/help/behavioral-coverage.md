# behavioral-coverage — how completely TestCases exercise behavioral elements

## SYNOPSIS
    syscribe -m <root> behavioral-coverage [<qname>] [--depth <N>] [--format text|json]
                                           [--uncovered-only] [--include-planned]

## DESCRIPTION
Reports how completely the **active** `TestCase` elements exercise the behavioral elements
(`ActionDef`, `Action`, `StateDef`, `State`) in scope (§20). A behavioral element B is
covered by a TestCase TC when any of four paths holds:

1. **Source overlap** — `TC.sourceFile` is under a path in `B.implementedBy`.
2. **Requirement chain** — `TC.verifies` → a requirement satisfied by an element that is
   `typedBy:`/`supertype:` B.
3. **Test function** — a `TC.testFunctions[].file` is under `B.implementedBy`.
4. **Allocation** — the satisfying element is `allocatedTo:` B.

## OPTIONS
    --depth N           Namespace depth limit below <qname>.
    --format F          text (default) | json ({scope, covered, total, coverage_pct, elements}).
    --uncovered-only    Show only uncovered elements.
    --include-planned   Count draft/review/approved TestCases in a separate "planned" column.

## SEE ALSO
    impact, safety-case, testplan
