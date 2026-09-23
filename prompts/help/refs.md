# refs — what references this element

## SYNOPSIS
    syscribe -m <root> refs <qname|id|module-path>

## DESCRIPTION
Reports the inbound references to an element, including user-defined links
(`links:` entries of a type declared in `[linkTypes]`, labelled with the type
name). For a Configuration it also lists the TestCases that run in it. Given a source module path, it reports the
architecture element(s) that declare it under implementedBy.

## EXAMPLES
    syscribe -m model/ refs Interfaces::PowerPortDef
    syscribe -m model/ refs src/flight_controller.rs

## SEE ALSO
    links, trace, follow
