---
type: PartDef
name: DeliveryPayload
supertype: Parts::Part
isVariant: true
domain: hardware
appliesWhen: Features::Payload::Delivery
satisfies:
  - REQ-UAV-CARGO-001
implementedBy:
  - repo:firmware/payload/cargo_release.rs
features:
  - name: capacityKg
    typedBy: ScalarValues::Real
    unit: SI::kg
    isReadonly: true
  - name: releaseActuator
    typedBy: ScalarValues::Boolean
    isDerived: true
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
---

Delivery payload variant — a cargo module with an electrically actuated release
mechanism. Active only in products that select the `Delivery` feature; satisfies
the cargo-release requirement.
