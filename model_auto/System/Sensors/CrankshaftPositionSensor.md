---
type: PartDef
name: Crankshaft Position Sensor
domain: hardware
supertype: System::Sensors::Sensor
features:
  - name: signalType
    type: ScalarValues::String
  - name: teethCount
    type: ScalarValues::Integer
---

Variable-reluctance crankshaft position sensor (CPS) providing engine speed
and top-dead-centre reference to the ECU.

The CPS signal is the primary input for ignition timing calculation and fuel
injection synchronisation. Loss of the CPS signal causes the engine to stall
(see `Safety::HARA::HE-ENG-002`).
