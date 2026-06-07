---
type: PartDef
name: BackupFlightController
supertype: Parts::Part
domain: software
asilLevel: C
appliesWhen: Features::DualFlightController
satisfies:
  - REQ-UAV-REDUN-001
implementedBy:
  - repo:firmware/flight_control/failover.rs
features:
  - name: heartbeatTimeoutMs
    typedBy: ScalarValues::Integer
    isReadonly: true
    isConstant: true
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
---

Redundant backup flight controller. Cross-monitors the primary flight controller
and assumes control authority on heartbeat loss. Active only in products that
select the `DualFlightController` feature; satisfies the failover requirement.
