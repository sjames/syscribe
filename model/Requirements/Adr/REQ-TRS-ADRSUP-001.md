---
type: Requirement
id: REQ-TRS-ADRSUP-001
name: "ADRs express, validate and show supersession"
status: draft
reqDomain: software
reqClass: system
tags:
  - adr
---

An ADR shall be able to supersede others, and the tool shall resolve, validate and show the chain (GH #232).

## Behavior

- `supersedes:` on an ADR accepts one reference or a list; each shall resolve to an `ADR`, else `E320`.
- A supersession cycle, including an ADR superseding itself, is `E321`.
- `W313`: an ADR that supersedes another ADR whose status is not `superseded`.
- `W314`: a non-draft element whose `breakdownAdr:` resolves to a `superseded` ADR (it should cite the successor).
- The reverse relation `supersededBy` is computed. `show` prints `supersededBy` and `supersedes` for an ADR, and `links` lists the `supersedes` edge.
- `Baseline.supersedes` keeps working unchanged (`E522`).
- The authoring prompt tells authors to put `supersedes:` on the new ADR (not `supersededBy:` on the old one).
