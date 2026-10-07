---
type: PlanningItem
id: PI-VIS-002
name: "Phase 1 — derived BDD and IBD generators, source selection by frontmatter, include/exclude, W417/W418"
status: done
itemType: feature
achieves: [REQ-TRS-VIS-003, REQ-TRS-VIS-004, REQ-TRS-VIS-005]
evidence:
  - path: repo:crates/syscribe-model/src/vis/derive/bdd.rs
  - path: repo:crates/syscribe-model/src/vis/derive/ibd.rs
  - path: repo:crates/syscribe-model/tests/vis_derive.rs
tags:
  - visualisation
---

`vis::derive::{bdd, ibd}` with golden IR tests, the source-selection rule and the two new
fields in spec §8.16.2, `W417`/`W418` catalogued. Design: `docs/design/visualisation.md` §5.2.
