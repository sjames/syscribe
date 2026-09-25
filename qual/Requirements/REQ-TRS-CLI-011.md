---
id: REQ-TRS-CLI-011
type: Requirement
name: The list command shall match every element type by its canonical type name
status: draft
reqDomain: software
verificationMethod: test
---

`syscribe list <Type>` **shall** select elements by the type's canonical name —
exactly the `type:` value an author writes (`ElementType::name`), compared
case-insensitively — for **every** element type in the implementation's type
inventory (`ElementType::ALL`). The type label printed by `list`, `show` and
the other query commands **shall** come from the same source, so no concrete
type is reported under a catch-all label (formerly `Other`) or missed by
`list`.

In particular `list Zone`, `list Conduit`, `list TestPlan`, `list ReviewRecord`,
`list TradeStudy` and the SysML types `OccurrenceDef`, `IndividualDef`, `Flow`,
`Case`, `Occurrence`, `Individual`, `Succession` and `Rendering` **shall** list
their elements.

**Source:** GH issue #167 — `list Zone` and `list TestPlan` printed "No …
elements found" while `zones` and `testplan` enumerated those elements, because
the hand-written type-label table lacked the newer types.

**Acceptance criteria:** in a model containing a `Zone`, a `Conduit` and a
`TestPlan`, `list Zone`, `list Conduit` and `list TestPlan` each print the
element and `show` labels the `Zone` with its type name; with a template
skeleton of every type rendered into one model, `list <T>` finds at least one
element for every type `T` in the inventory.
