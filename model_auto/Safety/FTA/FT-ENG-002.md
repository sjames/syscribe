---
type: FaultTree
id: FT-ENG-002
title: Fault tree — engine stall during high-speed operation
status: approved
topEvent: SG-ENG-002
missionTime: "8760 h"
---

Analyses all failure pathways leading to uncontrolled engine stall during
high-speed operation, as defined by safety goal SG-ENG-002 (HE-ENG-002, ASIL B).

## Top event

Engine stalls uncontrollably during high-speed operation — the engine speed
drops to zero without a controlled deceleration sequence, while the vehicle
is travelling above 100 km/h.

## Tree structure

The top event can occur via:

- **FTG-ENG-003** (OR gate): CPS signal is entirely lost due to either a
  wire harness open circuit (FTE-ENG-004) or a hardware OR of the common-cause
  combination (FTG-ENG-004) and ECU supply dropout (FTE-ENG-005).
- **FTG-ENG-004** (AND gate): CPS signal is degraded by target wheel corrosion
  (FTE-ENG-006) AND simultaneously disrupted by a high-intensity radiated field
  (FTE-ENG-007). This combination represents a common-cause failure that is
  controlled by EMC shielding requirements.

## Methodology

Binary fault tree per IEC 61025. Basic event failure rates sourced from
IEC TR 62380 (automotive component database). Common-cause analysis for the
FTG-ENG-004 AND gate performed using the beta-factor method per IEC 61508-6
Annex D. The low probability of the AND gate (~8.0e-13/h) confirms that the
EMC shielding design is sufficient to defeat the common-cause pathway at ASIL B.

Mission time is 8760 hours (one vehicle operating year at typical usage).
Probability figures represent per-hour failure rates at end of mission time.
