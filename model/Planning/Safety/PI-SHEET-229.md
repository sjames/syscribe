---
type: PlanningItem
id: PI-SHEET-229
name: "Exploded sheet rows inherit common fields (GH #229)"
status: done
itemType: bug
achieves: [REQ-TRS-SHEET-001]
evidence:
  - ref: TC-TRS-SHEET-001
  - path: repo:crates/syscribe-model/tests/sheet_field_inheritance.rs
tags:
  - safety
---

Requirement and test first; implemented in `walker::explode_tara_entries` / `explode_fmea_entries`.
