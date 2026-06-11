---
id: FTE-SIL-002
type: FaultTreeEvent
eventKind: basic
title: Channel B vital software — conflict check erroneous output (diverse)
ref: System::Software::ConflictChecker
failureRate: 1.0e-6
probability: 1.0e-3
---

Channel B's ConflictChecker (diverse implementation) independently produces the same erroneous conflict-check output as Channel A. Channel B is a completely separate software implementation: different programming language, different development team, different tool chain, different compiler, developed under the EN 50128 diverse software process.

**Diversity argument.** The probability of Channel B failing in the same direction as Channel A — that is, also concluding "no conflict" for the same conflicting route pair at the same time — is modelled as statistically independent from Channel A's failure. This independence assumption is the core of the software diversity safety argument. It is substantiated by the diverse development evidence package and is challenged during EN 50129 safety case assessment.

**Failure rate.** Same order of magnitude as Channel A (1.0 × 10⁻⁶ /h) because both implementations are subject to the same EN 50128 SIL 4 process, formal verification, and test coverage requirements. The actual implementations are independent, so the rates may differ; using the same rate is conservative in the sense that it maximises the AND-gate product.

**Contribution to AND gate.** FTE-SIL-001 × FTE-SIL-002 = 1.0 × 10⁻⁶ × 1.0 × 10⁻⁶ = 1.0 × 10⁻¹² /h². This is the key quantitative argument for software diversity at SIL 4: the product of two SIL 2-equivalent channels (each ~ 10⁻⁶ /h) achieves SIL 4 for the combined dangerous failure rate.
