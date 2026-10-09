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
parameters and expression text. ActionDef/Action and StateDef/State bodies (sub-actions, control nodes, successions, loops/if; entry/do/exit, substates, transitions) are written as the statements ingestion reads back; an entry it would not read back identically is a `// ... not exported (<reason>)` comment, never an approximation (a succession whose endpoint step was not exported is commented too). `if`/`while`/`loop`/`for`/`assign`/`terminate` entries with a hand-chosen name are written as `action <name> { <stmt> }`, and compound units as `[N*m]`. `via`/`to`, payload-less `trigger`, `referent` and `loopKind: until` are native `accept`/`send`/`assign`/`loop … until` syntax; only `valueKind` of an assign and a trigger beside a payload (no SysML syntax) travel in the deprecated `@SyscribeStep { … }` annotation inside the step. OccurrenceDef/IndividualDef/Occurrence/EventOccurrence and Dependency are written as `occurrence def`/`individual def`/`occurrence`/`event occurrence` (with `snapshot`/`timeslice` for `isPortion: true`)/`dependency … from … to …;`. Every `metadata:` application is written as a metadata annotation in the element's body (`@T { k = "v"; }`, `@n : T;`, `@T about Y;`) and MetadataDef as `metadata def` with its `features:` as attributes; `untilCondition`, control-node `parameters`, a succession's `typedBy` and a PartDef/Part's structural `successionConnections:` are native statements; an expression payload (`send new Cmd() via p;`) is written unquoted when it reads back identically. Enumerations, library packages, allocations, use/analysis/verification cases, views, viewpoints and concerns are exported too, as are package `imports:`/`aliases:`, `ends:`, port conjugation, `bindingConnections`/`flowConnections`, `performs`/`exhibitsStates`, `timeSlices`/`snapshots`, `dependsOn`, `isOrdered`/`isNonunique`/`isReference`/`isDerived`/`isConstant`, `isVariation` (`variation part`), parallel states and action `parameters`. A field with no SysML v2 text form is never dropped silently: it becomes a `// dropped: <field> on <qname>` comment and is counted in the summary. Element types with no mapping (TestCase, ADR, PlanningItem, ...) are written as
`// skipped: <qname> (<type>)` comments and counted. Names that are not basic SysML
identifiers (e.g. REQ-X-001) are single-quoted. Read-only on the model; ingestion of
`.sysml` files is unaffected.

## OPTIONS
    <package-qname>   export only this element's subtree (usage error if unknown)
    --out <file>      write one file instead of stdout
    --out <dir>       (existing directory or trailing /) one <TopLevel>.sysml per top-level element

A one-line exported/skipped/dropped summary (ending in a count of behaviour entries degraded to comments) is printed to stderr; the text itself ends in a
`// ---- export summary ----` comment block.

## EXAMPLES
    syscribe -m model/ export-sysml
    syscribe -m model/ export-sysml UAV --out uav.sysml
    syscribe -m model/ export-sysml --out build/sysml/

## SEE ALSO
    sysml, export-reqif, mcp
