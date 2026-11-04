# timeline — what each stage of a derivedFrom chain decided

## SYNOPSIS
    syscribe -m <root> timeline <Configuration> [--json]

## DESCRIPTION
For a staged Configuration, the chain of Configurations it is derived from (root first), what each one decided itself
(its own `features:` selections and `parameterBindings:` values) and the stage it is bound at, then the effective
values with their provenance — the source (`own`, `inherited from …`, `default`, `fixed`, `bindTo`, …) and the stage
that bound them — and what is **still open**, grouped by the stage it can still be bound at (the `residual` groups).

Use it to answer "who decided this, and when?" and to choose what to `bind` next. The JSON (schema `timeline`) is the
payload of the MCP tool `timeline` and the LSP request `syscribe/timeline`.

## OPTIONS
    --json   machine-readable output
An unknown Configuration exits 1.

## EXAMPLES
    syscribe -m examples/staged-configuration/model/ timeline CONF-UAV-DEPLOY-001
    syscribe -m examples/staged-configuration/model/ timeline CONF-UAV-DEPLOY-001 --json

## SEE ALSO
    bind, residual, configure, params
