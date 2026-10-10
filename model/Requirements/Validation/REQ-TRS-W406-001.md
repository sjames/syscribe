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

A manifest diagram (`shapes:` entries bound to model elements with `ref:`) renders its SVG from the manifest, so a body with no inline ` ```svg ` block shall not raise `W406`/`W407` (GH #263).

## Behavior

- A diagram whose `shapes:` sequence contains at least one entry with `ref:` and whose body has no ` ```svg ` block is skipped by the inline-SVG id check.
- A manifest diagram that does carry an inline ` ```svg ` block is still checked.
- A hand-drawn diagram (shape ids without `ref:`) with no SVG block is still checked.
