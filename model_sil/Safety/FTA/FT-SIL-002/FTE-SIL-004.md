---
id: FTE-SIL-004
type: FaultTreeEvent
eventKind: basic
title: Track circuit false-clear — train present but section reported clear
ref: System::Hardware::TrackCircuitInterface
failureRate: 3.0e-9
probability: 3.0e-6
---

An audio-frequency track circuit fails to shunt correctly and reports a "clear" indication while a train is present in the section. This is the dominant failure mode for the signal clearance condition check and the largest single contributor to the FT-SIL-002 top event.

**Failure mechanism.** Track circuit shunting relies on the train's wheels and axles providing a low-resistance path between the two running rails, reducing the rail current below the "occupied" detection threshold. The shunt can fail when:

1. **High ballast resistance**: Water ingress into the ballast between the rails reduces isolation, causing the track circuit to operate near the shunting threshold. A train with worn wheel treads or contaminated wheel/rail contact may not provide sufficient shunting.
2. **Insulated rail joint failure**: A failed insulated rail joint allows current from an adjacent track circuit to flow into the occupied section, maintaining the circuit above the "clear" threshold despite the train's shunting action.
3. **Wheel/rail contact film**: Oxidation, oil, or leaf contamination on the rail can increase wheel/rail contact resistance above the shunting threshold in some conditions.

**Defence in depth.** The probability accounts for the combined probability of the shunting failure AND the failure of the in-built track circuit self-supervision to detect the fault. Audio-frequency track circuits include continuous current and frequency monitoring; a shunting failure that produces an abnormal current level will in many cases be detected as a circuit fault (reporting "occupied") rather than a false-clear. The quoted failure rate is for the residual case where the failure mode produces a false-clear that is not self-detected.

**Failure rate basis.** 3.0 × 10⁻⁹ /h from field reliability data for audio-frequency track circuits on mainline operation, accounting for the self-supervision mechanism. The EN 50129 safety case for the track circuit equipment must substantiate this figure.
