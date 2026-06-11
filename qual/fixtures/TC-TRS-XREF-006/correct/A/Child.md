---
type: Requirement
id: REQ-XREF-CHILD-001
title: A child requirement whose derivedFrom is correctly written
status: approved
reqDomain: software
derivedFrom:
  - A::B
breakdownAdr: Decisions::BreakdownADR
---

derivedFrom is written correctly as A::B (root name omitted). It resolves, so no
unresolved-reference finding and no hint are produced.
