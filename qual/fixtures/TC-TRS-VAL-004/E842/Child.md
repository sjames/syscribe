---
id: REQ-CHD-001
type: Requirement
name: Child requirement missing ASIL inheritance
status: draft
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-PAR-001
breakdownAdr: ADR-TST-001
---

This element **shall** satisfy the test condition. Derived from ASIL C parent but declares no `asilLevel:` — should produce E842.
