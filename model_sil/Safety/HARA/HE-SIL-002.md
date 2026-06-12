---
id: HE-SIL-002
type: HazardousEvent
status: approved
name: Conflicting routes set simultaneously — risk of head-on or side collision at junction
consequence: Cd
freqExposure: Fb
avoidance: Pb
demandRate: W3
operationalSituation: Junction operation with simultaneous movements from multiple directions
---

Two conflicting train routes are set at the same time — for example, two trains both cleared onto a single-line section from opposite directions, or two routes sharing a common set of points. At junction speeds (40–80 km/h), braking distance exceeds the overlap by a significant margin, making a collision unavoidable once both routes are set.

The hazard is particularly severe at junctions with complex track geometry where multiple conflicting movements are possible and the interlocking must evaluate a large conflict matrix simultaneously. A single software or hardware failure that bypasses the conflict check exposes multiple trains to head-on or flank collision.

Risk parameters per IEC 61508-5 Annex D risk graph:

- **Consequence (Cd)**: Catastrophic — a head-on collision at junction approach speed is likely to be fatal to train crew and passengers in the leading vehicles.
- **Frequency of exposure (Fb)**: Frequent — junction operation is continuous; conflicting route combinations are present throughout the operational day.
- **Avoidance (Pb)**: Not possible — once both routes are set and signals are cleared, neither driver has sufficient advance warning to stop within the overlap.
- **Demand rate (W3)**: High — the conflict-check demand is continuous during normal junction operation.

**SIL target: 4** (Cd × Fb × Pb × W3 per IEC 61508-5 Annex D).
