---
type: Requirement
id: REQ-TRS-VIS-023
name: "The browser creates a new Diagram element from a dialog — derived from a subject or blank — and opens it for editing"
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

The web UI shall offer a **New diagram** control in the tab bar, visible whether or not a diagram
is open. It shall open a dialog collecting:

- a **name**, required, matching the basic-name grammar `^[A-Za-z_][A-Za-z0-9_]*$` (no hyphens or
  spaces, as for every name-identified element);
- a **kind**, one of the kinds that have a generator — `BDD`, `IBD`, `StateMachine`, `Action`,
  `Sequence`, `Requirement`, `Allocation`;
- a **start from** choice: *derive from a subject* (the diagram is two lines of frontmatter and
  follows the model, `REQ-TRS-VIS-003`) or *blank* (an empty manifest, `shapes: {}`, to be built
  with Add and Connect);
- a **subject**, required when deriving and optional when blank, with suggestions drawn from the
  model and limited to the element types spec §8.16.8 allows as a subject for the chosen kind
  (for example a `PartDef` or `Part` for an IBD, a `Package` for a Requirement diagram);
- a **package** to create the diagram in, defaulting to a package named `Diagrams` when one
  exists, else the model root.

Submitting shall create the `Diagram` element through `POST /api/elements` (the shared
guarded-write engine) with `diagramKind`, `subject` when given, and `shapes: {}` for a blank
diagram, then close the dialog, open the new diagram in a tab and let the model browser pick it
up. A refusal (a name already in use, a name outside the grammar, an engine refusal) shall be
shown in the dialog with the engine's reason and shall leave the dialog open and nothing written.
Validation warnings the new element raises (for example `W401` for a subject that resolves to
nothing) shall be shown after creation.

## Rationale

Today a diagram can be edited in the browser but only started by hand-writing a file. Starting
one is the first thing a user does, and for a derived diagram it needs only a kind and a subject.

## Scope

- `Mermaid`, `PlantUML` and `Custom` diagrams are not offered: they have no IR generator and their
  content is the Markdown body.
- *Add an existing element to this diagram* is a separate follow-on, not part of this requirement.
