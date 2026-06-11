---
id: FTE-SIL-006
type: FaultTreeEvent
eventKind: basic
title: SignalController software error — clears signal without all conditions true
ref: System::Software::SignalController
failureRate: 5.0e-10
probability: 5.0e-7
---

Both channels of the SignalController software independently produce an erroneous "conditions satisfied" output, causing a signal to be cleared when one or more mandatory interlocking conditions are not actually satisfied. Given the 2oo2 architecture with software diversity, this event represents the combined probability of both diverse implementations making the same error on the same input.

**Scope.** This event covers the SignalController's pre-clearance check logic — the code path that evaluates all five mandatory conditions (section clear, points confirmed, route locked, no conflict, level crossing confirmed) before asserting a signal clearance. The event does not cover the continuous supervision loop failure (which would be a separate event if the tree were extended) or the points/track-circuit condition check failures (covered by FTE-SIL-004 and FTE-SIL-005).

**Software diversity AND gate.** The 2oo2 architecture means that both Channel A and Channel B SignalControllers must independently produce the "conditions satisfied" output for the signal to be cleared. The probability of both diverse implementations making the same error is the product of their individual probabilities, by the independence assumption substantiated by the EN 50128 diverse development evidence package.

**Failure rate derivation.** The quoted 5.0 × 10⁻¹⁰ /h is the combined (AND) probability for the two-channel diverse system. Each channel's individual dangerous failure rate for the conditions-evaluation logic is approximately 2.2 × 10⁻⁵ /h (before diversity); the AND combination gives (2.2 × 10⁻⁵)² ≈ 5.0 × 10⁻¹⁰ /h. This is the software-dominated residual term in the FT-SIL-002 OR gate — smaller than the track circuit failure (FTE-SIL-004) by a factor of ~6.

**Residual risk justification.** This event is the "software floor" for the SG-SIL-002 fault tree. Further reduction in this term requires either higher-integrity software (e.g., independently assessed formal proof) or additional hardware diversity. The current architecture achieves the SIL 4 target with margin, so no further reduction is required at this stage.
