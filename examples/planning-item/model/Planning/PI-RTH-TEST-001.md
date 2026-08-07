---
type: PlanningItem
id: PI-RTH-TEST-001
name: "Verify RTH behavior end-to-end on hardware"
status: blocked
itemType: task
parent: PI-RTH-001
blockedBy: PI-RTH-IMPL-001
tags:
  - rth
---

End-to-end hardware-in-the-loop verification of the full return-to-home
behavior. `status: blocked`, `blockedBy: PI-RTH-IMPL-001` — the controller
logic isn't done yet (`REQ-TRS-PLANITEM-007`), so there's nothing to run
against the rig regardless of its own availability. Once `PI-RTH-IMPL-001`
reaches `done`, this `blockedBy:` should be cleared or repointed at whatever
blocks next (e.g. the hardware test rig itself, if that becomes the binding
constraint) — the field is a plain, author-maintained cross-reference, not
computed, exactly like `parent:`; leaving it stale after the blocker clears
is a `W308` warning, not an error. A leaf with no `evidence:`, which raises
nothing: `blocked` is not `done`, so `REQ-TRS-PLANITEM-006`'s leaf-evidence
rule does not apply here regardless of evidence content.
