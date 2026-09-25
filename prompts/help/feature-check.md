# feature-check — holistic feature-model validation

## SYNOPSIS
    syscribe -m <root> feature-check [--json]
        [--deny <CODES>] [--max-warnings <N>] [--warnings-as-errors]
        [--deep] [--count] [--enumerate] [--prove <dir>]

## DESCRIPTION
Validates the feature model as a whole (separate from `validate`): requires/
excludes resolution and satisfaction, dead/always-on optional features, circular
derivedFrom, bindTo ranges, parameterConstraints, binding-time ordering, and
orphan features (W024). With --deep, runs SAT-backed analysis over the whole
configuration space.

## OPTIONS
    --deny <CODES>         Treat the listed warning codes as gate failures (exit 2);
                           comma-separated or repeated, also --deny=<CODES>.
    --max-warnings <N>     Gate failure (exit 2) when more than N warnings remain.
    --warnings-as-errors   Every warning is a gate failure (exit 2).
    --deep           SAT analysis: void model, dead/core/false-optional features,
                     full-semantics configuration validity, with explanations.
    --count          Number of valid configurations.
    --enumerate      List the valid configurations.
    --prove <dir>    Write DIMACS CNF for each UNSAT finding.
    --json           Machine-readable output.

## EXAMPLES
    syscribe -m model/ feature-check
    syscribe -m model/ feature-check --deep
    syscribe -m model/ feature-check --count
    syscribe -m model/ feature-check --deny W024 --max-warnings 0

## EXIT CODES
    0  no errors and no gate tripped (also: dormant, with a notice, when the model
       has no feature model)
    1  error-severity findings, or a usage error (e.g. a non-integer --max-warnings)
    2  no errors, but a warning gate tripped (--deny / --max-warnings /
       --warnings-as-errors)

## SEE ALSO
    features, feature, configure, matrix --features, spec validation
