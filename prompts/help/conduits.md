# conduits — list IEC 62443 conduits and SL adequacy

## SYNOPSIS
    syscribe -m <root> conduits [--json]

## DESCRIPTION
Lists `Conduit` elements (§13) with their `fromZone`/`toZone`, `achievedSL`, the required SL
(the higher `targetSL` of the two connected zones), and whether the conduit boundary meets it.

## OPTIONS
    --json   Emit JSON ({ id, name, fromZone, toZone, achievedSL, requiredSL, pass }).

## SEE ALSO
    zones, cyber-risk
