---
type: Requirement
id: REQ-TRS-W406-001
name: "W406/W407 are not raised for manifest diagrams that have no inline SVG"
status: draft
reqDomain: software
reqClass: system
tags:
  - validation
  - diagrams
---

A manifest diagram (a non-null `shapes:` in list or mapping form, the same rule that selects the manifest as the diagram's source) renders its SVG from the manifest, so a body with no inline ` ```svg ` block shall not raise `W406`/`W407` (GH #263).

## Behavior

- A diagram with a non-null `shapes:` (list or mapping) and no ` ```svg ` block is skipped by the inline-SVG id check.
- A manifest diagram that does carry an inline ` ```svg ` block is still checked.
- A diagram with no `shapes:` has no ids to compare and raises nothing.
