---
type: Requirement
id: REQ-TRS-BT-027
name: "A parameter may declare the consumer that reads it"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

A parameter shall accept an optional `consumedBy:` naming the element or source that reads it, resolved like `implementedBy:` (§12.8); `trace` and `params export` shall show the link.

**Acceptance criteria:**
- An unresolvable `consumedBy:` is reported like an unresolvable `implementedBy:` (`W023`; a remote URI or package-registry reference is accepted as external).
- `trace` lists the consumer for the parameter.
