# path-for — file path for an element

## SYNOPSIS
    syscribe -m <root> path-for <qname|id>

## DESCRIPTION
Prints the on-disk file path for an element, given its qualified name or stable id
— handy for editor/script integration. The path is the model root exactly as
passed to `-m` (or discovered) joined with the element's path inside it, so with
a relative root it is relative to the current directory, not absolute
(`syscribe -m model/ path-for UAV::Avionics::FlightController` prints
`model/UAV/Avionics/FlightController.md`).

## EXAMPLES
    syscribe -m model/ path-for UAV::Avionics::FlightController
    syscribe -m model/ path-for REQ-UAV-NAV-001

## SEE ALSO
    show
