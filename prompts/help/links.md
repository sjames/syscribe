# links — all outbound and inbound relationships

## SYNOPSIS
    syscribe -m <root> links <qname|id> [--config <C>]

## DESCRIPTION
Lists every relationship touching an element in both directions — outbound
(supertype, typedBy, satisfies, verifies, implementedBy, …) and inbound (what
references it). Useful for impact analysis before editing.

User-defined links (`links:`, declared in `[linkTypes]` of `.syscribe.toml`) are
listed too: outbound under the link-type name, inbound under the type's declared
`inverse` (or `<type> (inbound)` when it declares none).

## OPTIONS
    --config <C>    Configuration lens (REQ-TRS-PROJ-001): answer over only the
                    elements active in configuration C (a stored Configuration
                    id/qname or an ad-hoc 'Features::A,Features::B' set).
                    Inactive elements are omitted; if the start element itself
                    is inactive in C the command exits 1 naming it.

## EXAMPLES
    syscribe -m model/ links UAV::Airframe

## SEE ALSO
    refs, connectivity, trace, follow, link-types
