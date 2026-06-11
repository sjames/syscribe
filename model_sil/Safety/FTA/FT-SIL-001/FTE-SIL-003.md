---
id: FTE-SIL-003
type: FaultTreeEvent
eventKind: basic
title: 2oo2 cross-comparison hardware failure — both channels produce matching erroneous outputs
ref: System::Hardware::VitalProcessor
failureRate: 2.0e-9
probability: 2.0e-6
---

The hardware cross-comparison circuit fails such that two discrepant channel outputs are reported as matching, or the fail-safe relay chain fails to open on detection of a disagreement. This event represents the hardware integrity floor for the 2oo2 architecture: even if the software diversity argument (FTG-SIL-002) is accepted, the hardware comparison mechanism must itself be reliable to a SIL 4 standard.

**Failure modes covered.**

1. **Comparison ASIC failure**: The dedicated comparison ASIC misclassifies a discrepant pair of channel outputs as matching, preventing the safe-state assertion. This could result from a permanent hardware fault (stuck-at) or a transient fault (SEU) in the comparison logic.
2. **Relay chain failure**: The fail-safe relay that opens on comparison failure fails in the energised (closed) position — a dangerous failure of the output relay. This is the "relay stuck closed" failure mode; it is defended against by using normally-energised relay logic (de-energising to safe state) with dual contacts in series.
3. **Communication path failure**: The inter-channel comparison bus fails in a way that causes Channel A's output to be substituted for Channel B's output, making both sides appear identical even when the underlying channel computations differ.

**Failure rate basis.** 2.0 × 10⁻⁹ /h is derived from IEC TR 62380 reliability data for the comparison ASIC and relay combination at the operating temperature profile of the vital processor unit. This rate is the dominant term in the FT-SIL-001 top event (≈ 2.0 × 10⁻⁹ /h vs. 1.0 × 10⁻¹² /h for the AND gate), confirming that the hardware comparison mechanism is the reliability-limiting element of the 2oo2 architecture.
