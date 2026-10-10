---
type: PlanningItem
id: PI-SUSLINK-251
name: "Suspect projection excludes workflow state (GH #251)"
status: done
itemType: bug
achieves: [REQ-TRS-SUS-LINKS-002]
evidence:
  - ref: TC-TRS-SUS-LINKS-002
  - path: repo:crates/syscribe-model/tests/suspect_workflow_fields.rs
  - path: repo:crates/syscribe/tests/suspect_links.rs
tags:
  - suspect-links
---

`status`, `claimedBy`, `claimedAt` and `assignedTo` no longer feed the suspect-link projection. Requirement and test case amended first. Existing baselines hash a different surface, so the repo's three own baselines were re-accepted.
