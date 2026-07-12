---
type: Requirement
id: REQ-TRS-SUS-LINKS-003
name: "Suspect detection covers all trace-link kinds"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-SUS-LINKS-000]
breakdownAdr: Decisions::SuspectLinksADR
tags:
  - traceability
  - suspect-links
---

The suspect-link mechanism shall apply to **every** trace-link kind, not a subset:

- `verifies`
- `derivedFrom`
- `satisfies`
- `satisfiedBy`
- `refines`
- `implementedBy`
- `supertype`
- `subsets`
- `redefines`
- `breakdownAdr`
- the domain-specific trace links (for example `hazardRef`, `mitigatedBy`, `supports`,
  `evidence`, `confirms`)

For each link kind, a target shall be resolved via the standard resolver (stable-id first,
then qualified name). A single `traceBaselines` map on the source (REQ-TRS-SUS-LINKS-001)
covers targets across all kinds; the key namespace is the target identifier, independent
of which link field referenced it.

Where the same target is referenced by more than one link kind from the same source, one
baseline entry suffices, since the projection hash is a property of the target alone.
