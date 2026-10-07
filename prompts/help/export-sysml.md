# export-sysml — export the model as SysML v2 textual notation

## SYNOPSIS
    syscribe -m <root> export-sysml [<package-qname>] [--out <file|dir>]

## DESCRIPTION
One-way, lossy export of native Syscribe elements to SysML v2 textual notation
(ADR-SYS-SYSMLV2-002). Package directories become nested `package`s; part, port,
attribute, connection, interface, item and requirement definitions/usages are rendered
with `:>` (supertype or usage subsets), `:>>` (redefines), `:` (typedBy), `[n]`
(multiplicity), `doc /* */` (body), `satisfy` (satisfies; a package-level
`satisfy R by X;` for non-part elements), `verify` (verifies on requirements) and
`= 5 [kg]` (inline attribute value with unit). Constraint/calc elements carry their
parameters and expression text. ActionDef/Action and StateDef/State bodies (sub-actions, control nodes, successions, loops/if; entry/do/exit, substates, transitions) are written as the statements ingestion reads back; an entry it would not read back identically is a `// ... not exported (<reason>)` comment, never an approximation (a succession whose endpoint step was not exported is commented too). `if`/`while`/`loop`/`for`/`assign`/`terminate` entries with a hand-chosen name are written as `action <name> { <stmt> }`, and compound units as `[N*m]`. Fields with no 0.54 syntax (`via`, `referent`, `valueKind`, `trigger`, `loopKind: until`) travel in an `@SyscribeStep { … }` annotation inside the step. Element types with no mapping (TestCase, ADR, PlanningItem, ...) are written as
`// skipped: <qname> (<type>)` comments and counted. Names that are not basic SysML
identifiers (e.g. REQ-X-001) are single-quoted. Read-only on the model; ingestion of
`.sysml` files is unaffected.

## OPTIONS
    <package-qname>   export only this element's subtree (usage error if unknown)
    --out <file>      write one file instead of stdout
    --out <dir>       (existing directory or trailing /) one <TopLevel>.sysml per top-level element

A one-line exported/skipped summary (ending in a count of behaviour entries degraded to comments) is printed to stderr; the text itself ends in a
`// ---- export summary ----` comment block.

## EXAMPLES
    syscribe -m model/ export-sysml
    syscribe -m model/ export-sysml UAV --out uav.sysml
    syscribe -m model/ export-sysml --out build/sysml/

## SEE ALSO
    sysml, export-reqif, mcp
