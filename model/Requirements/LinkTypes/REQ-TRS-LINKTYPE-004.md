---
type: Requirement
id: REQ-TRS-LINKTYPE-004
name: "A link type's declared cardinality shall bound the number of targets per source element"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-LINKTYPE-000]
breakdownAdr: Decisions::LinkTypesADR
tags:
  - link-types
---

A link type's `cardinality` **shall** bound the number of targets each source element holds for
that type. More targets than the upper bound **shall** raise error `E635`. For every element
whose `type:` is in the link type's `sourceTypes`, fewer targets than the lower bound
(including none) **shall** raise warning `W631`, except when the element has `status: draft`.

**Acceptance criteria:** within bounds validates clean; over the upper bound raises `E635`; an
in-scope non-draft element under the lower bound raises `W631`; a draft one does not.
