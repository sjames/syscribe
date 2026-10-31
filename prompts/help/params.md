# params — the late-bound parameter file

## SYNOPSIS
    syscribe -m <root> params export --config <Configuration> [--stage <stage>]
                                     [--out FILE] [--schema FILE] [--allow-open]
    syscribe -m <root> params check <FILE> --config <Configuration> [--stage <stage>]
                                     [--deny <CODES>]

## DESCRIPTION
A system whose parameters are bound at `load` or `runtime` needs a file to read them
from. `params export` generates it from the model (ADR-SYS-BT-001, spec §9.7–§9.8), so
the model stays the single source of truth; the file is never read back into it.

The file is YAML with two sections, keyed by stable `FEAT-*` id:

    features:      one boolean per feature toggle whose binding window contains the stage
    parameters:    FEAT-id → parameter name → value, for every parameter of a selected
                   feature whose window contains the stage

Each value is the effective value for the Configuration: its own or inherited binding,
the fixed `value:`, a `bindTo:` propagation, the `default:`, then the `fallback:`. A
trailing comment carries unit, range, source and fallback. A parameter that is
`derivedFrom:` an expression is the system's to compute and is not in the file. The
header names the Configuration, the stage and a `projection-hash` (blake3) of the
whole contract; keys are sorted, so two exports of the same model are byte-identical.

`export` fails, naming each, when a required parameter has no value, default or
fallback; `--allow-open` writes an explicit `null` placeholder marked `OPEN` instead.
`--schema FILE` also writes a JSON Schema (2020-12) for the file.

`check` validates a values file against the same contract: an unknown key (E760), a
wrong type (E761), a different value for a fixed parameter (E204), a value outside
`range:` (E205) or `enumValues:` (E206), a missing or still-open required parameter
(E762), and a file generated from a different model state (W751, gateable with
`--deny W751`). The stage defaults to the one in the file's header. Exit 1 on any error,
2 when a `--deny`'d warning is present.

## OPTIONS
    --config <Configuration>   the Configuration, by CONF-* id or qualified name (required)
    --stage <stage>            the stage (default: the Configuration's bindingStage)
    --out FILE                 write the YAML here instead of stdout
    --schema FILE              also write the JSON Schema
    --allow-open               emit placeholders instead of failing on unresolved parameters
    --deny <CODES>             (check) comma-separated warning codes that fail the check

## EXAMPLES
    syscribe -m examples/staged-configuration/model/ params export --config CONF-UAV-FLIGHT-001
    syscribe -m examples/staged-configuration/model/ params export --config CONF-UAV-DEPLOY-001 --stage runtime

## SEE ALSO
    residual, configure, binding-times, validate
