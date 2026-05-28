---
type: Part
name: Powertrain ECU
typedBy: System::EngineECU
domain: hardware
features:
  - name: supplyVoltage
    type: ScalarValues::Real
    unit: V
    redefines: [System::EngineECU::supplyVoltage]
connections:
  - typedBy: System::Interfaces::CANBusConnection
    from: softwareImage.throttleControl.canOut
    to: canTransceiver.canIn
---

The Engine ECU unit as installed in the vehicle powertrain bay.
Hosts the engine control software image and interfaces with all
engine sensors and actuators via dedicated wire harness connectors.
