---
id: REQ-CHD-001
type: Requirement
name: Child requirement with unresolvable breakdownAdr
status: draft
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-PAR-001
breakdownAdr: ADR-NONEXISTENT-001
---

This element **shall** satisfy the test condition. `breakdownAdr` points to an ADR id that does not exist — should produce E311.
