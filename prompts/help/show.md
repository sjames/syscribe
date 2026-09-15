# show — element details and documentation

## SYNOPSIS
    syscribe -m <root> show <qname|id> [--no-related]

## DESCRIPTION
Prints an element's frontmatter fields (type, status, integrity level, extRef,
domain, …), its inline features, and its documentation body. Accepts a qualified
name (Pkg::Sub::Name) or a stable id (REQ-*, TC-*, SG-*, …).

Ends with a type-appropriate "Related:" footer suggesting the traceability
commands that answer the natural next questions about this same element —
`trace`/`who-verifies`/`impact`/`refs` for a `Requirement`; `impact`/
`connectivity`/`n2`/`refs` for an architecture element (`PartDef`/`Part`/…);
just `impact`/`refs` for everything else. Text-mode only.

## OPTIONS
    --no-related    Suppress the "Related:" footer (scripting/piping).

## EXAMPLES
    syscribe -m model/ show UAV::Avionics::FlightController
    syscribe -m model/ show REQ-UAV-NAV-001
    syscribe -m model/ show REQ-UAV-NAV-001 --no-related

## SEE ALSO
    links, refs, trace, find
