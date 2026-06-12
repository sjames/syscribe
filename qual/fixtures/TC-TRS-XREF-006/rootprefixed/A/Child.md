---
type: Requirement
id: REQ-XREF-CHILD-001
name: A child requirement whose derivedFrom wrongly includes the root name
status: approved
reqDomain: software
derivedFrom:
  - EvRoot::A::B
breakdownAdr: Decisions::BreakdownADR
---

derivedFrom is written as EvRoot::A::B — wrongly prefixed with the root package
name. It does not resolve, so the normal unresolved-reference error fires; the
stripped form A::B does resolve, so the finding also carries a hint naming A::B.
