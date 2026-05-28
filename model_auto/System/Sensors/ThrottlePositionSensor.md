---
type: PartDef
name: Throttle Position Sensor
domain: hardware
supertype: System::Sensors::Sensor
features:
  - name: outputRange
    type: ScalarValues::String
  - name: supplyVoltage
    type: ScalarValues::Real
    unit: V
---

Dual-track potentiometric throttle position sensor (TPS) providing pedal
demand position to the electronic throttle control system.

Dual-track design provides redundancy; both tracks are monitored by the safety
monitor (`System::Software::SafetyMonitor`) for divergence. Track divergence
beyond 5 % is a safety fault (see `Safety::HARA::HE-ENG-001`).
