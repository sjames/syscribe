---
type: Requirement
id: REQ-TRS-ORDER-001
name: "Generic displayOrder field controls element presentation order"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - ordering
  - authoring
---

Syscribe shall recognise a first-class, optional `displayOrder` frontmatter field on
**any** element. The field carries a numeric value (integer or decimal) used as the
primary sort key when that element is presented alongside its peers in an ordered
output. Elements are ordered by ascending `displayOrder`; an element with no
`displayOrder` sorts **after** every element that declares one, and ties (equal or
absent `displayOrder`) fall back to the element's stable identifier order.

The `displayOrder` field shall govern presentation order in at least the following
outputs:

- the Requirements section of the Markdown validation report,
- the Requirement rows of the coverage matrix (`matrix` command), and
- the containment tree of the web UI.

The value shall accept a decimal so that a new element can be inserted between two
existing elements (for example `displayOrder: 15` between `10` and `20`) without
renumbering its neighbours.

## Rationale

Syscribe derives presentation order from the stable identifier (`REQ-*`, qualified
name) or from file-walk order. For most element types this is adequate, but for
**requirements** the order in which they are *read* is itself information: a
specification is authored as a deliberate narrative — context requirements before the
constraints they scope, general rules before their exceptions — and identifier order
rarely matches that intended reading sequence. Renumbering identifiers to force a
reading order is unacceptable because identifiers are stable and opaque by design
(they must never change once assigned).

A dedicated `displayOrder` field decouples *identity* from *presentation*: identifiers
stay stable and meaningless-by-design, while authors express the intended reading
sequence explicitly. Making the field generic (available on every element, not only
requirements) keeps the schema uniform and lets the same mechanism order any future
ordered view. Choosing a decimal value type lets authors leave gaps (the conventional
`10, 20, 30`) and later insert between neighbours without a cascading renumber.
