---
type: Requirement
id: REQ-TRS-VIS-024
name: "The browser adds an existing model element to a manifest diagram as a pinned shape, without creating or changing the element"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - sprotty
---

The diagram toolbar shall offer an **Add existing element** action. It opens a picker that
searches the model's elements by qualified name, name or stable id (a substring match, prefix
matches first, a bounded result list) and lets the user choose one. Choosing an element shall
add a shape for it to the open diagram through a new guarded-write route,
`POST /api/diagrams/shapes/{*qname}` with `{ "ref": <qualified name or id>, "x"?, "y"? }`, which:

- resolves `ref` to a model element, refusing an unresolved one with the reason;
- derives the shape's `kind` from the element's type, a shape id from its qualified name made
  unique within the diagram, and writes `shapes.<id> = {ref, kind}` and a `layout.<id>` pin at
  `x`/`y` (default a cascade position) in one guarded write — the element file itself is never
  touched;
- refuses an element already on the diagram, and refuses a **derived** diagram (one with a
  `subject:` and no `shapes:`, whose content follows the model) with a reason that points at
  `include:`/`exclude:`, since a hand-added shape would turn it into a manifest diagram;
- refuses a target that is not a `Diagram`.

On success the client shall re-open the diagram so the new shape appears laid out with the server's
sizes, pinned where it was placed. `Diagram` elements themselves are not offered by the picker.

## Rationale

Add today only creates new elements. Most diagram work starts from elements that already exist
in the model, and hand-listing them in YAML is exactly what the browser editor should replace.

## Scope

- Relationships between the added element and shapes already on the diagram are not added
  automatically; use Connect, or a derived diagram for generated relationships.
- Converting a derived diagram to a manifest, and editing `include:`/`exclude:` from the browser,
  are separate follow-ons.
