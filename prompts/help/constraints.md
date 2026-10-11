# constraints — evaluate the model's constraints against its attribute values

## SYNOPSIS
    syscribe -m <root> constraints check [--config <CONF> | --all-configs]
        [--set name=value]... [--strict] [--json]
    syscribe -m <root> constraints calc <CalculationDef> [--config <CONF>]
        [--set name=value]... [--json]
    syscribe -m <root> constraints sweep --param name=<lo>..<hi>:<step> [--param ...]
        [--config <CONF>] [--set name=value]... [--show '<expr>'] [--json]

## DESCRIPTION
`constraints check` evaluates, with the expression language of `syscribe help eval`:

- every `Constraint` usage (a top-level element, or an entry of `features:` with
  `type: Constraint`) whose `typedBy:` is a `ConstraintDef` with an `expression:`;
- every `requires:` and `assume:` clause of a `Requirement`/`RequirementDef`, written as
  an inline `expression:` or as a `typedBy:` constraint definition.

A `ConstraintDef`'s parameters are its `features:` with `direction: in` (or `inout`). A
usage binds them with `bindingConnections: [{left: <parameter>, right: <expression>}]`,
where `right` is any expression: an attribute path (`Pkg::Part::mass`), a literal, or a
calculation. A parameter with no binding takes its own `value:` (`valueKind:
default-bound`), read with its `unit:`; with neither it is unresolved. `isNegated: true`
inverts a check.

Attribute values come from inline `features:` entries with a `value:`
(`<owner qualified name>::<feature>`, also `<owner>.<feature>`) and from `Attribute`
elements. A `unit:` (`SI::kg`, `kg`) scales and dimensions the number; `typedBy:
ISQ::…Value` without a unit means SI base units. A parameter typed `ScalarValues::*` (or
untyped) receives the SI-base number with the dimension dropped; a parameter typed
`ISQ::…Value` keeps its dimension, so a mismatch is a dimension error.

`--config <CONF>` projects the model onto the Configuration and finds its feature
parameters (binding, else fixed value, else default) as `<feature>.<parameter>`
(`Features::Battery.capacity`). `--all-configs` evaluates once per Configuration and prints
a check × configuration table. `--set` binds or overrides any name, last.

Each check is `pass`, `fail`, `unresolved` (a name has no value; they are listed) or
`error` (parse, type, dimension or domain error, with the message).

## OPTIONS
    --config <CONF>    Evaluate on that Configuration (projection + its parameters).
    --all-configs      One column per Configuration.
    --set name=value   Bind or override a name (`Pkg::Part::mass=4.2kg`).
    --strict           Treat unresolved checks as failures.
    --json             {results:[{id, kind, constraint, expression, status, message,
                       bindings, unresolved}], summary} (--all-configs: configurations[]).

Exit 0 when nothing failed or errored; 2 when a check failed or errored (or, with
`--strict`, is unresolved); 1 for a usage error (unknown option or Configuration).

## CALC
`constraints calc` evaluates the `body:` of a `CalculationDef` (`bodyLanguage` absent,
`budget` or `syscribe`) with its `in` parameters from `--set`, else their `value:` defaults
(read with their `unit:`), over the same attribute and configuration environment as `check`.
It prints the result in SI base units (`--json`: {calculation, value, unit, dimension,
bindings}). A missing input or an evaluation error exits 2; something that is not an
evaluable CalculationDef (a ConstraintDef, an `alf`/`kerml`/`ocl` body, an unknown name) is a
usage error (1).

## SWEEP
`constraints sweep` evaluates the whole check set once per point of the grid spanned by the
`--param` ranges: `3..7:1`, `1000kg..2000kg:250kg` (inclusive of the upper bound, positive step
of the bounds' dimension) or a list `1,2,5`; several `--param`s form a Cartesian product, the
last varying fastest. Each point binds its values last. The table (`--json`: points:[{values,
summary, show}]) gives per point the pass/fail/unresolved/error counts and, with `--show`, the
value of an expression at that point. A malformed range, or more than 5000 points, is a usage
error (1); otherwise a sweep exits 0: it reports, it does not gate.
