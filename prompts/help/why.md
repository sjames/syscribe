# why — what requirements an element satisfies

## SYNOPSIS
    syscribe -m <root> why <qname> [--config <C>]

## DESCRIPTION
Reports the requirements a given architecture element satisfies (its `satisfies:`
targets), the upstream side of the trace.

## OPTIONS
    --config <C>    Configuration lens (REQ-TRS-PROJ-001): answer over only the
                    elements active in configuration C (a stored Configuration
                    id/qname or an ad-hoc 'Features::A,Features::B' set).
                    Inactive elements are omitted; if the start element itself
                    is inactive in C the command exits 1 naming it.

## EXAMPLES
    syscribe -m model/ why UAV::Avionics::FlightController

## SEE ALSO
    trace, who-verifies, links
