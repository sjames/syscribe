# residual — the variability a Configuration leaves open

## SYNOPSIS
    syscribe -m <root> residual --config <Configuration> [--json]

## DESCRIPTION
Lists every feature and parameter that declares a binding window
(`bindingTime:`/`latestBindingTime:`, spec §9.7) and that the Configuration's
effective (inherited plus own) selection and bindings do not decide — the
*residual variability* a later stage still has to bind (ADR-SYS-BT-001).

Open items are grouped by the stage at which they can next be bound (the later of
the window's start and the Configuration's `bindingStage:`) and by that stage's
`binder`. An item whose window closed before the Configuration's stage cannot be
bound by any later stage; it is listed apart as *closed*. A feature takes part only
when it is a real choice (not abstract or mandatory), a parameter only when its
feature is selected and it is not fixed or derived. Items with no window are not
listed.

Also shows the state of each applicable `parameterConstraints:` entry on the
partial binding: `satisfied` or `violated` whatever the open parameters become, or
`undetermined`. A staged Configuration is judged this way by `validate` too
(`E221`/`W025` once a constraint is violated regardless of the open parameters).

## OPTIONS
    --config <Configuration>   the Configuration, by CONF-* id or qualified name (required)
    --json                     {configuration, bindingStage, groups[{stage, binder, closed,
                               items[{kind, key, id, name, window{earliest,latest},
                               required, hasDefault}]}], constraints[{id, expression, state,
                               severity, open}]}

## EXAMPLES
    syscribe -m examples/staged-configuration/model/ residual --config CONF-UAV-BUILD-001
    syscribe -m examples/staged-configuration/model/ residual --config CONF-UAV-DEPLOY-001 --json

## SEE ALSO
    configure, binding-times, validate, mcp
