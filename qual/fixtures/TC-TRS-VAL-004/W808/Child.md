---
id: REQ-CHD-001
type: Requirement
name: Child requirement at lower ASIL without decomposition rationale
status: draft
reqDomain: system
asilLevel: B
verificationMethod: test
derivedFrom:
  - REQ-PAR-001
---

This element **shall** satisfy the test condition.
Derived from ASIL D parent but declares only ASIL B — no `breakdownAdr:` to document decomposition — should produce W808.
