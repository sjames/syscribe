# check-ref — verify a cross-reference resolves

## SYNOPSIS
    syscribe -m <root> check-ref <qname|id>
    syscribe -m <root> check-ref <REQ-id>#<anchor|N> | "<REQ-id> item N" | <PI-id>#Q<n>

## DESCRIPTION
Verifies that a cross-reference resolves to a known element and prints its type —
use it before writing a supertype/typedBy/verifies/derivedFrom value.

It also resolves an item or question reference (GH #453) before you cite it in prose:
`REQ-X#grant-keys` (item anchor), `REQ-X#5` or `"REQ-X item 5"` (item by position)
prints the item's number, anchor and first line; `PI-Y#Q3` prints the question's
status, text and answer. A reference that does not resolve exits 1 with the reason
(e.g. the anchors REQ-X does define) — the same condition `validate` reports as W092.

## EXAMPLES
    syscribe -m model/ check-ref Interfaces::PowerPortDef
    syscribe -m model/ check-ref REQ-UAV-NAV-001
    syscribe -m model/ check-ref REQ-TRS-ITEMREF-005#by-number
    syscribe -m examples/planning-item/model/ check-ref PI-RTH-DOCS-001#Q1

## SEE ALSO
    template, next-id, links
