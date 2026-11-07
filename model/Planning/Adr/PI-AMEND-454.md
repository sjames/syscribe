---
type: PlanningItem
id: PI-AMEND-454
name: "Amendment records for accepted ADRs and confirmed requirements (GH #454)"
status: done
itemType: feature
achieves: [REQ-TRS-AMEND-001, REQ-TRS-AMEND-002, REQ-TRS-AMEND-003, REQ-TRS-AMEND-004]
evidence:
  - ref: TC-TRS-AMEND-001
  - ref: TC-TRS-AMEND-002
  - ref: TC-TRS-AMEND-003
  - ref: TC-TRS-AMEND-004
  - path: repo:crates/syscribe/tests/amendments.rs
tags:
  - adr
  - amendments
---

Requirement and test first. `amendments:` on ADR and Requirement (E330), the opt-in W324 git check and the W325 owner-reference rule, with `set amendment.add`, `show` and `export-html` surfaces.
