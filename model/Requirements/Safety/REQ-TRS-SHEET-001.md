---
type: Requirement
id: REQ-TRS-SHEET-001
name: "Exploded TARA and FMEA rows inherit sheet-level common fields"
status: draft
reqDomain: software
reqClass: system
tags:
  - safety
  - security
---

A row exploded from a `TARASheet` (`assetTable`, `damageTable`, `threatTable`, `goalTable`, `controlTable`) or an `FMEASheet` (`entries`) shall inherit the sheet's `responsibility`, `appliesWhen`, `tags` and `status` when the row does not set them, so a common field is stated once on the sheet instead of repeated in every row.

## Behavior

- A row that sets the field itself keeps its own value (override). For `tags`, a row's list replaces the sheet's list, it is not merged.
- Applies to TARA rows. For FMEA entries it applies to `responsibility`, `appliesWhen`, `tags` and `status` taken from the sheet; FMEA rows have no per-row override of the first three.
- `W038` (missing `responsibility`) therefore does not fire once per row when the sheet declares it.
