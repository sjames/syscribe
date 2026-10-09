# zones — list IEC 62443 security zones and SL coverage

## SYNOPSIS
    syscribe -m <root> zones [--coverage] [--json] [--format dot|mermaid|plantuml|svg]

## DESCRIPTION
Lists `Zone` elements (§13) with their `targetSL` / `achievedSL`, member count, and SL gap
status (a gap is `achievedSL < targetSL`). `--coverage` prints a Zone × SecurityControl
cross-table: a control counts for a zone when it is allocated (an `Allocation` element or
`allocatedTo:`) to one of the zone's parts (`members:` or `inZone:`), to the zone or to a
conduit touching it, or when a touching conduit names it in `implementedBy:`. `--format`
draws the zones as compound nodes holding their parts and controls, and the conduits as
edges labelled `CD-… · SL achieved/required` (weak ones red, heavy and dashed).

## OPTIONS
    --coverage   Zone × SecurityControl coverage cross-table.
    --json       Emit JSON.
    --format <dot|mermaid|plantuml|svg>
                 Draw the whole model's zones and conduits (the ZoneConduit diagram
                 kind) instead of the table; exit 1 when there are none.

## EXAMPLES
    syscribe -m model_auto/ zones
    syscribe -m model_auto/ zones --coverage
    syscribe -m model_auto/ zones --format mermaid

## SEE ALSO
    conduits, cyber-risk, diagram
