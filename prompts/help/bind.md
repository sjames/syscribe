# bind — write one binding or selection into a Configuration, checked first

## SYNOPSIS
    syscribe -m <root> bind <Configuration> <feature | feature.parameter> <value> [--dry-run] [--json]

## DESCRIPTION
Writes a `parameterBindings:` value (`Features::Propulsion.motorKV 1000`) or a `features:` selection
(`Features::Telemetry true`) into a Configuration, at the stage that Configuration is bound. The target is a feature
or `<feature>.<parameter>`, by qualified name or `FEAT-*` id; a name the model does not have is refused with a
suggestion. A selection takes `true` or `false`; a binding takes a number or text.

The change is **previewed first** on a candidate copy of the model by the guarded-write engine. The model's own
checks decide whether it is acceptable — the parameter's `range` and `enumValues`, the cross-parameter `parameterConstraints:` (`E221`; one still undetermined because parameters are open is allowed), its unit, the stage window it may
be bound in, and the narrowing a `derivedFrom:` chain allows — so the result is exactly what `validate` would say.
The report carries the value it replaces and where that came from, the unified diff, and the errors and warnings the
change would add or resolve. If it would add an error nothing is written and the command exits 1. Otherwise the
file is updated in place (an existing entry keeps its spelling) unless `--dry-run` is given.

The report (`--json`, schema `bind`) carries the previous value, the diff and the errors and warnings added or
resolved. The MCP tool `bind` returns the standard guarded-write result (`written`, `validationDelta`, `diff`,
`reason`) with the same `target`, `previous` and `stage`, and previews unless `dry_run` is false. Use `timeline` to
see what each stage of a `derivedFrom:` chain decided.

## OPTIONS
    --dry-run   preview only; nothing is written (the global flag)
    --json      machine-readable report
Exit 0 on success, 1 when refused or the target is unknown, 2 on a usage error.

## EXAMPLES
    # against the bundled staged-configuration model
    syscribe -m examples/staged-configuration/model/ bind CONF-UAV-DEPLOY-001 Features::Propulsion.motorKV 1100 --dry-run
    syscribe -m examples/staged-configuration/model/ bind CONF-UAV-DEPLOY-001 FEAT-UAV-TELEM true --dry-run --json

## SEE ALSO
    timeline, residual, params, configure, set, validate
