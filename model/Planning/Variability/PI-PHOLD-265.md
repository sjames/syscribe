---
type: PlanningItem
id: PI-PHOLD-265
name: "Parameter placeholders v1: syntax, substitution, findings (GH #265)"
status: done
itemType: feature
achieves: [REQ-TRS-PHOLD-001]
evidence:
  - ref: TC-TRS-PHOLD-001
  - path: repo:crates/syscribe-model/tests/placeholders.rs
tags:
  - variability
---

Requirement and test first. v1 covers body and `name`; other frontmatter strings, per-configuration suspect semantics (#266), discoverability (#267) and typed fields (#268) follow.
