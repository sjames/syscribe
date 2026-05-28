---
type: Requirement
id: REQ-SIL-SW-003
title: Vital logic shall be formally specified using B-Method with machine-checked invariant proofs
status: approved
reqDomain: software
silLevel: 4
verificationMethod: analysis
derivedFrom:
  - REQ-SIL-SAFE-000
breakdownAdr: ADR-SIL-SW-001
---

All vital interlocking logic — route conflict detection, signal clearance conditions, points position management — **shall** be specified as B-Method abstract machines (Event-B notation). All safety invariants (e.g., "no two conflicting routes are simultaneously active") **shall** be expressed as machine invariants with machine-checked proofs using Atelier-B or ProB. The B-Method abstract machines **shall** be refined stepwise to a concrete implementation; each refinement step **shall** be verified to preserve the invariants. The complete proof obligation corpus **shall** be archived and included in the EN 50129 safety case. Reference: EN 50128 Table A.9 (Formal Methods — highly recommended at SIL 3/4).
