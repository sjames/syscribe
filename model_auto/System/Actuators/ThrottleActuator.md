---
type: PartDef
name: Throttle Actuator
domain: hardware
features:
  - name: motorType
    type: ScalarValues::String
  - name: returnSpringForce
    type: ScalarValues::Real
    unit: N
---

DC motor-driven electronic throttle body with integrated return spring.
The throttle plate position is controlled by `System::Software::ThrottleControl`
via an H-bridge driver.

## Safety design

A return spring provides a fail-safe mechanical default to approximately 7 %
opening (sufficient for limp-home but insufficient for normal acceleration).
If software command is lost, the spring returns the throttle to this position.

This is the primary actuator in the unintended-acceleration hazard chain
(see `Safety::HARA::HE-ENG-001` and `Safety::FTA::FT-ENG-001`).
