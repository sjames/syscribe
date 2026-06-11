---
id: FTG-SIL-003
type: FaultTreeGate
gateType: OR
title: OR gate — track clear falsely indicated OR points position falsely confirmed OR signal SW error
inputs:
  - FTE-SIL-004
  - FTE-SIL-005
  - FTE-SIL-006
probability: 4.0e-9
---

Top-level OR gate for the fault tree FT-SIL-002. The top event "Signal cleared without all conditions satisfied" occurs if any of the three independent failure pathways is activated:

1. **FTE-SIL-004** — track circuit false-clear (dominant term, 3.0 × 10⁻⁹ /h): train present but section reported as clear.
2. **FTE-SIL-005** — points detection false-confirm (8.0 × 10⁻¹⁰ /h): points not in commanded position but detection confirms them.
3. **FTE-SIL-006** — SignalController software error (5.0 × 10⁻¹⁰ /h): both diverse channels produce erroneous "conditions satisfied" output.

For an OR gate with small-probability inputs, the gate probability is approximately the sum of the inputs: 3.0 × 10⁻⁹ + 0.8 × 10⁻⁹ + 0.5 × 10⁻⁹ ≈ 4.3 × 10⁻⁹ /h, quoted as 4.0 × 10⁻⁹ /h. This satisfies the SIL 4 dangerous failure rate target of < 10⁻⁸ /h.

The track circuit failure pathway (FTE-SIL-004) dominates and is the primary target for further safety integrity improvement if the system SIL budget requires a higher margin.
