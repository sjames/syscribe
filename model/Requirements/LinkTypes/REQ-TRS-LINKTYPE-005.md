---
type: Requirement
id: REQ-TRS-LINKTYPE-005
name: "A link type declared acyclic shall reject cycles, including self-links"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-LINKTYPE-000]
breakdownAdr: Decisions::LinkTypesADR
tags:
  - link-types
---

When a link type declares `acyclic = true`, any cycle formed solely by links of that type —
including an element linking to itself — **shall** raise error `E636`, reported once per
cycle and naming its members. Link types that are not acyclic **shall not** be cycle-checked.

**Acceptance criteria:** a cycle in an acyclic type raises `E636`; a self-link raises `E636`;
the same cycle in a non-acyclic type raises nothing.
