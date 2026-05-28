---
id: FTG-SIL-001
type: FaultTreeGate
name: FTG-SIL-001
gateType: OR
title: OR gate — software conflict check bypassed OR 2oo2 comparison fails
inputs:
  - FTG-SIL-002
  - FTE-SIL-003
probability: 2.0e-9
---

Top-level OR gate for the fault tree FT-SIL-001. The top event "Conflicting route set without detection" occurs if either of the two independent failure paths is activated:

1. **FTG-SIL-002** — the software-level conflict check fails on both diverse channels simultaneously (AND gate).
2. **FTE-SIL-003** — the 2oo2 cross-comparison hardware mechanism itself fails to detect a channel discrepancy.

For an OR gate, the probability is approximately the sum of the input probabilities when both are small. Here FTE-SIL-003 dominates at 2.0 × 10⁻⁹ /h; FTG-SIL-002 at 1.0 × 10⁻¹² /h is negligible. The OR gate probability is therefore ≈ 2.0 × 10⁻⁹ /h, meeting the SIL 4 target of < 10⁻⁸ /h.
