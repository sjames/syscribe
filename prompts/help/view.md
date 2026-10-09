# view — materialise a View: the elements its `expose:` selects

## SYNOPSIS
    syscribe -m <root> view render <View|ViewDef> [--format tree|table|json|mermaid]

## DESCRIPTION
A `View` (or `ViewDef`) declares what it shows in `expose:` — a qualified name, an
import pattern (`Pkg::*` direct members, `Pkg::**` the whole subtree) or a map
`{target, isRecursive, filter}` — and may carry a `filterCondition:`. `view render`
evaluates that against the model and prints the selected elements. A view also
inherits the `expose:`/`filterCondition:` of its `typedBy:` ViewDef.

Filters understand `@MetadataName` (the element carries that metadata application),
`not`, `and`, `or`; any other clause is reported on stderr and treated as true.
Unresolved expose targets are reported on stderr (and as `W502` by `validate`).

The default format follows the view's `rendering:` — `asTreeDiagram` → tree,
`asTableView`/`asElementTable` → table, `asInterconnectionDiagram` → mermaid;
otherwise tree. Read-only.

## OPTIONS
    --format tree|table|json|mermaid   output form (default from `rendering:`)

## EXAMPLES
    syscribe -m model/ view render Views::SafetyView
    syscribe -m model/ view render Views::SafetyView --format table

## SEE ALSO
    show, diagram, validate
