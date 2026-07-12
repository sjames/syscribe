---
type: Requirement
id: REQ-TRS-SUS-LINKS-007
name: "Suspicion propagates implicitly, one hop per review"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-SUS-LINKS-000]
breakdownAdr: Decisions::SuspectLinksADR
tags:
  - traceability
  - suspect-links
---

Suspect status shall **not** be eagerly propagated across the trace graph. When a target
changes, only the direct links that reference it become suspect (REQ-TRS-SUS-LINKS-004);
the validator shall not compute or flag a transitive closure of affected links.

Propagation up the trace chain shall instead be **implicit**, one hop per review cycle:

1. Target `B` changes → link `A → B` becomes suspect (W090).
2. A reviewer inspects the change and clears it with `suspect accept` (REQ-TRS-SUS-LINKS-005).
3. If, in response, the reviewer edits source `A` such that `A`'s own **projection**
   (REQ-TRS-SUS-LINKS-002) changes, then any link `C → A` becomes suspect on the next
   validation — because `A` is now a changed target.

This rides the single baseline mechanism: no separate propagation engine, no
transitive-closure state, and no "suspect storm" from a change that does not actually
alter an intermediate element's projected content.
