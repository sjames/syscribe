---
type: FaultTreeEvent
id: FTE-ENG-004
name: CPS signal wire harness open circuit
eventKind: basic
ref: System::Sensors::CrankshaftPositionSensor
failureRate: 1.2e-6
probability: 1.2e-3
---

An open circuit in the CPS signal wire harness causes total loss of the
crankshaft position signal. The ECU receives no crank events and cannot
maintain injection or ignition timing, leading to engine stall within two
engine cycles.

Primary causes include connector corrosion, chafed wire insulation causing
a short-to-ground (open-circuit effect on the differential signal), or
mechanical fatigue fracture at the harness bend radius near the sensor connector.

This is the dominant failure mode in the engine stall fault tree, representing
approximately 93 % of the FTG-ENG-003 OR gate probability. Mitigation requires
harness routing away from heat sources and mechanical stress points, per the
engine bay harness design standard.
