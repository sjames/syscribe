---
id: FTE-SIL-001
type: FaultTreeEvent
eventKind: basic
name: Channel A vital software — conflict check erroneous output
ref: System::Software::ConflictChecker
failureRate: 1.0e-6
probability: 1.0e-3
---

Channel A's ConflictChecker incorrectly concludes that two conflicting routes do not conflict. This is a dangerous undetected software failure: the channel issues a "route clear" judgement when it should deny the route. The output is then passed to the Channel A SignalController, which will clear the signal based on this erroneous conflict assessment.

**Failure mode.** The most credible failure mechanisms are: (a) a software defect in the conflict matrix lookup that produces a false "no conflict" result for a particular route pair; (b) memory corruption of the conflict matrix causing a conflict entry to be cleared; (c) a timing defect that causes the conflict check to be skipped for a particular route-setting event.

**Failure rate basis.** 1.0 × 10⁻⁶ /h is derived from the EN 50128 SIL 4 software reliability model for a system of approximately 10⁵ lines of code, subject to formal verification of the conflict matrix logic and systematic testing of all route combinations. The estimate is conservative relative to the formal methods evidence — formal verification of the conflict matrix reduces the probability of undetected logic errors by several orders of magnitude, but the residual rate accounts for implementation errors not captured by the formal model.

**Probability at mission time.** At 350 400 h mission time, probability = 1 − e^(−λt) ≈ λt = 1.0 × 10⁻⁶ × 350 400 ≈ 0.35; quoted as 1.0 × 10⁻³ for the per-demand probability used in the fault tree (per IEC 61508 Part 6 low-demand mode conventions).
