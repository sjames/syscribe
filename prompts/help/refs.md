# refs — what references this element

## SYNOPSIS
    syscribe -m <root> refs <qname|id|module-path> [--config <C>]
    syscribe -m <root> refs <REQ-id>#<anchor|N> | <PI-id>#Q<n>
    syscribe -m <root> refs <Elem>.<attr> [--json]

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

**Value references and copied values (GH #452).** `refs <Elem>.<attr>` (an element id
or qualified name, then an attribute: a `features:` entry with a value, an Attribute
child, a tagged value `Stereotype.key` or a budget field such as `ram`) prints the
attribute's current value and unit, then three sections: every value reference
`{{Elem.attr}}` to it (*Uses*); every *literal copy* of the current value — a copied
value written as a number in the body of an element that links to `Elem` (W095 when
`[references] copied_values = true`); and every *stale copy* — a literal equal to a
previous value from the git history of the attribute's file (W096). Replace each copy
with the reference. Values below 10 and years are never searched for.

## OPTIONS
    --config <C>    Configuration lens (REQ-TRS-PROJ-001): answer over only the
                    elements active in configuration C (a stored Configuration
                    id/qname or an ad-hoc 'Features::A,Features::B' set).
                    Inactive elements are omitted; if the start element itself
                    is inactive in C the command exits 1 naming it.
    --json          With <Elem>.<attr> only: one object {attribute, element, path,
                    value, unit, file, uses, copies, stale}; each use/copy has
                    element, file and line, a stale copy also previous and commit.

## EXAMPLES
    syscribe -m model/ refs Interfaces::PowerPortDef
    syscribe -m model/ refs src/flight_controller.rs
    syscribe -m model/ refs REQ-TRS-ITEMREF-005#by-number                  # who cites that item
    syscribe -m examples/planning-item/model/ refs PI-RTH-DOCS-001#Q1      # who cites that question
    syscribe -m model/ refs UAV::Avionics::FlightController.flightMode     # uses and copies of an attribute value
    syscribe -m <root> refs PolicydBounds.snapshotFixedBytes --json        # the same as JSON

## SEE ALSO
    links, trace, follow, check-ref, impact
