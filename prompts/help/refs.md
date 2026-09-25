# refs — what references this element

## SYNOPSIS
    syscribe -m <root> refs <qname|id|module-path> [--config <C>]

## DESCRIPTION
Reports the inbound references to an element, including user-defined links
(`links:` entries of a type declared in `[linkTypes]`, labelled with the type
name). For a Configuration it also lists the TestCases that run in it. Given a source module path, it reports the
architecture element(s) that declare it under implementedBy.

## OPTIONS
    --config <C>    Configuration lens (REQ-TRS-PROJ-001): answer over only the
                    elements active in configuration C (a stored Configuration
                    id/qname or an ad-hoc 'Features::A,Features::B' set).
                    Inactive elements are omitted; if the start element itself
                    is inactive in C the command exits 1 naming it.

## EXAMPLES
    syscribe -m model/ refs Interfaces::PowerPortDef
    syscribe -m model/ refs src/flight_controller.rs

## SEE ALSO
    links, trace, follow
