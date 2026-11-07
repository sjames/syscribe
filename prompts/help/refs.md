# refs — what references this element

## SYNOPSIS
    syscribe -m <root> refs <qname|id|module-path> [--config <C>]
    syscribe -m <root> refs <REQ-id>#<anchor|N> | <PI-id>#Q<n>

## DESCRIPTION
Reports the inbound references to an element, including user-defined links
(`links:` entries of a type declared in `[linkTypes]`, labelled with the type
name). For a Configuration it also lists the TestCases that run in it. Given a source module path, it reports the
architecture element(s) that declare it under implementedBy.

**Item and question citations (GH #453).** `refs REQ-X#grant-keys` (an item anchor),
`refs REQ-X#5` (an item by position) or `refs PI-Y#Q3` (a PlanningItem question) lists
the elements whose body cites that item or question, with the citation as written and
its line. An anchor and a number naming the same item match each other. `refs REQ-X`
adds an *Item and question citations* section listing every item-level citation of
REQ-X. See `help Requirement` for the anchor and reference syntax.

## OPTIONS
    --config <C>    Configuration lens (REQ-TRS-PROJ-001): answer over only the
                    elements active in configuration C (a stored Configuration
                    id/qname or an ad-hoc 'Features::A,Features::B' set).
                    Inactive elements are omitted; if the start element itself
                    is inactive in C the command exits 1 naming it.

## EXAMPLES
    syscribe -m model/ refs Interfaces::PowerPortDef
    syscribe -m model/ refs src/flight_controller.rs
    syscribe -m model/ refs REQ-TRS-ITEMREF-005#by-number                  # who cites that item
    syscribe -m examples/planning-item/model/ refs PI-RTH-DOCS-001#Q1      # who cites that question

## SEE ALSO
    links, trace, follow, check-ref, impact
