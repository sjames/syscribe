# schema — the published JSON contracts

## SYNOPSIS
    syscribe schema [<name>]

## DESCRIPTION
Every `--json` form a front end depends on — `features`, `feature`, `configure`, `residual`, `matrix`, `diff`,
`feature-check`, `params-check` — has a JSON Schema (draft 2020-12) and carries a `schemaVersion`. With no name
the contracts are listed; with one, its schema is printed. Needs no model.

A new optional property is a compatible change. Removing or retyping a property, or changing its meaning,
bumps `schemaVersion` and is recorded in `docs/releases/json-contract.md`. Schemas do not forbid extra
properties, so a client must ignore ones it does not know. CI validates each command's real output against
its schema.

## EXAMPLES
    syscribe schema
    syscribe schema residual

## SEE ALSO
    capabilities, residual, configure, matrix
