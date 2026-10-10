---
type: PlanningItem
id: PI-VRES-257
name: "Requirement-level finding for a failing active verifier (GH #257 a)"
status: done
itemType: bug
achieves: [REQ-TRS-VRES-001]
evidence:
  - ref: TC-TRS-VRES-001
  - path: repo:crates/syscribe-model/tests/requirement_failing_verifier.rs
tags:
  - verification
---

Part (a) of #257. Parts (b) verification-depth counting and (c) verdict next to depth remain open on the issue.
