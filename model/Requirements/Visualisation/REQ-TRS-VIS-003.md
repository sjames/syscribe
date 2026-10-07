---
type: Requirement
id: REQ-TRS-VIS-003
name: "A Diagram with a subject and no shapes is derived from the model, with include/exclude filters and deterministic shape ids"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
---

A `Diagram` element that declares a `subject:` and no `shapes:` block shall have its IR
**derived** from the model by the generator registered for its `diagramKind`. A `Diagram` that
declares `shapes:` shall be manifest-sourced (`REQ-TRS-VIS-002`). No additional mode field is
introduced; the presence of `shapes:` selects the source.

A derived diagram may declare `include:` (restrict the content to the named members of the
subject and the edges joining them) and `exclude:` (remove named members from an otherwise
complete view), each a list of qualified names. `include:`/`exclude:` on a manifest diagram,
or an entry naming no member of the subject, shall raise warning `W417`. A `subject:` whose type
is not valid for the diagram's kind under spec §8.16.8 shall raise warning `W418` and the
diagram shall be drawn empty.

Derived shape ids shall be deterministic functions of the depicted element's qualified name
(`s-` followed by the name lower-cased with `::` and every non-alphanumeric run replaced by
`-`), so `layout:` pins keyed by shape id survive regeneration and a renamed element merely
loses its pin.

## Rationale

The point of holding a model is that views follow it. Today adding a part changes no diagram.
Inferring the source from `shapes:` keeps every existing diagram valid as written and makes a
new diagram two lines of frontmatter.

## Scope

- Generators are pure functions of `(subject, elements, resolver, filters)` with golden IR
  snapshot tests on fixture models.
- Spec §8.16.2 gains the source-selection rule and the two fields; `W417`/`W418` are
  catalogued in the same commit as the validator change.
- The generators for BDD and IBD are specified by `REQ-TRS-VIS-004` and `-005`; other kinds are
  `REQ-TRS-VIS-015`.
