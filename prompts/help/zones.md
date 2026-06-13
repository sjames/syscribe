# zones — list IEC 62443 security zones and SL coverage

## SYNOPSIS
    syscribe -m <root> zones [--coverage] [--json]

## DESCRIPTION
Lists `Zone` elements (§13) with their `targetSL` / `achievedSL`, member count, and SL gap
status (a gap is `achievedSL < targetSL`). `--coverage` prints a Zone × SecurityControl
cross-table (controls sourced from conduit `implementedBy:` and zone-member SecurityControls).

## OPTIONS
    --coverage   Zone × SecurityControl coverage cross-table.
    --json       Emit JSON.

## SEE ALSO
    conduits, cyber-risk
