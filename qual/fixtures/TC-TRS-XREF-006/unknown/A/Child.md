---
type: Requirement
id: REQ-XREF-CHILD-001
title: A child requirement whose derivedFrom is wholly unknown
status: approved
reqDomain: software
derivedFrom:
  - Totally::Unknown
breakdownAdr: Decisions::BreakdownADR
---

derivedFrom names Totally::Unknown, which does not exist and does not start with
the root package name EvRoot. The normal unresolved-reference error fires with no
root-name hint (stripping a non-matching prefix is not attempted).
