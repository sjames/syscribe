---
type: PlanningItem
id: PI-TMPL-001
name: "template Requirement: honour configured id prefixes (GH #246)"
status: done
itemType: bug
achieves: [REQ-TRS-TMPL-001]
evidence:
  - ref: TC-TRS-TMPL-001
  - path: repo:crates/syscribe/tests/template_prefix.rs
tags:
  - cli
---

Tracks GH #246. Requirement and test first, then the fix in `query::cmd_template`.
