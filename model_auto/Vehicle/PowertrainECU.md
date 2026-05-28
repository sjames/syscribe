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

The Engine ECU unit as installed in the vehicle powertrain bay. Hosts the engine control
software image and interfaces with all engine sensors and actuators via dedicated wire
harness connectors.

## Deployment context

The ECU is mounted to the engine bay bulkhead in a sealed aluminium housing (IP6K9K) to
withstand underhood temperatures up to +125 °C and road-splash ingress. Power supply is
taken directly from the vehicle battery via a 30 A fuse; supply voltage is redefined to
14.0 V nominal (battery + charging system) from the PartDef default range of 9–16 V.

## Internal connections

The `connections:` field binds the `ThrottleControl` software component's CAN output port
to the hardware `CANTransceiver` CAN input port. This is the primary safety-critical data
path: the throttle position command is serialised into a CAN frame, authenticated with a
24-bit CMAC-AES-128 MAC (SC-ENG-001), and transmitted onto the powertrain CAN bus at a
10 ms cycle time.

## Sub-parts

All sub-components — sensors, actuators, hardware ICs, and software SWC instances — are
owned as Part usages in the `Vehicle/PowertrainECU/` directory, conforming to the Syscribe
naming convention where `PowertrainECU/` is the namespace for the deployed ECU's contents.
