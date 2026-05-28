---
type: FaultTreeGate
id: FTG-ENG-004
title: AND gate — CPS signal degraded AND EMC noise burst (common cause)
gateType: AND
inputs:
  - FTE-ENG-006
  - FTE-ENG-007
probability: 8.0e-13
---

This AND gate represents the common-cause failure combination: the CPS signal
amplitude is already reduced by target wheel corrosion (FTE-ENG-006) AND
a high-intensity radiated field simultaneously causes a signal dropout
(FTE-ENG-007).

The extremely low combined probability (~8.0e-13/h) confirms that the vehicle's
EMC shielding and the required CPS harness routing separation are effective
defences against this pathway. The beta-factor for common-cause analysis is
estimated at β = 0.02, as specified in the EMC design requirements.

In isolation, either event is insufficient to cause a stall: degraded amplitude
is still decoded by the ECU's adaptive threshold, and a single HIRF burst without
pre-existing degradation is filtered by the signal conditioning circuit.
