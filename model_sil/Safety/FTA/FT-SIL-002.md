---
id: FT-SIL-002
type: FaultTree
title: Fault tree for SG-SIL-002 — signal cleared without all conditions satisfied
topEvent: SG-SIL-002
status: approved
missionTime: "350400 h"
---

Fault tree for safety goal SG-SIL-002: "Prevent signal clearance unless all route conditions are verified". The top event is **"Signal cleared without all conditions satisfied"** — a state in which a signal is displaying a proceed aspect but one or more of the mandatory interlocking conditions (section clear, points detected, route locked, no conflict, level crossing confirmed) is not satisfied.

**Methodology.** Binary fault tree per IEC 61025. Hardware failure rates from IEC TR 62380; software failure rates from EN 50128 SIL 4 reliability model. The dangerous failure rate target is < 10⁻⁸ /h (SIL 4, IEC 61508 Table 2, low-demand mode).

**Mission time.** 350 400 h (40 years).

**Tree structure.**

The top event is an OR gate (FTG-SIL-003) with three independent branches representing the three dominant failure pathways:

1. **Track circuit false-clear (FTE-SIL-004):** The audio-frequency track circuit reports a "clear" indication while a train is present in the section. This is the dominant failure mode (failure rate 3.0 × 10⁻⁹ /h).

2. **Points detection false-confirm (FTE-SIL-005):** The points detection contacts falsely confirm the points are in the commanded position when they are not. Failure rate 8.0 × 10⁻¹⁰ /h.

3. **SignalController software error (FTE-SIL-006):** Both diverse channels of the SignalController independently produce an erroneous "conditions satisfied" output. Failure rate 5.0 × 10⁻¹⁰ /h.

**Quantitative result.** For an OR gate, total ≈ 3.0 × 10⁻⁹ + 8.0 × 10⁻¹⁰ + 5.0 × 10⁻¹⁰ ≈ 4.0 × 10⁻⁹ /h. The track circuit failure (FTE-SIL-004) is the dominant contributor. The total satisfies the SIL 4 target of < 10⁻⁸ /h with margin of approximately 2.5×.
