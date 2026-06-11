---
id: FT-SIL-001
type: FaultTree
title: Fault tree for SG-SIL-001 — conflicting route set without detection
topEvent: SG-SIL-001
status: approved
missionTime: "350400 h"
---

Fault tree for safety goal SG-SIL-001: "Prevent conflicting train routes from being set simultaneously". The top event is **"Conflicting route set without detection"** — a state in which two routes sharing a common track section, points machine, or protection zone are simultaneously active and the interlocking has not asserted the safe state.

**Methodology.** Binary fault tree per IEC 61025. All gates and events are assigned failure probabilities derived from IEC TR 62380 (hardware) and EN 50128 SIL 4 software reliability models. The dangerous failure rate target is < 10⁻⁸ /h (SIL 4, IEC 61508 Table 2, low-demand mode).

**Mission time.** 350 400 h corresponds to 40 years of service life — the typical design life for railway interlocking equipment. All failure rate calculations are evaluated at this mission time.

**Tree structure.**

The top event is an OR gate (FTG-SIL-001) with two branches:

1. **Software conflict check bypassed (FTG-SIL-002 — AND gate):** Both diverse software channels simultaneously produce the same erroneous conflict-check output. This requires Channel A (FTE-SIL-001) AND Channel B (FTE-SIL-002) to independently fail in the same direction — the AND gate captures the diversity argument.

2. **2oo2 cross-comparison hardware failure (FTE-SIL-003):** The hardware comparison circuit itself fails to detect the channel discrepancy. This is the hardware integrity floor for the architecture.

**Quantitative result.** The dominant contribution is FTE-SIL-003 at 2.0 × 10⁻⁹ /h; FTG-SIL-002 contributes 1.0 × 10⁻¹² /h (negligible by comparison). Total top-event probability ≈ 2.0 × 10⁻⁹ /h, satisfying the SIL 4 target by one order of magnitude. This margin provides headroom for common-cause failure (CCF) contributions not individually modelled here.
