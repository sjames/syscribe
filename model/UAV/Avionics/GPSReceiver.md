---
type: PartDef
name: GPSReceiver
supertype: Parts::Part
domain: hardware
satisfies:
  - REQ-UAV-NAV-001
implementedBy:
  - repo:firmware/drivers/gnss.rs
features:
  - name: currentFix
    typedBy: Items::GPSFix
    isDerived: true
    isReadonly: true
  - name: fixQuality
    typedBy: ScalarValues::Integer
    isDerived: true
    isReadonly: true
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
---

Multi-constellation GNSS receiver providing position fixes to the flight controller. Reports fix quality for use in navigation state machine guards.
