# explain-feature — why a feature is in the state a selection leaves it in

## SYNOPSIS
    syscribe -m <root> explain-feature <feature> [--from <Configuration>]
                                       [--select <feature>=<true|false>]... [--json]

## DESCRIPTION
For one feature (qualified name or `FEAT-*` id) and a partial selection, reports its **state** — `selected`,
`deselected`, `forcedOn`, `forcedOff`, `free`, or `conflict` when the choices clash among themselves — and:

- **because** (for forced states and conflicts): the smallest set of your choices and the model's constraints that
  cause it. A constraint carries its kind (`requires`, `excludes`, `mandatory`, `groupAtLeast`, `groupAtMost`,
  `childImpliesParent`, `rootMandatory`), a label, the feature that declares it and its file, so a front end can link
  to it. The causes are a minimal core of the encoding `feature-check --deep` uses for a void model.
- **toSelect**: what would have to change to select the feature. Dropping any one of the listed choices makes it
  selectable (the list is minimal); `modelLevel` means no choice change helps because the model itself prevents it.
- **parameterConstraints**: with `--from`, the Configuration's parameter constraints that mention one of the feature's
  parameters, with their three-valued state.

`--from` and `--select` seed the choices exactly as for `propagate`. Nothing is written. The JSON (schema
`explain-feature`) is the payload of the MCP tool `explain_feature` and the LSP request `syscribe/explainFeature`.

## OPTIONS
    --from <Configuration>   seed the choices from this Configuration; unknown exits 1
    --select <f>=<bool>      one choice; repeatable; malformed exits 2
    --json                   machine-readable output
An unknown feature exits 1.

## EXAMPLES
    # against the bundled staged-configuration model
    syscribe -m examples/staged-configuration/model/ explain-feature Features::Telemetry --from CONF-UAV-BUILD-001
    syscribe -m examples/staged-configuration/model/ explain-feature FEAT-UAV-PROP --select FEAT-UAV-TELEM=false --json

## SEE ALSO
    propagate, configure, feature-check, feature
