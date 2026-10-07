# export-sysml — export the model as SysML v2 textual notation

## SYNOPSIS
    syscribe -m <root> export-sysml [<package-qname>] [--out <file|dir>]

## DESCRIPTION
One-way, lossy export of native Syscribe elements to SysML v2 textual notation
(ADR-SYS-SYSMLV2-002). Package directories become nested `package`s; part, port,
attribute, connection, interface, item and requirement definitions/usages are rendered
with `:>` (supertype), `:` (typedBy), `[n]` (multiplicity), `doc /* */` (body) and
`satisfy` (satisfies). Action/state/constraint/calc elements are emitted as header plus
doc. Element types with no mapping (TestCase, ADR, PlanningItem, ...) are written as
`// skipped: <qname> (<type>)` comments and counted. Names that are not basic SysML
identifiers (e.g. REQ-X-001) are single-quoted. Read-only on the model; ingestion of
`.sysml` files is unaffected.

## OPTIONS
    <package-qname>   export only this element's subtree (usage error if unknown)
    --out <file>      write one file instead of stdout
    --out <dir>       (existing directory or trailing /) one <TopLevel>.sysml per top-level element

A one-line exported/skipped summary is printed to stderr; the text itself ends in a
`// ---- export summary ----` comment block.

## EXAMPLES
    syscribe -m model/ export-sysml
    syscribe -m model/ export-sysml UAV --out uav.sysml
    syscribe -m model/ export-sysml --out build/sysml/

## SEE ALSO
    sysml, export-reqif, mcp
