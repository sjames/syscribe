# link-types — list the project's user-defined link types and their rules

## SYNOPSIS
    syscribe -m <root> link-types [--json]

## DESCRIPTION
Lists every valid link type declared in `[linkTypes.<name>]` tables of
`<model_root>/.syscribe.toml` (ADR-SYS-LINKTYPE-001) with its description,
`inverse`, source → target types, `cardinality`, `acyclic`/`suspect` settings, the
built-in link it `extends` (with the `relax`ed codes and `coverage`), and the number
of instances authored in the model. Read-only.

Run this before authoring `links:` — the vocabulary is per-project, and a
`links:` key that is not a declared type is error `E630`. An entry reported as
`W630` (malformed declaration) is ignored and not listed.

With no link types declared it says so, shows how to declare one, and exits 0.

Declaration keys (camelCase or snake_case):
    description   prose
    inverse       reverse-direction name (lowerCamel)
    sourceTypes   element types allowed to hold the link (default: any)
    targetTypes   element types allowed as targets (default: any)
    cardinality   targets per source: "N" | "N..M" | "N..*" (default "0..*")
    acyclic       reject cycles of this type (E636; default false)
    suspect       participate in suspect-link detection (default true)
    extends       satisfies | verifies | derivedFrom | refines
    relax         codes not raised for this type's instances (base-specific)
    coverage      count toward the base's reverse index / coverage (default true)

## OPTIONS
    --json   {"linkTypes": [{name, description, inverse, extends, relax, coverage,
              sourceTypes, targetTypes, cardinality, acyclic, suspect, count}]}

## EXAMPLES
    syscribe -m model/ link-types
    syscribe -m model/ link-types --json

## SEE ALSO
    follow, links, validate, mcp
