---
type: Requirement
id: REQ-TRS-REQKIND-001
name: "Requirement kinds that are not architecture-allocatable are exempt from W300 and W302"
status: draft
reqDomain: software
reqClass: system
tags:
  - requirements
---

Requirements whose subject is not a system element (process capability, regulatory compliance, deliverables) shall be expressible and shall not raise architecture-satisfaction warnings (GH #250).

## Behavior

- `requirementKind` accepts three additional values: `process`, `regulatory`, `deliverable`. `E022` still rejects any other value and lists all seven.
- A leaf requirement of one of these kinds at `approved`/`implemented` does not raise `W300` (no satisfying architecture element), because no architecture element can satisfy it.
- Such a requirement does not raise `W302` (`reqDomain: system` at `implemented`/`verified`), because it cannot be refined to hardware or software.
- Every other leaf requirement keeps both warnings. Verification coverage (`W002`/`W003`) is unchanged.
