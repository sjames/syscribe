# constraints — evaluate the model's constraints against its attribute values

## SYNOPSIS
    syscribe -m <root> constraints check [--config <CONF> | --all-configs]
        [--set name=value]... [--strict] [--json]

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
