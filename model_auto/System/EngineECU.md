---
type: PartDef
name: Engine ECU
domain: hardware
features:
  - name: supplyVoltage
    type: ScalarValues::Real
    unit: V
  - name: operatingTempMin
    type: ScalarValues::Real
    unit: degC
  - name: operatingTempMax
    type: ScalarValues::Real
    unit: degC
  - name: primarySpeedSensor
    typedBy: System::Sensors::Sensor
connections:
  - typedBy: System::Interfaces::CANBusConnection
    from: System::Software::ThrottleControl::canOut
    to: System::Hardware::CANTransceiver::canIn
---

The Engine ECU is the central hardware processing unit. It hosts the engine
control software and interfaces with sensors, actuators, and the vehicle CAN bus.

## Physical characteristics

Operating voltage range: 9–16 V DC. Temperature range: −40 °C to +125 °C.
Qualified to AEC-Q100 Grade 1 for automotive use.

## Interfaces

- **CAN bus** — high-speed ISO 11898 at 500 kbit/s (powertrain network)
- **OBD-II port** — K-line and CAN-based diagnostics per ISO 15031
- **Sensor inputs** — 0–5 V analogue and digital (crank/cam VR, throttle position, lambda)
- **Actuator outputs** — injector drivers (peak-and-hold), throttle motor H-bridge
